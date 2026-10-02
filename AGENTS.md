# Tiny Tune repository guide

## Product constraints

- Keep Tiny Tune native and lightweight. Do not add Electron, Node, React, or a resident WebView.
- Support macOS, Windows, and Linux.
- Spotify integration is read-only except for playback controls.
- Treat `librespot` as an isolated, replaceable adapter.

## Architecture

- `src/main.rs`: process and GPUI bootstrap only.
- `src/config.rs`: environment and application configuration.
- `src/domain.rs`: application-owned domain types.
- `src/spotify/`: OAuth, Web API, playback, runtime, and UI/backend contracts.
- `src/ui/`: GPUI state, reducer, theme, and composable views.
- GPUI state stays on the main thread.
- Spotify and Tokio work stays on the dedicated backend thread.
- UI code must not depend on concrete `librespot` or `reqwest` types.
- Keep channels bounded and avoid unbounded in-memory collections.

## Configuration and security

- Local development may use `.env`; never commit it.
- Update `.env.example` when adding configuration.
- Real environment variables override `.env` values.
- Never commit Spotify client secrets, access tokens, refresh tokens, cookies, or account data.
- Never log tokens, OAuth codes, account IDs, playlist names, or track names.
- Playback uses `librespot`'s default client configuration. `TINY_TUNE_SPOTIFY_CLIENT_ID` is an optional advanced override.
- Preserve the clear unofficial-integration and Premium-only warning in public documentation.

## Required checks

Run before considering work complete:

```sh
mise exec -- cargo fmt --all -- --check
mise exec -- cargo clippy --all-targets -- -D warnings
mise exec -- cargo test
mise exec -- cargo build --release
```

## Code style

- Prefer application-owned contracts at external boundaries.
- Keep DTOs private to their adapter and project them into domain types.
- Keep `main.rs` small and split GPUI views by responsibility.
- Use explicit state enums instead of boolean bags.
- Add tests for parsers, state transitions, and event projections.
- Keep test implementations in dedicated `__tests__/` subfolders and attach them with private `#[path]` test modules.
- Pin unstable dependencies such as GPUI to an explicit revision.
