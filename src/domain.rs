#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlaylistSummary {
    pub name: String,
    pub uri: String,
}

/// One Web API page. Cursors are opaque outside the Spotify adapter.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub next: Option<PageCursor>,
}

/// Offset of the next playlist-track page. The UI can pass it back and cannot inspect it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PageCursor {
    offset: u32,
}

impl PageCursor {
    pub const PAGE_SIZE: usize = 50;

    pub(crate) fn from_offset(offset: u32) -> Option<Self> {
        (offset > 0).then_some(Self { offset })
    }

    pub(crate) const fn offset(self) -> u32 {
        self.offset
    }
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
    pub(crate) fn unavailable() -> Self {
        Self {
            id: None,
            spotify_uri: None,
            name: "Unavailable".into(),
            artist_names: Vec::new(),
            album_name: String::new(),
            duration_ms: 0,
            image: None,
            is_playable: false,
        }
    }

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
    PlaybackUnreachable,
    RateLimited { retry_after_seconds: u64 },
    Offline,
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
            Self::PlaybackUnreachable => "Could not reach Spotify playback. Try again.".into(),
            Self::RateLimited {
                retry_after_seconds,
            } => format!("Spotify is rate limiting requests. Try again in {retry_after_seconds}s"),
            Self::Offline => "Spotify could not be reached. Try again.".into(),
            Self::Unexpected => "Tiny Tune could not complete the Spotify request".into(),
        }
    }

    pub fn tracks_message(&self) -> String {
        match self {
            Self::PlaylistUnavailable => "This playlist could not be loaded".into(),
            other => other.user_message(),
        }
    }

    pub(crate) const fn kind(&self) -> &'static str {
        match self {
            Self::ReauthenticationRequired => "reauthentication_required",
            Self::PlaylistUnavailable => "playlist_unavailable",
            Self::TrackUnavailable => "track_unavailable",
            Self::CredentialStoreUnavailable => "credential_store_unavailable",
            Self::BackendUnavailable => "backend_unavailable",
            Self::PlaybackUnreachable => "playback_unreachable",
            Self::RateLimited { .. } => "rate_limited",
            Self::Offline => "offline",
            Self::Unexpected => "unexpected",
        }
    }
}

pub fn is_spotify_track_uri(uri: &str) -> bool {
    spotify_resource_id(uri, "spotify:track:").is_some()
}

pub fn is_spotify_playlist_uri(uri: &str) -> bool {
    spotify_resource_id(uri, "spotify:playlist:").is_some()
}

/// Playback contexts Tiny Tune is willing to hand to the player.
pub fn is_spotify_context_uri(uri: &str) -> bool {
    let Some(rest) = uri.strip_prefix("spotify:") else {
        return false;
    };
    (1..=256).contains(&rest.len())
        && !rest.chars().any(|character| {
            character.is_whitespace() || matches!(character, '/' | '?' | '#' | '@')
        })
}

/// Keeps at most one page of new rows and drops duplicate track URIs.
pub fn merge_track_page(existing: &mut Vec<TrackSummary>, page: Vec<TrackSummary>, replace: bool) {
    let page = page.into_iter().take(PageCursor::PAGE_SIZE);
    if replace {
        existing.clear();
    }
    for track in page {
        if let Some(uri) = &track.spotify_uri
            && existing
                .iter()
                .any(|item| item.spotify_uri.as_ref() == Some(uri))
        {
            continue;
        }
        existing.push(track);
    }
}

fn spotify_resource_id<'a>(uri: &'a str, prefix: &str) -> Option<&'a str> {
    let id = uri.strip_prefix(prefix)?;
    if (1..=64).contains(&id.len())
        && id
            .chars()
            .all(|character| character.is_ascii_alphanumeric())
    {
        Some(id)
    } else {
        None
    }
}

#[cfg(test)]
#[path = "__tests__/library.rs"]
mod tests;
