use std::time::Duration;

use librespot::core::spotify_uri::SpotifyUri;
use librespot::playback::player::PlayerEvent;

use crate::{
    domain::{AppError, PlaybackState},
    spotify::SpotifyEvent,
};

use super::{
    MAX_PLAYLIST_RETRIES, PlaybackProjector, PlayerProjection, PlaylistRetry, SessionFollowUp,
    next_playlist_retry, volume_level, volume_percent,
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

#[test]
fn retry_timer_drives_five_retries_then_publishes_terminal_status() {
    tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .expect("runtime")
        .block_on(async {
            let (events, receiver) = async_channel::bounded(1);
            let mut retry = PlaylistRetry::default();
            for attempt in 1..=MAX_PLAYLIST_RETRIES {
                retry
                    .schedule(SessionFollowUp::RetryAfter(Duration::ZERO), &events)
                    .await;
                assert_eq!(
                    receiver.recv().await.expect("scheduled event"),
                    SpotifyEvent::PlaylistsRetryScheduled {
                        attempt,
                        max_attempts: MAX_PLAYLIST_RETRIES,
                    }
                );
                // Exercise the same timer future selected by drive_player, without Spotify.
                tokio::time::timeout(Duration::from_secs(1), retry.wait())
                    .await
                    .expect("scheduled retry must wake the driver");
            }
            retry
                .schedule(SessionFollowUp::RetryAfter(Duration::ZERO), &events)
                .await;
            assert_eq!(
                receiver.recv().await.expect("terminal event"),
                SpotifyEvent::PlaylistsRetriesExhausted
            );
            assert!(retry.sleep.is_none());
            assert!(
                std::future::Future::poll(
                    std::pin::pin!(retry.wait()).as_mut(),
                    &mut std::task::Context::from_waker(std::task::Waker::noop()),
                )
                .is_pending()
            );
            assert!(receiver.try_recv().is_err());
        });
}

#[test]
fn retry_preserves_spotify_delay_and_cancels_after_success_or_reauthentication() {
    tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .expect("runtime")
        .block_on(async {
            let (events, receiver) = async_channel::bounded(1);
            for follow_up in [SessionFollowUp::Continue, SessionFollowUp::Reauthenticate] {
                let mut retry = PlaylistRetry::default();
                let earliest = tokio::time::Instant::now() + Duration::from_secs(44);
                retry
                    .schedule(
                        SessionFollowUp::RetryAfter(Duration::from_secs(44)),
                        &events,
                    )
                    .await;
                receiver.recv().await.expect("scheduled event");
                let deadline = retry.sleep.as_ref().expect("armed timer").deadline();
                assert!(deadline >= earliest);
                assert!(
                    std::future::Future::poll(
                        std::pin::pin!(retry.wait()).as_mut(),
                        &mut std::task::Context::from_waker(std::task::Waker::noop()),
                    )
                    .is_pending()
                );
                // Cancelling a select branch must retain the original deadline.
                assert_eq!(
                    retry.sleep.as_ref().expect("retained timer").deadline(),
                    deadline
                );
                retry.schedule(follow_up, &events).await;
                assert!(retry.sleep.is_none());
                assert!(receiver.try_recv().is_err());
            }
        });
}
