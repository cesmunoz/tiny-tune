use std::thread;

use async_channel::{Receiver, Sender};
use tracing::{error, info};

use super::{
    messages::{SpotifyCommand, SpotifyEvent},
    playback::run_session,
};

pub fn spawn(commands: Receiver<SpotifyCommand>, events: Sender<SpotifyEvent>) {
    thread::Builder::new()
        .name("spotify-backend".into())
        .spawn(move || {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .thread_name("spotify-runtime")
                .build();

            match runtime {
                Ok(runtime) => runtime.block_on(run_supervisor(commands, events)),
                Err(error) => error!(%error, "failed to create Spotify runtime"),
            }
        })
        .expect("failed to spawn Spotify backend thread");
}

async fn run_supervisor(commands: Receiver<SpotifyCommand>, events: Sender<SpotifyEvent>) {
    while let Ok(command) = commands.recv().await {
        match command {
            SpotifyCommand::LoginAndPlayLikedSongs => {
                if let Err(error) = run_session(&commands, &events).await {
                    error!(%error, "Spotify session failed");
                    let _ = events
                        .send(SpotifyEvent::Error {
                            message: error.to_string(),
                        })
                        .await;
                }
            }
            SpotifyCommand::Shutdown => break,
            SpotifyCommand::PlayPlaylist { .. }
            | SpotifyCommand::TogglePlayPause
            | SpotifyCommand::Previous
            | SpotifyCommand::Next => {
                let _ = events
                    .send(SpotifyEvent::Error {
                        message: "Connect to Spotify first".into(),
                    })
                    .await;
            }
        }
    }

    info!("Spotify backend stopped");
}
