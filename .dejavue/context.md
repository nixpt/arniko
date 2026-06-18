---
name: arniko
purpose: Standalone Bliss/Mustang rendering stack + Arniko UI SDK (extracted from exosphere s277)
dcp: DCP/1.0
---

# Context

## Operating Rules

- `exo-bliss-net` and `exo-shell-client` stay in exosphere (mesh/xip deps) — reference via `path = "../../../exosphere/crates/..."` when optional.
- `bliss-core` stays in exosphere (JS engine deps: boa/v8/spidermonkey).
- git pins for taffy and parley must stay in sync with exosphere's `bliss-dom/Cargo.toml`.
- exosphere wires back via `[patch.crates-io]` → `../arniko/crates/*` for all bliss-* version deps.

## Build / Test

```bash
CARGO_TARGET_DIR=/build/target-arniko cargo check -p arniko
CARGO_TARGET_DIR=/build/target-arniko cargo check --workspace
```

## Architecture Map

| Crate | Role |
|-------|------|
| `bliss-traits` | Shared traits: NetProvider, ImageCache, Shell |
| `bliss-dom` | Headless HTML DOM + Servo/Stylo CSS + Taffy/Parley layout |
| `bliss-html` | HTML5 parser (html5ever sink) |
| `bliss-net` | Default reqwest NetProvider |
| `bliss-paint` | Vello scene painter (kurbo/peniko geometry) |
| `bliss-shell` | winit windowed application shell |
| `bliss` | Top-level re-export |
| `stylo_taffy` | Servo Stylo ↔ DioxusLabs Taffy bridge |
| `debug_timer` | Phase timing instrumentation |
| `accesskit_xplat` | Cross-platform accessibility (AccessKit) |
| `anyrender_vello` | Vello-backed anyrender implementation |
| `tui-shell` | Ratatui/crossterm TUI shell (optional exoshell feature) |
| `mustang` | GPU post-processor: blur/transform CSS synthetic effects (wgpu v27 / vello 0.7) |
| `arniko` | Unified UI SDK: 28 components, 6 themes (Dark/Light/System/Frosted/Cyberpunk/Aurora) |

## Memory

Decisions, blockers, and constraints are captured in `.dejavue/` — run
`dejavue context` for the boot packet and `dejavue recall <query>` to search.
