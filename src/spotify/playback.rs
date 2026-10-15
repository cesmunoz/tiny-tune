use std::{
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};

use anyhow::{Result, anyhow};
use async_channel::{Receiver, Sender};
use librespot::{
    connect::{ConnectConfig, LoadRequest, LoadRequestOptions, PlayingTrack, Spirc},
    core::{
        authentication::Credentials, config::SessionConfig, session::Session,
        spotify_uri::SpotifyUri,
    },
    metadata::audio::AudioItem,
    playback::player::PlayerEvent,
};
use tracing::warn;

use secrecy::SecretString;

use crate::{
    config::SpotifyConfig,
    domain::{
        AppError, NowPlaying, PageCursor, PlaybackState, TrackSummary, is_spotify_context_uri,
        is_spotify_track_uri,
    },
};

use super::{
    artwork::{self, DecodedArtwork},
    audio,
    auth::{authenticate, refresh},
    catalog::{self, error_needs_token_refresh, fetch_playlist_tracks, fetch_playlists},
    credentials::CredentialStore,
    messages::{ArtworkBytes, SpotifyCommand, SpotifyEvent},
    now_playing::project_audio_item,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SessionStart {
    Interactive,
    Stored,
}

pub async fn run_session(
    commands: &Receiver<SpotifyCommand>,
    events: &Sender<SpotifyEvent>,
    credentials: &dyn CredentialStore,
    start: SessionStart,
) -> Result<()> {
    let session_config = SessionConfig::default();
    let Some(signed_in) = sign_in(
        events,
        credentials,
        SpotifyConfig::from_env(session_config.client_id.clone()),
        start,
    )
    .await?
    else {
        return Ok(());
    };
    let SignedIn {
        mut catalog,
        credentials_persisted,
    } = signed_in;

    send(events, SpotifyEvent::Connecting).await;

    let session = Session::new(session_config, None);
    let audio = audio::initialize(session.clone())?;
    let mut player_events = audio.events;
    let spotify_credentials = Credentials::with_access_token(catalog.access_token.clone());
    let connect_config = ConnectConfig {
        name: "Tiny Tune".into(),
        ..ConnectConfig::default()
    };
    let (spirc, spirc_task) = Spirc::new(
        connect_config,
        session.clone(),
        spotify_credentials,
        audio.player,
        audio.mixer,
    )
    .await?;
    let mut spirc_task = Box::pin(spirc_task);
    let artwork_loader = ArtworkLoader::new();
    let mut playback = PlaybackProjector::default();
    let mut playback_publish = PlaybackPublish::default();

    if start == SessionStart::Interactive {
        play_liked_songs(&spirc, &session)?;
    }
    send(events, SpotifyEvent::Ready).await;
    if load_playlists(events, credentials, &mut catalog).await == SessionFollowUp::Reauthenticate {
        artwork_loader.cancel();
        spirc.shutdown()?;
        let _ = tokio::time::timeout(Duration::from_secs(2), &mut spirc_task).await;
        return Ok(());
    }
    if !credentials_persisted {
        send(
            events,
            SpotifyEvent::RecoverableError(AppError::CredentialStoreUnavailable),
        )
        .await;
    }

    drive_player(PlayerLoop {
        commands,
        events,
        credentials,
        spirc: &spirc,
        session: &session,
        spirc_task: &mut spirc_task,
        player_events: &mut player_events,
        artwork_loader: &artwork_loader,
        playback: &mut playback,
        playback_publish: &mut playback_publish,
        catalog: &mut catalog,
    })
    .await
}

struct SignedIn {
    catalog: CatalogSession,
    credentials_persisted: bool,
}

async fn sign_in(
    events: &Sender<SpotifyEvent>,
    credentials: &dyn CredentialStore,
    config: SpotifyConfig,
    start: SessionStart,
) -> Result<Option<SignedIn>> {
    let authenticated = match start {
        SessionStart::Interactive => {
            send(events, SpotifyEvent::Authenticating).await;
            authenticate(&config).await
        }
        SessionStart::Stored => {
            send(events, SpotifyEvent::RestoringSession).await;
            let refresh_token = match credentials.load_refresh_token() {
                Ok(Some(refresh_token)) => refresh_token,
                Ok(None) => {
                    send(events, SpotifyEvent::SignedOut).await;
                    return Ok(None);
                }
                Err(_) => {
                    warn!("secure credential storage is unavailable");
                    send(events, SpotifyEvent::SignedOut).await;
                    send(
                        events,
                        SpotifyEvent::RecoverableError(AppError::CredentialStoreUnavailable),
                    )
                    .await;
                    return Ok(None);
                }
            };
            refresh(&config, &refresh_token).await
        }
    };
    let Ok(authenticated) = authenticated else {
        warn!("Spotify authentication failed");
        send(
            events,
            SpotifyEvent::FatalError(AppError::ReauthenticationRequired),
        )
        .await;
        return Ok(None);
    };
    let credentials_persisted = credentials
        .save_refresh_token(&authenticated.refresh_token)
        .is_ok();
    if !credentials_persisted {
        warn!("secure credential storage is unavailable");
    }

    Ok(Some(SignedIn {
        catalog: CatalogSession {
            http: catalog::http_client(),
            access_token: authenticated.access_token,
            refresh_token: authenticated.refresh_token,
            config,
        },
        credentials_persisted,
    }))
}

struct PlayerLoop<'a, F> {
    commands: &'a Receiver<SpotifyCommand>,
    events: &'a Sender<SpotifyEvent>,
    credentials: &'a dyn CredentialStore,
    spirc: &'a Spirc,
    session: &'a Session,
    spirc_task: &'a mut std::pin::Pin<Box<F>>,
    player_events: &'a mut librespot::playback::player::PlayerEventChannel,
    artwork_loader: &'a ArtworkLoader,
    playback: &'a mut PlaybackProjector,
    playback_publish: &'a mut PlaybackPublish,
    catalog: &'a mut CatalogSession,
}

