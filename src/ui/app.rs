use std::{
    sync::mpsc::{Receiver, TryRecvError, sync_channel},
    thread,
    time::Instant,
};

use rune::{Pixels, Tx, Waker};
use tracing::{debug, info, warn};

#[cfg(debug_assertions)]
use crate::diagnostics::process_memory_mb;
use crate::{
    artwork_cache::{ArtworkCache, ArtworkInsert},
    config::AppConfig,
    spotify::{self, ArtworkBytes, SpotifyCommand, SpotifyEvent, SpotifyHandle},
};

use super::{artwork::render_artwork, state::AppState};

/// Backend events waiting for the next frame. The backend blocks when this is
/// full, so the queue stays bounded.
const UI_EVENT_QUEUE: usize = 32;

/// The running Spotify backend, seen from the UI thread.
struct Backend {
    handle: SpotifyHandle,
    events: Receiver<SpotifyEvent>,
}

pub struct TinyTune {
    pub(super) ui: AppState,
    #[cfg(debug_assertions)]
    pub(super) startup_ms: u128,
    #[cfg(debug_assertions)]
    pub(super) memory_mb: Option<u64>,
    backend: Option<Backend>,
    artwork: ArtworkCache<Pixels>,
    visible_artwork: Option<(String, Pixels)>,
    artwork_inflight: Option<String>,
    artwork_failed: Option<String>,
}

impl TinyTune {
    pub fn new(started_at: Instant) -> Self {
        let startup_ms = started_at.elapsed().as_millis();
        info!(startup_ms, "Tiny Tune state created");

        Self {
            ui: AppState::default(),
            #[cfg(debug_assertions)]
            startup_ms,
            #[cfg(debug_assertions)]
            memory_mb: process_memory_mb(),
            backend: None,
            artwork: ArtworkCache::new(AppConfig::from_env().image_cache_budget_bytes()),
            visible_artwork: None,
            artwork_inflight: None,
            artwork_failed: None,
        }
    }

    /// Rune `setup` hook: starts the Spotify backend and wakes the window
    /// whenever it reports an event. Headless snapshots run without Spotify.
    pub fn setup(app: &mut Self, waker: Waker) {
        if rune::tooling::headless_run().is_some() {
            return;
        }

        let (handle, backend_events) = spotify::spawn();
        let (ui_events, events) = sync_channel(UI_EVENT_QUEUE);
        thread::Builder::new()
            .name("spotify-events".into())
            .spawn(move || {
                while let Ok(event) = backend_events.recv_blocking() {
                    if ui_events.send(event).is_err() {
                        break;
                    }
                    waker();
                }
            })
            .expect("failed to spawn Spotify event thread");

        app.backend = Some(Backend { handle, events });
    }

    /// Rune `poll` hook: applies queued backend events before each frame.
    /// Returns true only when state changed.
    pub fn poll(app: &mut Self, _clock: f64, _tx: &mut Tx) -> bool {
        let mut changed = false;
        while let Some(backend) = app.backend.as_ref() {
            match backend.events.try_recv() {
                Ok(event) => {
                    app.handle_backend_event(event);
                    changed = true;
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    warn!("Spotify backend event stream closed");
                    break;
                }
            }
        }
        #[cfg(debug_assertions)]
        if changed {
            app.memory_mb = process_memory_mb();
        }
        changed
    }

    /// Rune `on_close` hook.
    pub fn on_close(app: &mut Self, _tx: &mut Tx) {
        app.shutdown();
    }

    fn shutdown(&mut self) {
        let Some(backend) = self.backend.take() else {
            return;
        };
        if !backend.handle.try_send(SpotifyCommand::Shutdown) {
            warn!("Spotify backend already stopped");
        }
    }

    fn handle_backend_event(&mut self, event: SpotifyEvent) {
        match event {
            SpotifyEvent::ArtworkReady(artwork) => self.store_artwork(*artwork),
            SpotifyEvent::ArtworkFailed { url } => {
                if self.artwork_inflight.as_deref() == Some(url.as_str()) {
                    self.artwork_inflight = None;
                }
                self.artwork_failed = Some(url);
            }
            other => {
                self.ui.apply(other);
                self.sync_artwork();
            }
        }
    }

    fn store_artwork(&mut self, artwork: ArtworkBytes) {
        if self.artwork_inflight.as_deref() == Some(artwork.url.as_str()) {
            self.artwork_inflight = None;
        }
        if !self.ui.connected() {
            return;
        }
        let url = artwork.url.clone();
        let bytes = artwork.rgba.len();
        let Some(image) = render_artwork(artwork) else {
            self.artwork_failed = Some(url);
            return;
        };
        if self.artwork_failed.as_deref() == Some(url.as_str()) {
            self.artwork_failed = None;
        }
        let protected = self.ui.current_image_url();
        let ArtworkInsert { stored, evicted } =
            self.artwork
                .insert(url.clone(), image.clone(), bytes, protected);
        for eviction in evicted {
            debug!(
                bytes = eviction.bytes,
                retained = self.artwork.used_bytes(),
                "evicted artwork from cache"
            );
        }
        if stored {
            if self
                .visible_artwork
                .as_ref()
                .is_some_and(|(cached, _)| cached == &url)
            {
                self.visible_artwork = None;
            }
        } else if protected == Some(url.as_str()) {
            self.visible_artwork = Some((url, image));
        }
    }

    fn sync_artwork(&mut self) {
        if !self.ui.connected() {
            self.artwork.clear();
            self.visible_artwork = None;
            self.artwork_inflight = None;
            self.artwork_failed = None;
            return;
        }

        let Some(url) = self.ui.current_image_url().map(str::to_owned) else {
            return;
        };
        if self.artwork.contains(&url) {
            self.artwork.touch(&url);
            if self.artwork_inflight.as_deref() == Some(url.as_str()) {
                self.artwork_inflight = None;
            }
            return;
        }
        if self
            .visible_artwork
            .as_ref()
            .is_some_and(|(cached, _)| cached == &url)
        {
            return;
        }
        if self.artwork_failed.as_deref() == Some(url.as_str())
            || self.artwork_inflight.as_deref() == Some(url.as_str())
        {
            return;
        }
        if self.try_send(SpotifyCommand::LoadArtwork { url: url.clone() }) {
            self.artwork_inflight = Some(url);
        }
    }

    pub(super) fn displayed_artwork(&self) -> Option<&Pixels> {
        let url = self.ui.current_image_url()?;
        if let Some(image) = self.artwork.peek(url) {
            return Some(image);
        }
        self.visible_artwork
            .as_ref()
            .filter(|(cached, _)| cached == url)
            .map(|(_, image)| image)
    }

    /// Sends a command from a UI handler. Rune re-runs the view after every
    /// handler, so a full queue only needs to update the status.
    pub(super) fn send(&mut self, command: SpotifyCommand) {
        if !self.try_send(command) {
            self.ui.backend_busy();
        }
    }

    fn try_send(&self, command: SpotifyCommand) -> bool {
        self.backend
            .as_ref()
            .is_some_and(|backend| backend.handle.try_send(command))
    }

    pub(super) fn connected(&self) -> bool {
        self.ui.connected()
    }
}

impl Drop for TinyTune {
    fn drop(&mut self) {
        self.shutdown();
    }
}
