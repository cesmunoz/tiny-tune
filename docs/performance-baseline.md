# Performance baseline

## 2026-10-02 — feasibility alpha

Machine: Apple Silicon macOS 27.0

Build: `cargo build --release`, thin LTO, stripped symbols

State: signed out, window idle, sampled with `ps`

| Metric | Result |
|---|---:|
| Window creation | 92 ms |
| RSS at window creation | 56 MB |
| RSS after 2 seconds | 95 MB |
| RSS after 10 seconds | 94 MB |
| RSS after 30 seconds | 94 MB |
| Release binary | 13 MB |

The original 50 MB idle target is aspirational and is not met by the initial GPUI shell. The current development environment lacks the full Xcode Metal compiler, so GPUI uses runtime shader compilation; a packaged build with precompiled shaders must be measured before drawing final conclusions.

Playback RSS remains unmeasured until interactive Spotify OAuth is completed.
