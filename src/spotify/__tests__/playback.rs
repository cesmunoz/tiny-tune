use librespot::core::spotify_uri::SpotifyUri;
use librespot::playback::player::PlayerEvent;

use crate::domain::{AppError, PlaybackState};

use super::{PlaybackProjector, PlayerProjection, volume_level, volume_percent};

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
        playback.apply(PlayerEvent::PlayRequestIdChanged { play_request_id: 7 }),
        PlayerProjection::Ignore
    ));
}

#[test]
fn projects_volume_changes_as_percent() {
    let mut playback = PlaybackProjector::default();

    for (volume, percent) in [(0, 0), (u16::MAX / 2, 50), (u16::MAX, 100)] {
        assert!(matches!(
            playback.apply(PlayerEvent::VolumeChanged { volume }),
            PlayerProjection::Volume(projected) if projected == percent
        ));
    }
}

#[test]
fn volume_percent_round_trips_through_librespot_levels() {
    for percent in 0..=100 {
        assert_eq!(volume_percent(volume_level(percent)), percent);
    }
    assert_eq!(volume_level(0), 0);
    assert_eq!(volume_level(100), u16::MAX);
    assert_eq!(volume_level(250), u16::MAX);
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
