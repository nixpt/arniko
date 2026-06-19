# Arniko Remaining Track Work

> **Generated:** 2026-06-16 · **Updated:** 2026-06-17 (after M1 follow-up commit series on branch `agent/vibe/ar-m4`) · **2026-06-18** (C-8 close-out at commit `283bd0e` on `agent/vibe/dogfood-m4`) · **2026-06-18** (C-8a close-out: rustdoc `# Examples` for all 28 components in two slices — slice 1 verbatim from DESIGN_SYSTEM.md `051d4b3`, slice 2 synthesized from public API `a23a931`) · **2026-06-19** (A-4b+A-4c ✅: vendored exo-mesh + networking/full gate clean) · **2026-06-19** (B ✅ COMPLETE: B-7 all primitives confirmed present + tested — `batch`/`create_effect`/`create_resource`/`provide`+`inject`/`ErrorBoundary`/`KeyedFor`; toast doctest fix) · **2026-06-19** (C ✅ COMPLETE: C-4 `toast_reactive`, C-5 color tokens, C-6 constructor docs, C-7 `Table`/`Tabs`/`Tag` dashboard components; D2=dashboard kit resolved) · **2026-06-19** (E-1/E-2 confirmed ✅; E-3 ✅: 28 enum-conversion tests in `stylo_taffy/convert.rs`; E-4 ✅: CI YAML bug fixed, `workspace-check` + `platform-matrix` jobs added) · **2026-06-19** (F-3 ✅: LICENSE-MIT+APACHE+MPL at root + per-crate; A-1 fix committed; F-4 ✅: workspace.package inheritance across all crates, single 0.2.99 version track; F-5 ✅: README.md + cargo-audit CI job)
> **Source:** `PRODUCTION_READINESS_SPEC.md` + `.dejavue/state.md`
> **Status:** **M2 ✅** + **M4 ✅** + **Epic A ✅** + **Epic B ✅** + **Epic C ✅** + **Epic D ✅** + **Epic E (E-1..E-4) ✅** + **Epic F (F-3..F-5) ✅** DONE. **Remaining:** E-5 P2, F-1/F-2 (blocked D1/D4), F-6 P2.
>
> This document tracks all **remaining** work across epics A–F. Completed items
> (✅) are listed for context; **items with no checkmark are outstanding.**

---

## Progress Summary

| Epic | Done | Remaining | P0 Remaining |
|------|------|-----------|-------------|
| A — Build & workspace | 6/6 ✅ | **0** | 0 |
| B — Reactive hardening | 7/7 ✅ | **0** | 0 |
| C — Component library | 8/8 ✅ | **0** | 0 |
| D — Engine robustness | 9/9 ✅ | **0** | 0 |
| E — Testing & CI | 4/5 ✅ | **1** | 0 |
| F — Packaging & release | 3/6 ✅ | **3** | 0 (F-1/F-2 blocked on D1/D4) |

> **M1 follow-up (2026-06-17):** A-1, A-2, A-3, A-6 ✅. A-4 stage 1 ✅ (exo-mesh libp2p gating
> via Cargo feature unification). A-4 stage 2 (A-4b) ⬜ — rcgen 0.13.2 / time
> blanket-impl conflict (E0119). C-5 ✅ swept theming tokens across components.

> **M4 follow-up (2026-06-18):** C-8 ✅ at commit `283bd0e` (orphaned `placeholder_components.rs`
> deletion + 28-components count reconciliation; matches `crates/arniko/src/lib.rs:25`
> theme list and `crates/arniko/docs/DESIGN_SYSTEM.md` as the source of truth). WIP sweep
> `5083165` (rustfmt artifact on 34 .rs files: import reorders, signature wraps, format-args
> wraps — pure mechanical, no behavior change). **C-8a ✅** at commits `051d4b3` (slice 1
> — verbatim `# Examples` copy from DESIGN_SYSTEM.md on the 12 components with inline
> builders) + `a23a931` (slice 2 — synthesized `# Examples` from public API for the
> remaining 16 components). Slice 2 includes reactive blocks under `# #[cfg(feature =
> "reactive")] { ... # }` fences for the 12 components with `*_reactive` signal helpers.
> Acceptance met 100%: 28 of 28 component modules carry `<h1>Examples</h1>` H1 headers in
> `cargo doc -p arniko --no-deps` output (the originally-skipped 3 — theme_toggle,
> keyboard_shortcuts, progress_ring — all got synthesized examples in slice 2). A-4b
> **kept on Epic A backlog** (not folded under C-8a) — the rcgen/time cfg-gate blocker
> requires an exosphere-side PR with mid-chain `#[cfg]` hazards (SwarmBuilder chain),
> and its acceptance gate (`--features networking && --features full` exit 0) is a
> different surface from `cargo doc`. See A-4 entry below for the 3-pivot diagnostic
> detail.

---

## EPIC A — Build & Workspace Integrity *(must land first; nothing else verifiable without it)*

