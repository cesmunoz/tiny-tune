use std::time::Duration;

use anyhow::{Result, anyhow};
use async_channel::{Receiver, Sender};
use librespot::{
    connect::{ConnectConfig, LoadRequest, LoadRequestOptions, Spirc},
    core::{authentication::Credentials, config::SessionConfig, session::Session},
    playback::player::PlayerEvent,
};
use tracing::{error, warn};

use crate::{config::SpotifyConfig, domain::AppError};

use super::{
    audio,
    auth::{authenticate, refresh},
    catalog::fetch_playlists,
    credentials::CredentialStore,
    messages::{SpotifyCommand, SpotifyEvent},
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
    let config = SpotifyConfig::from_env(session_config.client_id.clone());
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
                    return Ok(());
                }
                Err(_) => {
                    warn!("secure credential storage is unavailable");
                    send(events, SpotifyEvent::SignedOut).await;
                    send(
                        events,
                        SpotifyEvent::RecoverableError(AppError::CredentialStoreUnavailable),
                    )
                    .await;
                    return Ok(());
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
        return Ok(());
    };
    let credentials_persisted = credentials
        .save_refresh_token(&authenticated.refresh_token)
        .is_ok();
    if !credentials_persisted {
        warn!("secure credential storage is unavailable");
    }
    let access_token = authenticated.access_token;

    send(events, SpotifyEvent::Connecting).await;

    let session = Session::new(session_config, None);
    let audio = audio::initialize(session.clone())?;
    let mut player_events = audio.events;
    let spotify_credentials = Credentials::with_access_token(access_token.clone());
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

    if start == SessionStart::Interactive {
        play_liked_songs(&spirc, &session)?;
    }
    send(events, SpotifyEvent::Ready).await;
    load_playlists(events, &access_token).await;
    if !credentials_persisted {
        send(
            events,
            SpotifyEvent::RecoverableError(AppError::CredentialStoreUnavailable),
        )
        .await;
    }

    loop {
        tokio::select! {
            () = &mut spirc_task => {
                return Err(anyhow!("Spotify Connect session ended"));
            }
            command = commands.recv() => {
                if let Some(exit) = handle_command(command, &spirc, &session, &mut spirc_task).await? {
                    if exit == SessionExit::Logout {
                        finish_logout(events, credentials).await;
                    }
                    return Ok(());
                }
            }
            event = player_events.recv() => {
                let Some(event) = event else {
                    return Err(anyhow!("Spotify player event stream ended"));
                };
                if let Some(event) = project_player_event(event) {
                    send(events, event).await;
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SessionExit {
    Logout,
    Shutdown,
}

async fn handle_command<F>(
    command: Result<SpotifyCommand, async_channel::RecvError>,
    spirc: &Spirc,
    session: &Session,
    spirc_task: &mut std::pin::Pin<Box<F>>,
) -> Result<Option<SessionExit>>
where
    F: Future<Output = ()>,
{
    match command {
        Ok(SpotifyCommand::PlayPlaylist { uri }) => {
            play_context(spirc, uri)?;
        }
        Ok(SpotifyCommand::TogglePlayPause) => spirc.play_pause()?,
        Ok(SpotifyCommand::Previous) => spirc.prev()?,
        Ok(SpotifyCommand::Next) => spirc.next()?,
        Ok(SpotifyCommand::LoginAndPlayLikedSongs) => play_liked_songs(spirc, session)?,
        Ok(SpotifyCommand::RestoreSession) => {}
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
    )
}

fn play_context(spirc: &Spirc, uri: String) -> Result<()> {
    spirc.load(LoadRequest::from_context_uri(
        uri,
        LoadRequestOptions {
            start_playing: true,
            ..LoadRequestOptions::default()
        },
    ))?;
    Ok(())
}

async fn load_playlists(events: &Sender<SpotifyEvent>, access_token: &str) {
    match fetch_playlists(access_token).await {
        Ok(playlists) => send(events, SpotifyEvent::PlaylistsLoaded(playlists)).await,
        Err(error) => {
            error!(%error, "failed to load Spotify playlists");
            send(
                events,
                SpotifyEvent::RecoverableError(AppError::PlaylistUnavailable),
            )
            .await;
        }
    }
}

fn project_player_event(event: PlayerEvent) -> Option<SpotifyEvent> {
    match event {
        PlayerEvent::Loading { .. } => Some(SpotifyEvent::Loading),
        PlayerEvent::Playing { track_id, .. } => track_id
            .to_uri()
            .ok()
            .map(|track_uri| SpotifyEvent::Playing { track_uri }),
        PlayerEvent::Paused { track_id, .. } => track_id
            .to_uri()
            .ok()
            .map(|track_uri| SpotifyEvent::Paused { track_uri }),
        PlayerEvent::Stopped { .. } => Some(SpotifyEvent::Stopped),
        PlayerEvent::Unavailable { .. } => {
            Some(SpotifyEvent::RecoverableError(AppError::TrackUnavailable))
        }
        _ => None,
    }
}

async fn send(events: &Sender<SpotifyEvent>, event: SpotifyEvent) {
    let _ = events.send(event).await;
}

#[cfg(test)]
#[path = "__tests__/playback.rs"]
mod tests;
