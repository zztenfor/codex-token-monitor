use super::QuotaError;
use serde_json::Value;
use std::{fs, path::Path};

pub struct AuthCredentials {
    pub(super) access_token: String,
    pub(super) account_id: String,
}

impl AuthCredentials {
    pub fn access_token(&self) -> &str {
        &self.access_token
    }

    pub fn account_id(&self) -> &str {
        &self.account_id
    }
}

fn string_at<'a>(value: &'a Value, pointers: &[&str]) -> Option<&'a str> {
    pointers
        .iter()
        .filter_map(|pointer| value.pointer(pointer).and_then(Value::as_str))
        .find(|value| !value.is_empty())
}

pub fn read_auth(codex_home: &Path) -> Result<AuthCredentials, QuotaError> {
    let path = codex_home.join("auth.json");
    if !path.is_file() {
        return Err(QuotaError::AuthNotFound);
    }
    let contents = fs::read_to_string(path).map_err(|_| QuotaError::AuthNotFound)?;
    let value: Value = serde_json::from_str(&contents).map_err(|_| QuotaError::AuthInvalid)?;
    let access_token = string_at(&value, &["/tokens/access_token", "/access_token"])
        .ok_or(QuotaError::AuthInvalid)?;
    let account_id =
        string_at(&value, &["/tokens/account_id", "/account_id"]).ok_or(QuotaError::AuthInvalid)?;
    Ok(AuthCredentials {
        access_token: access_token.to_owned(),
        account_id: account_id.to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_nested_auth_without_persisting_credentials() {
        let temp = tempfile::tempdir().unwrap();
        fs::write(
            temp.path().join("auth.json"),
            r#"{"tokens":{"access_token":"secret-access","account_id":"account-123","refresh_token":"must-not-be-read"}}"#,
        )
        .unwrap();
        let auth = read_auth(temp.path()).unwrap();
        assert_eq!(auth.access_token(), "secret-access");
        assert_eq!(auth.account_id(), "account-123");
        assert!(!std::any::type_name_of_val(&auth).contains("secret-access"));
    }

    #[test]
    fn reports_missing_auth_file() {
        let temp = tempfile::tempdir().unwrap();
        assert!(matches!(
            read_auth(temp.path()),
            Err(QuotaError::AuthNotFound)
        ));
    }
}
