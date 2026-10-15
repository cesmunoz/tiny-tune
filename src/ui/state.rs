use crate::{
    domain::{AppError, PlaybackState, PlaylistSummary},
    spotify::SpotifyEvent,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ViewState {
    SignedOut,
    RestoringSession,
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
    pub playback: PlaybackState,
    pub playlists: Vec<PlaylistSummary>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            view: ViewState::SignedOut,
            status: "Ready to connect".into(),
            playback: PlaybackState::Idle,
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

    pub fn now_playing(&self) -> Option<&crate::domain::NowPlaying> {
        self.playback.now_playing()
    }

    pub fn current_image_url(&self) -> Option<&str> {
        self.now_playing()
            .and_then(|now_playing| now_playing.track.image.as_ref())
            .map(|image| image.url.as_str())
    }

    pub fn apply(&mut self, event: SpotifyEvent) {
        match event {
            SpotifyEvent::RestoringSession => {
                self.view = ViewState::RestoringSession;
                self.status = "Restoring your Spotify session".into();
            }
            SpotifyEvent::SignedOut => *self = Self::default(),
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
            SpotifyEvent::PlaybackChanged(playback) => self.apply_playback(playback),
            SpotifyEvent::ArtworkReady(_) | SpotifyEvent::ArtworkFailed { .. } => {}
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
            self.playback = PlaybackState::Idle;
            self.playlists.clear();
        } else if fatal {
            self.view = ViewState::Failed;
        }
    }

    fn apply_playback(&mut self, playback: PlaybackState) {
        self.playback = playback;
        match &self.playback {
            PlaybackState::Idle => {
                if self.connected() || self.view == ViewState::Loading {
                    self.view = ViewState::Ready;
                }
                self.status = "Stopped".into();
            }
            PlaybackState::Loading { .. } => {
                self.view = ViewState::Loading;
                self.status = "Loading your music".into();
            }
            PlaybackState::Playing { .. } => {
                self.view = ViewState::Playing;
                self.status = "Playing".into();
            }
            PlaybackState::Paused { .. } => {
                self.view = ViewState::Paused;
                self.status = "Paused".into();
            }
        }
    }
}

#[cfg(test)]
#[path = "__tests__/state.rs"]
mod tests;
