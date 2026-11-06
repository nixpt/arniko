# What bliss-engine and mustang could borrow from Rune

> Companion to [`RUNE_BORROW.md`](RUNE_BORROW.md), which borrows Rune concepts **into
> arniko**. This note is the other direction: patterns worth porting **into the engine
> repos** — [`nixpt/bliss-engine`](https://github.com/nixpt/bliss-engine) (packages:
> `bliss-dom`, `bliss-paint`, `bliss-shell`, …) and [`nixpt/mustang`](https://github.com/nixpt/mustang).
> Analysis only; the changes land in those repos, not arniko.

Reference read: Rune (`crates/rune`, `rune-motion`, `rune-fx`, `rune-light`,
`crates/rune/src/tooling`). Targets read locally: `projects/libs/bliss-engine/packages/*`,
`projects/mustang/src/*`.

## What already exists (so we don't re-propose it)

- **bliss-dom already animates CSS.** `stylo.rs` tracks transitions/animations
  (`AnimationState::{Pending,Running,Finished}`, `has_css_animations` /
  `has_css_transitions`), `is_animating` (`document.rs:1567`) drives per-frame redraw, and
  `resolve(time)` (`resolve.rs:41`) is the animation clock. Fling scrolling exists
  (`events/pointer.rs::generate_fling`, `ScrollAnimationState::Fling`). `position: sticky` is
  in layout.
- **mustang has the effect model.** Typed `Effect` (Blur/Transform/Color/DropShadow params),
  `SceneScheduler` with per-region dirty tracking, and skip/completion
  (`SceneEffectResult::is_complete`). Its **animation layer is tween-only**:
  `animation/easing.rs` (`EasingFunction::apply`) + `AnimatedProperty::interpolate(progress)`
  — linear `from..to`, no spring, no retarget.
- **bliss-paint** owns shadows (`render/box_shadow.rs`), gradients (`gradient.rs`), layers
  (`layers.rs`); text is pushed as `anyrender` glyph commands (`text.rs`).

## Borrow candidates

### A. Springs + velocity-preserving retarget — bliss-dom and mustang

*Rune:* analytic `Spring(duration:bounce:)`, closed form for every damping regime, retarget
that keeps position **and** velocity, a reduced-motion policy and a time scale
(`crates/rune-motion`).

*Gap:* bliss-dom's transitions run on stylo's CSS timing (cubic-bezier/steps); mustang's
animation is linear `interpolate(progress)`. Neither has a spring timing function or a
retarget that preserves velocity.

*Propose:* a small shared **motion math** crate — arniko's `crates/arniko/src/motion`
(`Spring`/`Curve`/`Track`) is a working reference — that (a) bliss-dom offers as a
spring-equivalent timing function for CSS transitions and for programmatic ones, and (b)
mustang's animation layer adopts so an `AnimatedProperty` retargets instead of restarting.

### B. Stop stripping CSS transitions — arniko (feeds the engine)

*Finding:* arniko's `css::visitor::transform` rebuilds the stylesheet and **drops**
`transition*`, `animation*` and `@keyframes` (records them in `dropped_features`, then
`continue`) — so the document never sees them and bliss-dom's animation engine never runs.
This is the cheap, in-arniko path to engine-driven motion: stop dropping them (or gate it),
and the engine that already exists does the work.

**✅ Done.** `normalize` now **keeps** `transition`/`animation`/`@keyframes` by default
(they reach bliss-dom's animation engine); `NormalizeOptions { keep_animations: false }` /
`normalize_with_options` restores the old drop-and-record behaviour. `@keyframes` bodies are
passed through verbatim so their nested steps aren't mis-parsed. Tests: keep-by-default,
opt-out-drops, and arniko's own component animations (`arniko-spin`, `arniko-pulse`) survive
normalization.

### C. Headless agent tooling — bliss-shell + bliss-dom

*Rune:* a fixed virtual clock, an element-tree JSON dump (`--tree`), a synthetic input script
(click/type/key/scroll/fling), motion traces (`crates/rune/src/tooling`).

*Gap:* bliss-dom has only stdout debug (`debug.rs::debug_log_node` / `print_taffy_tree`);
bliss-shell has no headless script or virtual clock.

*Propose:* a JSON tree dump (arniko's `crates/arniko/src/inspect.rs` is a reference
serializer), a virtual clock, and synthetic input — making the engine verifiable headlessly by
agents.

### D. Scroll polish — bliss-dom

*Rune:* virtual rows, scroll edge fade, sticky with a stuck-shadow, tuned fling decay,
scroll-snap / overscroll, cached subtrees.

*Gap:* bliss-dom has sticky and a fling; no virtualization, edge fade, snap, overscroll, or
cached-subtree rasterization.

*Propose:* edge fade and scroll-snap first (small, high-visibility); virtualization and
cached subtrees later (larger).

### E. Effect breadth + determinism — mustang

*Rune-fx:* many effect kinds, typed params, **deterministic given `(time, seed)`**, and a
layer skipped when intensity is 0 / its one-shot finished / it is off screen.

*Gap:* mustang has typed params and the skip; it lacks kind breadth and a determinism
contract.

*Propose:* a `(time, seed)` hashing contract so a headless frame is reproducible, a layer
stack, and a few ported kinds (glow, pulse) before the long tail.

### F. SDF lighting — mustang (or the renderer)

*Rune-light:* every element is an analytic rounded rect, so the frame is a 2D distance field;
occluders become slabs and lights sphere-trace soft shadows, bevels and speculars.

*Gap:* mustang has blur/shadow effects but no lighting pass. Speculative and larger; depends
on the element rounded-rect geometry Bliss already provides.

### G. Out of both trees — anyrender / vello

Rune's single instanced uber-quad (one draw call/frame), glyph-atlas ping-pong pool, the
CoreGraphics-matched glyph pipeline with font-smoothing modes, and sRGB-vs-linear blending
are **renderer internals**. bliss-paint pushes `anyrender` commands implemented by
`anyrender_vello`, so those belong to the `anyrender`/vello owners, not bliss-engine or
mustang. Noted here so they aren't mis-scoped into the wrong repo.

## Filed

Tracked in the engine repos so the items aren't lost:

- **bliss-engine** ([issues enabled on 2026-11-06](https://github.com/nixpt/bliss-engine) — they were off):
  [#15 headless tooling](https://github.com/nixpt/bliss-engine/issues/15),
  [#16 spring timing](https://github.com/nixpt/bliss-engine/issues/16),
  [#17 scroll polish](https://github.com/nixpt/bliss-engine/issues/17).
- **mustang**:
  [#10 springs + retarget](https://github.com/nixpt/mustang/issues/10),
  [#11 effect breadth + determinism](https://github.com/nixpt/mustang/issues/11).

## Priority (cheapest high-value first)

1. ✅ **(arniko)** stopped stripping `transition`/`animation`/`@keyframes` → the engine's
   animation runtime now runs them.
2. **(bliss-shell)** headless tree dump + virtual clock + synthetic input.
3. **(shared)** spring timing + retarget in bliss-dom and mustang.
4. **(mustang)** effect kinds + `(time, seed)` determinism.
5. **(bliss-dom)** scroll edge fade / snap; later virtualization.
6. **(mustang / anyrender)** lighting and renderer internals.
