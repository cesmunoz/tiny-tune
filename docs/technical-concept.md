# Tiny Tune MVP Technical Concept

## Summary

Tiny Tune will be a deliberately small, private Spotify Premium desktop player for macOS, Windows, and Linux.

The first version will use:

- Rust for the entire application;
- GPUI for a native GPU-accelerated interface without Electron, Node, React, or a resident WebView;
- `librespot` for Spotify-compatible authentication, Spotify Connect, and local playback;
- Spotify Web API for read-only access to the user's playlists when the acquired token supports it;
- the operating system credential store for refresh tokens;
- bounded caches and paginated loading to prevent unnecessary memory growth.

The first delivery gate is a private spike that must demonstrate login, at least 30 seconds of local audio, and compilation on all three target platforms. The complete UI must not be built on top of an integration that has not passed this gate.

Initial performance targets are directional rather than hard release requirements:

- idle RSS: at most 50 MB;
- playback RSS: at most 100 MB;
- first visible window: at most one second on a reasonable development machine;
- no persistent Chromium or Node processes.

## Context / Current State

The repository began empty, without existing standards, a toolchain, code, or tests. The feasibility alpha now contains a GPUI shell, a bounded UI/backend channel boundary, OAuth wiring, `librespot` playback, a read-only playlist list, paginated tracks for the single open playlist, tests, CI, and a first performance baseline. Track pages hold at most 50 items. Stale track responses are dropped with a generation id, and raw Web API JSON is discarded after projection.

Verified technical evidence:

- GPUI `0.2.x` supports macOS, Linux/FreeBSD, and Windows through `gpui_platform`.
- GPUI remains pre-1.0 and warns about frequent breaking changes.
- `librespot` `0.8.x` provides separate core, OAuth, Connect, and playback crates.
- `librespot` exposes browser OAuth, sessions, `Player`, and `Spirc` APIs.
- Spotify Web API supports playlist reads and playback control for Premium users.
- `librespot` is unofficial. Reimplementing Spotify's private protocol would not remove that policy risk and would multiply maintenance cost.
- Tiny Tune intentionally uses `librespot`'s default client configuration for zero-setup playback and documents the upstream project's risk disclaimer.

## User / Maintainer Goals

- Play Spotify Premium audio directly through Tiny Tune.
- View owned and followed playlists, open one, and start playback.
- Control play/pause, previous/next, and volume.
- Display the current track, artist, and artwork.
- Keep startup, binary size, and RAM substantially below an Electron client.
- Measure startup and RSS to catch large regressions.
- Isolate Spotify integration so `librespot` can be replaced later.
- Keep the project understandable as it grows through explicit modules and contracts.

## Non-Goals

- Creating, editing, deleting, or reordering playlists.
- Search across artists, albums, or tracks.
- Podcasts, audiobooks, lyrics, recommendations, social features, or ads.
- Offline downloads.
- Equalizer controls, configurable normalization, or manual quality selection in the MVP.
- Synchronizing, modifying, rebroadcasting, or exporting Spotify audio.
- Public or commercial distribution before policy review.
- Reimplementing Spotify's private protocol.
- React: GPUI renders the interface directly in Rust.

## Invariants and Constraints

1. Playback requires Spotify Premium.
2. Source code, logs, and build artifacts contain no client secrets, access tokens, refresh tokens, or credentials.
3. Playback uses `librespot`'s default client configuration. A custom client ID remains an optional advanced override.
4. OAuth uses the system browser; Tiny Tune does not embed a browser.
5. The OAuth callback listens only on loopback and stops after completion, expiry, or cancellation.
6. UI state and GPUI objects remain on the main thread.
7. `librespot` and its Tokio runtime remain behind one application-owned boundary. UI modules never depend on `librespot` types.
8. UI/backend queues are bounded. Progress updates may coalesce; user commands are never silently dropped.
9. Playlists and tracks load in pages rather than materializing the complete library in memory.
10. Artwork loads on demand, is decoded near display size, and uses a bounded cache.
11. Network, token, and audio work never blocks the GPUI thread.
12. Structured logs redact OAuth codes, tokens, account IDs, signed URLs, playlist names, and track names.
13. GPUI and `librespot` versions/revisions are pinned and updated deliberately.
14. Spotify developer policy and client-ID compatibility are release gates, not assumptions.

