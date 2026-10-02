use std::sync::Mutex;

use secrecy::{ExposeSecret, SecretString};

use super::{CredentialError, CredentialStore};

#[derive(Default)]
struct MemoryCredentialStore {
    refresh_token: Mutex<Option<String>>,
}

impl MemoryCredentialStore {
    fn saved_refresh_token(&self) -> Result<Option<String>, CredentialError> {
        Ok(self
            .refresh_token
            .lock()
            .map_err(|_| CredentialError)?
            .clone())
    }
}

impl CredentialStore for MemoryCredentialStore {
    fn save_refresh_token(&self, token: &SecretString) -> Result<(), CredentialError> {
        *self.refresh_token.lock().map_err(|_| CredentialError)? =
            Some(token.expose_secret().to_owned());
        Ok(())
    }
}

#[test]
fn credential_store_saves_without_exposing_the_token() {
    let store = MemoryCredentialStore::default();
    let token = SecretString::from("refresh-token-value");

    store.save_refresh_token(&token).expect("save token");

    assert_eq!(
        store.saved_refresh_token().expect("read test store"),
        Some("refresh-token-value".into())
    );
    assert!(!format!("{token:?}").contains("refresh-token-value"));
}
