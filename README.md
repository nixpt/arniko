# Arniko

Unified UI SDK for Exosphere agent capsules, built on the Bliss rendering engine
(forked from [Blitz](https://github.com/DioxusLabs/blitz)).

Arniko provides a component library (28 components, 6 themes), reactive signal bindings,
and a headless HTML+CSS layout engine built on Blitz's Stylo (Servo) + Taffy + Parley stack.

## Architecture

| Crate | Role |
|-------|------|
| `arniko` | Unified UI SDK: 28 components, 6 themes (Dark/Light/System/Frosted/Cyberpunk/Aurora) |
| `exo-bliss-net` | Arniko Crush capsule bindings (networking) |
| `arniko-crush` | Crush capability bindings for Arniko components |
| `mustang` | GPU post-processor: blur/transform CSS effects (wgpu/vello) — [nixpt/mustang](https://github.com/nixpt/mustang), published `arniko-mustang` |

**BLISS-HOME-2 (s506): the Bliss rendering stack moved out of this repo.**
`bliss`, `bliss-traits`, `bliss-dom`, `bliss-html`, `bliss-net`, `bliss-paint`,
`bliss-shell`, `stylo_taffy`, `accesskit_xplat`, and `anyrender_vello` now live
in their own repository, [nixpt/bliss-engine](https://github.com/nixpt/bliss-engine),
published to crates.io at 0.3.0. `arniko` depends on them as ordinary registry
deps (see `[workspace.dependencies]` in this repo's root `Cargo.toml`).

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
cargo test  -p arniko --lib                  # 164 tests
```

Bliss's own test suites (`bliss-dom`, `stylo_taffy`, ...) now live in
[nixpt/bliss-engine](https://github.com/nixpt/bliss-engine) — see that repo's
README for its own build/test commands.

## Relationship to Upstream

The Bliss rendering stack (`bliss`, `bliss-dom`, `bliss-html`, `bliss-net`,
`bliss-paint`, `bliss-shell`, `bliss-traits`, `stylo_taffy`, `accesskit_xplat`,
`anyrender_vello`) is a hard fork of [Blitz](https://github.com/DioxusLabs/blitz)
by Dioxus Labs (and, for `anyrender_vello`, of
[DioxusLabs/anyrender](https://github.com/DioxusLabs/anyrender)) — now
published from its own standalone repo, [nixpt/bliss-engine](https://github.com/nixpt/bliss-engine),
which carries the full upstream attribution in its own NOTICE.
The `tornado`/`tornado-*` TUI toolkit (which vendored code from
[ratatui](https://github.com/ratatui/ratatui) and
[xai-org/grok-build](https://github.com/xai-org/grok-build)) now lives in its own
repository, renamed `tui-easy`: https://github.com/nixpt/tui-easy. Upstream copyright
and license terms are preserved in [NOTICE](NOTICE) and each crate's own `NOTICE`.

## License

MIT OR Apache-2.0 — see [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE).

Third-party attributions: [NOTICE](NOTICE). (The Bliss stack's own license
exceptions — `accesskit_xplat` Apache-2.0-only, `stylo_taffy` with an
MPL-2.0 option — now live in [nixpt/bliss-engine](https://github.com/nixpt/bliss-engine),
not here.)
