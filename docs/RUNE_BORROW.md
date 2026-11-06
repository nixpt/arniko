# Borrowing concepts from Rune

> **Branch:** `agent/opencode/ARNIKO-RUNE-BORROW-1` · **Base:** `origin/main` @ `566cc3d`
> **Status:** design memo (no code yet). Source of truth for the borrow tracks.
> **Provenance:** comparison of arniko against `Rune` (`github.com/anomalyco/rune`, MIT,
> © Anomaly), a native GPU Rust UI framework. This memo borrows **concepts and API shape**,
> not code. Rune is public; any later code port must keep MIT attribution per `NOTICE`.

## Why this exists

Rune and arniko occupy the same category — a Rust GPU UI framework with a component
library and headless rendering — and land on opposite architectures. Rune rolls its own
renderer, view DSL, motion runtime and text pipeline; arniko rides the web stack
(HTML + CSS on the Bliss/Blitz engine) with a reactive layer on top. Reading Rune against
arniko surfaces capabilities arniko simply does not have yet. This memo picks the ones
worth porting and says where they land.

## The two projects

| Axis | **arniko** (this repo) | **Rune** |
|---|---|---|
| Rendering | Bliss engine (Blitz/Servo fork) → vello; `mustang` blur/transform | Own renderer: single instanced uber-quad + native Metal backend (wgpu fallback) |
| Layout | Stylo (Servo CSS) + Taffy | Taffy 0.15 directly via a Rust builder |
| Text | Parley | Parley 0.12 + swash + a CoreGraphics glyph-pipeline port + font-smoothing modes |
| View model | HTML + CSS, reactive signals drive the DOM | Fluent `El` builder: `fn view(s, cx) -> El`, diff retargets property tracks |
| Reactivity | poll-on-flush, version counter, `Computed`, `For`/`KeyedFor`, `Resource`, `ErrorBoundary` | handler queue re-runs the view; `live_only()` escapes the re-run; `Live` high-frequency values |
| Motion | **none** — CSS `transition`/`animation` are parsed then dropped (`crates/arniko/src/css/visitor.rs:14`) | `rune-motion`: analytic springs, retarget-keeping-velocity, presence, keyframes, choreography, reduced-motion |
| Components | 28 (`crates/arniko/src/components/`) | 40+ (`rune-kit`) over a `Kit` token schema |
| Effects | `mustang` (blur/transform) | `rune-fx` (WGSL effect layers), `rune-light` (2D-SDF real-time lighting) |
| Agent tooling | headless HTML+CSS engine (no dump/script harness of its own) | PNG render, element-tree JSON (`--tree`), scripted input, motion traces, virtual clock |
| Perf devices | keyed diff | virtual rows, `memo`, growth stack, cached subtrees, sticky headers |
| Extras | Crush capsule bindings | `rune-knobs`, `procwatch`, `rune-mac`, `rune-mermaid` |

**Key architectural difference for every borrow below:** Rune exposes a Rust DSL; arniko is
CSS-native. So a rune concept must be re-expressed either as a **CSS feature the engine
actually drives** or as an addition to arniko's **reactive layer** — never copied verbatim.

## Adopted tracks

The five tracks selected for this branch. Design-system tokens (`Kit` schema, OKLCH tokens)
were deliberately deferred to a later branch.

### 1. Motion runtime

**Rune concept.** `rune-motion` is pure math the render thread drives once per frame
(`crates/rune-motion/src/lib.rs`):

- `Spring` — SwiftUI's `Spring(duration:bounce:)`, closed form for every damping regime
  (`spring.rs`); `sample(x0, v0, target, t) -> (x, v)`, `settling_duration`.
- A **token palette**: `INSTANT / SNAP / SETTLE / REFLOW / GLIDE / MORPH / POP / DRIFT`,
  each with an `_EXIT` variant, plus aliases `SNAPPY`/`DEFAULT`/`CALM` (`tokens.rs`).
- `AnimationStore` (`store.rs`) — struct-of-arrays tracks, **retargeting that keeps position
  and velocity**, add/merge blending, delays, settle events, zero allocation per frame;
  `set_reduced_motion`, `set_pixel_scale`, `set_node_time_scale`.