## Affected Systems

- Rust bootstrap and CI;
- GPUI application shell and views;
- OAuth and secure credential storage;
- Spotify Web API adapter;
- `librespot` session, Connect, and audio runtime;
- domain models and application state;
- non-sensitive configuration and caches;
- local performance diagnostics and logs;
- cross-platform packaging.

## Current Code Paths

```txt
Tiny Tune
  -> system browser -> Spotify OAuth
  -> Spotify Web API over HTTPS
  -> Spotify protocol through librespot
  -> operating-system audio device
  -> operating-system credential store (planned)
  -> filesystem for non-sensitive configuration/cache (planned)
```

The feasibility implementation currently groups too many responsibilities in `backend.rs` and `main.rs`. The target module map in this document replaces that temporary spike structure.

## Scope Split / Delivery Plan

### Slice 0: Feasibility gate

**Behavior**

- Open a minimal GPUI window.
- Start browser-based login.
- Create a `librespot` session and play local audio for at least 30 seconds.
- Record startup and idle/playback RSS.

**Engineering**

- Bootstrap the toolchain and minimal structure.
- Keep `librespot` behind bounded command/event channels.
- Run `cargo check`, tests, formatting, and Clippy in CI.
- Check macOS, Windows, and Linux.

**Gate**

Do not continue without a documented answer if:

- OAuth does not produce credentials accepted for streaming;
- `librespot`'s default authentication flow no longer produces credentials accepted for streaming;
- `librespot` cannot maintain stable local playback;
- GPUI does not compile on a target platform;
- licensing or terms make even the intended private use unacceptable;
- memory behavior is structurally incompatible with the lightweight goal.

### Slice 1: Application shell and session

- Signed-out, authenticating, ready, expired, and error states.
- OAuth state/PKCE validation.
- Refresh token in the system credential store.
- Logout deletes credentials and closes session/player resources.
- Recoverable login errors in the UI.
- Extract temporary spike code into the module boundaries defined below.

### Slice 2: Read-only playlists

- Paginated list of owned and followed playlists.
- Lazy artwork and placeholders.
- Playlist detail with paginated tracks.
- Explicit retry behavior for 429, offline, and expired-token failures.

### Slice 3: Playback

- Start local playback from a playlist or track.
- Play/pause, previous/next, and volume.
- Now-playing track, artist, artwork, and state.
- Controlled reconnection and session refresh.

### Slice 4: Hardening and private alpha

- Bounded caches and cleanup.
- Keyboard accessibility and labels.
- Private installers for all three platforms.
- Manual audio, sleep/wake, and network-loss matrix.
- Per-platform memory baseline.

## Alternatives Considered

### Option 1: GPUI with embedded librespot

```txt
GPUI AppModel
  <-> bounded Command/Event ports
  <-> dedicated Tokio runtime
  <-> librespot Session + Spirc + Player
  -> OS audio
```

- **Advantages:** expected low overhead, one Rust binary, direct lifecycle/memory control, no browser runtime.
- **Disadvantages:** GPUI is pre-1.0, its ecosystem is smaller, `librespot` is unofficial, and UI/packaging require more work.
- **Testing:** application tests use fake ports; a small ignored smoke suite requires a Premium account.

### Option 2: Tauri v2 with a Rust/librespot backend

```txt
System WebView UI
  <-> Tauri IPC
  <-> Rust/librespot backend
```

- **Advantages:** faster UI development, stronger documentation, optional React.
- **Disadvantages:** resident WebView, duplicated state/IPC, higher expected memory, and WebKit/WebView2/WebKitGTK differences.
- **Fit:** fallback if GPUI blocks accessibility, text rendering, or platform support.

### Option 3: GPUI as an official Spotify controller

```txt
GPUI -> official OAuth/Web API -> another Spotify device
```

- **Advantages:** official protocol and smaller technical/policy surface.
- **Disadvantages:** Tiny Tune does not emit audio and requires another active Spotify device.
- **Fit:** safe fallback if embedded playback fails.

