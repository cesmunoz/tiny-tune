use crate::{
    domain::{AppError, PlaylistSummary},
    spotify::SpotifyEvent,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ViewState {
    SignedOut,
    Authenticating,
    Connecting,
    Ready,
    ReauthRequired,
    Loading,
    Playing,
    Paused,
    Failed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct AppState {
    pub view: ViewState,
    pub status: String,
    pub track_uri: Option<String>,
    pub playlists: Vec<PlaylistSummary>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            view: ViewState::SignedOut,
            status: "Ready to connect".into(),
            track_uri: None,
            playlists: Vec::new(),
        }
    }
}

impl AppState {
    pub fn connected(&self) -> bool {
        matches!(
            self.view,
            ViewState::Ready | ViewState::Loading | ViewState::Playing | ViewState::Paused
        )
    }

    pub fn apply(&mut self, event: SpotifyEvent) {
        match event {
            SpotifyEvent::Authenticating => {
                self.view = ViewState::Authenticating;
                self.status = "Finish signing in in your browser".into();
            }
            SpotifyEvent::Connecting => {
                self.view = ViewState::Connecting;
                self.status = "Starting the lightweight player".into();
            }
            SpotifyEvent::Ready => {
                self.view = ViewState::Ready;
                self.status = "Connected as Tiny Tune".into();
            }
            SpotifyEvent::PlaylistsLoaded(playlists) => {
                let count = playlists.len();
                self.playlists = playlists;
                self.status = format!("{count} playlists ready");
            }
            SpotifyEvent::Loading => {
                self.view = ViewState::Loading;
                self.status = "Loading your music".into();
            }
            SpotifyEvent::Playing { track_uri } => {
                self.view = ViewState::Playing;
                self.status = "Playing".into();
                self.track_uri = Some(track_uri);
            }
            SpotifyEvent::Paused { track_uri } => {
                self.view = ViewState::Paused;
                self.status = "Paused".into();
                self.track_uri = Some(track_uri);
            }
            SpotifyEvent::Stopped => {
                self.view = ViewState::Ready;
                self.status = "Stopped".into();
                self.track_uri = None;
            }
            SpotifyEvent::RecoverableError(error) => self.apply_error(&error, false),
            SpotifyEvent::FatalError(error) => self.apply_error(&error, true),
        }
    }

    pub fn backend_busy(&mut self) {
        self.apply(SpotifyEvent::FatalError(AppError::BackendUnavailable));
    }

    fn apply_error(&mut self, error: &AppError, fatal: bool) {
        self.status = error.user_message();

        if matches!(error, AppError::ReauthenticationRequired) {
            self.view = ViewState::ReauthRequired;
            self.track_uri = None;
            self.playlists.clear();
        } else if fatal {
            self.view = ViewState::Failed;
        }
    }
}

#[cfg(test)]
#[path = "__tests__/state.rs"]
mod tests;