- `Scene` (`scene.rs`) — presence (exits that stay mounted until opacity 0), `Transition`
  (incl. blur-replace), **layout animation per edge**, matched geometry, `Presented` output.
- `Curve` (`curve.rs`) — springs, CSS tweens (`Easing`), decay (fling), keyframes.
- `Animatable` (`value.rs`) — f32, `Vec2`, `Rect` (as edges), `Color` (premultiplied OKLab),
  `Corners`, `Transform`, `Blur`, tuples, arrays.

**Arniko gap.** arniko has no motion at all. `transition-property/-duration/-timing-function/
-delay` are parsed into `FeatureType::Transition` metadata and dropped
(`crates/arniko/src/css/visitor.rs:178-184`); two components (`toast.rs:203`,
`progress_ring.rs:178`) emit inline `transition:` CSS that the static engine never animates.

**Proposed borrow — two layers:**

- **(a) Engine-side CSS motion driver.** Make the dropped CSS properties real: a timeline in
  the engine that interpolates the animatable subset of the style tree between computed
  snapshots, with springs as the timing function. This is the CSS-native path and fixes the
  dead inline transitions already shipping in components.
- **(b) Reactive-layer spring API.** `Spring`/`Curve` math plus an `AnimationStore`-style
  driver in `crates/arniko/src/reactive/`, so signal changes can animate (retarget-keeping-
  velocity), the way rune retargets view-diff tracks.
- **Reduced motion** as a first-class token from day one (`REDUCED_FADE` precedent), plus a
  time scale for a settings slider.

**First slice.** Port `Spring` + `Curve` math into a new `crates/arniko/src/motion/`
with the token palette; unit-test `sample`/`settling_duration` against closed-form
expectations. No engine wiring yet. **(✅ landed — `crates/arniko/src/motion/`:
`Spring`/`SpringPair`, `Easing`/`CubicBezier`, `Curve`/`Decay`, the token palette,
and a stateful `Track` that retargets keeping position + velocity; 36 tests.)**

### 2. Agent-native headless tooling

**Rune concept.** `crates/rune/src/tooling.rs` + `tooling/*` is an agent-first harness on a
**fixed virtual clock** (`args.rs`, `headless.rs`, `headless_run.rs`): render to PNG
(`--out`/`--video`), **element-tree JSON dump (`--tree`)**, a **scripted-input file**
(`--script`: `click`, `double-click`, `type`, `key`, `paste`, `scroll`, `fling`), motion-lint
**traces (`--trace`)**, frame strips (`--strip`), keycap overlays (`--keys`), and scene
selection (`--scene`). Determinism is the point: same script → same frames.

**Arniko gap.** arniko's engine is already headless (HTML+CSS → layout), and its Crush-capsule
mission is agent-facing, but it ships no dump/script harness of its own.

**Proposed borrow.**

- **Element-tree JSON dump** — walk the Bliss DOM + resolved layout into a JSON tree
  (tag, key/id, rect, text, computed style subset, inert/focus flags). Highest leverage for
  an agent editing markup blind.
- **Deterministic virtual clock + synthetic input script** — a driver that feeds
  click/type/key/scroll events and renders deterministic frames, so a capsule can self-verify.
- **Motion traces** — once track 1 lands, a trace that reports settle cuts / growing rows,
  rune's own `scripts/motion-audit` analogue.

**First slice.** A `--tree <file>` equivalent: a function that serializes the arniko document
tree + layout to JSON, plus a test that pins its shape. **(✅ landed — `crates/arniko/src/inspect.rs`
(feature `inspect`): `document_tree` / `document_tree_string`; plus the **`arniko-tree` CLI**
(feature `inspect-cli`): reads HTML from a FILE, `--html`, or stdin, lays it out at a
`--width`×`--height` viewport, and writes the JSON to `--out` or stdout. 5 tests. Rects are
block-layout boxes; inline content reports zero.)**

### 3. Reactive / performance model

