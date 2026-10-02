use librespot::core::spotify_uri::SpotifyUri;
use librespot::playback::player::PlayerEvent;

use crate::domain::AppError;

use super::{SpotifyEvent, project_player_event};

#[test]
fn ignores_non_ui_player_events() {
    assert_eq!(
        project_player_event(PlayerEvent::VolumeChanged { volume: 42 }),
        None
    );
}

#[test]
fn projects_playing_event_without_leaking_backend_types() {
    let track_id =
        SpotifyUri::from_uri("spotify:track:4iV5W9uYEdYUVa79Axb7Rh").expect("valid Spotify URI");

    assert_eq!(
        project_player_event(PlayerEvent::Playing {
            play_request_id: 1,
            track_id,
            position_ms: 0,
        }),
        Some(SpotifyEvent::Playing {
            track_uri: "spotify:track:4iV5W9uYEdYUVa79Axb7Rh".into(),
        })
    );
}

#[test]
fn projects_unavailable_track_as_an_application_error() {
    let track_id =
        SpotifyUri::from_uri("spotify:track:4iV5W9uYEdYUVa79Axb7Rh").expect("valid Spotify URI");

    assert_eq!(
        project_player_event(PlayerEvent::Unavailable {
            play_request_id: 1,
            track_id,
        }),
        Some(SpotifyEvent::RecoverableError(AppError::TrackUnavailable))
    );
}
