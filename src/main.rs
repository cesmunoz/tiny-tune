mod artwork_cache;
mod config;
mod diagnostics;
mod domain;
mod spotify;
mod ui;

use std::time::Instant;

use gpui::{App, Bounds, WindowBounds, WindowOptions, prelude::*, px, size};
use gpui_platform::application;
use tracing_subscriber::EnvFilter;

use ui::{TinyTune, shell_size};

fn main() {
    dotenvy::dotenv().ok();
    initialize_logging();

    let started_at = Instant::now();
    application().run(move |cx: &mut App| {
        let chrome = shell_size();
        let bounds = Bounds::centered(None, size(px(chrome.width), px(chrome.height)), cx);
        cx.on_window_closed(|cx, _| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                window_min_size: Some(size(px(chrome.min_width), px(chrome.min_height))),
                ..WindowOptions::default()
            },
            |_, cx| cx.new(|cx| TinyTune::new(cx, started_at)),
        )
        .expect("failed to open Tiny Tune window");
        cx.activate(true);
    });
}

fn initialize_logging() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("tiny_tune=info")),
        )
        .without_time()
        .init();
}
