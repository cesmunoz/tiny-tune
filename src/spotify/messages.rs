use crate::domain::{AppError, PlaylistSummary};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SpotifyCommand {
    RestoreSession,
    LoginAndPlayLikedSongs,
    PlayPlaylist { uri: String },
    TogglePlayPause,
    Previous,
    Next,
    Shutdown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SpotifyEvent {
    RestoringSession,
    SignedOut,
    Authenticating,
    Connecting,
    Ready,
    PlaylistsLoaded(Vec<PlaylistSummary>),
    Loading,
    Playing { track_uri: String },
    Paused { track_uri: String },
    Stopped,
    RecoverableError(AppError),
    FatalError(AppError),
}
