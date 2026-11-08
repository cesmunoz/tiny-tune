use std::{sync::Arc, time::Instant};

use gpui::{Context, RenderImage};
use tracing::{debug, info, warn};

#[cfg(debug_assertions)]
use crate::diagnostics::process_memory_mb;
use crate::{
    artwork_cache::{ArtworkCache, ArtworkInsert},
    config::AppConfig,
    spotify::{self, ArtworkBytes, SpotifyCommand, SpotifyEvent, SpotifyHandle},
};

use super::{artwork::render_artwork, state::AppState};

pub struct TinyTune {
    pub(super) backend: SpotifyHandle,
    pub(super) ui: AppState,
    #[cfg(debug_assertions)]
    pub(super) startup_ms: u128,
    #[cfg(debug_assertions)]
    pub(super) memory_mb: Option<u64>,
    artwork: ArtworkCache<Arc<RenderImage>>,
    visible_artwork: Option<(String, Arc<RenderImage>)>,
    artwork_inflight: Option<String>,
    artwork_failed: Option<String>,
}

impl TinyTune {
    pub fn new(cx: &mut Context<Self>, started_at: Instant) -> Self {
        let (backend, events) = spotify::spawn();

        cx.spawn(async move |this, cx| {
            while let Ok(event) = events.recv().await {
                if this
                    .update(cx, |app, cx| {
                        app.handle_backend_event(event);
                        #[cfg(debug_assertions)]
                        {
                            app.memory_mb = process_memory_mb();
                        }
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
        info!(startup_ms, "Tiny Tune window created");

        Self {
            backend,
            ui: AppState::default(),
            #[cfg(debug_assertions)]
            startup_ms,
            #[cfg(debug_assertions)]
            memory_mb: process_memory_mb(),
            artwork: ArtworkCache::new(AppConfig::from_env().image_cache_budget_bytes()),
            visible_artwork: None,
            artwork_inflight: None,
            artwork_failed: None,
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
        let bytes = artwork.bgra.len();
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
                .insert(url.clone(), Arc::clone(&image), bytes, protected);
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
        if self
            .backend
            .try_send(SpotifyCommand::LoadArtwork { url: url.clone() })
        {
            self.artwork_inflight = Some(url);
        }
    }

    pub(super) fn displayed_artwork(&self) -> Option<Arc<RenderImage>> {
        let url = self.ui.current_image_url()?;
        if let Some(image) = self.artwork.peek(url) {
            return Some(Arc::clone(image));
        }
        self.visible_artwork
            .as_ref()
            .filter(|(cached, _)| cached == url)
            .map(|(_, image)| Arc::clone(image))
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