### Rejected option: implement Spotify's private protocol

This requires authentication, Connect state, encryption, streaming, codecs, cache behavior, reconnection, and continuous tracking of private server changes. It would take months to approach `librespot` maturity while preserving the same policy risk.

## Recommendation

Adopt Option 1 after Slice 0 validates `librespot` authentication and playback. Preserve application-owned interfaces so Option 2 can replace GPUI or Option 3 can replace playback without changing domain/UI contracts.

Start with one binary crate and explicit modules. Extract additional crates only when a second application or a real package boundary appears.

## Proposed Design

### Target module structure

```txt
src/
  main.rs                    # process bootstrap only
  app.rs                     # AppModel and event reducer
  config.rs                  # environment and versioned app configuration
  diagnostics.rs             # sanitized startup/performance diagnostics
  domain/
    mod.rs
    playlist.rs
    playback.rs
    error.rs
  spotify/
    mod.rs
    messages.rs              # SpotifyCommand / SpotifyEvent
    client.rs                # UI-facing handle and bounded channels
    runtime.rs               # Tokio thread and supervisor
    auth.rs                  # OAuth and refresh
    audio.rs                 # player/mixer initialization
    catalog.rs               # Web API DTOs and projection
    playback.rs              # Spirc adapter and session loop
  ui/
    mod.rs
    theme.rs
    app_view.rs
    login_view.rs
    playlist_list.rs
    player_controls.rs
assets/
  oauth-success.html
```

This is component-oriented like a React application—state, reducer, views, and reusable components—without importing React's runtime or forcing React-specific patterns onto GPUI.

### Runtime topology

```txt
Main OS thread
  GPUI Application
    AppModel (single UI source of truth)
    screens/components
    event bridge task

Spotify backend thread
  Tokio runtime (bounded worker count)
    supervisor
    auth coordinator
    Web API client
    librespot Session
    librespot Spirc/Player

Shared boundaries
  async_channel::Sender<SpotifyCommand>
  async_channel::Receiver<SpotifyEvent>
  CredentialStore trait
  CachePaths
```

GPUI never owns a `Session`, `Spirc`, `Player`, or token. The backend publishes small, owned snapshots.

### Runtime supervision

The backend supervisor intentionally consumes commands in a loop:

```txt
wait for command
  -> BeginLogin starts one authenticated session
  -> session owns its command/event select loop
  -> normal logout/error returns to supervisor
  -> supervisor can accept another login
  -> Shutdown exits permanently
```

`while let Ok(command) = commands.recv().await` is idiomatic for this channel lifecycle. It should be named `run_supervisor` and live in `spotify/runtime.rs`; it should not be replaced with recursion or polling.

### Memory strategy

- Playlist pages contain 20–50 summaries; track pages contain at most 50 items.
- Retain detail for only one active playlist.
- Download artwork with a byte limit, resize before retention, and use a budgeted LRU.
- Drop raw JSON after projecting it into domain types.
- Use bounded channels; replace stale progress with the newest value.
- Bound and clean disk audio cache rather than accumulating memory buffers.
- Evaluate thin LTO using measured size/startup tradeoffs.
- Optimize from profiling evidence rather than intuition.

## Domain Model and Contracts

```rust
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PlaylistId(String);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TrackId(String);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImageRef {
    pub url: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlaylistSummary {
    pub id: PlaylistId,
    pub spotify_uri: String,
    pub name: String,
    pub owner_name: String,
    pub track_count: u32,
    pub image: Option<ImageRef>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrackSummary {
    pub id: Option<TrackId>, // unavailable/local items may have no usable id
    pub spotify_uri: Option<String>,
    pub name: String,
    pub artist_names: Vec<String>,
    pub album_name: String,
    pub duration_ms: u32,
    pub image: Option<ImageRef>,
    pub is_playable: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub next: Option<PageCursor>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PageCursor(String); // opaque outside the Spotify adapter

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SessionState {
    SignedOut,
    Authenticating,
    Ready { display_name: Option<String> },
    ReauthRequired,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlaybackState {
    Idle,
    Loading { context_name: String },
    Playing { now_playing: NowPlaying },
    Paused { now_playing: NowPlaying },
    Recovering { attempt: u8, now_playing: Option<NowPlaying> },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NowPlaying {
    pub track: TrackSummary,
    pub position_ms: u32,
    pub volume_percent: u8,
}
```

