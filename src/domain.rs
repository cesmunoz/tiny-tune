#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlaylistSummary {
    pub name: String,
    pub uri: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TrackId(String);

impl TrackId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

/// Remote artwork address. Bytes stay out of the domain model until a bounded cache decodes them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImageRef {
    pub url: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrackSummary {
    pub id: Option<TrackId>,
    pub spotify_uri: Option<String>,
    pub name: String,
    pub artist_names: Vec<String>,
    pub album_name: String,
    pub duration_ms: u32,
    pub image: Option<ImageRef>,
    pub is_playable: bool,
}

impl TrackSummary {
    pub(crate) fn pending(spotify_uri: String) -> Self {
        let id = spotify_uri
            .strip_prefix("spotify:track:")
            .filter(|id| !id.is_empty())
            .map(TrackId::new);
        Self {
            id,
            spotify_uri: Some(spotify_uri),
            name: String::new(),
            artist_names: Vec::new(),
            album_name: String::new(),
            duration_ms: 0,
            image: None,
            is_playable: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NowPlaying {
    pub track: TrackSummary,
    pub position_ms: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlaybackState {
    Idle,
    Loading { now_playing: Option<NowPlaying> },
    Playing { now_playing: NowPlaying },
    Paused { now_playing: NowPlaying },
}

impl PlaybackState {
    pub fn now_playing(&self) -> Option<&NowPlaying> {
        match self {
            Self::Idle => None,
            Self::Loading { now_playing } => now_playing.as_ref(),
            Self::Playing { now_playing } | Self::Paused { now_playing } => Some(now_playing),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AppError {
    ReauthenticationRequired,
    PlaylistUnavailable,
    TrackUnavailable,
    CredentialStoreUnavailable,
    BackendUnavailable,
    Unexpected,
}

impl AppError {
    pub fn user_message(&self) -> String {
        match self {
            Self::ReauthenticationRequired => "Sign in to Spotify again".into(),
            Self::PlaylistUnavailable => "Connected, but playlists could not be loaded".into(),
            Self::TrackUnavailable => "This track is unavailable".into(),
            Self::CredentialStoreUnavailable => {
                "Connected, but the session could not be saved securely".into()
            }
            Self::BackendUnavailable => "Spotify backend is unavailable".into(),
            Self::Unexpected => "Tiny Tune could not complete the Spotify request".into(),
        }
    }
}
