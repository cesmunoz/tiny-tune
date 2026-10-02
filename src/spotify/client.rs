use async_channel::{Receiver, Sender};

use super::{
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
    runtime::spawn(command_rx, event_tx);

    (
        SpotifyHandle {
            commands: command_tx,
        },
        event_rx,
    )
}
