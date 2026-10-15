use std::{sync::Arc, thread};

use async_channel::{Receiver, Sender};
use tracing::{error, info, warn};

use crate::domain::AppError;

use super::{
    credentials::CredentialStore,
    messages::{SpotifyCommand, SpotifyEvent},
    playback::{SessionStart, run_session},
};

pub fn spawn(
    commands: Receiver<SpotifyCommand>,
    events: Sender<SpotifyEvent>,
    credentials: Arc<dyn CredentialStore>,
) {
    thread::Builder::new()
        .name("spotify-backend".into())
        .spawn(move || {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .thread_name("spotify-runtime")
                .build();

            match runtime {
                Ok(runtime) => runtime.block_on(run_supervisor(commands, events, credentials)),
                Err(error) => error!(%error, "failed to create Spotify runtime"),
            }
        })
        .expect("failed to spawn Spotify backend thread");
}

async fn run_supervisor(
    commands: Receiver<SpotifyCommand>,
    events: Sender<SpotifyEvent>,
    credentials: Arc<dyn CredentialStore>,
) {
    while let Ok(command) = commands.recv().await {
        match command {
            SpotifyCommand::RestoreSession => {
                run_and_report(
                    &commands,
                    &events,
                    credentials.as_ref(),
                    SessionStart::Stored,
                )
                .await;
            }
            SpotifyCommand::LoginAndPlayLikedSongs => {
                run_and_report(
                    &commands,
                    &events,
                    credentials.as_ref(),
                    SessionStart::Interactive,
                )
                .await;
            }
            SpotifyCommand::Logout => {
                let credentials_deleted = credentials.delete_refresh_token().is_ok();
                let _ = events.send(SpotifyEvent::SignedOut).await;
                if !credentials_deleted {
                    warn!("secure credential storage is unavailable");
                    let _ = events
                        .send(SpotifyEvent::RecoverableError(
                            AppError::CredentialStoreUnavailable,
                        ))
                        .await;
                }
            }
            SpotifyCommand::Shutdown => break,
            SpotifyCommand::LoadArtwork { .. } => {}
            SpotifyCommand::PlayPlaylist { .. }
            | SpotifyCommand::TogglePlayPause
            | SpotifyCommand::Previous
            | SpotifyCommand::Next => {
                let _ = events
                    .send(SpotifyEvent::RecoverableError(AppError::BackendUnavailable))
                    .await;
            }
        }
    }

    info!("Spotify backend stopped");
}

async fn run_and_report(
    commands: &Receiver<SpotifyCommand>,
    events: &Sender<SpotifyEvent>,
    credentials: &dyn CredentialStore,
    start: SessionStart,
) {
    if let Err(error) = run_session(commands, events, credentials, start).await {
        error!(%error, "Spotify session failed");
        let _ = events
            .send(SpotifyEvent::FatalError(AppError::Unexpected))
            .await;
    }
}
