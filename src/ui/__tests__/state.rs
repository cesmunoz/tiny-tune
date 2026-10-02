use crate::{
    domain::{AppError, PlaylistSummary},
    spotify::SpotifyEvent,
    ui::state::{AppState, ViewState},
};

#[test]
fn starts_signed_out_without_session_data() {
    let state = AppState::default();

    assert_eq!(state.view, ViewState::SignedOut);
    assert_eq!(state.status, "Ready to connect");
    assert_eq!(state.track_uri, None);
    assert_eq!(state.playlists, []);
    assert!(!state.connected());
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
    let track_uri = "spotify:track:example".to_owned();

    state.apply(SpotifyEvent::Playing {
        track_uri: track_uri.clone(),
    });
    assert_eq!(state.view, ViewState::Playing);
    assert_eq!(state.track_uri.as_deref(), Some(track_uri.as_str()));

    state.apply(SpotifyEvent::Paused {
        track_uri: track_uri.clone(),
    });
    assert_eq!(state.view, ViewState::Paused);
    assert_eq!(state.track_uri.as_deref(), Some(track_uri.as_str()));

    state.apply(SpotifyEvent::Stopped);
    assert_eq!(state.view, ViewState::Ready);
    assert_eq!(state.track_uri, None);
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
    state.apply(SpotifyEvent::Playing {
        track_uri: "spotify:track:example".into(),
    });

    state.apply(SpotifyEvent::FatalError(AppError::ReauthenticationRequired));

    assert_eq!(state.view, ViewState::ReauthRequired);
    assert_eq!(state.track_uri, None);
    assert_eq!(state.playlists, []);
    assert!(!state.connected());
}
