use std::time::Instant;

use gpui::Context;
use tracing::{info, warn};

use crate::{
    diagnostics::process_memory_mb,
    spotify::{self, SpotifyCommand, SpotifyHandle},
};

use super::state::AppState;

pub struct TinyTune {
    pub(super) backend: SpotifyHandle,
    pub(super) ui: AppState,
    pub(super) startup_ms: u128,
    pub(super) memory_mb: Option<u64>,
}

impl TinyTune {
    pub fn new(cx: &mut Context<Self>, started_at: Instant) -> Self {
        let (backend, events) = spotify::spawn();

        cx.spawn(async move |this, cx| {
            while let Ok(event) = events.recv().await {
                if this
                    .update(cx, |app, cx| {
                        app.ui.apply(event);
                        app.memory_mb = process_memory_mb();
                        cx.notify();
                    })
                    .is_err()
                {
                    break;
                }
            }
        })
        .detach();

        let startup_ms = started_at.elapsed().as_millis();
        let memory_mb = process_memory_mb();
        info!(startup_ms, ?memory_mb, "Tiny Tune window created");

        Self {
            backend,
            ui: AppState::default(),
            startup_ms,
            memory_mb,
        }
    }

    pub(super) fn send(&mut self, command: SpotifyCommand, cx: &mut Context<Self>) {
        if self.backend.try_send(command) {
            return;
        }

        self.ui.backend_busy();
        cx.notify();
    }

    pub(super) fn connected(&self) -> bool {
        self.ui.connected()
    }
}

impl Drop for TinyTune {
    fn drop(&mut self) {
        if self.backend.try_send(SpotifyCommand::Shutdown) {
            return;
        }

        warn!("Spotify backend already stopped");
    }
}