IDs and cursors are created only by parsers/adapters. UI views never consume Spotify JSON directly.

## API / Boundary Contracts

### UI to backend

```rust
#[derive(Debug)]
pub enum SpotifyCommand {
    BeginLogin,
    CancelLogin,
    Logout,
    LoadPlaylists { cursor: Option<PageCursor> },
    OpenPlaylist { id: PlaylistId, cursor: Option<PageCursor> },
    PlayPlaylist { playlist_uri: String, start_track_uri: Option<String> },
    TogglePlayPause,
    Next,
    Previous,
    SetVolume { percent: u8 },
    Shutdown,
}

pub struct SpotifyHandle {
    commands: async_channel::Sender<SpotifyCommand>,
}

impl SpotifyHandle {
    pub async fn send(&self, command: SpotifyCommand) -> Result<(), BackendUnavailable>;
}
```

The boundary validates/clamps volume to 0–100. Spotify URIs are parsed and constrained to expected resource kinds before reaching `librespot`.

### Backend to UI

```rust
#[derive(Clone, Debug)]
pub enum SpotifyEvent {
    SessionChanged(SessionState),
    PlaylistsLoaded(Page<PlaylistSummary>),
    PlaylistLoaded {
        playlist: PlaylistSummary,
        tracks: Page<TrackSummary>,
    },
    PlaybackChanged(PlaybackState),
    RecoverableError(AppError),
    FatalError(AppError),
    LoggedOut,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AppError {
    LoginCancelled,
    LoginExpired,
    ReauthenticationRequired,
    PremiumRequired,
    PermissionDenied,
    RateLimited { retry_after_seconds: u64 },
    Offline,
    PlaylistUnavailable,
    TrackUnavailable,
    AudioDeviceUnavailable,
    BackendUnavailable,
    Unexpected { correlation_id: String },
}
```

Concrete `reqwest`, OAuth, and `librespot` errors never cross into the UI. Adapters classify them and retain sanitized details only in logs.

### Internal ports

```rust
#[async_trait::async_trait]
pub trait PlaylistCatalog: Send + Sync {
    async fn playlists(
        &self,
        cursor: Option<PageCursor>,
    ) -> Result<Page<PlaylistSummary>, SpotifyFailure>;

    async fn playlist_tracks(
        &self,
        playlist: &PlaylistId,
        cursor: Option<PageCursor>,
    ) -> Result<Page<TrackSummary>, SpotifyFailure>;
}

#[async_trait::async_trait]
pub trait PlaybackEngine: Send {
    async fn play_context(
        &mut self,
        context_uri: &str,
        start_track_uri: Option<&str>,
    ) -> Result<(), SpotifyFailure>;

    async fn toggle(&mut self) -> Result<(), SpotifyFailure>;
    async fn next(&mut self) -> Result<(), SpotifyFailure>;
    async fn previous(&mut self) -> Result<(), SpotifyFailure>;
    async fn set_volume(&mut self, percent: u8) -> Result<(), SpotifyFailure>;
    async fn shutdown(&mut self);
}

pub trait CredentialStore: Send + Sync {
    fn load_refresh_token(&self) -> Result<Option<SecretString>, CredentialError>;
    fn save_refresh_token(&self, token: &SecretString) -> Result<(), CredentialError>;
    fn delete_refresh_token(&self) -> Result<(), CredentialError>;
}
```

Prefer native `impl Future` trait methods if they remain clear with the pinned Rust version; the semantic boundary does not change.

## Configuration Contract

```rust
pub struct SpotifyConfig {
    pub client_id: String,
    pub redirect_uri: String,
    pub ap_port: ApPortSetting,
}

impl SpotifyConfig {
    pub fn from_env(default_client_id: impl Into<String>) -> Self;
}
```

Environment variables:

