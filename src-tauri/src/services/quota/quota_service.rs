use super::{read_auth, AccountQuota, UsageApiClient};
use std::{fmt, path::Path};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum QuotaError {
    AuthNotFound,
    AuthInvalid,
    AuthenticationExpired,
    QuotaUnavailable,
    ConnectionFailed,
}

impl QuotaError {
    pub fn code(self) -> &'static str {
        match self {
            Self::AuthNotFound => "AUTH_NOT_FOUND",
            Self::AuthInvalid => "AUTH_INVALID",
            Self::AuthenticationExpired => "AUTH_EXPIRED",
            Self::QuotaUnavailable => "QUOTA_UNAVAILABLE",
            Self::ConnectionFailed => "CONNECTION_FAILED",
        }
    }

    pub fn user_message(self) -> &'static str {
        match self {
            Self::AuthNotFound | Self::AuthInvalid => "Please login Codex first",
            Self::AuthenticationExpired => "Authentication expired",
            Self::QuotaUnavailable => "Quota unavailable",
            Self::ConnectionFailed => "Connection failed",
        }
    }
}

impl fmt::Debug for QuotaError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.code())
    }
}

impl fmt::Display for QuotaError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.user_message())
    }
}

impl std::error::Error for QuotaError {}

pub struct QuotaService {
    client: UsageApiClient,
}

impl QuotaService {
    pub fn new() -> Result<Self, QuotaError> {
        Ok(Self {
            client: UsageApiClient::new()?,
        })
    }

    pub fn fetch(&self, codex_home: &Path) -> Result<AccountQuota, QuotaError> {
        let auth = read_auth(codex_home)?;
        self.client.fetch(&auth)
    }
}

#[cfg(test)]
mod live_tests {
    use super::*;

    #[test]
    #[ignore = "requires the current user's Codex login and network access"]
    fn live_quota_probe() {
        let codex_home = std::env::var_os("CODEX_HOME")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::env::var_os("USERPROFILE")
                    .map(std::path::PathBuf::from)
                    .unwrap()
                    .join(".codex")
            });
        let service = QuotaService::new().unwrap();
        let auth = read_auth(&codex_home).unwrap();
        println!("{}", service.client.diagnostic_shape(&auth).unwrap());
        let quota = service.client.fetch(&auth).unwrap();
        if let Some(five_hour) = quota.five_hour {
            println!(
                "5H used={:.1}% remaining={:.1}% reset={}s",
                five_hour.used_percent, five_hour.remaining_percent, five_hour.reset_after_seconds
            );
        } else {
            println!("5H unavailable (official 5-hour window was not returned)");
        }
        if let Some(weekly) = quota.weekly {
            println!(
                "7D used={:.1}% remaining={:.1}% reset={}s",
                weekly.used_percent, weekly.remaining_percent, weekly.reset_after_seconds
            );
        } else {
            println!("7D unavailable (official secondary_window is null)");
        }
    }
}
