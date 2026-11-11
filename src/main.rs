mod artwork_cache;
mod config;
#[cfg(debug_assertions)]
mod diagnostics;
mod domain;
mod spotify;
mod ui;

use std::time::Instant;

use rune::{App, AppSpec};
use tracing_subscriber::EnvFilter;

use ui::{SCENES, TinyTune, shell_size, view};

fn main() {
    rune::prewarm_gpu();
    dotenvy::dotenv().ok();
    initialize_logging();

    let started_at = Instant::now();
    let chrome = shell_size();
    let spec = AppSpec {
        min_size: Some((chrome.min_width, chrome.min_height)),
        scenes: SCENES,
        setup: Some(TinyTune::setup),
        poll: Some(TinyTune::poll),
        ..AppSpec::new(
            "Tiny Tune",
            (chrome.width, chrome.height),
            || TinyTune::new(Instant::now()),
            view,
        )
    };
    App::new(spec)
        .init(move || TinyTune::new(started_at))
        .on_close(TinyTune::on_close)
        .run();
}

fn initialize_logging() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("tiny_tune=info")),
        )
        .without_time()
        .init();
}
