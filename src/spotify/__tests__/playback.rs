use std::time::Duration;

use librespot::core::spotify_uri::SpotifyUri;
use librespot::playback::player::PlayerEvent;

use crate::domain::{AppError, PlaybackState};

use super::{
    MAX_PLAYLIST_RETRIES, PlaybackProjector, PlayerProjection, SessionFollowUp, next_playlist_retry,
};

fn track_uri() -> SpotifyUri {
    SpotifyUri::from_uri("spotify:track:4iV5W9uYEdYUVa79Axb7Rh").expect("valid Spotify URI")
}

fn playing(position_ms: u32) -> PlayerEvent {
    PlayerEvent::Playing {
        play_request_id: 1,
        track_id: track_uri(),
        position_ms,
    }
}

#[test]
fn ignores_non_ui_player_events() {
    let mut playback = PlaybackProjector::default();

    assert!(matches!(
        playback.apply(PlayerEvent::VolumeChanged { volume: 42 }),
        PlayerProjection::Ignore
    ));
}

#[test]
fn projects_playing_event_without_leaking_backend_types() {
    let mut playback = PlaybackProjector::default();

    let PlayerProjection::Playback(PlaybackState::Playing { now_playing }) =
        playback.apply(playing(1_500))
    else {
        panic!("playing should publish now-playing state");
    };

    assert_eq!(
        now_playing.track.spotify_uri.as_deref(),
        Some("spotify:track:4iV5W9uYEdYUVa79Axb7Rh")
    );
    assert_eq!(now_playing.position_ms, 1_500);
    assert_eq!(now_playing.track.name, "");
}

#[test]
fn coalesces_repeated_position_updates_for_the_same_track() {
    let mut playback = PlaybackProjector::default();
    assert!(matches!(
        playback.apply(playing(0)),
        PlayerProjection::Playback(_)
    ));

    assert!(matches!(
        playback.apply(PlayerEvent::PositionChanged {
            play_request_id: 1,
            track_id: track_uri(),
            position_ms: 1_000,
        }),
        PlayerProjection::Ignore
    ));
    assert!(matches!(
        playback.apply(playing(2_000)),
        PlayerProjection::Ignore
    ));
    assert_eq!(playback.position_ms, 2_000);
}

#[test]
fn projects_unavailable_track_as_an_application_error() {
    let mut playback = PlaybackProjector::default();

    assert!(matches!(
        playback.apply(PlayerEvent::Unavailable {
            play_request_id: 1,
            track_id: track_uri(),
        }),
        PlayerProjection::Error(AppError::TrackUnavailable)
    ));
}

#[test]
fn stopped_clears_now_playing() {
    let mut playback = PlaybackProjector::default();
    playback.apply(playing(0));

    assert!(matches!(
        playback.apply(PlayerEvent::Stopped {
            play_request_id: 1,
            track_id: track_uri(),
        }),
        PlayerProjection::Playback(PlaybackState::Idle)
    ));
    assert_eq!(playback.summary, None);
    assert_eq!(playback.track_uri, None);
}

#[test]
fn retries_rate_limited_playlists_a_bounded_number_of_times() {
    let delay = Duration::from_secs(51);
    let rate_limited = SessionFollowUp::RetryAfter(delay);

    assert_eq!(next_playlist_retry(rate_limited, 0), Some(delay));
    assert_eq!(
        next_playlist_retry(rate_limited, MAX_PLAYLIST_RETRIES - 1),
        Some(delay)
    );
    assert_eq!(
        next_playlist_retry(rate_limited, MAX_PLAYLIST_RETRIES),
        None
    );
    assert_eq!(next_playlist_retry(SessionFollowUp::Continue, 0), None);
    assert_eq!(
        next_playlist_retry(SessionFollowUp::Reauthenticate, 0),
        None
    );
}
