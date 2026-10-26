use crate::{
    domain::{
        AppError, Page, PageCursor, PlaybackState, PlaylistSummary, TrackSummary, merge_track_page,
    },
    spotify::{SpotifyCommand, SpotifyEvent},
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
pub(super) enum LibraryPhase {
    NotLoaded,
    Loading,
    Ready,
    Failed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum TrackPagePhase {
    Loading,
    Ready,
    Failed(AppError),
}

/// Tracks for the one open playlist. Other playlists stay summaries only.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct PlaylistDetail {
    pub summary: PlaylistSummary,
    pub generation: u64,
    pub tracks: Vec<TrackSummary>,
    pub next: Option<PageCursor>,
    pub phase: TrackPagePhase,
    pub pending_cursor: Option<PageCursor>,
    pub awaiting: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct AppState {
    pub view: ViewState,
    pub status: String,
    pub playback: PlaybackState,
    pub playlists: Vec<PlaylistSummary>,
    pub library_phase: LibraryPhase,
    pub selected_playlist: Option<PlaylistDetail>,
    playlist_generation: u64,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            view: ViewState::SignedOut,
            status: "Ready to connect".into(),
            playback: PlaybackState::Idle,
            playlists: Vec::new(),
            library_phase: LibraryPhase::NotLoaded,
            selected_playlist: None,
            playlist_generation: 0,
        }
    }
}

impl AppState {
    pub fn library_placeholder(&self) -> &'static str {
        match self.library_phase {
            LibraryPhase::NotLoaded => "Connect to see your playlists",
            LibraryPhase::Loading => "Loading your playlists…",
            LibraryPhase::Ready => "No playlists in your library",
            LibraryPhase::Failed => "Your playlists could not be loaded",
        }
    }

    pub fn connected(&self) -> bool {
        matches!(
            self.view,
            ViewState::Ready | ViewState::Loading | ViewState::Playing | ViewState::Paused
        )
    }

    pub fn now_playing(&self) -> Option<&crate::domain::NowPlaying> {
        self.playback.now_playing()
    }

    pub fn selected_playlist_name(&self) -> Option<&str> {
        self.selected_playlist
            .as_ref()
            .map(|detail| detail.summary.name.as_str())
    }

    pub fn current_image_url(&self) -> Option<&str> {
        self.now_playing()
            .and_then(|now_playing| now_playing.track.image.as_ref())
            .map(|image| image.url.as_str())
    }

    pub fn open_playlist(&mut self, playlist: PlaylistSummary) -> SpotifyCommand {
        self.playlist_generation = self.playlist_generation.wrapping_add(1);
        let generation = self.playlist_generation;
        let uri = playlist.uri.clone();
        self.selected_playlist = Some(PlaylistDetail {
            summary: playlist,
            generation,
            tracks: Vec::new(),
            next: None,
            phase: TrackPagePhase::Loading,
            pending_cursor: None,
            awaiting: true,
        });
        SpotifyCommand::OpenPlaylist {
            uri,
            cursor: None,
            generation,
        }
    }

    pub fn close_playlist(&mut self) {
        self.selected_playlist = None;
        self.playlist_generation = self.playlist_generation.wrapping_add(1);
    }

    pub fn load_next_tracks(&mut self) -> Option<SpotifyCommand> {
        let detail = self.selected_playlist.as_mut()?;
        if detail.awaiting || !matches!(detail.phase, TrackPagePhase::Ready) {
            return None;
        }
        let cursor = detail.next?;
        detail.pending_cursor = Some(cursor);
        detail.awaiting = true;
        detail.phase = TrackPagePhase::Loading;
        Some(tracks_command(detail))
    }

    pub fn retry_tracks(&mut self) -> Option<SpotifyCommand> {
        let detail = self.selected_playlist.as_mut()?;
        if detail.awaiting || !matches!(detail.phase, TrackPagePhase::Failed(_)) {
            return None;
        }
        detail.awaiting = true;
        detail.phase = TrackPagePhase::Loading;
        Some(tracks_command(detail))
    }

    pub fn play_track(&self, index: usize) -> Option<SpotifyCommand> {
        let detail = self.selected_playlist.as_ref()?;
        let track = detail.tracks.get(index)?;
        let uri = track.spotify_uri.as_deref()?;
        if !track.is_playable || !crate::domain::is_spotify_track_uri(uri) {
            return None;
        }
        Some(SpotifyCommand::PlayPlaylist {
            uri: detail.summary.uri.clone(),
            start_track_uri: Some(uri.to_owned()),
        })
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
            SpotifyEvent::PlaylistsLoading => self.library_phase = LibraryPhase::Loading,
            SpotifyEvent::PlaylistsLoaded(playlists) => {
                let count = playlists.len();
                self.playlists = playlists;
                self.library_phase = LibraryPhase::Ready;
                self.status = format!("{count} playlists ready");
            }
            SpotifyEvent::PlaylistsFailed(error) => {
                self.library_phase = LibraryPhase::Failed;
                self.apply_error(&error, false);
            }
            SpotifyEvent::PlaylistTracksLoaded {
                playlist_uri,
                generation,
                request_cursor,
                tracks,
            } => self.merge_loaded_tracks(&playlist_uri, generation, request_cursor, tracks),
            SpotifyEvent::PlaylistTracksFailed {
                playlist_uri,
                generation,
                request_cursor,
                error,
            } => self.fail_loaded_tracks(&playlist_uri, generation, request_cursor, error),
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
            self.library_phase = LibraryPhase::NotLoaded;
            self.selected_playlist = None;
        } else if fatal {
            self.view = ViewState::Failed;
            self.library_phase = LibraryPhase::Failed;
        }
    }

    fn merge_loaded_tracks(
        &mut self,
        playlist_uri: &str,
        generation: u64,
        request_cursor: Option<PageCursor>,
        tracks: Page<TrackSummary>,
    ) {
        if !self.accepts_track_response(playlist_uri, generation, request_cursor) {
            return;
        }
        let replace = request_cursor.is_none();
        let detail = self
            .selected_playlist
            .as_mut()
            .expect("accepted track response has a selected playlist");
        merge_track_page(&mut detail.tracks, tracks.items, replace);
        detail.next = tracks.next;
        detail.phase = TrackPagePhase::Ready;
        detail.awaiting = false;
    }

    fn fail_loaded_tracks(
        &mut self,
        playlist_uri: &str,
        generation: u64,
        request_cursor: Option<PageCursor>,
        error: AppError,
    ) {
        if !self.accepts_track_response(playlist_uri, generation, request_cursor) {
            return;
        }
        let detail = self
            .selected_playlist
            .as_mut()
            .expect("accepted track response has a selected playlist");
        detail.phase = TrackPagePhase::Failed(error);
        detail.awaiting = false;
    }

    fn accepts_track_response(
        &self,
        playlist_uri: &str,
        generation: u64,
        request_cursor: Option<PageCursor>,
    ) -> bool {
        self.selected_playlist.as_ref().is_some_and(|detail| {
            detail.awaiting
                && detail.generation == generation
                && detail.summary.uri == playlist_uri
                && detail.pending_cursor == request_cursor
        })
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

fn tracks_command(detail: &PlaylistDetail) -> SpotifyCommand {
    SpotifyCommand::OpenPlaylist {
        uri: detail.summary.uri.clone(),
        cursor: detail.pending_cursor,
        generation: detail.generation,
    }
}

#[cfg(test)]
#[path = "__tests__/state.rs"]
mod tests;