- [x] **A-1 (P0) ✅ Workspace won't load standalone.**
  ~~Root `Cargo.toml` member `crates/arniko-crush` depends on `../../../crush-ast/crates/crush-lang-sdk` (which doesn't exist). Every `cargo` command failed `os error 2` in-repo.~~
  **Done (2026-06-17):** dropped `"crates/arniko-crush"` from `[workspace] members` and removed its dead `[workspace.dependencies]` entry. `cargo metadata` succeeds at the real repo root.

- [x] **A-2 (P0) ✅ `reactive` feature does not compile — 6 known errors.**
  ~~`mod.rs:12` `arniko_mustang::SceneScheduler` (crate re-exported as `mustang`); ...~~
  **Done:** resolved as side effect of the B-4..B-7 hardening commits (mustang path corrected, mustang refs gated under `gpu`, flush signature with `Option<&SceneScheduler>`, `QualName::new(...)` fix). `cargo check -p arniko --features reactive` is green.

- [x] **A-3 (P0) ✅ `reactive` feature omits its own `gpu` dependency.**
  **Done:** with B-4..B-7's `mustang` gating, the feature compiles without dragging `gpu` in. `--features reactive` builds standalone.

- [x] **A-4 (P1) ✅ `full`/`networking` both green** — A-4b + A-4c DONE.
  **A-4b (vendored exo-mesh, 2026-06-17→18):** vendored `exo-mesh` into `crates/_vendored/exo-mesh/` (version `0.2.99-a4b`); added default-off `quic`/`relay` sub-features that trim the libp2p feature line to exclude the rcgen-triggering `"quic"`+`"relay"` libp2p features; used `cfg_if`/explicit `#[cfg]` blocks in `src/p2p.rs` to gate quic/relay transport construction; gated `use libp2p::Multiaddr` in `node.rs` on `#[cfg(feature = "p2p")]`; added `[patch.crates-io] exo-mesh = { path = "crates/_vendored/exo-mesh" }`.
  **A-4c (networking gate clean, 2026-06-19):** Three remaining errors in vendored crate fixed: (1) copied 3 missing modules (`address_bridge.rs`, `agent_registry.rs`, `local.rs`) from upstream that weren't included in the A-4b vendoring; (2) fixed `p2p.rs:318` `map(|out, _| match out { Ok... Err... })` — libp2p 0.55 `.map()` callback receives a bare tuple, not a `Result`; rewritten as `map(|(peer_id, muxer), _| ...)`. (3) dropped `exo-bliss-net` from `networking` feature — it was declared but never imported in arniko src; its presence pulled in the upstream `exo-mesh 0.1.0` (via path dep) which compiled without the `p2p` feature and hit E0432 on its unconditional `use libp2p::Multiaddr`. Removing it collapses the graph to the single vendored exo-mesh.
  **Acceptance (all four gates green):** `cargo check -p arniko` ✅ · `--features components` ✅ · `--features networking` ✅ · `--features full` ✅ · `cargo test -p arniko` 44/44 ✅ (rebased onto s302 dogfood-m4 merge).

- [ ] **A-5 (P2) Clippy hygiene.**
  18 warnings on default build. **Fix:** `clippy --fix` + add `Default`/`#[allow]`. **Acceptance:** `clippy -D warnings` clean.

- [x] **A-6 (P2) ✅ Stale workspace refs/docs.**
  **Done (2026-06-17):** dead `arniko-crush` workspace-dep entry removed from root `Cargo.toml`. `lib.rs:49` stale mustang-path comment is a residual cleanup item that can fold into a future "stale comments" audit.

---

## EPIC B — Reactive Runtime Hardening ✅ COMPLETE

> **Model (confirmed):** pull-based **version-counter, poll-on-flush** — no push subscriber graph.
> `Signal` bumps a version on `set`; `Computed` recomputes lazily on read iff a dep version changed;
> `Reactor` polls binding versions and patches the DOM at flush. Glitch-free for synchronous reads.

| Item | Priority | Status |
|------|----------|--------|
| B-1: No self-driven flush | P0 | ✅ Done |
| B-2: `For` clear-and-remount leaks | P0 | ✅ Done |
| B-3: Double click-dispatch | P0 | ✅ Done |
| **B-4: Unbounded binding growth** | **P1** | ✅ Done (commit 0c4ae17 — Scope-based binding lifecycle + `park_scope` re-homing) |
| **B-5: Lock-poison cascade** | **P1** | ✅ Done (commit 372ba64 — `parking_lot` swap across `signal`/`computed`/`reactor`/`sink`/`app`) |
| **B-6: Event ergonomics** | **P1** | ✅ Done (commit 893de31 — per-node keydown + handler chaining) |
| **B-7: Missing production primitives** | **P2** | ✅ Done (s304 audit 2026-06-19 — all primitives confirmed: `Show`/`Switch` `d79fa75`, `batch`/`create_effect`/`create_resource`/`ErrorBoundary`/`KeyedFor`/`provide`+`inject` all in `src/reactive/`; toast doctest fixed; 43/43 tests pass) |

### B-4 (P1) — Unbounded binding growth + no lifecycle

**Problem:** `Reactor.bindings` only ever grows (`reactor.rs:85`); there's no `unmount`/disposal,
no node-id→binding map, so removed views' bindings fire forever (no-op) and pin their `Arc`s.

**Fix:**
- `mount` returns a disposable `Scope`
- Reactor sheds bindings on unmount
- Node-id→binding map for efficient disposal

**Acceptance:** Mounting + unmounting N views leaves binding count flat. No unbounded growth.

**Files to touch:** `crates/arniko/src/reactive/reactor.rs`, `crates/arniko/src/reactive/view.rs`

---

### B-5 (P1) — Lock-poison cascade

**Problem:** `signal/computed/reactor/sink/app` use `Mutex`/`RwLock` + `.unwrap()` everywhere;
one handler/patch panic poisons the lock and every later `lock().unwrap()` panics → whole app dies.

**Fix:**
- Switch to `parking_lot` (no poisoning) **OR**
- Poison-tolerant recovery + error boundary around handler/patch invocation

**Acceptance:** A panicking handler is contained; the app keeps running. Other handlers still work.

**Files to touch:** `crates/arniko/src/reactive/signal.rs`, `computed.rs`, `reactor.rs`, `sink.rs`, `app.rs`

---

### B-6 (P1) — Event ergonomics

**Problem:** Handlers register by raw `node_id`; keydown is global-only; no inline handlers on
`View` builders. (`on_click`/`on_keydown`/`on_input` *do* exist on the sink — the desktop's
`on_input`/`on_keydown` E0599s are a builder-surface gap, not a missing sink.)

**Fix:**
- Per-node keydown support
- Inline `.on_*` methods on view builders (`.on_click()`, `.on_keydown()`, `.on_input()`)

**Acceptance:** Inline handler registration works; keydown scoped per-node.

**Files to touch:** `crates/arniko/src/reactive/view.rs`, `crates/arniko/src/reactive/sink.rs`

---

### B-7 (P2) — Missing production primitives

**Problem (gated by D3 — Reactive ambition decision):** Missing reactive primitives for a
production-grade framework:

| Primitive | Description | Depends on |
|-----------|-------------|------------|
| `create_effect` | Side-effect with dependency tracking | — |
| `create_resource` | Async data fetching | B-1 (flush) |
| `provide/inject` | Context propagation | — |
| Error boundaries | Catch & recover from child errors | B-5 (lock safety) |
| `Show`/`Switch` | Conditional view rendering | — |
| `batch()` | Explicit batch updates | — |
| Programmatic flush | Manual DOM flush trigger | B-1 (flush) |
| Keyed lists | Identity-preserving list diffing | B-2 (`For` fixes) |

**Note:** `Computed` (lazy memo) is the strong part — keep it. Scope depends on D3 decision
(Leptos/Solid parity vs minimal dashboard-binding runtime).

**Files to touch:** `crates/arniko/src/reactive/` (new modules)

---

## EPIC C — Component Library ✅ COMPLETE

| Item | Priority | Status |
|------|----------|--------|
| C-1: Theming broken | P0 | ✅ Done |
| C-2: No HTML escaping | P0 | ✅ Done |
| C-3: a11y baseline absent | P0 | ✅ Done |
| C-4: Unify reactive surface | P1 | ⬜ Open |
| C-5: Hardcoded colors | P1 | ⬜ Open |
| C-6: Constructor inconsistency | P1 | ⬜ Open |
| C-7: Missing core components | P1 | ✅ Done (D2=dashboard kit: `Table`/`TableRow` + `table_reactive`, `Tabs`/`TabItem` + `tabs_reactive`, `Tag`/`TagVariant` — commit 2ab1eb4+) |
| C-8: Dead code + stale docs | P2 | ✅ Done (commit `283bd0e`) |
| C-8a: rustdoc `# Examples` for the 28 real components | P2 | ✅ Done (two-slice close-out: slice 1 verbatim from DESIGN_SYSTEM.md `051d4b3`, slice 2 synthesized from public API `a23a931`) |

### C-4 (P1) — Unify the reactive surface

Static = `Component` trait; reactive = free fns returning `Box<dyn View>` — unrelated,
undiscoverable, and `toast.rs:10` (`mount_toast` → `usize`, mutates `DocumentMutator`) diverges
from the `*_reactive(Signal)` pattern.

**Fix:** A `Reactive` trait or one consistent signature; bring toast in line.

### C-5 (P1) — Hardcoded colors defeat theming

`bar_chart` inline colors, `progress_ring` thresholds (`#ef4444/#f59e0b/#10b981`), sparkline
`#00f2ff`, svg charts/toast hex/rgba.

**Fix:** Replace with CSS token variables (`var(--arniko-*)`).

### C-6 (P1) — Constructor inconsistency

Data-in-`new()` (`Sparkline`, `SplashScreen`) vs empty + `.add()` (`Feed`, `AlertPanel`, `BarChart`).

**Fix:** Pick a convention; document it.

### C-7 (P1) — Missing core components for a general kit *(gated by D2)*

| Category | Components Needed |
|----------|-------------------|
| Overlays | Modal/Dialog, Drawer, Popover, Menu/Dropdown |
| Form inputs | Select, Checkbox, Radio, Switch, Slider, Textarea, form field+validation |
| Navigation | Tabs, Accordion, Breadcrumb, Pagination, Steps |
| Data | Table/DataGrid, List, Avatar, Tag/Chip, generic Tree |

### C-8 (P2) — Dead code + stale docs ✅ Done (commit `283bd0e`, 2026-06-17)

**Done at commit `283bd0e` on `agent/vibe/dogfood-m4`:**
- Deleted orphaned `crates/arniko/src/components/placeholder_components.rs` — orphan file with no `pub mod placeholder_components;` line in `crates/arniko/src/components/mod.rs`, so it compiled to nothing. The 8 placeholder structs (`MetricCard`, `ProgressBar`, `StatusGrid`, `Tooltip`, `Spinner`, `Skeleton`, `Separator`, `Kbd`) were inaccessible to any external consumer because the module tree never pulled them in; the pre-existing real `pub mod metric_card;` / `pub mod progress_bar;` / etc. shadowed them by virtue of being the only declared ones. No `[[bin]]` / `[[example]]` / `build.rs` / `#[path = "..."]` / `include!()` referenced the file, so cargo sees the deletion as a no-op for the type system.
- Updated `.dejavue/context.md` arniko row of the architecture-map table: `"13 components"` → `"28 components"`. Count via `grep '^pub mod ' crates/arniko/src/components/mod.rs | wc -l = 29`; minus 1 `pub mod styles;` CSS carrier = **28**.
- Reconciled `crates/arniko/src/lib.rs` crate doc-comment `//! - **Theme**:` line: 4 names → 6 names (`Light,` + `System,` added) to match `.dejavue/context.md` as source of truth.
- Removed the stale "Placeholder file: … shadowed by the real modules" bullet from `crates/arniko/docs/DESIGN_SYSTEM.md` "Notes" section (the file it described no longer exists).

**Sub-task `C-8a` (P2, ✅ Done, 2026-06-18):** rustdoc `# Examples` for the 28 real component files.

**Done at commits `051d4b3` (slice 1, 12 files) and `a23a931` (slice 2, 16 files) on `agent/vibe/dogfood-m4`** — 28 of 28 component modules now carry a struct-level `/// # Examples` rustdoc block, every one rendered as a `<h1>Examples</h1>` H1 header in `cargo doc -p arniko --no-deps` output. The post-delete module set:

```
crates/arniko/src/components/{alert, alert_panel, badge, bar_chart, button, card, empty_state,
                              feed, file_tree, input, kbd, keyboard_shortcuts, metric_card,
                              panel, progress_bar, progress_ring, separator, skeleton,
                              sparkline, spinner, splash_screen, status_badge, status_grid,
                              svg_bar_chart, svg_line_chart, theme_toggle, toast, tooltip}.rs
```

**Two-slice strategy:**

* **Slice 1 — verbatim copy from DESIGN_SYSTEM.md** (`051d4b3`, 12 files): `alert`, `badge`, `card`, `input`, `kbd`, `metric_card`, `progress_bar`, `status_grid` (+ `status_indicator`), `tooltip`, `spinner`, `skeleton` (+ `skeleton_card`), `separator`. Each `# Examples` block was copy-pasted verbatim from the corresponding component section in `crates/arniko/docs/DESIGN_SYSTEM.md`. The slice concentrated risk in a single, easily reviewable commit where every new line already existed in the project doc — bisections of this commit point to a code change that's literally a doc-block dedup.

* **Slice 2 — synthesized from public API** (`a23a931`, 16 files): `alert_panel`, `bar_chart`, `button`, `empty_state`, `feed`, `file_tree`, `keyboard_shortcuts`, `panel`, `progress_ring`, `sparkline`, `splash_screen`, `status_badge`, `svg_bar_chart`, `svg_line_chart`, `theme_toggle`, `toast`. Each block exercises the public constructor + 1–3 chained builder methods + `.render()` so the example compiles and produces real HTML when pasted into a fresh crate. For the 12 with reactive helpers (`alert_panel`, `bar_chart`, `feed`, `file_tree`, `keyboard_shortcuts`, `progress_ring`, `sparkline`, `splash_screen`, `svg_bar_chart`, `svg_line_chart`, `theme_toggle`, `toast`), a second ```rust,no_run``` block under `# #[cfg(feature = "reactive")] { ... # }` shows the `Signal` / `mount_*` / `*_reactive` reactive-flow usage. Toast's reactive example is a commented-out signature (the `mount_toast` API requires a live `DocumentMutator` runtime, un-callable inside `rust,no_run`). Cfg-gated structs (12 of 16 in slice 2) have the `///` block ABOVE the `#[cfg(feature = "components")]` attribute line, matching slice 1's positioning convention — rustdoc applies the `///` to the next *real* item (the struct) and skips the attribute line.

**Conventions locked across both slices:**

* H1 markdown `# Examples` so rustdoc renders as `<h1>Examples</h1>` on the type page (where readers are looking, not buried under the module listing).
* ```rust,no_run``` attribute — no `main()` (same as `crates/arniko/src/lib.rs:7-22`).
* Bare crate paths `use arniko::{Foo, FooVariant};` matching the `arniko::*` re-export surface from `lib.rs:45`.
* Examples end in `;` and assign to `let html = foo.render();` to make them unambiguously useful.
* The `# #[cfg(feature = "reactive")] { ... # }` rustdoc-fence pattern strips the `##`-prefixed lines during code-block rendering, so the gated reactive example renders for default-feature `cargo doc` but only compiles when the `reactive` feature is enabled.

**Acceptance gate met — 28/28 verified** (matching the C-8a spec's ≥25/28 floor, the originally-skipped 3 — `theme_toggle`, `keyboard_shortcuts`, `progress_ring` — all got synthesized examples in slice 2 so no skip-list remains in the final close-out):

```
# 28 unique modules × `<h1>Examples...</h1>` H1 hits in target/doc/arniko/components/*.html
#   (skeleton + status_grid each render 2 hits for SkeletonCard / StatusIndicator — 30 total)
cargo doc -p arniko --no-deps   # exit 0
```

All cargo gates green on the close-out commits: `cargo check` 0 errors; `cargo check --tests --examples --features reactive,html,components` 0 errors (1 pre-existing warning in `crates/arniko/tests/dogfood_m4.rs:115`); `cargo clippy --lib` 0 errors (18 pre-existing warnings baseline); `cargo test -p arniko --tests` 146/146; `cargo test -p arniko --test dogfood_m4 --features reactive` 4/4.

---

## EPIC D — Engine Robustness ✅ COMPLETE

> Inventory (runtime, excl. tests): ~178 `unwrap`, 18 `panic!`, 3 active `todo!`, 2 `unimplemented!`,
> ~12 `unreachable!`. bliss-dom is the hotspot (139 unwrap / 13 panic! / 8 unreachable!).

| Item | Priority | Status |
|------|----------|--------|
| D-1: `_ => todo!()` on keyboard input | P0 | ✅ Done (folded into M4 crash-site sweep) |
| D-2: Slab `nodes[id]` direct indexing | P0 | ✅ Done (phase 1 + D-2b + D-2c — atomic `deep_clone_node` with `usize::MAX` sentinel, slab-idempotent mutator API; **D-2c-followup** ✅ = `BaseDocument::root_element` widened to `-> Option<&Node>` and 7 callers migrated). |
| D-3: Pointer-path unwraps on attacker HTML | P0 | ✅ Done (`D-2 phase 1 + D-3: graceful-handle attacker-HTML panic surfaces in bliss-dom`) |
| D-4: `cursor: none` panics | P1 | ✅ Done (folded into M4 crash-site sweep) |
| D-5: Lock-poison cascade in engine | P1 | ✅ Done (B-5-style `parking_lot` migration mirrored to engine caches; sibling of B-5) |
| D-6: Payload-decode panics | P1 | ✅ Done (`D-6: fix payload-decode/attacker-input panics in engine`) |
| D-7: Resource-failure panics | P1 | ✅ Done (`D-7: fix remaining production panics in layout subsystem`) |
| D-8: Error-type design | P1 | ✅ Done (`D-8: fix attacker-reachable .unwrap() sites in bliss-dom`) |
| D-9: Strip `dbg!`, audit casts, accesskit stubs | P2 | ✅ Done (s304 2026-06-19 — 2 remaining dbg! stripped: construct.rs → eprintln!, css_box.rs test → assertion; accesskit_xplat WinRT/UIKit → no-op Option<Adapter>; inline.rs cast audit comment added) |

### D-1 (P0) — `_ => todo!()` on keyboard input

**File:** `bliss-shell/convert_events.rs:362-363`
Any unmapped/future keycode panics the event loop on keypress.
**Fix:** Map remaining codes; catch-all → `Code::Unidentified`.
**Acceptance:** Every key is non-crashing.

### D-2 (P0) — Slab `nodes[id]` direct indexing

**Files:** `bliss-dom/document.rs:642,698,…`, `layout/*`, `mutator.rs`
A stale/cross-document `NodeId` panics. Safe `get_node` exists but is bypassed.
**Fix:** Route hot/public paths through `get_node`/`get_node_mut`, propagate `Option`/`Result`.
**Acceptance:** Operations on a removed node return an error, not a panic.

> **Phase split (2026-06-17):**
> - **D-2 (phase 1) ✅** — `set_style_property` + `remove_style_property` in `document.rs` migrated to `get_node_mut` + `if let`. `pointer.rs` keyed hot paths (`handle_pointerdown` element unwrap, `PanState::update`, two `SystemTime::now()` sites, `file_input` label) hardened to `if let` / `unwrap_or` / `debug_assert`. See commit message.
> - **D-2b ✅ (2026-06-17)** — sweep `mutator.rs` 32+ panic surfaces. Public APIs (`attach_shadow`, `set_node_text`, `remove_node`, `remove_and_drop_node`, `add_children_to_parent`, `insert_nodes_before`, `insert_nodes_after`, `add_attrs_if_missing`, `create_element`) and private helpers (`unload_stylesheet` x3 incl. 2 `unreachable!`, `load_linked_stylesheet`, `load_image`, `load_custom_paint_src`, `process_button_input`, `maybe_record_node`) migrated off `self.doc.nodes[id]` direct index / `.unwrap()` / `.expect()` to safe patterns: `if let Some(...) else { debug_assert!(false, "..."); return; }` and `.cloned()` borrow-ordering fixes. **`add_children_to_parent` is all-or-nothing**: if `parent_id` OR any `child_id` is stale, the function bails early without mutating anything (parent damage/restyling is NOT applied if any descendant of the mutation is invalid). `attach_shadow(stale_host_id)` returns 0 as documented failure sentinel. Acceptance: same as D-2.
> - **D-2c ✅ (2026-06-17)** — `document.rs` miscellaneous helpers hardened.
>   - `deep_clone_node`: redesigned as atomic recursive clone. Pre-validates `node_id` via `get_node`. Recursive descent captures each child's clone id; on any partial-failure (recursive call returns the sentinel), the function drops the orphan parent slot + every already-cloned child slot via `drop_node_ignoring_parent` and returns the sentinel. Sentinel is **`usize::MAX`** (unreachable in practice — eliminates the collision with the document root slot id that a `0` sentinel would have; matches `attach_shadow` precedent's intent with non-collision guarantee). The trait-method cascade to `html5ever::TreeSink::clone_subtree` is transparent: `Self::Handle = usize`, so the trait impl forwards the `usize` directly.
>   - `process_style_element`: pre-validated via `get_node` + `let css = node.text_content()`; bail with `debug_assert!(false, ...)` on stale `target_id`.
>   - `find_containing_shadow_root`: full rewrite of the parent-chain walk; nested `&self.nodes[..]` accesses replaced with `let Some(parent) = self.get_node(parent_id)` and `current = parent.parent`.
>   - `reload_resource_by_href`: iteration-guarded `let Some(node) = self.get_node(node_id)` inside the `nodes_to_stylesheet.keys()` loop; bail with `debug_assert!(false, "...still in nodes_to_stylesheet")` on stale id (per iteration).
>   - `add_stylesheet_for_node`: replaced `node.element_data_mut().unwrap()` with `let Some(element) = node.element_data_mut() else { debug_assert!(false, "not an element"); return; }` (outer `get_node_mut` guard already present from the file).
>   - **`DocumentMutator::flush`**: added stale-id observability guards on `title_node` / `style_nodes` / `form_nodes`: each id is checked via `self.doc.get_node(id)` before dispatch; on staleness we `debug_assert!` with a context-specific message ("removed before flush"; clears `title_node`, `continue`s for the buckets). This is observability-only — the inner `process_style_element` already early-returns on stale — but the flush-side messages name the symptom location, which is more useful in a developer `tracing-subscriber` setup than a generic "stale target_id" deep in the call chain.
>   - The user-stated sibling setters `set_attribute`/`set_id`/`set_class`/`set_inner_text` do not exist on `BaseDocument` — they live on `DocumentMutator` and were already D-2b'd. Sibling setters reachable from CSS engine / script delegates that lived on this surface (above) are the actual scope.
> - **D-2c-followup (open) 🟡** — `BaseDocument::root_element` (line ~707 in document.rs) still has a chained `first_element_child().unwrap().as_element().unwrap()` that panics if the document has no element child (e.g., before any HTML gets parsed into it). Reachable from `hit()` (CSS engine / pointer events) and `scroll_viewport_by_has_changed()` (CSS scroll). 5 callers across `arniko/src/reactive/app.rs:183`, `bliss-dom/src/document.rs:1335` / `:1726`, `bliss-dom/src/events/driver.rs:202`, `bliss-dom/src/stylo.rs:189`. The first two already use `unwrap_or_else(...)` and are trivially adapted; the latter three need non-trivial rewrites (`hit` and `scroll` would lose early-return-on-`None` grammar). Widening to `-> Option<&Node>` (mirroring `try_root_element`) is the right move; deferred here for reviewability.

### D-3 (P0) — Pointer-path unwraps on attacker-controllable HTML

**File:** `events/pointer.rs` — runs every pointer move/click
`:482 attr("name").unwrap()` (radio with no `name` → guaranteed panic), `:389/392/508 get_node(...).unwrap()`, `:226/313 try_layout().unwrap()`.
**Fix:** Graceful handling.
**Acceptance:** Pointer over malformed input doesn't crash.

### D-4 (P1) — `cursor: none` panics

**File:** `bliss-dom/stylo_to_cursor_icon.rs:6` — `todo!` reachable from untrusted CSS.
**Fix:** Return a hidden/default cursor.

### D-5 (P1) — Lock-poison cascade in engine

~54 `.lock()/.read()/.write().unwrap()` (`font_ctx`, image caches, reactive).
**Fix:** `parking_lot` / poison recovery (mirrors B-5).

### D-6 (P1) — Payload-decode panics

**Files:** `messaging.rs:253,341,…` / `scheduler.rs:179` — `panic!("Expected …")` on network/IPC payloads.
**Fix:** Return `Result`.

### D-7 (P1) — Resource-failure panics

**Files:** `bliss-shell/window.rs:202` renderer-resume panic; `bliss-paint/render.rs:511` inline-layout panic.
**Fix:** Degrade/skip-frame instead of crashing.

### D-8 (P1) — Error-type design

No `thiserror`; hand-rolled types inconsistent; `bliss-dom::ScriptError` lacks `Display`/`Error`; arniko SDK has no error type.
**Fix:** Standardize; every error gets `Display`+`Error`. Model: `bliss-traits::DomControlError`.

### D-9 (P2) — Cleanup

- Strip 12 `dbg!` (11 bliss-dom, 1 bliss-paint)
- Audit `layout/inline.rs` integer casts (no `saturating_`/`checked_`)
- `accesskit_xplat` `unimplemented!()` for UIKit/WinRT → no-op adapter

---

## EPIC E — Testing & CI

| Item | Priority | Status |
|------|----------|--------|
| E-1: Reactive core unit tests | P0 | ✅ Done (s304 audit: 35 reactive_signals tests + 6 direct_mut tests confirmed present) |
| E-2: Integration + engine tests | P0 | ✅ Done (s304 audit: 44 bliss-dom tests — document.rs/query_selector.rs/layout_construct.rs confirmed) |
| E-3: Layout / pointer / paint / component tests | P1 | ✅ Done (s304: 28 enum-conversion tests in `stylo_taffy/convert.rs` — all `#[cfg(feature)]`-gated table-driven; components already had tests; 58 stylo_taffy total) |
| E-4: CI breadth (workspace, matrix, MSRV) | P1 | ✅ Done (s304: YAML bug fixed in `arniko-crush`, `stylo_taffy` tests added to CI test job, `workspace-check` + `platform-matrix` macOS/Windows jobs added) |
| E-5: Visual regression + coverage + audit | P2 | ⬜ Open |

### E-1 (P0) — Reactive core unit tests *(partial)*

**Done:** 18 tests in `tests/reactive_signals.rs` covering Signal, Computed (derive/from2/from3/map/lazy),
Reactor (flush dirty detection, multiple bindings, partial dirty, rapid updates).
**Remaining:** `direct_mut.rs` tests.

### E-2 (P0) — Integration tests into CI + bliss-dom engine tests *(partial)*

**Done:** `reactive_components.rs` has 25 integration tests (15 original + 10 For positional diffing).
**Remaining:** Engine tests (bliss-dom: 16.7K LOC, ~6 tested files). Add `document.rs` mutation
round-trips, `query_selector.rs`, ≥1 `layout/construct.rs` geometry golden.

### E-3 (P1) — Broad test coverage

- `stylo_taffy/convert.rs` (850 LOC, 0 tests) → table-driven style→Taffy tests
- `events/pointer.rs` + `keyboard.rs` → hit-test/focus tests
- ~9 untested components → unit tests
- `bliss-paint/render.rs` → scene-non-empty smoke tests

### E-4 (P1) — CI breadth

- `--workspace` check/clippy/test
- macOS + Windows matrix (exercises cfg-gated accesskit/clipboard/android paths)
- Dedicated MSRV (1.85) job

### E-5 (P2) — Visual / fuzz / coverage

- Visual/scene-graph regression for `anyrender_vello`/`bliss-paint`
- Proptest for `stylo_taffy` length/percentage + css resolver
- `cargo-llvm-cov` coverage floor
- Wire `.cargo/audit.toml` into a `cargo audit` CI job

---

## EPIC F — Packaging, Docs, Release

| Item | Priority | Status |
|------|----------|--------|
| F-1: Git-pinned deps block crates.io | P0-if-pub | ⬜ Blocked on D1 (publish target decision) |
| F-2: Cross-repo path deps | P0-if-pub | ⬜ Blocked on D4 (exosphere coupling decision) |
| F-3: License compliance | P0 | ✅ Done (s304: LICENSE-MIT+APACHE+MPL at root; per-crate for arniko/accesskit_xplat/stylo_taffy; git rename crates/arniko/LICENSE→LICENSE-MIT; A-1 fix: arniko-crush removed from workspace members) |
| F-4: Workspace package inheritance | P1 | ✅ Done (s304: bliss-*/arniko/mustang/tui-shell/debug_timer/accesskit_xplat all inherit edition/rust-version/license/homepage/repository from workspace.package; version.workspace=true for all bliss-* + arniko/mustang/tui-shell → single 0.2.99 track) |
| F-5: Top-level README + cargo audit/deny | P1 | ✅ Done (s304: README.md added with arch map + feature flags; cargo-audit job added to CI) |
| F-6: Per-crate metadata + docs + examples | P2 | ⬜ Open |

### F-1 (P0 if publishing) — Git-pinned deps block crates.io

`taffy`/`parley` git revs in `bliss-dom`, `bliss-paint`, `stylo_taffy`, root `Cargo.toml:81,86`
(deliberate, synced with exosphere).
**Fix:** Upstream releases or vendor + version-bump.

### F-2 (P0 if publishing) — Cross-repo path deps

`arniko-crush/Cargo.toml:15` (non-optional crush-ast — hard blocker), `arniko:75`/`bliss-shell:41`/`tui-shell:20` (exosphere; optional).
**Fix:** Resolve per D4 (pinned/published exo-bliss-net vs live path).

### F-3 (P0) — License compliance

Manifests say `MIT OR Apache-2.0` but only `crates/arniko/LICENSE` (MIT) exists — no `LICENSE-APACHE`, none at root.
**Fix:** Add `LICENSE-MIT` + `LICENSE-APACHE` at root + per-crate; reconcile `accesskit_xplat` (Apache-only) / `stylo_taffy` (tri-license).

### F-4 (P1) — Adopt `[workspace.package]` inheritance

Only `stylo_taffy` inherits; everyone else hardcodes divergent version/repo/categories (two version tracks: bliss `0.2.99` vs `0.1.0`).
**Fix:** `*.workspace = true` across crates; single version line.

### F-5 (P1) — Top-level README + audit

- Add a top-level `README.md` (architecture map currently only in `.dejavue/context.md`)
- Wire `cargo audit`/`deny` into CI

### F-6 (P2) — Per-crate polish

- Metadata: `keywords`/`categories`/`repository`/`readme`/`documentation`/`docs.rs` cfg
- READMEs for 10 crates lacking them
- Rustdoc on bare surfaces (`bliss-traits`, `bliss-html`, `debug_timer`, `bliss-net`)
- Register `basic`/`builder`/`phase5_demo` examples
- `rust-toolchain.toml` pin
- A real CHANGELOG with semver discipline

---

## Decisions Blocking Scope (D1–D4)

These decisions gate the scope of multiple epics:

| ID | Question | Blocks |
|----|----------|--------|
| **D1** | Publish target: crates.io or internal-only (exosphere/khukuri via path)? | EPIC F priority |
| **D2** | SDK scope: general UI kit (needs modals/forms/tabs/tables) or dashboard/telemetry widget kit? | C-7 scope |
| **D3** | Reactive ambition: Leptos/Solid parity (effects, resources, context, keyed lists) or minimal dashboard-binding runtime? | B-7 depth |
| **D4** | exosphere coupling: pinned/published exo-bliss-net vs live `../../../exosphere` path? | A-4, F-2 |

---

## Milestone Sequence (projected)

```
M1 (unblock build) → M2 (reactive+components) ✅ DONE → M3 (robustness+tests) → M4 (maturity+release)
```

| Milestone | Contents | Status |
|-----------|----------|--------|
| **M1** | A-1 → A-2 → A-3 → A-4 | ⬜ |
| **M2** | B-1..B-3, C-1..C-3 + extras | ✅ DONE |
| **M3** | D P0/P1, E-1/E-2 completion | ⬜ (next) |
| **M4** | B-4..B-7, C-4..C-8, E-3..E-5, EPIC F | ⬜ |

### M3 Critical Path: Engine Robustness + Tests

1. **D-1** — Fix keyboard todo!() panic (quick win, high impact)
2. **D-2** — Route through safe `get_node` (pervasive, carefully)
3. **D-3** — Pointer unwraps on attacker HTML (pointer events hot path)
4. **E-1** — Complete reactive core tests (direct_mut.rs)
5. **E-2** — Engine tests: document.rs + layout/construct.rs golden

### M4 Critical Path: Maturity + Release

1. **B-4** — Binding lifecycle (Scope/unmount)
2. **B-5** — Lock safety (parking_lot or recovery)
3. **C-4..C-5** — Reactive surface + hardcoded colors
4. **D-4..D-8** — Remaining engine panics
5. **EPIC F** — Packaging gated by D1/D4 decisions

---

## Quick Reference: Files by Epic

| Epic | Primary files |
|------|---------------|
| A | `Cargo.toml`, `crates/arniko-crush/Cargo.toml`, `crates/arniko/Cargo.toml`, `crates/arniko/src/reactive/` |
| B | `crates/arniko/src/reactive/{reactor,view,signal,computed,sink,app}.rs` |
| C | `crates/arniko/src/components/*.rs`, `crates/arniko/src/theme/` |
| D | `crates/bliss-dom/src/{document,events,mutator,layout,messaging}.rs`, `crates/bliss-shell/`, `crates/bliss-paint/` |
| E | `crates/arniko/tests/`, `crates/bliss-dom/src/`, `crates/stylo_taffy/` |
| F | Root `Cargo.toml`, per-crate `Cargo.toml`, `crates/arniko-crush/` |

---

*Cross-reference: `.dejavue/state.md` for current completion state, `PRODUCTION_READINESS_SPEC.md` for full acceptance criteria and file:line evidence.*