**Rune concept.**

- **`live_only` escape hatch.** High-frequency values (`Live<T>`, `LiveText`,
  `live_offset`/`live_scale`/`live_opacity`/`live_fill`, `live.rs`) move without re-running the
  view: a handler that only touches them calls `tx.live_only()` and the frame skips view,
  diff and layout. Measured: a 120-frame pinch drops from 13.7 ms / 42k allocs per frame
  (view re-runs) to 2.0 ms / 78 allocs (`crates/rune/README.md`, "live-only").
- **`memo` + `cached`** (`memo.rs`) — `memo(id, version, build)` rebuilds a subtree only when
  its version hash changes; `digest`/`cached` for explicit invalidation.
- **Virtual rows** (`.virtualize(est_row_height)` on a growth stack in a scroll viewport) —
  only an overscanned window is realized; unvisited heights are estimates; measured
  corrections preserve the visible key.
- **Cached subtrees** (`.cache()`) — rasterize a settled subtree into an offscreen layer,
  composite one quad, auto-invalidated from relative draw output.

**Arniko gap.** arniko's Reactor is poll-on-flush with a version counter; `For`/`KeyedFor`
do positional/keyed diff but there is no virtualization, no memoization boundary, and every
flush re-runs the bound subtree. No high-frequency channel exists.

**Proposed borrow.** In priority order: (1) a **live-value channel** that mutates a style
property without a full flush; (2) a **`memo`-style boundary** keyed on a hash of inputs;
(3) **virtualization** for `For` over long lists. Cached-subtree rasterization is a later,
GPU-coupled item.

**First slice.** The `live`/`live_only` channel: a `Live` type + a way to bind it to a
property such that a set schedules a repaint, not a reactive flush. Test: setting a live
value does not bump the Reactor version. **(✅ landed — `crates/arniko/src/live.rs`:
`Live<T>` (f32/u32/i32, lock-free, change-detecting, waker on change) and `SpringBinding`
(smooths a `Live<f32>` onto a `motion::Track`); 7 tests. Wiring a live value to a specific
style property, and the reactor-version assertion, are follow-ups.)**

### 4. Interaction & focus

**Rune concept** (`crates/rune/src/el.rs`, `crates/rune/src/runtime/`):

- Focus: `.focusable(true)`, `.tab_index(n)` (positive overrides, 0 doc order, negative
  pointer-only), `.focus_group()` roving with type-select, `.focus_follows()`,
  `.focus_selected()`, `.focus_ring(w, offset, color)`, `.focus_highlight(color)`.
- `.modal(click_outside, close)` — traps Tab, handles Escape, makes the background **inert**,
  restores the opener; nested modals restore in stack order.
- **A 9-step key-routing order** (AppKit-faithful; `crates/rune/README.md` "Key routing"):
  IME preedit → default button on Return → window `on_key` → focused field → focus navigation
  → text editor → window `on_key` again → selected-text keys → page-scroll fallback.
- Pointer feel: `.press_scale(0.97)` (iOS tile shrink + spring back), `.scale`/`.rotate` about
  `.origin`, press-on-release-inside, double/triple press, `Drag::clicks`.
- `.inert()` — draws/lays out but routes all input past the subtree.

**Arniko gap.** arniko has `accesskit_xplat` for the a11y tree but no first-class focus model,
no roving groups, no modal/inert contract, and no documented key-routing order.

**Proposed borrow.** A focus model expressed in CSS/attributes (a `tabindex`/`role`/`inert`
attribute layer the engine understands), a roving focus-group convention, and a written
key-routing order matching the AppKit rules. `press_scale`/`scale`/`rotate` are motion-track
work (see track 1) and follow it.

**First slice.** A written key-routing spec + an `inert` subtree implementation in the engine
(input routing skips it), mirroring rune's `.inert()`. **(✅ partly landed —
`docs/KEY_ROUTING.md` is the routing spec, and `crates/arniko/src/focus.rs` implements the
focus math it references: `tab_order`/`tab_step` (CSS `tabindex`), `roving` and
`type_select` (focus groups); 6 tests. The `inert` subtree lives in the engine (bliss-dom),
so engine-side work is out of arniko's tree.)**

