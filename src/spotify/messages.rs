use crate::domain::{AppError, Page, PageCursor, PlaybackState, PlaylistSummary, TrackSummary};

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
    PlayPlaylist {
        uri: String,
        start_track_uri: Option<String>,
    },
    OpenPlaylist {
        uri: String,
        cursor: Option<PageCursor>,
        generation: u64,
    },
    TogglePlayPause,
    Previous,
    Next,
    SetVolume {
        percent: u8,
    },
    LoadArtwork {
        url: String,
    },
    Shutdown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SpotifyEvent {
    RestoringSession,
    SignedOut,
    Authenticating,
    Connecting,
    Ready,
    PlaylistsLoading,
    PlaylistsRetryScheduled {
        attempt: u8,
        max_attempts: u8,
    },
    PlaylistsRetriesExhausted,
    PlaylistsLoaded(Vec<PlaylistSummary>),
    PlaylistsFailed(AppError),
    PlaylistTracksLoaded {
        playlist_uri: String,
        generation: u64,
        request_cursor: Option<PageCursor>,
        tracks: Page<TrackSummary>,
    },
    PlaylistTracksFailed {
        playlist_uri: String,
        generation: u64,
        request_cursor: Option<PageCursor>,
        error: AppError,
    },
    PlaybackChanged(PlaybackState),
    VolumeChanged {
        percent: u8,
    },
    ArtworkReady(Box<ArtworkBytes>),
    ArtworkFailed {
        url: String,
    },
    RecoverableError(AppError),
    FatalError(AppError),
}