- `TINY_TUNE_SPOTIFY_CLIENT_ID`: optional advanced override; defaults to the client configuration selected by `librespot`.
- `TINY_TUNE_SPOTIFY_REDIRECT_URI`: optional; defaults to `http://127.0.0.1:8898/login`.
- `TINY_TUNE_SPOTIFY_AP_PORT`: optional; unset tries librespot's default access-point port and then 443. A numeric port forces that port only.

Tiny Tune accepts `librespot`'s zero-setup authentication model. Client secrets are never accepted by the desktop application. Public documentation must retain the upstream warning that connecting to Spotify through this unofficial implementation may be forbidden and is used at the user's own risk.

## Client / UI Contracts

```rust
pub struct AppModel {
    pub session: SessionState,
    pub playlists: Loadable<Page<PlaylistSummary>>,
    pub selected_playlist: Option<PlaylistDetailState>,
    pub playback: PlaybackState,
    pub transient_error: Option<AppError>,
}

pub enum Loadable<T> {
    NotAsked,
    Loading,
    Ready(T),
    Failed { error: AppError },
}
```

Views/components:

- `LoginView`: login, cancellation, and actionable errors.
- `LibraryView`: virtualized/paginated playlists.
- `PlaylistView`: paginated tracks with unavailable items disabled.
- `PlayerBar`: current playback state and controls.

The root view composes these components; it does not contain the complete UI tree in one method. Theme tokens live in `ui/theme.rs`.

Minimum accessibility:

- complete keyboard navigation;
- visible focus;
- accessible names for icon buttons;
- playback state is not communicated by color alone;
- reduced motion is respected when GPUI exposes the preference.

## Persistence, Migrations, and Backfills

There is no database or initial migration.

```rust
#[derive(serde::Serialize, serde::Deserialize)]
pub struct AppConfigV1 {
    pub schema_version: u8,
    pub volume_percent: u8,
    pub image_cache_budget_mb: u16,
    pub audio_cache_budget_mb: u16,
}
```

- Secrets: operating-system credential store behind an adapter.
- Configuration: platform-standard directory with atomic temp-and-rename writes.
- Images/audio: cache directory, never the configuration directory.
- Corrupt data: move to `.invalid`, restore defaults, and show a non-blocking notice.
- Logout deletes tokens; cache cleanup remains a separate explicit action.

## Events, Metrics, Audit, and Observability

The MVP has no remote analytics, email, background jobs, or external audit stream.

```rust
pub enum DiagnosticEvent {
    AppStarted { visible_window_ms: u64 },
    LoginFinished { outcome: &'static str, duration_ms: u64 },
    PlaybackStarted { startup_ms: u64 },
    BackendRecovered { attempts: u8 },
    CacheEvicted { cache: &'static str, bytes: u64 },
}
```

Rules:

- use `tracing`;
- info by default, debug by opt-in;
- never log track/playlist names, account IDs, OAuth codes, tokens, or signed URLs;
- assign a random correlation ID to unexpected errors;
- document release-build startup, idle, and stable-playback measurements.

## Authorization and Failure Handling

Minimum intended scopes:

- `streaming` for the `librespot`-compatible playback flow;
- `playlist-read-private`;
- `playlist-read-collaborative`;
- player scopes only if the final adapter uses Web API playback control.

Do not request email, extended profile, or write permissions.

Failure policy:

- 401/invalid refresh -> `ReauthenticationRequired`; stop new protected commands.
- 403 -> `PremiumRequired` or `PermissionDenied` when the response supports that classification.
- 429 -> honor `Retry-After`; never retry aggressively.
- offline/5xx -> bounded exponential backoff with jitter and an explicit Retry action.
- lost audio device -> pause, publish an error, and allow retry.
- unavailable playlist/track -> preserve navigation and disable only the affected item.
- backend panic/disconnect -> `BackendUnavailable`; do not restart forever.

## Call Stacks and Data Flow

### Login flow

```txt
Login click
  -> SpotifyCommand::BeginLogin
  -> validate required runtime configuration
  -> ensure no login is already running
  -> generate state + PKCE verifier
  -> bind loopback callback
  -> open authorization URL in system browser
  -> receive callback code/state
  -> reject wrong state or expired callback
  -> exchange code for token
  -> establish librespot Session
  -> persist only refresh credential
  -> SpotifyEvent::SessionChanged(Ready)
  -> update AppModel on GPUI main thread
```

