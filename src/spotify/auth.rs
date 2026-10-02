use anyhow::Result;
use librespot::oauth::OAuthClientBuilder;
use secrecy::{ExposeSecret, SecretString};

use crate::config::SpotifyConfig;

const OAUTH_SUCCESS_HTML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/oauth-success.html"
));
const OAUTH_SCOPES: &[&str] = &[
    "streaming",
    "playlist-read-private",
    "playlist-read-collaborative",
];

pub struct AuthenticatedSession {
    pub access_token: String,
    pub refresh_token: SecretString,
}

pub async fn authenticate(config: &SpotifyConfig) -> Result<AuthenticatedSession> {
    let oauth = OAuthClientBuilder::new(
        &config.client_id,
        &config.redirect_uri,
        OAUTH_SCOPES.to_vec(),
    )
    .open_in_browser()
    .with_custom_message(OAUTH_SUCCESS_HTML)
    .build()?;

    let token = oauth.get_access_token_async().await?;

    Ok(AuthenticatedSession {
        access_token: token.access_token,
        refresh_token: SecretString::from(token.refresh_token),
    })
}

pub async fn refresh(
    config: &SpotifyConfig,
    refresh_token: &SecretString,
) -> Result<AuthenticatedSession> {
    let oauth = OAuthClientBuilder::new(
        &config.client_id,
        &config.redirect_uri,
        OAUTH_SCOPES.to_vec(),
    )
    .build()?;
    let token = oauth
        .refresh_token_async(refresh_token.expose_secret())
        .await?;
    let refresh_token = if token.refresh_token.is_empty() {
        refresh_token.clone()
    } else {
        SecretString::from(token.refresh_token)
    };

    Ok(AuthenticatedSession {
        access_token: token.access_token,
        refresh_token,
    })
}
