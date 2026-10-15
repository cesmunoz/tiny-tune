use crate::domain::{
    PageCursor, TrackId, TrackSummary, is_spotify_context_uri, is_spotify_track_uri,
    merge_track_page,
};

fn track(uri: &str, playable: bool) -> TrackSummary {
    TrackSummary {
        id: Some(TrackId::new(uri.trim_start_matches("spotify:track:"))),
        spotify_uri: Some(uri.into()),
        name: "Track".into(),
        artist_names: vec!["Ada".into()],
        album_name: "Hours".into(),
        duration_ms: 1_000,
        image: None,
        is_playable: playable,
    }
}

#[test]
fn merges_pages_without_duplicate_uris() {
    let mut tracks = vec![track("spotify:track:aaaaaaaaaaaaaaaaaaaaaa", true)];
    merge_track_page(
        &mut tracks,
        vec![
            track("spotify:track:aaaaaaaaaaaaaaaaaaaaaa", true),
            track("spotify:track:bbbbbbbbbbbbbbbbbbbbbb", true),
            TrackSummary::unavailable(),
            TrackSummary::unavailable(),
        ],
        false,
    );

    assert_eq!(tracks.len(), 4);
    assert_eq!(
        tracks[1].spotify_uri.as_deref(),
        Some("spotify:track:bbbbbbbbbbbbbbbbbbbbbb")
    );
    assert!(tracks[2].spotify_uri.is_none());
    assert!(tracks[3].spotify_uri.is_none());
}

#[test]
fn replacing_a_page_drops_the_previous_playlist_rows() {
    let mut tracks = vec![track("spotify:track:aaaaaaaaaaaaaaaaaaaaaa", true)];
    merge_track_page(
        &mut tracks,
        vec![track("spotify:track:bbbbbbbbbbbbbbbbbbbbbb", true)],
        true,
    );

    assert_eq!(tracks.len(), 1);
    assert_eq!(
        tracks[0].spotify_uri.as_deref(),
        Some("spotify:track:bbbbbbbbbbbbbbbbbbbbbb")
    );
}

#[test]
fn merge_keeps_only_one_incoming_page() {
    let incoming = (0..60)
        .map(|index| track(&format!("spotify:track:{index:022}"), true))
        .collect();
    let mut tracks = Vec::new();
    merge_track_page(&mut tracks, incoming, true);

    assert_eq!(tracks.len(), PageCursor::PAGE_SIZE);
}

#[test]
fn accepts_playlist_and_track_uris_used_for_playback() {
    assert!(is_spotify_track_uri("spotify:track:4iV5W9uYEdYUVa79Axb7Rh"));
    assert!(!is_spotify_track_uri(
        "spotify:episode:4iV5W9uYEdYUVa79Axb7Rh"
    ));
    assert!(!is_spotify_track_uri("spotify:track:"));
    assert!(is_spotify_context_uri(
        "spotify:playlist:37i9dQZF1DXcBWIGoYBM5M"
    ));
    assert!(is_spotify_context_uri("spotify:user:ada:collection"));
    assert!(!is_spotify_context_uri(
        "https://open.spotify.com/playlist/abc"
    ));
    assert!(!is_spotify_context_uri("spotify:playlist:abc?market=US"));
}
