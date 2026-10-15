use std::env;

const CLIENT_ID_ENV: &str = "TINY_TUNE_SPOTIFY_CLIENT_ID";
const REDIRECT_URI_ENV: &str = "TINY_TUNE_SPOTIFY_REDIRECT_URI";
const IMAGE_CACHE_BUDGET_MB_ENV: &str = "TINY_TUNE_IMAGE_CACHE_BUDGET_MB";
const DEFAULT_REDIRECT_URI: &str = "http://127.0.0.1:8898/login";
pub const DEFAULT_IMAGE_CACHE_BUDGET_MB: u16 = 8;
pub const MAX_IMAGE_CACHE_BUDGET_MB: u16 = 64;

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

/// Non-secret settings. Image bytes retained by the UI cache stay within this budget.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AppConfig {
    pub image_cache_budget_mb: u16,
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self {
            image_cache_budget_mb: image_cache_budget_mb(
                env::var(IMAGE_CACHE_BUDGET_MB_ENV).ok().as_deref(),
            ),
        }
    }

    pub fn image_cache_budget_bytes(self) -> usize {
        usize::from(self.image_cache_budget_mb) * 1024 * 1024
    }
}

fn image_cache_budget_mb(configured: Option<&str>) -> u16 {
    configured
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .and_then(|value| value.parse::<u16>().ok())
        .filter(|value| *value <= MAX_IMAGE_CACHE_BUDGET_MB)
        .unwrap_or(DEFAULT_IMAGE_CACHE_BUDGET_MB)
}

#[cfg(test)]
#[path = "__tests__/config.rs"]
mod tests;
