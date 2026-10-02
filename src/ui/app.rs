use std::time::Instant;

use gpui::Context;
use tracing::{info, warn};

use crate::{
    diagnostics::process_memory_mb,
    domain::PlaylistSummary,
    spotify::{self, SpotifyCommand, SpotifyEvent, SpotifyHandle},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ViewState {
    SignedOut,
    Authenticating,
    Connecting,
    Ready,
    Loading,
    Playing,
    Paused,
    Failed,
}

pub struct TinyTune {
    pub(super) backend: SpotifyHandle,
    pub(super) state: ViewState,
    pub(super) status: String,
    pub(super) track_uri: Option<String>,
    pub(super) playlists: Vec<PlaylistSummary>,
    pub(super) startup_ms: u128,
    pub(super) memory_mb: Option<u64>,
}

impl TinyTune {
    pub fn new(cx: &mut Context<Self>, started_at: Instant) -> Self {
        let (backend, events) = spotify::spawn();

        cx.spawn(async move |this, cx| {
            while let Ok(event) = events.recv().await {
                if this
                    .update(cx, |app, cx| {
                        app.apply_event(event);
                        cx.notify();
                    })
                    .is_err()
                {
                    break;
                }
            }
        })
        .detach();

        let startup_ms = started_at.elapsed().as_millis();
        let memory_mb = process_memory_mb();
        info!(startup_ms, ?memory_mb, "Tiny Tune window created");

        Self {
            backend,
            state: ViewState::SignedOut,
            status: "Ready to connect".into(),
            track_uri: None,
            playlists: Vec::new(),
            startup_ms,
            memory_mb,
        }
    }

    pub(super) fn send(&mut self, command: SpotifyCommand, cx: &mut Context<Self>) {
        if self.backend.try_send(command) {
            return;
        }

        self.state = ViewState::Failed;
        self.status = "Spotify backend is busy".into();
        cx.notify();
    }

    pub(super) fn connected(&self) -> bool {
        matches!(
            self.state,
            ViewState::Ready | ViewState::Loading | ViewState::Playing | ViewState::Paused
        )
    }

    fn apply_event(&mut self, event: SpotifyEvent) {
        match event {
            SpotifyEvent::Authenticating => {
                self.state = ViewState::Authenticating;
                self.status = "Finish signing in in your browser".into();
            }
            SpotifyEvent::Connecting => {
                self.state = ViewState::Connecting;
                self.status = "Starting the lightweight player".into();
            }
            SpotifyEvent::Ready => {
                self.state = ViewState::Ready;
                self.status = "Connected as Tiny Tune".into();
            }
            SpotifyEvent::PlaylistsLoaded(playlists) => {
                let count = playlists.len();
                self.playlists = playlists;
                self.status = format!("{count} playlists ready");
            }
            SpotifyEvent::Loading => {
                self.state = ViewState::Loading;
                self.status = "Loading your music".into();
            }
            SpotifyEvent::Playing { track_uri } => {
                self.state = ViewState::Playing;
                self.status = "Playing".into();
                self.track_uri = Some(track_uri);
            }
            SpotifyEvent::Paused { track_uri } => {
                self.state = ViewState::Paused;
                self.status = "Paused".into();
                self.track_uri = Some(track_uri);
            }
            SpotifyEvent::Stopped => {
                self.state = ViewState::Ready;
                self.status = "Stopped".into();
                self.track_uri = None;
            }
            SpotifyEvent::Notice { message } => {
                self.status = message;
            }
            SpotifyEvent::Error { message } => {
                self.state = ViewState::Failed;
                self.status = message;
            }
        }

        self.memory_mb = process_memory_mb();
    }
}

impl Drop for TinyTune {
    fn drop(&mut self) {
        if self.backend.try_send(SpotifyCommand::Shutdown) {
            return;
        }

        warn!("Spotify backend already stopped");
    }
}