### 5. Effects & lighting

**Rune concept.**

- `rune-fx`: effect layers — glow, pulse, electricity, fire, motion blur, distortion, plus a
  family of fills (liquid metal, mesh gradient, dither, metaballs, voronoi, halftone, …). Each
  is a WGSL fragment shader on an instanced quad with **typed params**, deterministic given
  `(time, seed)`; a layer at intensity 0 / finished one-shot / off screen is never drawn.
- `rune-light`: real-time lighting — every Rune element is an analytic rounded rect, so the
  frame is an exact 2D distance field; occluders become slabs on the UI plane, lights have a
  3D height + physical radius, sphere-traced through the field for soft shadows / bevels /
  specular highlights / bulb glow. Runs as passes over the frame already drawn.

**Arniko gap (corrected).** `mustang` 0.4 (separate repo) already provides the effect-layer
*model* this track asks for: typed `Effect` (blur / transform / color / shadow params), a
`SceneScheduler` with per-region dirty tracking, and skip/completion
(`SceneEffectResult::is_complete`, `deferred_count`). What it lacks is rune-fx's *breadth*
(glow / fire / liquid-metal / … kinds) and rune-light's SDF lighting — and those belong in
the GPU compositor (`nixpt/mustang`), not in the UI SDK.

**Status — engine-side, no arniko code.** arniko's side of this track is already satisfied by
the existing mustang integration (`arniko::mustang`, `reactive::SceneScheduler`,
`reactive::direct_mut`); a parallel effect model in arniko would duplicate mustang. The
effect kinds and lighting are follow-ups in `nixpt/mustang`.

## Deferred (not this branch)

- **Design-system tokens** — rune's `Kit` schema (roles: `Palette`, `Elevation`, `Type`,
  `Radius`, `Motion`, `Glyphs`) separating core from theme, and `Theme::from_json`
  (themes-as-data) + OKLCH tokens. arniko has 6 themes; a follow-up branch can decide whether
  to adopt the data-driven form.
- Text-quality pipeline (CoreGraphics glyph port, font-smoothing modes, atlas management).
- `rune-knobs` live tuning, `procwatch` memory/CPU gauge, `rune-mermaid`.

## Open questions

1. Motion: engine-side CSS driver (track 1a) first, or reactive-layer springs (1b) first?
   The CSS driver fixes dead inline transitions already shipping; the reactive layer is the
   smaller, testable first slice.
2. Should the motion math live in-tree (`crates/arniko/src/motion/`) or as a sibling crate
   (mirroring `rune-motion`)? In-tree is smaller; a sibling is reusable by bliss-engine.
3. Headless dump: JSON schema is ours to define — pin it with a test before components depend
   on it.

## Next steps

1. Approve the track order; pick the first slice per track.
2. ✅ Track 1's motion math + driver landed (`crates/arniko/src/motion/`, 36 tests).
3. ✅ Track 2 landed (`crates/arniko/src/inspect.rs` + the `arniko-tree` CLI, 5 tests).
4. ✅ Track 3's first slice landed (`crates/arniko/src/live.rs`, 7 tests).
5. ✅ Track 4's first slice landed (`crates/arniko/src/focus.rs` + `docs/KEY_ROUTING.md`, 6 tests).
6. ✅ Track 5: engine-side — arniko's side is already met by the mustang integration; the
   effect kinds and lighting live in `nixpt/mustang`.
7. ✅ Motion→reactive wiring: `Reactor::animate` / `tick` drive a `Track` onto a CSS property
   (`crates/arniko/src/reactive/animation.rs`), and the shell frame loop
   (`ReactiveApplication::about_to_wait` in `reactive/app.rs`) ticks each frame, writes changed
   values, requests a redraw, and schedules `WaitUntil` (~60 fps) while animating. Still next:
   live-value style bindings, and routing keys through the `docs/KEY_ROUTING.md` wheel.
8. Capture each decision in `.dejavue/` as it is made.
