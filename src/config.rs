use std::env;

const CLIENT_ID_ENV: &str = "TINY_TUNE_SPOTIFY_CLIENT_ID";
const REDIRECT_URI_ENV: &str = "TINY_TUNE_SPOTIFY_REDIRECT_URI";
const DEFAULT_REDIRECT_URI: &str = "http://127.0.0.1:8898/login";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpotifyConfig {
    pub client_id: String,
    pub redirect_uri: String,
}

impl SpotifyConfig {
    pub fn from_env(default_client_id: impl Into<String>) -> Self {
        let client_id = resolve_client_id(env::var(CLIENT_ID_ENV).ok(), default_client_id.into());
        let redirect_uri = env::var(REDIRECT_URI_ENV)
            .ok()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_REDIRECT_URI.into());

        Self {
            client_id,
            redirect_uri,
        }
    }
}

fn resolve_client_id(configured: Option<String>, default: String) -> String {
    configured
        .filter(|value| {
            let value = value.trim();
            !value.is_empty() && value != "your-client-id"
        })
        .unwrap_or(default)
}

#[cfg(test)]
#[path = "__tests__/config.rs"]
mod tests;
