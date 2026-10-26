use crate::{
    domain::{
        AppError, ImageRef, NowPlaying, Page, PageCursor, PlaybackState, PlaylistSummary, TrackId,
        TrackSummary,
    },
    spotify::{SpotifyCommand, SpotifyEvent},
    ui::state::{AppState, TrackPagePhase, ViewState},
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
    assert!(state.selected_playlist.is_none());
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
fn empty_library_finishes_loading_even_while_playback_changes() {
    let mut state = AppState::default();
    state.apply(SpotifyEvent::Ready);
    state.apply(SpotifyEvent::PlaylistsLoading);
    assert_eq!(state.library_placeholder(), "Loading your playlists…");

    state.apply(SpotifyEvent::PlaylistsLoaded(vec![]));
    state.apply(SpotifyEvent::PlaybackChanged(PlaybackState::Playing {
        now_playing: now_playing("Track", &["Artist"]),
    }));
    assert!(state.connected());
    assert_eq!(state.library_placeholder(), "No playlists in your library");

    state.apply(SpotifyEvent::SignedOut);
    assert_eq!(state.library_placeholder(), "Connect to see your playlists");
}

#[test]
fn catalog_failures_end_loading_and_only_a_new_request_restarts_it() {
    for error in [
        AppError::PlaylistUnavailable,
        AppError::Offline,
        AppError::RateLimited {
            retry_after_seconds: 3,
        },
    ] {
        let mut state = AppState::default();
        state.apply(SpotifyEvent::Ready);
        state.apply(SpotifyEvent::PlaylistsLoading);
        state.apply(SpotifyEvent::PlaylistsFailed(error.clone()));
        assert_eq!(state.status, error.user_message());
        assert_eq!(
            state.library_placeholder(),
            "Your playlists could not be loaded"
        );

        state.apply(SpotifyEvent::RecoverableError(AppError::TrackUnavailable));
        assert_eq!(
            state.library_placeholder(),
            "Your playlists could not be loaded"
        );

        state.apply(SpotifyEvent::PlaylistsLoading);
        assert_eq!(state.library_placeholder(), "Loading your playlists…");
        state.apply(SpotifyEvent::PlaylistsLoaded(vec![]));
        assert_eq!(state.library_placeholder(), "No playlists in your library");
    }
}

#[test]
fn unrelated_errors_do_not_end_a_library_request() {
    let mut state = AppState::default();
    state.apply(SpotifyEvent::Ready);
    state.apply(SpotifyEvent::PlaylistsLoading);
    state.apply(SpotifyEvent::RecoverableError(
        AppError::CredentialStoreUnavailable,
    ));
    assert_eq!(state.library_placeholder(), "Loading your playlists…");

    state.apply(SpotifyEvent::FatalError(AppError::BackendUnavailable));
    assert_eq!(
        state.library_placeholder(),
        "Your playlists could not be loaded"
    );
    state.apply(SpotifyEvent::FatalError(AppError::ReauthenticationRequired));
    assert_eq!(state.library_placeholder(), "Connect to see your playlists");
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
    assert!(state.selected_playlist.is_none());
    assert!(!state.connected());
}

fn playlist(uri: &str) -> PlaylistSummary {
    PlaylistSummary {
        name: "Focus".into(),
        uri: uri.into(),
    }
}

fn summary(uri: &str, playable: bool) -> TrackSummary {
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

fn loaded(
    uri: &str,
    generation: u64,
    request_cursor: Option<PageCursor>,
    tracks: Vec<TrackSummary>,
    next: Option<PageCursor>,
) -> SpotifyEvent {
    SpotifyEvent::PlaylistTracksLoaded {
        playlist_uri: uri.into(),
        generation,
        request_cursor,
        tracks: Page {
            items: tracks,
            next,
        },
    }
}

#[test]
fn merges_track_pages_for_the_open_playlist_only() {
    let mut state = AppState::default();
    let playlist_uri = "spotify:playlist:focus";
    let open = state.open_playlist(playlist(playlist_uri));
    assert_eq!(
        open,
        SpotifyCommand::OpenPlaylist {
            uri: playlist_uri.into(),
            cursor: None,
            generation: 1,
        }
    );

    let first = summary("spotify:track:aaaaaaaaaaaaaaaaaaaaaa", true);
    let second = summary("spotify:track:bbbbbbbbbbbbbbbbbbbbbb", true);
    let third = summary("spotify:track:cccccccccccccccccccccc", false);
    let next = PageCursor::from_offset(2);
    state.apply(loaded(
        playlist_uri,
        1,
        None,
        vec![first.clone(), second.clone()],
        next,
    ));

    let detail = state.selected_playlist.as_ref().expect("open playlist");
    assert_eq!(detail.phase, TrackPagePhase::Ready);
    assert_eq!(detail.tracks.len(), 2);
    assert!(!detail.awaiting);

    let load_next = state.load_next_tracks().expect("next page");
    assert_eq!(
        load_next,
        SpotifyCommand::OpenPlaylist {
            uri: playlist_uri.into(),
            cursor: next,
            generation: 1,
        }
    );
    assert!(state.load_next_tracks().is_none());

    state.apply(loaded(
        playlist_uri,
        1,
        next,
        vec![second.clone(), third.clone()],
        None,
    ));
    let detail = state.selected_playlist.as_ref().expect("open playlist");
    assert_eq!(detail.tracks.len(), 3);
    assert_eq!(detail.tracks[2].spotify_uri, third.spotify_uri);
    assert!(detail.next.is_none());
    assert_eq!(
        match state.play_track(0).expect("playable") {
            SpotifyCommand::PlayPlaylist {
                start_track_uri, ..
            } => start_track_uri,
            other => panic!("unexpected command {other:?}"),
        },
        first.spotify_uri
    );
    assert!(state.play_track(2).is_none());

    state.apply(loaded(
        playlist_uri,
        1,
        next,
        vec![summary("spotify:track:dddddddddddddddddddddd", true)],
        None,
    ));
    assert_eq!(
        state.selected_playlist.as_ref().expect("open").tracks.len(),
        3
    );
}

#[test]
fn discards_stale_playlist_track_responses() {
    let mut state = AppState::default();
    let first = "spotify:playlist:one";
    let second = "spotify:playlist:two";
    state.open_playlist(playlist(first));
    state.open_playlist(playlist(second));

    state.apply(loaded(
        first,
        1,
        None,
        vec![summary("spotify:track:aaaaaaaaaaaaaaaaaaaaaa", true)],
        None,
    ));
    assert_eq!(
        state
            .selected_playlist
            .as_ref()
            .expect("second playlist")
            .tracks,
        []
    );

    state.apply(loaded(
        second,
        2,
        None,
        vec![summary("spotify:track:bbbbbbbbbbbbbbbbbbbbbb", true)],
        None,
    ));
    assert_eq!(
        state
            .selected_playlist
            .as_ref()
            .expect("second playlist")
            .tracks
            .len(),
        1
    );

    state.close_playlist();
    state.apply(loaded(
        second,
        2,
        None,
        vec![summary("spotify:track:cccccccccccccccccccccc", true)],
        None,
    ));
    assert!(state.selected_playlist.is_none());
}

#[test]
fn retries_a_failed_track_page_without_dropping_loaded_rows() {
    let mut state = AppState::default();
    let playlist_uri = "spotify:playlist:focus";
    state.apply(SpotifyEvent::PlaybackChanged(PlaybackState::Playing {
        now_playing: now_playing("Night Owl", &["Ada"]),
    }));
    state.open_playlist(playlist(playlist_uri));
    state.apply(SpotifyEvent::PlaylistTracksFailed {
        playlist_uri: playlist_uri.into(),
        generation: 1,
        request_cursor: None,
        error: AppError::Offline,
    });
    assert_eq!(state.view, ViewState::Playing);
    assert_eq!(state.status, "Playing");
    assert_eq!(
        state.selected_playlist.as_ref().expect("open").phase,
        TrackPagePhase::Failed(AppError::Offline)
    );

    let retry = state.retry_tracks().expect("retry");
    assert_eq!(
        retry,
        SpotifyCommand::OpenPlaylist {
            uri: playlist_uri.into(),
            cursor: None,
            generation: 1,
        }
    );

    let next = PageCursor::from_offset(1);
    state.apply(loaded(
        playlist_uri,
        1,
        None,
        vec![summary("spotify:track:aaaaaaaaaaaaaaaaaaaaaa", true)],
        next,
    ));
    state.load_next_tracks().expect("next");
    state.apply(SpotifyEvent::PlaylistTracksFailed {
        playlist_uri: playlist_uri.into(),
        generation: 1,
        request_cursor: next,
        error: AppError::RateLimited {
            retry_after_seconds: 3,
        },
    });

    let detail = state.selected_playlist.as_ref().expect("open");
    assert_eq!(detail.tracks.len(), 1);
    assert_eq!(
        detail.phase,
        TrackPagePhase::Failed(AppError::RateLimited {
            retry_after_seconds: 3
        })
    );
    let retry_next = state.retry_tracks().expect("retry next");
    assert_eq!(
        retry_next,
        SpotifyCommand::OpenPlaylist {
            uri: playlist_uri.into(),
            cursor: next,
            generation: 1,
        }
    );
}

#[test]
fn titles_the_stage_with_the_open_playlist() {
    let mut state = AppState::default();
    assert_eq!(state.selected_playlist_name(), None);

    let _ = state.open_playlist(playlist("spotify:playlist:focus"));
    assert_eq!(state.selected_playlist_name(), Some("Focus"));

    state.close_playlist();
    assert_eq!(state.selected_playlist_name(), None);
}
