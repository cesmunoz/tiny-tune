use std::time::Duration;

use serde::Deserialize;

use crate::domain::{AppError, Page, PageCursor, PlaylistSummary, TrackId, TrackSummary};

const CURRENT_USER_PLAYLISTS_URL: &str = "https://api.spotify.com/v1/me/playlists";
const PLAYLIST_TRACKS_URL: &str = "https://api.spotify.com/v1/playlists";
const PAGE_LIMIT: usize = PageCursor::PAGE_SIZE;
const MAX_RETRY_AFTER_SECONDS: u64 = 24 * 60 * 60;

#[derive(Deserialize)]
struct PlaylistPageDto {
    #[serde(default)]
    items: Vec<PlaylistDto>,
}

#[derive(Deserialize)]
struct PlaylistDto {
    name: String,
    uri: String,
}

#[derive(Deserialize)]
struct TrackPageDto {
    #[serde(default)]
    items: Vec<PlaylistItemDto>,
    next: Option<String>,
    #[serde(default)]
    offset: u32,
}

#[derive(Deserialize)]
struct PlaylistItemDto {
    #[serde(default)]
    is_local: bool,
    track: Option<TrackDto>,
}

#[derive(Deserialize)]
struct TrackDto {
    uri: Option<String>,
    name: Option<String>,
    #[serde(default)]
    duration_ms: u64,
    is_playable: Option<bool>,
    #[serde(rename = "type")]
    kind: Option<String>,
    #[serde(default)]
    artists: Vec<ArtistDto>,
    album: Option<AlbumDto>,
}

#[derive(Deserialize)]
struct ArtistDto {
    name: Option<String>,
}

#[derive(Deserialize)]
struct AlbumDto {
    name: Option<String>,
}

pub(crate) fn http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}

pub(crate) fn error_needs_token_refresh(error: &AppError) -> bool {
    matches!(error, AppError::ReauthenticationRequired)
}

pub async fn fetch_playlists(
    http: &reqwest::Client,
    access_token: &str,
) -> Result<Vec<PlaylistSummary>, AppError> {
    let response = http
        .get(CURRENT_USER_PLAYLISTS_URL)
        .query(&[("limit", PAGE_LIMIT.to_string())])
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|error| map_transport_error(&error))?;
    let page = read_json::<PlaylistPageDto>(response).await?;
    Ok(project_playlist_page(page))
}

pub(crate) async fn fetch_playlist_tracks(
    http: &reqwest::Client,
    access_token: &str,
    playlist_uri: &str,
    cursor: Option<&PageCursor>,
) -> Result<Page<TrackSummary>, AppError> {
    let url = playlist_tracks_url(playlist_uri, cursor)?;
    let response = http
        .get(url)
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|error| map_transport_error(&error))?;
    let page = read_json::<TrackPageDto>(response).await?;
    Ok(project_track_page(page))
}

pub(crate) fn playlist_tracks_url(
    playlist_uri: &str,
    cursor: Option<&PageCursor>,
) -> Result<String, AppError> {
    let Some(id) = playlist_uri.strip_prefix("spotify:playlist:") else {
        return Err(AppError::PlaylistUnavailable);
    };
    if !crate::domain::is_spotify_playlist_uri(playlist_uri) {
        return Err(AppError::PlaylistUnavailable);
    }
    let offset = cursor.map_or(0, |cursor| cursor.offset());
    Ok(format!(
        "{PLAYLIST_TRACKS_URL}/{id}/tracks?limit={PAGE_LIMIT}&offset={offset}"
    ))
}

pub(crate) fn classify_http_status(status: u16, retry_after: Option<&str>) -> Result<(), AppError> {
    match status {
        200..=299 => Ok(()),
        401 => Err(AppError::ReauthenticationRequired),
        403 | 404 => Err(AppError::PlaylistUnavailable),
        429 => Err(AppError::RateLimited {
            retry_after_seconds: parse_retry_after(retry_after),
        }),
        500..=599 => Err(AppError::Offline),
        _ => Err(AppError::Unexpected),
    }
}

