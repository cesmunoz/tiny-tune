use anyhow::Result;
use librespot::oauth::OAuthClientBuilder;

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

pub async fn authenticate(config: &SpotifyConfig) -> Result<String> {
    let oauth = OAuthClientBuilder::new(
        &config.client_id,
        &config.redirect_uri,
        OAUTH_SCOPES.to_vec(),
    )
    .open_in_browser()
    .with_custom_message(OAUTH_SUCCESS_HTML)
    .build()?;

    Ok(oauth.get_access_token_async().await?.access_token)
}
