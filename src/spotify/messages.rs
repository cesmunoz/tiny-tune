use crate::domain::PlaylistSummary;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SpotifyCommand {
    LoginAndPlayLikedSongs,
    PlayPlaylist { uri: String },
    TogglePlayPause,
    Previous,
    Next,
    Shutdown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SpotifyEvent {
    Authenticating,
    Connecting,
    Ready,
    PlaylistsLoaded(Vec<PlaylistSummary>),
    Loading,
    Playing { track_uri: String },
    Paused { track_uri: String },
    Stopped,
    Notice { message: String },
    Error { message: String },
}
