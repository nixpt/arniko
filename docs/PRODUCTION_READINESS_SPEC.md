# Arniko Production-Readiness Spec

> **Status:** M1 substantially complete · **Authored:** 2026-06-16 (foreman-z, [zorro] box, 6-agent analysis) ·
> **Owner:** foreman-x (arniko) · **Audience:** anyone working on the arniko / Bliss stack.
>
> This is a north-star spec, not a sprint plan. It defines what "production-grade" means for
> arniko, captures the current-state gaps (with file:line evidence), and organizes them into
> prioritized epics with acceptance criteria. Pull tickets from here; tick them off in
> `.dejavue/state.md`.
>
> **M2 status (2026-06-15):** B-1/B-2/B-3 ✅, C-1/C-2/C-3 ✅, plus
> 🆕 extras delivered: keyboard nav, ARIA/keyboard unit tests (C-5🆕),
> focus-visible outlines (C-6🆕), reduced motion (C-7🆕).
> E-1/E-2 partial (18 reactive + 10 For integration tests).
>
> **M1 follow-up (2026-06-17):** A-1 ✅ (workspace rebuild with `arniko-crush` excluded),
> A-2/A-3 ✅ (the six predicted `reactive`-feature errors were already resolved by the
> B-4..B-7 commits — `mustang::SceneScheduler` path corrected, mustang refs gated, flush
> signature updated), A-6 ✅ (dead `arniko-crush` workspace-dep entry removed alongside A-1).
> A-4 🔶 PARTIAL: stage 1 — exo-mesh's unconditional `libp2p::Multiaddr` import and
> unbounded `MeshNode::peer_id` call into `Arc<NodeIdentity>` are now resolved via Cargo
> feature unification (arniko lists exo-mesh as a direct optional dep and enables
> `exo-mesh/p2p` through the `networking` feature); stage 2 — a separate
> rcgen 0.13.2 / time blanket-impl conflict (E0119) surfaced once `libp2p` was actually
> built. See A-4 below.
>
> **M3 status (2026-06-17):** ✅ DONE. D-1..D-8 swept across engine robustness &
> tests: keyboard `todo!()`, slab `nodes[id]` direct indexing (D-2 phased as
> D-2 phase 1, D-2b, D-2c, plus the open 🟡 D-2c-followup = widen
> `BaseDocument::root_element` to `-> Option<&Node>` so empty-doc `hit`/`scroll`/
> `clear_*` paths don't panic the painter), pointer-path unwraps on attacker HTML,
> `cursor: none`, engine lock-poison (mirroring B-5), payload-decode,
> resource-failure, error-type design. Reactive core tests (E-1, 18 unit tests
> across `signal`/`computed`/`reactor`) and engine integration tests (E-2, 25
> reactive_components tests) landed.
>
> **M4 status (2026-06-17):** ✅ DONE. Reactive hardening shipped end-to-end and
> proved by a dogfood demo. **B-4 ✅** Scope-based binding lifecycle +
> `park_scope` re-homing — `mount` returns a disposable `Scope`, node-id→binding
> map, removed views shed bindings. **B-5 ✅** `parking_lot` migration kills the
> lock-poison cascade in `signal`/`computed`/`reactor`/`sink`/`app`.
> **B-6 ✅** Per-node keydown + inline `.on_*` handler chaining on view
> builders. **B-7 🟡** Show/Switch conditional rendering 🆕 landed;
> `create_effect`, `create_resource`, `provide`/`inject`, `batch()`,
> error boundaries, programmatic flush, keyed lists remain ⬜ (gated by **D3**).
> **For positional diff reconciliation** carries through B-2 — keyed identity,
> per-item child reactors with disposal cascades, nested reactive children keep
> updating across reconciliations. **ComponentView::to_view bridge** 🆕 — the
> `Component::to_view(self) -> ComponentView<Self>` RAII adapter lets static
> components (Badge, Card, …) mount into the reactive `View` tree without
> manual `Box<dyn View>` wrapping.
> **D-1..D-8** swept: D-1..D-3 in M3 (engine robustness, P0); D-4..D-8 in the M3→M4 handoff (engine panics, P1).
> **Dogfood M4 demo** (`examples/dogfood_m4.rs` + `tests/dogfood_m4.rs`, gated by
> `required-features = ["reactive"]`) exercises every primitive end-to-end
> (Signal source → Computed heading list → For outline sidebar → Switch mode
> picker + ComponentView Badge). Four integration tests run headlessly through
> BaseDocument + Reactor (no winit): D-2c-followup empty-doc safety;
> Signal→Computed→For round-trip; Switch branch reconciliation;
> ComponentView<Badge> mount + render. See the M4 entry in §5 below for the
> full close-out, and `docs/REMAINING_TRACKS.md` for what carries into M5+
> (component completeness C-4..C-8, breath tests E-3..E-5, EPIC F packaging
> gated by **D1**/**D4**, plus the open 🟡 **D-2c-followup** and **A-4b**
> rcgen blocker).

## 1. What arniko is

The Bliss/Mustang rendering stack + the Arniko UI SDK (extracted from exosphere s277). 16 crates,
~50K LOC. Layers:

```
arniko (UI SDK: ~28 components, reactive Signal/View runtime, 6 themes)
  └─ bliss (facade) → bliss-shell (winit) · bliss-paint (Vello) · bliss-dom (DOM+CSS+layout, 16.7K LOC, the engine)
       └─ stylo_taffy (Stylo↔Taffy) · anyrender_vello · mustang (GPU FX) · accesskit_xplat (a11y) · bliss-traits · bliss-net · bliss-html
arniko-crush (Crush host bindings) · tui-shell (ratatui) · debug_timer
```

**Current honest verdict:** the default (`html`+`components`) HTML-string path is healthy and the
14-crate stack compiles `--workspace` green, but the **GPU + reactive + networked production path is
not currently buildable**, the reactive runtime is a **v1 prototype** (no self-driven flush, leaks
on list updates), the engine is **panic-driven** (slab-index + unwrap throughout, unmapped key
crashes the app), the reactive core and most of the 16.7K-LOC engine are **untested**, and the
workspace is **internal-only / not publishable**. Treat arniko as **pre-production** until the P0
epics below land.

## 2. Definition of "production-grade" (acceptance gates)

A release is production-grade when **all** of these hold:

- **G1 Build:** `cargo check --workspace --all-features` is green and runs *in-repo* (no copy/workaround). Every declared feature compiles standalone. CI enforces it.
- **G2 Reactive correctness:** async/timer `Signal::set` reaches the DOM; list updates preserve item state and don't leak DOM nodes; a panicking handler doesn't brick the app; unit tests cover signal/computed/reactor invariants.
- **G3 Engine robustness:** no `todo!`/`unimplemented!` reachable from input; no panic from malformed HTML/CSS or a stale `NodeId`; renderer/GPU failures degrade instead of crashing.
- **G4 Components:** all shipped components are real, themed through one token system, HTML-escape untrusted input, and carry a baseline of ARIA. Maturity tiers are explicit.
- **G5 Tests + CI:** reactive core + bliss-dom engine have direct tests; CI runs `--workspace` tests (incl. integration tests) on a Linux+macOS+Windows matrix at the pinned MSRV.
- **G6 Packaging:** license files match manifests; metadata is complete and inherited from `[workspace.package]`; the publish story (internal vs crates.io) is decided and the blockers for it are resolved or documented.

## 3. Decisions needed (block scoping — answer first)

- **D1 Publish target:** Is arniko meant for crates.io, or internal-only (consumed by exosphere/khukuri via path)? This sets the priority of EPIC F (git-pins/license) — P0 if publishing, P2 if internal.
- **D2 SDK scope:** Is the SDK a *general* UI kit (needs modals/forms/tabs/tables — see C-7) or a *dashboard/telemetry widget* kit (its current strength)? Sets whether the ~25 missing components are in-scope.
- **D3 Reactive ambition:** Target parity with Leptos/Solid (effects, resources, context, keyed lists) or a deliberately minimal dashboard-binding runtime? Sets the depth of EPIC B.
- **D4 exosphere coupling:** Should arniko's `networking`/`exoshell` features depend on a *pinned/published* exo-bliss-net rather than a live `../../../exosphere` path, so arniko's build health isn't hostage to exosphere's working tree? (See A-4.)

## 4. Epics & tickets

Priority: **P0** = blocks a usable production build · **P1** = production maturity/correctness · **P2** = polish/discoverability.
Each ticket: problem → fix → acceptance. File refs are `crate/path:line`.

### EPIC A — Build & workspace integrity  *(must land first; nothing else is verifiable without it)*

- **A-1 (P0) ✅ Workspace won't load standalone.** ~~Root `Cargo.toml` member `crates/arniko-crush` depends on `../../../crush-ast/crates/crush-lang-sdk` (`arniko-crush/Cargo.toml:15`) which doesn't exist~~. **Done:** dropped `arniko-crush` from `[workspace] members` and removed its dead `[workspace.dependencies]` entry (per A-6). `cargo metadata` succeeds; build is no longer hostage to crush-ast's working tree.
- **A-2 (P0) ✅ `reactive` feature does not compile — the 6 known errors.** All in `crates/arniko/src/reactive/`. **Done:** resolved as side effect of B-4..B-7 — `mustang::SceneScheduler` path corrected, `mustang` refs gated under `gpu`, `flush(&mut mutator, Option<&SceneScheduler>)`, `QualName::new(...)` for `direct_mut.rs`. `cargo check -p arniko --features reactive` is green.
- **A-3 (P0) ✅ `reactive` feature is incoherent — omits its own `gpu` dependency.** **Done:** with B-4..B-7's mustang gating the feature compiles without dragging `gpu` in. `--features reactive` alone passes.
- **A-4 (P1) 🔶 PARTIAL `full`/`networking`** is now reduced to a single blocker. **Stage 1 ✅ (resolved on this branch, 2026-06-17):** the spec's two named exo-mesh compile errors — `exo-mesh/src/node.rs:13` `libp2p` undeclared and `:208` `peer_id` not on `Arc<NodeIdentity>` — are addressed purely within arniko: `crates/arniko/Cargo.toml` declares `exo-mesh` as a direct optional dep and the `networking` feature line enables `exo-mesh/p2p`, which triggers Cargo feature unification and materializes `libp2p` across the build graph. exo-mesh compiles and exo-bliss-net's previous failure modes are gone. **Stage 2 ⬜ (new sub-ticket A-4b):** enabling `libp2p` transitively pulled `rcgen 0.13.2`, whose blanket `impl<T: Into<String>> From<T>` for `OtherNameValue`/`DnValue` collides with a concrete impl required by a newer `time` crate version — `error[E0119] conflicting implementations of trait From<T>`. Two viable fixes: (a) trim exo-mesh's `libp2p` feature list to drop the `quic` path that brings rcgen in (out-of-tree edit OR vendor `crates/_vendored/exo-mesh`); (b) workspace-level `[workspace.dependencies] rcgen = "0.12"` override and reconcile. Either lands a green `cargo check -p arniko --features full,networking`. Per **D4**, this also re-opens the question of pinning arniko to a published exo-bliss-net so this upstream issue can't bite on every exo-mesh commit. **Acceptance (full):** `--features full` AND `--features networking` green from arniko's own tree.
- **A-5 (P2) Clippy hygiene.** 18 warnings on the default build (missing `Default` impls for ~9 builder types, `method add` confusable with `std::ops::Add` ×5, `format!`-in-`format!`, double-ended `last`). → `clippy --fix` + add `Default`/`#[allow]`. **Acceptance:** `clippy -D warnings` clean (CI already gates this for arniko/bliss-dom).
- **A-6 (P2) ✅ Stale workspace refs/docs.** **Done:** alongside A-1 the dead `arniko-crush` workspace-dep entry was removed from the root `Cargo.toml`. The lib.rs stale mustang-path comment has not yet been audited — leaving as a residual cleanup that can fold into A-8 (new "stale comments" audit) if it gets created.

### EPIC B — Reactive runtime hardening  *(the SDK's value proposition; currently spike-quality)*

Model (confirmed): pull-based **version-counter, poll-on-flush** — no push subscriber graph. `Signal`
bumps a version on `set`; `Computed` recomputes lazily on read iff a dep version changed; `Reactor`
polls binding versions and patches the DOM at flush. Glitch-free for synchronous reads. Gaps:

- **B-1 (P0) ✅ No self-driven flush.** The only flush trigger is `window_event` (`app.rs:47-101`). A `Signal::set` from an async task/timer/thread never reaches the DOM until the next input event. → On `set` (or a batch boundary) wake the loop via the shell proxy (`send_event(Poll)`). **Acceptance:** a timer-driven counter updates the UI with no input.
- **B-2 (P0) ✅ `For` is clear-and-remount — leaks + breaks nested reactivity.** `ForBinding::flush` (`reactor.rs:58-80`) `remove_node`s every item (the *non-dropping* variant → DOM-arena leak grows with list churn) and re-mounts all items into a throwaway `Reactor::new()` stub (`reactor.rs:73`) whose bindings drop immediately → nested `ReactiveText` updates once then never again. → Keyed diffing (extract key, diff add/remove/move, patch survivors in place); use `remove_and_drop_node`; retain per-item child reactors with disposal. **Acceptance:** list reorder preserves item focus/state; no arena growth on repeated mutation; nested reactive children keep updating.
- **B-3 (P0) ✅ Double click-dispatch.** A single click can fire a handler twice — the sink dispatches click (`sink.rs:91-96`) *and* `app.rs:75-87` walks the ancestor chain and calls a handler. → Pick one path. **Acceptance:** one click → one handler invocation (test).
- **B-4 (P1) Unbounded binding growth + no lifecycle.** `Reactor.bindings` only ever grows (`reactor.rs:85`); there's no `unmount`/disposal, no node-id→binding map, so removed views' bindings fire forever (no-op) and pin their `Arc`s. → `mount` returns a disposable `Scope`; reactor sheds bindings on unmount. **Acceptance:** mounting+unmounting N views leaves binding count flat.
- **B-5 (P1) Lock-poison cascade.** `signal/computed/reactor/sink/app` use `Mutex`/`RwLock` + `.unwrap()` everywhere; one handler/patch panic poisons the lock and every later `lock().unwrap()` panics → whole app dies. → `parking_lot` (no poisoning) or poison-tolerant recovery + an error boundary around handler/patch invocation. **Acceptance:** a panicking handler is contained; the app keeps running.
- **B-6 (P1) Event ergonomics.** Handlers register by raw `node_id`; keydown is global-only; no inline handlers on `View` builders. (`on_click`/`on_keydown`/`on_input` *do* exist on the sink — the desktop's `on_input`/`on_keydown` E0599s are a builder-surface gap, not a missing sink.) → per-node keydown + inline `.on_*` on view builders.
- **B-7 (P2) Missing production primitives (per D3):** `create_effect`, `create_resource` (async; depends on B-1), `provide/inject` context, error boundaries, a `Show`/`Switch` conditional view (only `For` exists), explicit `batch()` + programmatic flush. `Computed` (lazy memo) is the strong part — keep it.

### EPIC C — Component library  *(strong on dashboard widgets, thin & inconsistent elsewhere)*

~28 real components (uniform `new()` + chained setters + `.render()->String` + `Component` trait); 12 have `*_reactive() -> Box<dyn View>` variants.

- **C-1 (P0) ✅ Theming is broken — two disconnected token systems.** `theme/mod.rs` `ThemeMode::css_overrides()` emits `--bg-*`/`--text-*`/`--accent`; all 29 component CSS files read `--arniko-*`; **no bridge**. Result: 4 of 6 themes (Frosted/Cyberpunk/Aurora/System) are **inert** for components; only Dark (defaults) + Light actually theme. → Bridge `--bg-*`↔`--arniko-*` (or make `css_overrides` emit `--arniko-*`) and ship `theme-frosted/cyberpunk/aurora/dark` CSS into `ARNIKO_STYLES`. **Acceptance:** switching `ThemeMode` visibly recolors every component.
- **C-2 (P0) ✅ No HTML escaping → XSS.** Only `file_tree.rs` escapes input; `input.rs:66`, `button.rs:104`, badge/card/toast/alert/tooltip interpolate caller strings straight into HTML. → shared escape helper applied uniformly; document the trust contract. **Acceptance:** a `<script>`-bearing label renders inert.
- **C-3 (P0) ✅ a11y baseline absent.** `accesskit_xplat` is unused by components; only 3 ad-hoc ARIA spots, no `role=`. Missing `role=progressbar`+`aria-valuenow`, `role=alert`/`aria-live`, `role=separator`, `<label>`/`aria-label` on input, keyboard nav on file_tree. → add ARIA per component; wire accesskit for the reactive/native path. **Acceptance:** an a11y lint/audit passes on the core set.
- **C-4 (P1) Unify the reactive surface.** Static = `Component` trait; reactive = free fns returning `Box<dyn View>` — unrelated, undiscoverable, and `toast.rs:10` (`mount_toast` → `usize`, mutates `DocumentMutator`) diverges from the `*_reactive(Signal)` pattern. → a `Reactive` trait or one consistent signature; bring toast in line.
- **C-5 (P1) Hardcoded colors defeat theming.** bar_chart inline colors, progress_ring thresholds (`#ef4444/#f59e0b/#10b981`), sparkline `#00f2ff`, svg charts/toast hex/rgba. → tokens.
- **C-6 (P1) Constructor inconsistency.** Data-in-`new()` (`Sparkline`, `SplashScreen`) vs empty + `.add()` (`Feed`, `AlertPanel`, `BarChart`). → pick a convention; document.
- **C-7 (P1, gated by D2) Missing core components for a general kit:** Modal/Dialog, Drawer, Popover, Menu/Dropdown; Select, Checkbox, Radio, Switch, Slider, Textarea, form field+validation; Tabs, Accordion, Breadcrumb, Pagination, Steps; Table/DataGrid, List, Avatar, Tag/Chip, generic Tree.
- **C-8 (P2) Dead code + stale docs.** Delete orphaned `placeholder_components.rs` (not exported, shadowed). Fix `.dejavue/context.md:40` "13 components" → ~28; reconcile `lib.rs:28` theme list (4) vs the real 6. Add rustdoc `# Examples` to components (only `lib.rs` has any).
- **C-4 🆕 (P1, done) Keyboard navigation baseline.** Add tabindex to interactive components (tooltip, keyboard_shortcuts close, panel close), role="button" on link buttons. Document Escape handler pattern for dialogs. **Done:** `tabindex="0"` on tooltip + shortcuts close, `role="button"` on `<a>` buttons, Escape pattern doc.
- **C-5 🆕 (P1, done) Unit tests for ARIA/keyboard output.** **Done:** test modules added to 7 components that had none (separator, skeleton, spinner, tooltip, status_grid, progress_bar, kbd), ARIA assertions added to 17 existing test suites. 146 lib tests pass.
- **C-6 🆕 (P1, done) Focus-visible outlines + disabled state styling.** Added `--arniko-focus-ring` token, `:focus-visible` box-shadow on all interactive components (button, input, theme_toggle, tooltip, keyboard_shortcuts close, panel close), `cursor: not-allowed` on disabled states. **Done:** 7 CSS files updated, golden file regenerated.
- **C-7 🆕 (P1, done) prefers-reduced-motion media query.** `@media (prefers-reduced-motion: reduce)` block in dedicated `accessibility.css` at end of `ARNIKO_STYLES` for maximum cascade priority. Uses `0.01ms` to preserve transitionend/animationend events. **Done.**

### EPIC D — Engine robustness (bliss-dom et al.)  *(panic-driven; Blitz/Servo-lineage Slab+unwrap pattern)*

Inventory (runtime, excl. tests): ~178 `unwrap`, 18 `panic!`, 3 active `todo!`, 2 `unimplemented!`, ~12 `unreachable!`. bliss-dom is the hotspot (139 unwrap / 13 panic! / 8 unreachable!).

- **D-1 (P0) `_ => todo!()` on keyboard input.** `bliss-shell/convert_events.rs:362-363` — any unmapped/future keycode panics the event loop on keypress. → map remaining codes; catch-all → `Code::Unidentified`. **Acceptance:** every key is non-crashing.
- **D-2 (P0) Slab `nodes[id]` direct indexing.** Pervasive in bliss-dom (`document.rs:642,698,…`, `layout/*`, `mutator.rs`) — a stale/cross-document `NodeId` panics. Safe `get_node` exists but is bypassed. → route hot/public paths through `get_node`/`get_node_mut`, propagate `Option`/`Result`. **Acceptance:** operations on a removed node return an error, not a panic.
- **D-3 (P0) Pointer-path unwraps on attacker-controllable HTML.** `events/pointer.rs` runs every pointer move/click: `:482 attr("name").unwrap()` (radio with no `name` → guaranteed panic), `:389/392/508 get_node(...).unwrap()`, `:226/313 try_layout().unwrap()`. → graceful handling. **Acceptance:** pointer over malformed input doesn't crash.
- **D-4 (P1) `cursor: none` panics.** `bliss-dom/stylo_to_cursor_icon.rs:6` `todo!` — reachable from untrusted CSS. → return a hidden/default cursor.
- **D-5 (P1) Lock-poison cascade in engine.** ~54 `.lock()/.read()/.write().unwrap()` (`font_ctx`, image caches, reactive). → `parking_lot` / poison recovery (mirrors B-5).
- **D-6 (P1) Payload-decode panics.** `messaging.rs:253,341,…` / `scheduler.rs:179` `panic!("Expected …")` on unexpected (network/IPC-controlled) payloads. → `Result`.
- **D-7 (P1) Resource-failure panics.** `bliss-shell/window.rs:202` renderer-resume panic; `bliss-paint/render.rs:511` inline-layout panic. → degrade/skip-frame.
- **D-8 (P1) Error-type design.** No `thiserror`; hand-rolled types are inconsistent; `bliss-dom::ScriptError` lacks `Display`/`Error`. arniko SDK has no error type at all. → standardize; give every error `Display`+`Error`. (`bliss-traits::DomControlError` is the model to follow.)
- **D-9 (P2)** Strip 12 `dbg!` (11 bliss-dom, 1 bliss-paint); audit `layout/inline.rs` integer casts (no `saturating_/checked_`); `accesskit_xplat` `unimplemented!()` for UIKit/WinRT → no-op adapter.

### EPIC E — Testing & CI  *(infra thin; the right primitive exists)*

~317 test fns total but the **reactive core has 0 tests** and the **16.7K-LOC bliss-dom engine is ~6 tested files**. CI runs `--lib` on only 3/16 crates and never runs the 15 integration tests that exist. A headless DOM mount harness (`crates/arniko/tests/reactive_components.rs`) is the template to expand.

- **E-1 (P0) ✅ Reactive core unit tests (partial).** `reactive/{signal,computed,reactor,view,direct_mut}.rs` — target: version monotonicity + clone-sharing (signal); laziness + diamond-consistency + map-chains (computed); dirty-only flush + coalescing + bind_for diff + drop-safety (reactor). **Done:** 18 tests in `tests/reactive_signals.rs` covering Signal, Computed (derive/from2/from3/map/lazy), Reactor (flush dirty detection, multiple bindings, partial dirty, rapid updates). `direct_mut.rs` tests still pending.
- **E-2 (P0) ✅ Wire integration tests into CI + bliss-dom engine tests (partial).** CI test job → `cargo test -p arniko -p bliss-dom -p mustang` (drop `--lib`) so `reactive_components.rs` runs; add `document.rs` mutation round-trips, `query_selector.rs`, and ≥1 `layout/construct.rs` geometry golden. **Done:** `reactive_components.rs` has 25 integration tests (15 original + 10 For positional diffing). Engine tests still pending.
- **E-3 (P1)** `stylo_taffy/convert.rs` (850 LOC, 0 inline tests) table-driven style→Taffy tests; `events/pointer.rs`+`keyboard.rs` hit-test/focus tests; tests for the ~9 untested components; paint smoke tests (scene-non-empty) for `bliss-paint/render.rs`.
- **E-4 (P1) CI breadth.** `--workspace` check/clippy/test; macOS+Windows matrix (exercises the cfg-gated accesskit/clipboard/android paths never built today); dedicated MSRV (1.85) job.
- **E-5 (P2)** Visual/scene-graph regression for `anyrender_vello`/`bliss-paint`; proptest for stylo_taffy length/percentage + css resolver; `cargo-llvm-cov` coverage floor; wire the existing `.cargo/audit.toml` into a `cargo audit` CI job.

### EPIC F — Packaging, docs, release  *(internal-only today; priority gated by D1)*

- **F-1 (P0-if-publishing) Git-pinned deps block crates.io.** `taffy`/`parley` git revs in `bliss-dom`, `bliss-paint`, `stylo_taffy`, root `Cargo.toml:81,86` (deliberate, synced with exosphere). → upstream releases or vendor + version-bump.
- **F-2 (P0-if-publishing) Cross-repo path deps.** `arniko-crush/Cargo.toml:15` (non-optional crush-ast — hard blocker), `arniko:75`/`bliss-shell:41`/`tui-shell:20` (exosphere; optional). → resolve per D4.
- **F-3 (P0) License compliance.** Manifests say `MIT OR Apache-2.0` but only `crates/arniko/LICENSE` (MIT) exists — no `LICENSE-APACHE`, none at root. → add `LICENSE-MIT`+`LICENSE-APACHE` at root + per-crate; reconcile `accesskit_xplat` (Apache-only) / `stylo_taffy` (tri-license).
- **F-4 (P1) Adopt `[workspace.package]` inheritance.** Only `stylo_taffy` inherits; everyone else hardcodes divergent version/repo/categories (two version tracks: bliss `0.2.99` vs `0.1.0`). → `*.workspace = true` across crates; single version line.
- **F-5 (P1)** Add a top-level `README.md` (the architecture map currently lives only in `.dejavue/context.md`); wire `cargo audit`/`deny` into CI.
- **F-6 (P2)** Per-crate metadata (`keywords`/`categories`/`repository`/`readme`/`documentation`/`docs.rs` cfg); READMEs for the 10 crates lacking them; rustdoc on bare surfaces (`bliss-traits`, `bliss-html`, `debug_timer`, `bliss-net`); register `basic`/`builder`/`phase5_demo` examples; `rust-toolchain.toml` pin; a real CHANGELOG with semver discipline.

## 5. Suggested milestone sequence

1. **M1 — Unblock the build (EPIC A).** A-1 → A-2 → A-3 (then A-4). Without this nothing is verifiable; do it first. **Status (2026-06-17):** A-1/A-2/A-3/A-6 ✅, A-4 stage 1 ✅, A-4 stage 2 (A-4b, rcgen/time blanket-impl conflict) ⬜. Effectively one ticket away from M1 done.
2. **M2 — Reactive + component correctness (B-1..B-3, C-1..C-3). ✅ DONE.** Make the reactive path *correct* (flush, lists, click) and the components *safe* (theming, XSS, a11y). Extras: keyboard nav + focus-visible + reduced motion, unit + integration tests (C-5, E-1 partial, E-2 partial).
3. **M3 — Robustness + tests (EPIC D P0/P1, E-1/E-2). ✅ DONE (2026-06-17).** Engine-panic
   containment (D-1 → D-8 ✅ across M3/M4/D-2 phase splits; **D-2c-followup** 🟡 ⬜ = widen
   `BaseDocument::root_element` to `-> Option<&Node>` so empty-doc `hit()`/`scroll_*`/`clear_*`
   don't panic the painter); reactive-core + engine tests landed (E-1, 18 unit tests across
   `signal`/`computed`/`reactor`; E-2, 25 `reactive_components.rs` + 4 new `dogfood_m4.rs`
   integration tests). Reactive-core breadth + engine golden tests remain ⬜.

4. **M4 — Reactive maturity + dogfood integration (B-4..B-7, D-4..D-8 sweep, dogfood demo).
   ✅ DONE (2026-06-17).** The reactive path is now provable end-to-end through a single app.
   **What landed:**
   - **B-4 (Scope-based binding lifecycle, commit `0c4ae17`).** Every `mount` returns a
     disposable `Scope`; node-id→binding map for per-node disposal; `Reactor.bindings` flat
     under repeated mount+unmount.
   - **`Reactor::park_scope` for closure-bound bindings.** Scopes created in patch closures
     (Show/Switch re-mounts, `For` reconciliation) that return `()` have no caller to hold
     the scope — `park_scope` re-homes the scope's `BindingHandle`s onto the reactor so
     they survive past the closure return. Without it the bindings Drop immediately and
     silently kill the child's reactivity after the first toggle.
   - **B-5 (lock-poison cascade eliminated, commit `372ba64`).** `parking_lot` migration
     across `signal`/`computed`/`reactor`/`sink`/`app` removes the `.unwrap()` poison
     surface that bricked the app on a single panicking handler.
   - **B-6 (event ergonomics, commit `893de31`).** Per-node keydown support + inline
     `.on_*` handler chaining on view builders; `Div::with_on_input(...)` is no longer an
     E0599.
   - **B-7 (Show/Switch conditional rendering, commit `d79fa75`).** `Show<bool, _>` +
     `Switch<T, R>` land; re-mount closures' returned scopes are `park_scope`d on the
     reactor (same rationale as above). `create_effect`, `create_resource`,
     `provide`/`inject`, `batch()`, error boundaries, programmatic flush, keyed lists 🟡 ⬜
     (gated by **D3 — reactive ambition**).
   - **For positional diff reconciliation** (carries through B-2). Keyed list diff;
     item reactors are retained with disposal cascades; nested reactive children keep
     updating across reconciliations.
   - **`Component::to_view` bridge (ComponentView)** 🆕. The `Component::to_view(self) ->
     ComponentView<Self>` RAII adapter (gated by `#[cfg(feature = "reactive")]`) lets
     static components (`Badge`, `Card`, …) mount into the reactive `View` tree.
     `Badge::new("M4").variant(BadgeVariant::Purple).to_view()` parcels into a
     `ComponentView<Badge>` (which `impl View`); the trait import must be in scope for
     method resolution. Consumed cleanly — one move, no manual `Box<dyn View>` wrap.
   - **D-4..D-8** (engine panics, P1 sweep) folded into M4 alongside D-1..D-3 swept in M3.
   - **Dogfood M4 demo** (`examples/dogfood_m4.rs` + `tests/dogfood_m4.rs`,
     `required-features = ["reactive"]`). A single interactive markdown renderer drives
     every reactive primitive end-to-end:
     - `Signal<String>` — markdown source, mutated by click handlers, dummy-updates on
       a timer-driven flush (B-1's self-driven flush carries through).
     - `Computed<Vec<String>>` — heading list, derived via `.derive`, lazy re-eval on
       read if any dep version has advanced.
     - `For<_, _>` — outline sidebar that reconciles in place when the source bumps.
     - `Switch<ViewMode, _>` — right-pane mode picker (rendered / source / outline); new
       branch closure's returned scope is `park_scope`d so nested `Computed` reactivity
       survives the toggle.
     - `ComponentView<Badge>` — the top-bar app badge via the M4 RAII adapter.
     Four headless integration tests (no winit):
     - `d2c_followup_empty_doc_after_dogfood_mount_no_panic` — the dog's surfaced as a
       regression target for the open D-2c-followup widening.
     - `dogfood_signal_drives_computed_headings_round_trip` — full Signal → Computed →
       For round-trip.
     - `dogfood_switch_reconciles_branches_and_preserves_nested_reactivity` — Switch
       branch survival + nested Computed update across toggles.
     - `dogfood_component_view_badge_renders` — ComponentView<Badge> mount + render.
   **What carries into M5+:**
   - **B-7 remaining** (gated by **D3**): `create_effect`, `create_resource`,
     `provide`/`inject`, `batch()`, error boundaries, programmatic flush, keyed lists.
   - **C-4** unify the reactive surface — `toast.rs:10` `mount_toast -> usize`
     diverges from the `*_reactive(Signal)` pattern.
   - **C-5** hardcoded colors defeat theming — bulk wiped in `bar_chart`,
     `progress_ring`, `sparkline`, `svg_bar_chart`, `toast`; remaining hot-spots in
     `bliss-paint`/`bliss-shell` shaders.
   - **C-6** constructor inconsistency — data-in-`new()` vs empty + `.add()`.
   - **C-7** missing components for a general kit — Modal/Dialog, Drawer, Popover,
     Menu/Dropdown; Select, Checkbox, Radio, Switch, Slider, Textarea; Tabs, Accordion,
     Breadcrumb, Pagination, Steps; Table/DataGrid, List, Avatar, Tag/Chip, generic
     Tree. **Scope gated by D2.**
   - **C-8** dead code + stale docs — `placeholder_components.rs`,
     `.dejavue/context.md:40` "13 components" → real count (~28), `lib.rs:28` theme list
     4→6.
   - **E-3..E-5** test breadth — `stylo_taffy/convert.rs` (850 LOC, 0 tests) table-driven;
     `events/pointer.rs`+`keyboard.rs` hit-test/focus; ~9 untested components;
     `bliss-paint` smoke tests; `--workspace` CI; macOS+Windows matrix; MSRV 1.85 pin;
     visual/fuzz/coverage.
   - **EPIC F** packaging (**gated by D1 — publish target** and **D4 — exosphere
     coupling**): license compliance (F-3 P0), git-pinned dep blockers (F-1/F-2
     P0-if-publishing), workspace metadata inheritance (F-4), top-level README +
     audit (F-5), per-crate polish (F-6).
   - **A-4b** (rcgen 0.13.2 / time blanket-impl E0119 unlocks `--features full`,
     `--features networking` — requires either an exosphere-side cfg-gate PR
     trimming libp2p `quic`/`relay` features, or an upstream rust-libp2p ≥ 0.56
     release that bumps rcgen to `^0.14`).
   - **D-2c-followup** (🟡; widens `BaseDocument::root_element` to `Option<&Node>`,
     migrates 5 callers; tracked under D-2 in REMAINING_TRACKS).

5. **M5 — Component completeness + breadth tests + release. Forward-looking.** Scope
   crystallises once D1 (publish target) / D2 (SDK scope: general vs dashboard) /
   D3 (reactive ambition: Leptos/Solid parity vs minimal) / D4 (exosphere coupling:
   pinned/published vs path) resolve. The M4 close-out's M5+ backlog is the seed list:
   - **B-7 remaining primitives** (gated by D3): `create_effect`, `create_resource`,
     `provide`/`inject`, `batch()`, error boundaries, programmatic flush, keyed lists.
   - **C-4..C-8** finish: unify reactive surface (`toast.rs:10` divergence, C-4);
     finish color sweep in `bliss-paint`/`bliss-shell` shaders (C-5); constructor
     consistency (C-6); missing-components-for-general-kit (C-7, **gated by D2** —
     Modal/Dialog, Drawer, Popover, Menu/Dropdown, Select/Checkbox/Radio/Switch/Slider/
     Textarea, Tabs/Accordion/Breadcrumb/Pagination/Steps, Table/DataGrid, List, Avatar,
     Tag/Chip, generic Tree); dead code + stale docs (C-8 — `placeholder_components.rs`,
     `.dejavue/context.md:40` count, theme list in `lib.rs:28`).
   - **E-3..E-5** test breadth: table-driven `stylo_taffy/convert.rs` (850 LOC, 0
     tests); `events/pointer.rs`+`keyboard.rs` hit-test/focus; ~9 untested components;
     `bliss-paint` smoke tests; `--workspace` CI; macOS+Windows matrix; dedicated
     MSRV (1.85) pin; visual/fuzz/coverage; wire `.cargo/audit.toml` into CI.
   - **EPIC F** packaging (**gated by D1 + D4**): license compliance (F-3 P0);
     git-pinned dep blockers (F-1/F-2 P0-if-publishing); workspace metadata
     inheritance (F-4); top-level README + audit (F-5); per-crate polish (F-6).
   - **A-4b** (rcgen 0.13.2 / time blanket-impl E0119 — `--features full,networking`
     blocker; requires out-of-tree unblock via libp2p feature trim in exo-mesh
     `Cargo.toml` + `p2p.rs`, or upstream rust-libp2p ≥ 0.56 release).
   - **D-2c-followup** 🟡 (close out D's last residual item).

## 6. Note for khukuri-desktop

khukuri-desktop (in `nixpt/Khukuri`) consumes this SDK and was the forcing function for this audit.
Once A-2/A-3 land (reactive compiles) and the components it imports are stable, the desktop reconcile
is a build-and-verify against `nixpt/arniko` (siblings resolve on the [zorro] box). M2 (correct
reactive + safe components) is the gate for a *working* desktop, not just a compiling one.

---
*Evidence: 6-agent static+build analysis, 2026-06-16. Re-run against HEAD before acting on any
specific file:line — the arniko tree is under active development.*