async fn drive_player<F>(player: PlayerLoop<'_, F>) -> Result<()>
where
    F: Future<Output = ()>,
{
    loop {
        player.playback_publish.flush(player.events);
        tokio::select! {
            () = player.spirc_task.as_mut() => {
                player.artwork_loader.cancel();
                return Err(anyhow!("Spotify Connect session ended"));
            }
            command = player.commands.recv() => {
                if let Ok(SpotifyCommand::LoadArtwork { url }) = &command {
                    player.artwork_loader.request(url.clone(), player.events.clone());
                    continue;
                }
                if let Ok(SpotifyCommand::OpenPlaylist { uri, cursor, generation }) = command {
                    if handle_open_playlist(
                        player.catalog,
                        player.credentials,
                        player.events,
                        uri,
                        cursor,
                        generation,
                    )
                    .await
                        == SessionFollowUp::Reauthenticate
                    {
                        player.artwork_loader.cancel();
                        player.spirc.shutdown()?;
                        let _ = tokio::time::timeout(Duration::from_secs(2), player.spirc_task.as_mut()).await;
                        return Ok(());
                    }
                    continue;
                }
                if let Some(exit) = handle_command(command, player.events, player.spirc, player.session, player.spirc_task).await? {
                    player.artwork_loader.cancel();
                    if exit == SessionExit::Logout {
                        finish_logout(player.events, player.credentials).await;
                    }
                    return Ok(());
                }
            }
            event = player.player_events.recv() => {
                let Some(event) = event else {
                    player.artwork_loader.cancel();
                    return Err(anyhow!("Spotify player event stream ended"));
                };
                match player.playback.apply(event) {
                    PlayerProjection::Playback(state) => {
                        player.playback_publish.push(player.events, state);
                    }
                    PlayerProjection::Error(error) => {
                        send(player.events, SpotifyEvent::RecoverableError(error)).await;
                    }
                    PlayerProjection::Ignore => {}
                }
            }
        }
    }
}

#[derive(Default)]
struct PlaybackPublish {
    pending: Option<PlaybackState>,
}

impl PlaybackPublish {
    fn push(&mut self, events: &Sender<SpotifyEvent>, state: PlaybackState) {
        self.pending = Some(state);
        self.flush(events);
    }

    fn flush(&mut self, events: &Sender<SpotifyEvent>) {
        let Some(state) = self.pending.clone() else {
            return;
        };
        if events
            .try_send(SpotifyEvent::PlaybackChanged(state))
            .is_ok()
        {
            self.pending = None;
        }
    }
}

struct ArtworkLoader {
    http: reqwest::Client,
    generation: Arc<AtomicU64>,
}

impl ArtworkLoader {
    fn new() -> Self {
        Self {
            http: reqwest::Client::new(),
            generation: Arc::new(AtomicU64::new(0)),
        }
    }

    fn cancel(&self) {
        self.generation.fetch_add(1, Ordering::Relaxed);
    }

    fn request(&self, url: String, events: Sender<SpotifyEvent>) {
        let ticket = self.generation.fetch_add(1, Ordering::Relaxed) + 1;
        let generation = Arc::clone(&self.generation);
        let http = self.http.clone();
        tokio::spawn(async move {
            let outcome = artwork::load_display_artwork(&http, &url).await;
            if generation.load(Ordering::Relaxed) != ticket {
                return;
            }
            let event = if let Ok(decoded) = outcome {
                SpotifyEvent::ArtworkReady(Box::new(artwork_bytes(url, decoded)))
            } else {
                warn!("artwork download failed");
                SpotifyEvent::ArtworkFailed { url }
            };
            let _ = events.send(event).await;
        });
    }
}

fn artwork_bytes(url: String, decoded: DecodedArtwork) -> ArtworkBytes {
    ArtworkBytes {
        url,
        width: decoded.width,
        height: decoded.height,
        bgra: decoded.bgra,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SessionExit {
    Logout,
    Shutdown,
}

async fn handle_command<F>(
    command: Result<SpotifyCommand, async_channel::RecvError>,
    events: &Sender<SpotifyEvent>,
    spirc: &Spirc,
    session: &Session,
    spirc_task: &mut std::pin::Pin<Box<F>>,
) -> Result<Option<SessionExit>>
where
    F: Future<Output = ()>,
{
    match command {
        Ok(SpotifyCommand::PlayPlaylist {
            uri,
            start_track_uri,
        }) => match start_playback(spirc, uri, start_track_uri)? {
            StartPlayback::Started => {}
            StartPlayback::Unavailable => {
                send(
                    events,
                    SpotifyEvent::RecoverableError(AppError::TrackUnavailable),
                )
                .await;
            }
        },
        Ok(SpotifyCommand::TogglePlayPause) => spirc.play_pause()?,
        Ok(SpotifyCommand::Previous) => spirc.prev()?,
        Ok(SpotifyCommand::Next) => spirc.next()?,
        Ok(SpotifyCommand::LoginAndPlayLikedSongs) => play_liked_songs(spirc, session)?,
        Ok(
            SpotifyCommand::LoadArtwork { .. }
            | SpotifyCommand::RestoreSession
            | SpotifyCommand::OpenPlaylist { .. },
        ) => {}
        Ok(SpotifyCommand::Logout) => {
            spirc.shutdown()?;
            let _ = tokio::time::timeout(Duration::from_secs(2), spirc_task).await;
            return Ok(Some(SessionExit::Logout));
        }
        Ok(SpotifyCommand::Shutdown) | Err(_) => {
            spirc.shutdown()?;
            let _ = tokio::time::timeout(Duration::from_secs(2), spirc_task).await;
            return Ok(Some(SessionExit::Shutdown));
        }
    }

    Ok(None)
}

async fn finish_logout(events: &Sender<SpotifyEvent>, credentials: &dyn CredentialStore) {
    let credentials_deleted = credentials.delete_refresh_token().is_ok();
    send(events, SpotifyEvent::SignedOut).await;

    if !credentials_deleted {
        warn!("secure credential storage is unavailable");
        send(
            events,
            SpotifyEvent::RecoverableError(AppError::CredentialStoreUnavailable),
        )
        .await;
    }
}

fn play_liked_songs(spirc: &Spirc, session: &Session) -> Result<()> {
    spirc.activate()?;
    play_context(
        spirc,
        format!("spotify:user:{}:collection", session.username()),
        None,
    )
}

fn play_context(spirc: &Spirc, uri: String, start_track_uri: Option<String>) -> Result<()> {
    match start_playback(spirc, uri, start_track_uri)? {
        StartPlayback::Started => Ok(()),
        StartPlayback::Unavailable => Err(anyhow!("unsupported Spotify playback target")),
    }
}

fn start_playback(
    spirc: &Spirc,
    context_uri: String,
    start_track_uri: Option<String>,
) -> Result<StartPlayback> {
    if !is_spotify_context_uri(&context_uri) {
        return Ok(StartPlayback::Unavailable);
    }
    let playing_track = match start_track_uri {
        Some(uri) if is_spotify_track_uri(&uri) => Some(PlayingTrack::Uri(uri)),
        Some(_) => return Ok(StartPlayback::Unavailable),
        None => None,
    };
    spirc.load(LoadRequest::from_context_uri(
        context_uri,
        LoadRequestOptions {
            start_playing: true,
            playing_track,
            ..LoadRequestOptions::default()
        },
    ))?;
    Ok(StartPlayback::Started)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StartPlayback {
    Started,
    Unavailable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SessionFollowUp {
    Continue,
    Reauthenticate,
}

struct CatalogSession {
    http: reqwest::Client,
    access_token: String,
    refresh_token: SecretString,
    config: SpotifyConfig,
}

async fn load_playlists(
    events: &Sender<SpotifyEvent>,
    credentials: &dyn CredentialStore,
    catalog: &mut CatalogSession,
) -> SessionFollowUp {
    match catalog_call(catalog, credentials, |http, token| {
        let http = http.clone();
        let token = token.to_owned();
        async move { fetch_playlists(&http, &token).await }
    })
    .await
    {
        Ok(playlists) => {
            send(events, SpotifyEvent::PlaylistsLoaded(playlists)).await;
            SessionFollowUp::Continue
        }
        Err(AppError::ReauthenticationRequired) => {
            warn!(
                kind = AppError::ReauthenticationRequired.kind(),
                "spotify catalog request failed"
            );
            send(
                events,
                SpotifyEvent::FatalError(AppError::ReauthenticationRequired),
            )
            .await;
            SessionFollowUp::Reauthenticate
        }
        Err(error) => {
            warn!(kind = error.kind(), "spotify catalog request failed");
            send(events, SpotifyEvent::RecoverableError(error)).await;
            SessionFollowUp::Continue
        }
    }
}

async fn handle_open_playlist(
    catalog: &mut CatalogSession,
    credentials: &dyn CredentialStore,
    events: &Sender<SpotifyEvent>,
    playlist_uri: String,
    cursor: Option<PageCursor>,
    generation: u64,
) -> SessionFollowUp {
    let request_cursor = cursor;
    match catalog_call(catalog, credentials, |http, token| {
        let http = http.clone();
        let token = token.to_owned();
        let playlist_uri = playlist_uri.clone();
        let page_cursor = cursor;
        async move {
            fetch_playlist_tracks(&http, &token, &playlist_uri, page_cursor.as_ref()).await
        }
    })
    .await
    {
        Ok(tracks) => {
            send(
                events,
                SpotifyEvent::PlaylistTracksLoaded {
                    playlist_uri,
                    generation,
                    request_cursor,
                    tracks,
                },
            )
            .await;
            SessionFollowUp::Continue
        }
        Err(AppError::ReauthenticationRequired) => {
            warn!(
                kind = AppError::ReauthenticationRequired.kind(),
                "playlist tracks request failed"
            );
            send(
                events,
                SpotifyEvent::FatalError(AppError::ReauthenticationRequired),
            )
            .await;
            SessionFollowUp::Reauthenticate
        }
        Err(error) => {
            warn!(kind = error.kind(), "playlist tracks request failed");
            send(
                events,
                SpotifyEvent::PlaylistTracksFailed {
                    playlist_uri,
                    generation,
                    request_cursor,
                    error,
                },
            )
            .await;
            SessionFollowUp::Continue
        }
    }
}

async fn catalog_call<T, F, Fut>(
    catalog: &mut CatalogSession,
    credentials: &dyn CredentialStore,
    mut call: F,
) -> Result<T, AppError>
where
    F: FnMut(&reqwest::Client, &str) -> Fut,
    Fut: Future<Output = Result<T, AppError>>,
{
    let first = call(&catalog.http, &catalog.access_token).await;
    if !matches!(first, Err(ref error) if error_needs_token_refresh(error)) {
        return first;
    }
    refresh_catalog_token(catalog, credentials).await?;
    call(&catalog.http, &catalog.access_token).await
}

async fn refresh_catalog_token(
    catalog: &mut CatalogSession,
    credentials: &dyn CredentialStore,
) -> Result<(), AppError> {
    let refreshed = refresh(&catalog.config, &catalog.refresh_token)
        .await
        .map_err(|_| AppError::ReauthenticationRequired)?;
    catalog.access_token = refreshed.access_token;
    catalog.refresh_token = refreshed.refresh_token;
    if credentials
        .save_refresh_token(&catalog.refresh_token)
        .is_err()
    {
        warn!("secure credential storage is unavailable");
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PlaybackPhase {
    Idle,
    Loading,
    Playing,
    Paused,
}

enum PlayerProjection {
    Playback(PlaybackState),
    Error(AppError),
    Ignore,
}

#[derive(Debug)]
struct PlaybackProjector {
    phase: PlaybackPhase,
    position_ms: u32,
    track_uri: Option<String>,
    summary: Option<TrackSummary>,
}

impl Default for PlaybackProjector {
    fn default() -> Self {
        Self {
            phase: PlaybackPhase::Idle,
            position_ms: 0,
            track_uri: None,
            summary: None,
        }
    }
}

impl PlaybackProjector {
    fn apply(&mut self, event: PlayerEvent) -> PlayerProjection {
        match event {
            PlayerEvent::Loading {
                track_id,
                position_ms,
                ..
            } => self.transition(PlaybackPhase::Loading, &track_id, position_ms),
            PlayerEvent::Playing {
                track_id,
                position_ms,
                ..
            } => self.transition(PlaybackPhase::Playing, &track_id, position_ms),
            PlayerEvent::Paused {
                track_id,
                position_ms,
                ..
            } => self.transition(PlaybackPhase::Paused, &track_id, position_ms),
            PlayerEvent::Stopped { .. } => {
                *self = Self::default();
                PlayerProjection::Playback(PlaybackState::Idle)
            }
            PlayerEvent::TrackChanged { audio_item } => self.apply_track(*audio_item),
            PlayerEvent::Seeked {
                track_id,
                position_ms,
                ..
            } => {
                self.observe_track(&track_id, position_ms);
                self.snapshot()
                    .map_or(PlayerProjection::Ignore, PlayerProjection::Playback)
            }
            PlayerEvent::PositionChanged {
                track_id,
                position_ms,
                ..
            }
            | PlayerEvent::PositionCorrection {
                track_id,
                position_ms,
                ..
            } => {
                self.observe_track(&track_id, position_ms);
                PlayerProjection::Ignore
            }
            PlayerEvent::Unavailable { .. } => PlayerProjection::Error(AppError::TrackUnavailable),
            _ => PlayerProjection::Ignore,
        }
    }

    fn transition(
        &mut self,
        phase: PlaybackPhase,
        track_id: &SpotifyUri,
        position_ms: u32,
    ) -> PlayerProjection {
        let track_changed = self.observe_track(track_id, position_ms);
        let phase_changed = self.phase != phase;
        self.phase = phase;
        if track_changed || phase_changed {
            self.snapshot()
                .map_or(PlayerProjection::Ignore, PlayerProjection::Playback)
        } else {
            PlayerProjection::Ignore
        }
    }

    fn apply_track(&mut self, item: AudioItem) -> PlayerProjection {
        let summary = project_audio_item(&item);
        drop(item);
        if let Some(uri) = summary.spotify_uri.clone()
            && self.track_uri.as_deref() != Some(uri.as_str())
        {
            self.track_uri = Some(uri);
        }
        self.summary = Some(summary);
        if self.phase == PlaybackPhase::Idle {
            PlayerProjection::Ignore
        } else {
            self.snapshot()
                .map_or(PlayerProjection::Ignore, PlayerProjection::Playback)
        }
    }

    fn observe_track(&mut self, track_id: &SpotifyUri, position_ms: u32) -> bool {
        self.position_ms = position_ms;
        let Ok(uri) = track_id.to_uri() else {
            return false;
        };
        if self.track_uri.as_deref() == Some(uri.as_str()) {
            return false;
        }
        self.track_uri = Some(uri);
        self.summary = None;
        true
    }

    fn snapshot(&self) -> Option<PlaybackState> {
        let now_playing = self.now_playing();
        Some(match self.phase {
            PlaybackPhase::Idle => PlaybackState::Idle,
            PlaybackPhase::Loading => PlaybackState::Loading { now_playing },
            PlaybackPhase::Playing => PlaybackState::Playing {
                now_playing: now_playing?,
            },
            PlaybackPhase::Paused => PlaybackState::Paused {
                now_playing: now_playing?,
            },
        })
    }

    fn now_playing(&self) -> Option<NowPlaying> {
        let track = self
            .summary
            .clone()
            .or_else(|| self.track_uri.clone().map(TrackSummary::pending))?;
        Some(NowPlaying {
            track,
            position_ms: self.position_ms,
        })
    }
}

async fn send(events: &Sender<SpotifyEvent>, event: SpotifyEvent) {
    let _ = events.send(event).await;
}

#[cfg(test)]
#[path = "__tests__/playback.rs"]
mod tests;
