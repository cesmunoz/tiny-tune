use std::{error::Error, fmt};

use keyring::Entry;
use secrecy::{ExposeSecret, SecretString};

const SERVICE: &str = "com.tiny-tune.spotify";
const REFRESH_TOKEN_ACCOUNT: &str = "refresh-token";

pub trait CredentialStore: Send + Sync {
    fn save_refresh_token(&self, token: &SecretString) -> Result<(), CredentialError>;
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
    fn save_refresh_token(&self, token: &SecretString) -> Result<(), CredentialError> {
        Self::entry()?
            .set_password(token.expose_secret())
            .map_err(|_| CredentialError)
    }
}

#[cfg(test)]
#[path = "__tests__/credentials.rs"]
mod tests;
