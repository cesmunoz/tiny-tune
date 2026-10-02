use std::{error::Error, fmt};

use keyring::{Entry, Error as KeyringError};
use secrecy::{ExposeSecret, SecretString};

const SERVICE: &str = "com.tiny-tune.spotify";
const REFRESH_TOKEN_ACCOUNT: &str = "refresh-token";

pub trait CredentialStore: Send + Sync {
    fn load_refresh_token(&self) -> Result<Option<SecretString>, CredentialError>;
    fn save_refresh_token(&self, token: &SecretString) -> Result<(), CredentialError>;
    fn delete_refresh_token(&self) -> Result<(), CredentialError>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CredentialError;

impl fmt::Display for CredentialError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("secure credential storage is unavailable")
    }
}

impl Error for CredentialError {}

pub struct SystemCredentialStore;

impl SystemCredentialStore {
    pub fn new() -> Self {
        Self
    }

    fn entry() -> Result<Entry, CredentialError> {
        Entry::new(SERVICE, REFRESH_TOKEN_ACCOUNT).map_err(|_| CredentialError)
    }
}

impl CredentialStore for SystemCredentialStore {
    fn load_refresh_token(&self) -> Result<Option<SecretString>, CredentialError> {
        match Self::entry()?.get_password() {
            Ok(token) => Ok(Some(SecretString::from(token))),
            Err(KeyringError::NoEntry) => Ok(None),
            Err(_) => Err(CredentialError),
        }
    }

    fn save_refresh_token(&self, token: &SecretString) -> Result<(), CredentialError> {
        Self::entry()?
            .set_password(token.expose_secret())
            .map_err(|_| CredentialError)
    }

    fn delete_refresh_token(&self) -> Result<(), CredentialError> {
        match Self::entry()?.delete_credential() {
            Ok(()) | Err(KeyringError::NoEntry) => Ok(()),
            Err(_) => Err(CredentialError),
        }
    }
}

#[cfg(test)]
#[path = "__tests__/credentials.rs"]
mod tests;
