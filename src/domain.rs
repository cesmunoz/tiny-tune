#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlaylistSummary {
    pub name: String,
    pub uri: String,
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