pub(crate) fn parse_retry_after(header: Option<&str>) -> u64 {
    header
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(1)
        .clamp(1, MAX_RETRY_AFTER_SECONDS)
}

pub(crate) fn map_transport_kind(offline: bool) -> AppError {
    if offline {
        AppError::Offline
    } else {
        AppError::Unexpected
    }
}

fn map_transport_error(error: &reqwest::Error) -> AppError {
    map_transport_kind(error.is_connect() || error.is_timeout())
}

fn project_playlist_page(page: PlaylistPageDto) -> Vec<PlaylistSummary> {
    page.items
        .into_iter()
        .take(PAGE_LIMIT)
        .map(|playlist| PlaylistSummary {
            name: playlist.name,
            uri: playlist.uri,
        })
        .collect()
}

fn project_track_page(page: TrackPageDto) -> Page<TrackSummary> {
    let offset = page.offset;
    let raw_len = page.items.len();
    let items: Vec<TrackSummary> = page
        .items
        .into_iter()
        .take(PAGE_LIMIT)
        .map(project_playlist_item)
        .collect();
    let next = next_cursor(page.next.as_deref(), offset, items.len(), raw_len);
    Page { items, next }
}

fn project_playlist_item(item: PlaylistItemDto) -> TrackSummary {
    let Some(track) = item.track else {
        return TrackSummary::unavailable();
    };
    let spotify_uri = track
        .uri
        .filter(|uri| crate::domain::is_spotify_track_uri(uri));
    let playable = !item.is_local
        && track.is_playable.unwrap_or(true)
        && track.kind.as_deref() != Some("episode")
        && spotify_uri.is_some();
    let name = track.name.filter(|name| !name.is_empty());
    TrackSummary {
        id: spotify_uri
            .as_deref()
            .and_then(|uri| uri.strip_prefix("spotify:track:"))
            .map(TrackId::new),
        spotify_uri,
        name: name.unwrap_or_else(|| "Unavailable".into()),
        artist_names: track
            .artists
            .into_iter()
            .filter_map(|artist| artist.name)
            .filter(|name| !name.is_empty())
            .collect(),
        album_name: track.album.and_then(|album| album.name).unwrap_or_default(),
        duration_ms: u32::try_from(track.duration_ms).unwrap_or(0),
        image: None,
        is_playable: playable,
    }
}

fn next_cursor(next: Option<&str>, offset: u32, kept: usize, raw_len: usize) -> Option<PageCursor> {
    if let Some(cursor) = next.and_then(cursor_from_next_url) {
        return Some(cursor);
    }
    if next.is_some() || raw_len > PAGE_LIMIT {
        let step = u32::try_from(kept).unwrap_or(50);
        return PageCursor::from_offset(offset.saturating_add(step));
    }
    None
}

fn cursor_from_next_url(next: &str) -> Option<PageCursor> {
    let rest = next.strip_prefix("https://api.spotify.com/")?;
    let query = rest.split_once('?')?.1.split('#').next()?;
    for pair in query.split('&') {
        let (key, value) = pair.split_once('=')?;
        if key == "offset" {
            return value.parse::<u32>().ok().and_then(PageCursor::from_offset);
        }
    }
    None
}

async fn read_json<T: serde::de::DeserializeOwned>(
    response: reqwest::Response,
) -> Result<T, AppError> {
    let status = response.status().as_u16();
    let retry_after = response
        .headers()
        .get(reqwest::header::RETRY_AFTER)
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    if let Err(error) = classify_http_status(status, retry_after.as_deref()) {
        let _ = response.bytes().await;
        return Err(error);
    }
    response.json::<T>().await.map_err(|_| AppError::Unexpected)
}

#[cfg(test)]
#[path = "__tests__/catalog.rs"]
mod tests;
