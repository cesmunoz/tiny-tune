use anyhow::Result;
use serde::Deserialize;

use crate::domain::PlaylistSummary;

const CURRENT_USER_PLAYLISTS_URL: &str = "https://api.spotify.com/v1/me/playlists";
const PAGE_LIMIT: &str = "50";

#[derive(Deserialize)]
struct PlaylistPageDto {
    items: Vec<PlaylistDto>,
}

#[derive(Deserialize)]
struct PlaylistDto {
    name: String,
    uri: String,
}

pub async fn fetch_playlists(access_token: &str) -> Result<Vec<PlaylistSummary>> {
    let page = reqwest::Client::new()
        .get(CURRENT_USER_PLAYLISTS_URL)
        .query(&[("limit", PAGE_LIMIT)])
        .bearer_auth(access_token)
        .send()
        .await?
        .error_for_status()?
        .json::<PlaylistPageDto>()
        .await?;

    Ok(page
        .items
        .into_iter()
        .map(|playlist| PlaylistSummary {
            name: playlist.name,
            uri: playlist.uri,
        })
        .collect())
}

#[cfg(test)]
#[path = "__tests__/catalog.rs"]
mod tests;
