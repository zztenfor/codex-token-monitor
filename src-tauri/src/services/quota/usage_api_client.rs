use super::{parse_quota_response, AccountQuota, AuthCredentials, QuotaError};
use reqwest::{blocking::Client, StatusCode};
use std::time::Duration;

const USAGE_URL: &str = "https://chatgpt.com/backend-api/wham/usage";

pub struct UsageApiClient {
    client: Client,
    endpoint: String,
}

impl UsageApiClient {
    pub fn new() -> Result<Self, QuotaError> {
        Self::with_endpoint(USAGE_URL)
    }

    fn with_endpoint(endpoint: &str) -> Result<Self, QuotaError> {
        let client = Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(20))
            .user_agent("Codex Token Monitor")
            .build()
            .map_err(|_| QuotaError::ConnectionFailed)?;
        Ok(Self {
            client,
            endpoint: endpoint.to_owned(),
        })
    }

    pub fn fetch(&self, auth: &AuthCredentials) -> Result<AccountQuota, QuotaError> {
        let response = self
            .client
            .get(&self.endpoint)
            .bearer_auth(auth.access_token())
            .header("ChatGPT-Account-Id", auth.account_id())
            .send()
            .map_err(|_| QuotaError::ConnectionFailed)?;
        if matches!(
            response.status(),
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN
        ) {
            return Err(QuotaError::AuthenticationExpired);
        }
        if !response.status().is_success() {
            return Err(QuotaError::QuotaUnavailable);
        }
        let value = response.json().map_err(|_| QuotaError::QuotaUnavailable)?;
        parse_quota_response(&value)
    }

    #[cfg(test)]
    pub fn diagnostic_shape(&self, auth: &AuthCredentials) -> Result<String, QuotaError> {
        let response = self
            .client
            .get(&self.endpoint)
            .bearer_auth(auth.access_token())
            .header("ChatGPT-Account-Id", auth.account_id())
            .send()
            .map_err(|_| QuotaError::ConnectionFailed)?;
        let status = response.status().as_u16();
        let value: serde_json::Value = response.json().unwrap_or(serde_json::Value::Null);
        let keys = |candidate: Option<&serde_json::Value>| -> String {
            let mut values = candidate
                .and_then(serde_json::Value::as_object)
                .map(|object| object.keys().cloned().collect::<Vec<_>>())
                .unwrap_or_default();
            values.sort();
            values.join(",")
        };
        let rate = value
            .get("rate_limit")
            .or_else(|| value.get("rate_limits"))
            .or_else(|| value.get("usage"));
        Ok(format!(
            "http_status={status};top_keys={};rate_keys={};primary_keys={};secondary_keys={}",
            keys(Some(&value)),
            keys(rate),
            keys(rate.and_then(|item| item.get("primary_window").or_else(|| item.get("primary")))),
            keys(rate.and_then(|item| {
                item.get("secondary_window")
                    .or_else(|| item.get("secondary"))
            }))
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::{Read, Write},
        net::TcpListener,
        thread,
    };

    #[test]
    fn calls_usage_endpoint_with_required_headers() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buffer = [0_u8; 4096];
            let count = stream.read(&mut buffer).unwrap();
            let request = String::from_utf8_lossy(&buffer[..count]);
            assert!(request.starts_with("GET /usage "));
            assert!(request
                .to_ascii_lowercase()
                .contains("authorization: bearer fake-access"));
            assert!(request
                .to_ascii_lowercase()
                .contains("chatgpt-account-id: fake-account"));
            let body = r#"{"primary_window":{"used_percent":10,"reset_after_seconds":60},"secondary_window":{"used_percent":20,"reset_after_seconds":120}}"#;
            write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", body.len(), body).unwrap();
        });
        let client = UsageApiClient::with_endpoint(&format!("http://{address}/usage")).unwrap();
        let auth = AuthCredentials {
            access_token: "fake-access".into(),
            account_id: "fake-account".into(),
        };
        let quota = client.fetch(&auth).unwrap();
        server.join().unwrap();
        assert_eq!(quota.five_hour.unwrap().used_percent, 10.0);
        assert_eq!(quota.weekly.unwrap().used_percent, 20.0);
    }
}