The spike must prove that `librespot`'s default authentication flow still produces credentials accepted for streaming. Advanced users may override the client ID, but ordinary Spotify developer applications may not receive the unofficial `streaming` scope.

### Playlist flow

```txt
Library becomes visible
  -> SpotifyCommand::LoadPlaylists { cursor: None }
  -> ensure/refresh valid access token
  -> GET current user's playlists
  -> validate status and deserialize boundary DTO
  -> project DTO -> Page<PlaylistSummary>
  -> discard raw response
  -> SpotifyEvent::PlaylistsLoaded
  -> merge page by PlaylistId in AppModel
  -> render virtualized GPUI rows
  -> fetch artwork only for visible rows
```

### Playback flow

```txt
playlist/track click
  -> validate Spotify URI and playability
  -> SpotifyCommand::PlayPlaylist
  -> backend ensures authenticated Session + active Player/Spirc
  -> Spirc load(context URI, optional track offset)
  -> Spirc play
  -> librespot opens OS audio sink and buffers stream
  -> map player events to PlaybackState
  -> SpotifyEvent::PlaybackChanged
  -> AppModel updates PlayerBar
```

### Failure flow

```txt
external/library/player error
  -> adapter classifies SpotifyFailure
  -> auth failure attempts one serialized refresh
  -> original operation retries at most once
  -> expected failure maps to AppError
  -> send sanitized event to UI
  -> UI presents retry/login/device action
```

### Async, retry, and idempotency

- Only one refresh runs at a time; concurrent callers await the same result.
- `BeginLogin` during active login does not create another listener.
- `SetVolume` retains only the newest value when backpressured.
- Playlist selection carries an internal generation ID; stale responses are discarded.
- GET requests retry bounded offline/5xx failures with jitter.
- Playback commands are not retried automatically when the result is ambiguous.
- `Shutdown` stops command acceptance, Spirc/Player, configuration writes, and the runtime with a timeout.

## Files to Add / Change / Delete

```txt
Cargo.toml
Cargo.lock
rust-toolchain.toml
.gitignore
AGENTS.md
README.md
LICENSE                         # required before public distribution
assets/oauth-success.html
src/main.rs
src/app.rs
src/config.rs
src/diagnostics.rs
src/domain/mod.rs
src/domain/playlist.rs
src/domain/playback.rs
src/domain/error.rs
src/spotify/mod.rs
src/spotify/messages.rs
src/spotify/client.rs
src/spotify/runtime.rs
src/spotify/auth.rs
src/spotify/audio.rs
src/spotify/catalog.rs
src/spotify/playback.rs
src/spotify/credentials.rs
src/ui/mod.rs
src/ui/theme.rs
src/ui/app_view.rs
src/ui/login_view.rs
src/ui/playlist_list.rs
src/ui/player_controls.rs
tests/app_flows.rs
tests/spotify_contracts.rs
.github/workflows/ci.yml
docs/technical-concept.md
docs/performance-baseline.md
```

Delete temporary `src/backend.rs` after its responsibilities move into the modules above. Do not preserve compatibility for private module paths.

## Release, Rollout, and Rollback

1. **Local spike:** no installer or external users.
2. **Private alpha:** signed artifacts where possible, limited to the owner.
3. **Open-source code:** includes a prominent unofficial-integration disclaimer and preserves attribution/license notices for `librespot`.
4. **No public binary release:** until Spotify Developer Policy, client-ID use, and `librespot` risk have been reviewed.

Compatibility:

- version configuration with `schema_version`;
- invalid tokens may force login without losing non-sensitive settings;
- update GPUI/librespot only in dedicated changes with three-platform smoke checks;
- commit the lockfile for reproducible application builds.

Rollback:

- return to the last known artifact;
- preserve V1 configuration readability;
- if Spotify breaks `librespot`, disable embedded playback and retain library/controller mode when authorized;
- no remote backfill is required because playlists remain read-only.

