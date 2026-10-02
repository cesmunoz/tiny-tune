use std::sync::Arc;

use async_channel::{Receiver, Sender};

use super::{
    credentials::SystemCredentialStore,
    messages::{SpotifyCommand, SpotifyEvent},
    runtime,
};

#[derive(Clone)]
pub struct SpotifyHandle {
    commands: Sender<SpotifyCommand>,
}

impl SpotifyHandle {
    pub fn try_send(&self, command: SpotifyCommand) -> bool {
        self.commands.try_send(command).is_ok()
    }
}

pub fn spawn() -> (SpotifyHandle, Receiver<SpotifyEvent>) {
    let (command_tx, command_rx) = async_channel::bounded(16);
    let (event_tx, event_rx) = async_channel::bounded(32);
    command_tx
        .try_send(SpotifyCommand::RestoreSession)
        .expect("new Spotify command queue should accept session restoration");
    runtime::spawn(command_rx, event_tx, Arc::new(SystemCredentialStore::new()));

    (
        SpotifyHandle {
            commands: command_tx,
        },
        event_rx,
    )
}
