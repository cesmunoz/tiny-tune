use crate::domain::{AppError, PageCursor, TrackSummary};

use super::{
    PlaylistPageDto, TrackPageDto, classify_http_status, cursor_from_next_url,
    error_needs_token_refresh, map_transport_kind, parse_retry_after, playlist_tracks_url,
    project_playlist_page, project_track_page,
};

#[test]
fn deserializes_and_projects_playlist_page() {
    let page: PlaylistPageDto =
        serde_json::from_str(r#"{"items":[{"name":"Focus","uri":"spotify:playlist:123"}]}"#)
            .expect("valid playlist fixture");

    assert_eq!(
        project_playlist_page(page),
        vec![crate::domain::PlaylistSummary {
            name: "Focus".into(),
            uri: "spotify:playlist:123".into(),
        }]
    );
}

#[test]
fn projects_track_page_and_drops_unrelated_payload() {
    let raw = r#"{
        "accessToken": "super-secret-token",
        "offset": 0,
        "next": "https://api.spotify.com/v1/playlists/abc/tracks?offset=50&limit=50",
        "items": [
            {
                "is_local": false,
                "track": {
                    "id": "track-id",
                    "uri": "spotify:track:aaaaaaaaaaaaaaaaaaaaaa",
                    "name": "Night Owl",
                    "duration_ms": 180000,
                    "artists": [{"name": "Ada"}, {"name": ""}, {"name": "Bea"}],
                    "album": {
                        "name": "Hours",
                        "images": [{"url": "https://i.scdn.co/image/signed-secret"}]
                    },
                    "type": "track"
                }
            },
            {"is_local": false, "track": null},
            {
                "is_local": true,
                "track": {
                    "uri": "spotify:local:demo",
                    "name": "Desk Recording",
                    "duration_ms": 1000,
                    "artists": [{"name": "Local"}],
                    "album": {"name": "Files"},
                    "type": "track"
                }
            },
            {
                "is_local": false,
                "track": {
                    "uri": "spotify:episode:bbbbbbbbbbbbbbbbbbbbbb",
                    "name": "Show notes",
                    "duration_ms": 5000,
                    "type": "episode",
                    "artists": [{"name": "Host"}]
                }
            },
            {
                "is_local": false,
                "track": {
                    "uri": "spotify:track:cccccccccccccccccccccc",
                    "name": "Hidden",
                    "duration_ms": 2000,
                    "is_playable": false,
                    "artists": [{"name": "Cee"}],
                    "album": {"name": "No"}
                }
            }
        ]
    }"#;
    let page: TrackPageDto = serde_json::from_str(raw).expect("valid track fixture");
    let projected = project_track_page(page);
    let debug = format!("{projected:?}");

    assert!(!debug.contains("super-secret-token"));
    assert!(!debug.contains("signed-secret"));
    assert_eq!(projected.items.len(), 5);
    assert_eq!(projected.next.map(PageCursor::offset), Some(50));
    assert_eq!(
        projected.items[0],
        TrackSummary {
            id: Some(crate::domain::TrackId::new("aaaaaaaaaaaaaaaaaaaaaa")),
            spotify_uri: Some("spotify:track:aaaaaaaaaaaaaaaaaaaaaa".into()),
            name: "Night Owl".into(),
            artist_names: vec!["Ada".into(), "Bea".into()],
            album_name: "Hours".into(),
            duration_ms: 180_000,
            image: None,
            is_playable: true,
        }
    );
    assert!(!projected.items[1].is_playable);
    assert_eq!(projected.items[1].name, "Unavailable");
    assert!(!projected.items[2].is_playable);
    assert!(projected.items[2].spotify_uri.is_none());
    assert!(!projected.items[3].is_playable);
    assert!(!projected.items[4].is_playable);
    assert_eq!(
        projected.items[4].spotify_uri.as_deref(),
        Some("spotify:track:cccccccccccccccccccccc")
    );
}

#[test]
fn caps_a_track_page_at_fifty_items() {
    let items = (0..51)
        .map(|index| {
            format!(
                r#"{{"track":{{"uri":"spotify:track:{index:022}","name":"Row","duration_ms":1,"artists":[]}}}}"#
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let page: TrackPageDto =
        serde_json::from_str(&format!(r#"{{"offset":0,"items":[{items}]}}"#)).expect("page");
    let projected = project_track_page(page);

    assert_eq!(projected.items.len(), 50);
    assert_eq!(projected.next.map(PageCursor::offset), Some(50));
}

#[test]
fn ignores_next_links_outside_the_spotify_api() {
    assert_eq!(
        cursor_from_next_url("https://evil.example/tracks?offset=50"),
        None
    );
    assert_eq!(
        cursor_from_next_url("http://api.spotify.com/v1/playlists/abc/tracks?offset=50"),
        None
    );
    assert_eq!(
        cursor_from_next_url("https://api.spotify.com/v1/playlists/abc/tracks?limit=50&offset=100")
            .map(PageCursor::offset),
        Some(100)
    );

    let page: TrackPageDto = serde_json::from_str(
        r#"{"offset":0,"next":"https://evil.example/?offset=50","items":[{"track":{"uri":"spotify:track:aaaaaaaaaaaaaaaaaaaaaa","name":"One"}}]}"#,
    )
    .expect("page");
    let projected = project_track_page(page);
    assert_eq!(projected.next.map(PageCursor::offset), Some(1));
    assert!(!format!("{projected:?}").contains("evil.example"));
}

#[test]
fn builds_a_bounded_tracks_url_from_a_playlist_uri() {
    let cursor = PageCursor::from_offset(50);
    let url =
        playlist_tracks_url("spotify:playlist:abc123", cursor.as_ref()).expect("playlist uri");

    assert_eq!(
        url,
        "https://api.spotify.com/v1/playlists/abc123/tracks?limit=50&offset=50"
    );
    assert!(playlist_tracks_url("spotify:album:abc", None).is_err());
    assert!(playlist_tracks_url("spotify:playlist:../admin", None).is_err());
}

#[test]
fn maps_catalog_status_codes_and_retry_after() {
    assert!(classify_http_status(200, None).is_ok());
    assert_eq!(
        classify_http_status(401, None),
        Err(AppError::ReauthenticationRequired)
    );
    assert!(error_needs_token_refresh(
        &classify_http_status(401, None).expect_err("401")
    ));
    assert_eq!(
        classify_http_status(403, None),
        Err(AppError::PlaylistUnavailable)
    );
    assert_eq!(
        classify_http_status(404, None),
        Err(AppError::PlaylistUnavailable)
    );
    assert_eq!(
        classify_http_status(429, Some("12")),
        Err(AppError::RateLimited {
            retry_after_seconds: 12
        })
    );
    assert!(!error_needs_token_refresh(
        &classify_http_status(429, Some("12")).expect_err("429")
    ));
    assert_eq!(classify_http_status(503, None), Err(AppError::Offline));
    assert_eq!(classify_http_status(418, None), Err(AppError::Unexpected));
    assert_eq!(parse_retry_after(None), 1);
    assert_eq!(parse_retry_after(Some("0")), 1);
    assert_eq!(parse_retry_after(Some(" 8 ")), 8);
    assert_eq!(parse_retry_after(Some("Wed, 21 Oct 2015 07:28:00 GMT")), 1);
    assert_eq!(parse_retry_after(Some("999999999")), 24 * 60 * 60);
    assert_eq!(map_transport_kind(true), AppError::Offline);
    assert_eq!(map_transport_kind(false), AppError::Unexpected);
}
