use std::sync::Mutex;

use secrecy::{ExposeSecret, SecretString};

use super::{CredentialError, CredentialStore};

#[derive(Default)]
struct MemoryCredentialStore {
    refresh_token: Mutex<Option<String>>,
}

impl CredentialStore for MemoryCredentialStore {
    fn load_refresh_token(&self) -> Result<Option<SecretString>, CredentialError> {
        Ok(self
            .refresh_token
            .lock()
            .map_err(|_| CredentialError)?
            .clone()
            .map(SecretString::from))
    }

    fn save_refresh_token(&self, token: &SecretString) -> Result<(), CredentialError> {
        *self.refresh_token.lock().map_err(|_| CredentialError)? =
            Some(token.expose_secret().to_owned());
        Ok(())
    }

    fn delete_refresh_token(&self) -> Result<(), CredentialError> {
        self.refresh_token
            .lock()
            .map_err(|_| CredentialError)?
            .take();
        Ok(())
    }
}

#[test]
fn credential_store_saves_without_exposing_the_token() {
    let store = MemoryCredentialStore::default();
    let token = SecretString::from("refresh-token-value");

    store.save_refresh_token(&token).expect("save token");

    let loaded = store
        .load_refresh_token()
        .expect("load token")
        .expect("stored token");
    assert_eq!(loaded.expose_secret(), "refresh-token-value");
    assert!(!format!("{token:?}").contains("refresh-token-value"));
}

#[test]
fn missing_credentials_load_as_none() {
    let store = MemoryCredentialStore::default();

    assert!(store.load_refresh_token().expect("load token").is_none());
}

#[test]
fn deleting_credentials_is_idempotent() {
    let store = MemoryCredentialStore::default();
    store
        .save_refresh_token(&SecretString::from("refresh-token-value"))
        .expect("save token");

    store.delete_refresh_token().expect("delete token");
    store.delete_refresh_token().expect("delete token again");

    assert!(store.load_refresh_token().expect("load token").is_none());
}
