# Arniko

Unified UI SDK for Exosphere agent capsules, built on the Bliss rendering engine
(forked from [Blitz](https://github.com/DioxusLabs/blitz)).

Arniko provides a component library (28 components, 6 themes), reactive signal bindings,
and a headless HTML+CSS layout engine built on Blitz's Stylo (Servo) + Taffy + Parley stack.

## Architecture

| Crate | Role |
|-------|------|
| `arniko` | Unified UI SDK: 28 components, 6 themes (Dark/Light/System/Frosted/Cyberpunk/Aurora) |
| `bliss-traits` | Shared traits: NetProvider, ImageCache, ShellProvider |
| `bliss-dom` | Headless HTML DOM + Servo/Stylo CSS + Taffy/Parley layout |
| `bliss-html` | HTML5 parser (html5ever sink) |
| `bliss-net` | Default reqwest NetProvider |
| `bliss-paint` | Vello scene painter (kurbo/peniko geometry) |
| `bliss-shell` | winit windowed application shell |
| `bliss` | Top-level re-export |
| `stylo_taffy` | Servo Stylo ↔ DioxusLabs Taffy style bridge |
| `mustang` | GPU post-processor: blur/transform CSS effects (wgpu/vello) |
| `debug_timer` | Phase timing instrumentation |
| `accesskit_xplat` | Cross-platform accessibility (AccessKit) |
| `anyrender_vello` | Vello-backed anyrender implementation |

The `tornado`/`tornado-*` TUI toolkit was extracted to its own repository,
renamed `tui-easy`: https://github.com/nixpt/tui-easy.

## Feature Flags

`arniko` has four feature gates:

| Feature | Enables |
|---------|---------|
| `default` | `html` + `components` |
| `components` | 28 UI components + themes |
| `networking` | HTTP networking via `exo-bliss-net` |
| `full` | All of the above |

## Build

```bash
export CARGO_TARGET_DIR=/build/target-arniko

cargo check -p arniko                        # default features
cargo check -p arniko --features components  # components
cargo check -p arniko --features networking  # networking gate
cargo check -p arniko --features full        # all gates
cargo test  -p arniko --lib                  # 163 tests
cargo test  -p bliss-dom --lib               # 44 tests
cargo test  -p stylo_taffy --lib             # 58 tests
```

## Relationship to Upstream

The `bliss*`, `stylo_taffy`, `debug_timer` and `accesskit_xplat` crates are a hard fork of
[Blitz](https://github.com/DioxusLabs/blitz) by Dioxus Labs, carried here with Arniko's
modifications; the standalone fork is [nixpt/bliss-engine](https://github.com/nixpt/bliss-engine).
`anyrender_vello` is vendored from [DioxusLabs/anyrender](https://github.com/DioxusLabs/anyrender).
The `tornado`/`tornado-*` TUI toolkit (which vendored code from
[ratatui](https://github.com/ratatui/ratatui) and
[xai-org/grok-build](https://github.com/xai-org/grok-build)) now lives in its own
repository, renamed `tui-easy`: https://github.com/nixpt/tui-easy. Upstream copyright
and license terms are preserved in [NOTICE](NOTICE) and each crate's own `NOTICE`.

## License

Most crates: MIT OR Apache-2.0 — see [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE).

Exceptions:

- `accesskit_xplat`: Apache-2.0 only.
- `stylo_taffy`: MIT OR Apache-2.0 OR MPL-2.0 (Stylo/Servo CSS engine is MPL-licensed) — see [LICENSE-MPL](LICENSE-MPL).
- `crates/_vendored/exo-mesh` (vendored source, not built by any crate): OCPL-1.1, Exosphere's runtime-tier license — see [its LICENSE](crates/_vendored/exo-mesh/LICENSE).

Third-party attributions: [NOTICE](NOTICE).
