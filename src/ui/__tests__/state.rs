use crate::{
    domain::{
        AppError, ImageRef, NowPlaying, PlaybackState, PlaylistSummary, TrackId, TrackSummary,
    },
    spotify::SpotifyEvent,
    ui::state::{AppState, ViewState},
};

fn now_playing(name: &str, artists: &[&str]) -> NowPlaying {
    NowPlaying {
        track: TrackSummary {
            id: Some(TrackId::new("4iV5W9uYEdYUVa79Axb7Rh")),
            spotify_uri: Some("spotify:track:4iV5W9uYEdYUVa79Axb7Rh".into()),
            name: name.into(),
            artist_names: artists.iter().map(|artist| (*artist).to_owned()).collect(),
            album_name: "Hours".into(),
            duration_ms: 180_000,
            image: Some(ImageRef {
                url: "https://i.scdn.co/image/cover".into(),
                width: Some(160),
                height: Some(160),
            }),
            is_playable: true,
        },
        position_ms: 1_500,
    }
}

#[test]
fn starts_signed_out_without_session_data() {
    let state = AppState::default();

    assert_eq!(state.view, ViewState::SignedOut);
    assert_eq!(state.status, "Ready to connect");
    assert_eq!(state.playback, PlaybackState::Idle);
    assert!(state.now_playing().is_none());
    assert_eq!(state.playlists, []);
    assert!(!state.connected());
}

#[test]
fn restores_or_returns_to_signed_out_on_startup() {
    let mut state = AppState::default();

    state.apply(SpotifyEvent::RestoringSession);
    assert_eq!(state.view, ViewState::RestoringSession);
    assert!(!state.connected());

    state.apply(SpotifyEvent::SignedOut);
    assert_eq!(state, AppState::default());
}

#[test]
fn follows_authentication_and_connection_transitions() {
    let mut state = AppState::default();

    state.apply(SpotifyEvent::Authenticating);
    assert_eq!(state.view, ViewState::Authenticating);
    assert!(!state.connected());

    state.apply(SpotifyEvent::Connecting);
    assert_eq!(state.view, ViewState::Connecting);
    assert!(!state.connected());

    state.apply(SpotifyEvent::Ready);
    assert_eq!(state.view, ViewState::Ready);
    assert!(state.connected());
}

#[test]
fn stores_playlists_without_changing_the_ready_view() {
    let mut state = AppState::default();
    state.apply(SpotifyEvent::Ready);

    state.apply(SpotifyEvent::PlaylistsLoaded(vec![PlaylistSummary {
        name: "Focus".into(),
        uri: "spotify:playlist:focus".into(),
    }]));

    assert_eq!(state.view, ViewState::Ready);
    assert_eq!(state.status, "1 playlists ready");
    assert_eq!(state.playlists[0].name, "Focus");
}

#[test]
fn follows_playback_transitions_and_clears_stopped_track() {
    let mut state = AppState::default();
    let playing = now_playing("Night Owl", &["Ada", "Bea"]);

    state.apply(SpotifyEvent::PlaybackChanged(PlaybackState::Playing {
        now_playing: playing.clone(),
    }));
    assert_eq!(state.view, ViewState::Playing);
    assert_eq!(
        state.now_playing().map(|item| item.track.name.as_str()),
        Some("Night Owl")
    );
    assert_eq!(
        state
            .now_playing()
            .map(|item| item.track.artist_names.as_slice()),
        Some(["Ada".to_owned(), "Bea".to_owned()].as_slice())
    );
    assert_eq!(
        state.now_playing().map(|item| item.position_ms),
        Some(1_500)
    );
    assert_eq!(
        state.current_image_url(),
        Some("https://i.scdn.co/image/cover")
    );

    state.apply(SpotifyEvent::PlaybackChanged(PlaybackState::Paused {
        now_playing: playing,
    }));
    assert_eq!(state.view, ViewState::Paused);
    assert_eq!(
        state.now_playing().map(|item| item.track.name.as_str()),
        Some("Night Owl")
    );

    state.apply(SpotifyEvent::PlaybackChanged(PlaybackState::Idle));
    assert_eq!(state.view, ViewState::Ready);
    assert!(state.now_playing().is_none());
    assert_eq!(state.current_image_url(), None);
}

#[test]
fn reports_backend_failures_as_failed_state() {
    let mut state = AppState::default();

    state.backend_busy();

    assert_eq!(state.view, ViewState::Failed);
    assert_eq!(state.status, "Spotify backend is unavailable");
}

#[test]
fn keeps_current_view_for_recoverable_errors() {
    let mut state = AppState::default();
    state.apply(SpotifyEvent::Ready);

    state.apply(SpotifyEvent::RecoverableError(
        AppError::PlaylistUnavailable,
    ));

    assert_eq!(state.view, ViewState::Ready);
    assert_eq!(state.status, "Connected, but playlists could not be loaded");
}

#[test]
fn reauthentication_clears_session_data() {
    let mut state = AppState::default();
    state.apply(SpotifyEvent::PlaylistsLoaded(vec![PlaylistSummary {
        name: "Focus".into(),
        uri: "spotify:playlist:focus".into(),
    }]));
    state.apply(SpotifyEvent::PlaybackChanged(PlaybackState::Playing {
        now_playing: now_playing("Night Owl", &["Ada"]),
    }));

    state.apply(SpotifyEvent::FatalError(AppError::ReauthenticationRequired));

    assert_eq!(state.view, ViewState::ReauthRequired);
    assert!(state.now_playing().is_none());
    assert_eq!(state.playlists, []);
    assert!(!state.connected());
}
