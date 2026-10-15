# Tiny Tune

A small, native Spotify Premium player for macOS, Windows, and Linux.

Tiny Tune is built in Rust with GPUI and `librespot`. It does not embed Electron, Node, React, Chromium, or a WebView.

> [!WARNING]
> Tiny Tune uses `librespot`, an unofficial Spotify client. The upstream project states: “Using this code to connect to Spotify's API is probably forbidden by them. Use at your own risk.” Tiny Tune requires Spotify Premium, is not affiliated with Spotify, and may stop working without notice.

## Current alpha

- Spotify OAuth in the system browser
- local playback of Liked Songs
- read-only list of up to 50 playlists
- open one playlist and browse its tracks in pages of 50
- start playback from a playlist or a playable track
- play/pause, previous, and next
- now playing title, artists, and cover art
- startup and RSS diagnostics

The player bar shows the current track while audio is playing or paused. Cover art is fetched only for that track, decoded near the display size (128 px), and kept in an LRU whose budget defaults to 8 MB (`TINY_TUNE_IMAGE_CACHE_BUDGET_MB`, maximum 64). A muted placeholder stays in the cover slot while artwork is missing or still loading.

Opening a playlist loads one page of tracks. **Load next page** requests the following page and appends it to that playlist only. Switching playlists drops the previous track list. Unavailable rows stay visible and cannot be played. If Spotify rate-limits the request, the network is unreachable, or the access token needs a refresh, the track list keeps the rows already loaded and offers **Try again**. A rejected refresh asks you to sign in again. Raw Web API JSON is not kept after it is turned into track rows.

Session persistence, volume, and installers are not implemented yet.

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

Tiny Tune uses the client configuration provided by `librespot`, so no Spotify developer setup is required. Advanced users can copy `.env.example` to `.env` to override the client ID, loopback callback, or access-point port.

Click **Connect & play liked songs**, finish login in the browser, and return to Tiny Tune. Playback tries Spotify's default access point, then port 443, and gives each attempt a short timeout. If both fail, the window leaves "Waiting for Spotify…" and shows **Retry**.

## Quality checks

Run the complete local quality gate with Cargo only:

```sh
cargo xtask check
```

Individual tasks are also available:

```sh
cargo xtask format
cargo xtask format-check
cargo xtask lint
cargo xtask test
cargo xtask release
```

The equivalent `mise run ...` tasks delegate to these Cargo commands.

## Architecture

```text
GPUI main thread
  <-> bounded command/event channels
  <-> dedicated Tokio runtime thread
  <-> Spotify Web API + librespot Spirc/Player
  -> system audio
```

The complete implementation handoff is in [`docs/technical-concept.md`](docs/technical-concept.md). Current measurements are in [`docs/performance-baseline.md`](docs/performance-baseline.md).
