# Tiny Tune

A small, native Spotify Premium player for macOS, Windows, and Linux.

Tiny Tune is built in Rust with GPUI and `librespot`. It does not embed Electron, Node, React, Chromium, or a WebView.

> [!WARNING]
> Tiny Tune uses `librespot`, an unofficial Spotify client. The upstream project states: “Using this code to connect to Spotify's API is probably forbidden by them. Use at your own risk.” Tiny Tune requires Spotify Premium, is not affiliated with Spotify, and may stop working without notice.

## Current alpha

- Spotify OAuth in the system browser
- local playback of Liked Songs
- read-only list of up to 50 playlists
- start playlist playback
- play/pause, previous, and next
- startup and RSS diagnostics

Session persistence, secure token storage, playlist tracks, artwork, volume, and installers are not implemented yet.

## Requirements

- Spotify Premium
- Rust stable
- macOS, Windows, or Linux
- platform build dependencies required by [GPUI](https://gpui.rs/) and the system audio backend

The repo includes `mise.toml`, but any current stable Rust installation works.

## Run

```sh
mise install
mise exec -- cargo run --release
```

Tiny Tune uses the client configuration provided by `librespot`, so no Spotify developer setup is required. Advanced users can copy `.env.example` to `.env` to override the client ID or loopback callback.

Click **Connect & play liked songs**, finish login in the browser, and return to Tiny Tune.

## Quality checks

```sh
mise exec -- cargo fmt --all -- --check
mise exec -- cargo clippy --all-targets -- -D warnings
mise exec -- cargo test
```

## Architecture

```text
GPUI main thread
  <-> bounded command/event channels
  <-> dedicated Tokio runtime thread
  <-> Spotify Web API + librespot Spirc/Player
  -> system audio
```

The complete implementation handoff is in [`docs/technical-concept.md`](docs/technical-concept.md). Current measurements are in [`docs/performance-baseline.md`](docs/performance-baseline.md).
