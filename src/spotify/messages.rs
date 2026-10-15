use crate::domain::{AppError, PlaybackState, PlaylistSummary};

/// Display-sized BGRA pixels. The encoded download is not retained.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArtworkBytes {
    pub url: String,
    pub width: u32,
    pub height: u32,
    pub bgra: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SpotifyCommand {
    RestoreSession,
    LoginAndPlayLikedSongs,
    Logout,
    PlayPlaylist { uri: String },
    TogglePlayPause,
    Previous,
    Next,
    LoadArtwork { url: String },
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
    PlaybackChanged(PlaybackState),
    ArtworkReady(Box<ArtworkBytes>),
    ArtworkFailed { url: String },
    RecoverableError(AppError),
    FatalError(AppError),
}