Temporary cleanup:

- remove diagnostics that provide no value after alpha;
- if `librespot` is replaced, delete only its adapter while preserving application contracts.

## RGR TDD Test Plan

### Slice 0: backend viability

- **Red:** a fake backend proves the window handles `Ready`, `Playing`, and failure without blocking.
- **Green:** bounded channels and runtime connect GPUI to the backend.
- **Refactor:** extract lifecycle/shutdown and run a real 30-second smoke test.

### Slice 1: authentication

- **Red:** missing config, wrong state, expired callback, cancellation, invalid refresh, and credential-store failure.
- **Green:** login -> persist refresh -> ready; restart -> refresh -> ready; logout -> signed out.
- **Refactor:** separate OAuth HTTP parsing, orchestration, and credential storage.

### Slice 2: playlists

- **Red:** fixtures for valid pages, optional fields, unavailable items, 401, 403, and 429.
- **Green:** login -> library -> next page; playlist -> tracks; visible retry.
- **Refactor:** ID deduplication, opaque cursors, and bounded artwork loading.

### Slice 3: playback

- **Red:** playlist/track commands, loading/playing/paused transitions, unavailable content, and audio-device failure.
- **Green:** public AppModel -> command -> event -> PlayerBar flow.
- **Refactor:** coalesce progress/volume and isolate `librespot` event mapping.

### Slice 4: hardening

- **Red:** shutdown during work, corrupt configuration, over-budget cache, and out-of-order responses.
- **Green:** cleanup, eviction, and generation IDs.
- **Refactor:** remove allocations/copies found by profiling.

### Test layers

- Pure unit tests for parsers, reducers, and state machines.
- Integration tests through `SpotifyHandle` with fake adapters.
- Sanitized contract fixtures for Spotify responses.
- GPUI keyboard/focus tests where `gpui::test` supports them.
- Real `#[ignore]` smoke tests, never in public CI and never with persisted credentials.
- Manual QA on macOS, Windows, and Linux for audio, sleep/wake, network loss, and logout.

## Estimate

Estimate for one senior engineer, assuming auth/playback passes the spike and no complex visual design is required:

| Slice | Implementation | Test/QA | Buffer | Total |
|---|---:|---:|---:|---:|
| 0. Feasibility gate | 2–3 days | 1 day | 1 day | 4–5 days |
| 1. Shell and session | 2–3 days | 1–2 days | 1 day | 4–6 days |
| 2. Playlists | 2–3 days | 1–2 days | 1 day | 4–6 days |
| 3. Playback | 3–4 days | 2 days | 1–2 days | 6–8 days |
| 4. Hardening/private alpha | 3–5 days | 2–3 days | 2 days | 7–10 days |

**Private MVP total:** approximately 5–7 calendar weeks of focused work. A reproducible audio/memory result should exist by the end of the first week.

The estimate decreases if GPUI/librespot integrate cleanly and macOS is validated first. It increases for packaging, signing, platform audio differences, upstream changes, or incompatible authentication.

## Risks and Open Questions

### Blockers and gates

1. **Authentication and scopes:** prove that `librespot`'s default client configuration continues to obtain both playback and playlist access. Spotify may change or revoke this behavior without notice.
2. **Spotify policy:** `librespot` is unofficial. Open source does not imply permission to distribute working binaries.
3. **GPUI stability:** pre-1.0 with breaking changes; pin revisions and never track `main` implicitly.
4. **Cross-platform audio:** validate audio backends and system dependencies on all three operating systems.

### Important risks

- Spotify may change OAuth eligibility, redirects, sessions, metadata, or playback without notice.
- GPUI may lack an accessibility or packaging primitive required for release.
- Artwork and large lists can dominate RAM without strict budgets.
- The initial GPUI idle baseline exceeds the aspirational 50 MB target.
- RSS is not directly comparable across platforms.

### Open questions deferred beyond Slice 0

- Final Spotify Connect device name.
- Initial fixed audio quality.
- Exact disk-cache budgets.
- Auto-update, signing/notarization, and release channels.
- Repository license.
- Detailed visual design, tray integration, media keys, and OS now-playing integration.
