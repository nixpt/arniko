# Arniko Remaining Track Work

> **Generated:** 2026-06-16 · **Updated:** 2026-06-17 (after M1 follow-up commit series on branch `agent/vibe/ar-m4`)
> **Source:** `PRODUCTION_READINESS_SPEC.md` + `.dejavue/state.md`
> **Status:** M2 complete + M1 substantially complete (only A-4b outstanding) → M3 (engine robustness + D P0s) is the next frontier.
>
> This document tracks all **remaining** work across epics A–F. Completed items
> (✅) are listed for context; **items with no checkmark are outstanding.**

---

## Progress Summary

| Epic | Done | Remaining | P0 Remaining |
|------|------|-----------|-------------|
| A — Build & workspace | 4/6 + 1 partial | **1** | 0 (A-4b is P1 rcgen/time blocker) |
| B — Reactive hardening | 3/7 | **4** | 0 |
| C — Component library | 4/8 + 4 extras | **4** | 0 |
| D — Engine robustness | 8/9 (D-1..D-8 ✅ across `M3`/`M4`/`D-2` phases) + D-2c-followup tracked | **1** (D-9) + D-2c-followup | 0 |
| E — Testing & CI | 2/5 partial | **3** | 0 (partial done) |
| F — Packaging & release | 0/6 | **6** | 3 (F-1, F-2, F-3) |

> **M1 follow-up (2026-06-17):** A-1, A-2, A-3, A-6 ✅. A-4 stage 1 ✅ (exo-mesh libp2p gating
> via Cargo feature unification). A-4 stage 2 (A-4b) ⬜ — rcgen 0.13.2 / time
> blanket-impl conflict (E0119). C-5 ✅ swept theming tokens across components.

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

- [ ] **A-4 (P1) 🔶 PARTIAL `full`/`networking`** — Stage 1 ✅; Stage 2 ⬜ ESCALATED (A-4b).
  **Stage 1 ✅ (resolved on this branch):** exo-mesh's two named errors (`:13 libp2p undeclared`, `:208 peer_id on Arc<NodeIdentity>`) are addressed without out-of-tree edits — `crates/arniko/Cargo.toml` lists `exo-mesh` as a direct optional dep and the `networking` feature enables `exo-mesh/p2p`, which triggers Cargo feature unification and materializes `libp2p` across the graph. exo-mesh compiles.
  **Stage 2 ⬜ ESCALATED (A-4b, 2026-06-17):** rcgen 0.13.2 vs time blanket-impl E0119 was the only remaining blocker on `--features networking` and `--features full`. **Three in-ariko pivots rejected** (full diagnostic in `.dejavue/decisions.md` `A-4b: 3-pivot diagnostic result`):
  1. `[patch.crates-io] time = "=0.3.35"` — rejected by Cargo as same-source no-op patch.
  2. `[patch.crates-io] time = { git = "https://github.com/time-rs/time.git", tag = "v0.3.35" }` — registered but **unused**: x509-parser v0.17 (transitive via libp2p-quic) hard-pulls `time >= 0.3.36` for an internal feature flag, defeating the resolver.
  3. Trim `"quic"`+`"relay"` from exo-mesh's `libp2p` features — `exo-mesh/src/p2p.rs` (1083 LOC) uses `libp2p::quic` and `libp2p::relay` UNCONDITIONALLY in transport + swarm behaviour; just trimming breaks exo-mesh's compile before rcgen is even reached.
  **Out-of-tree unblock options** (any one of these resolves A-4b):
  * **Exosphere-side cfg-gate PR.** Modify `crates/exo/net/mesh/Cargo.toml` to expose `quic` + `relay` as default-off sub-features; modify `crates/exo/net/mesh/src/p2p.rs` to gate `libp2p::quic`/`libp2p::relay` imports, transport setup, and behaviour wiring via `#[cfg(feature = "...")]`. **SwarmBuilder chain hazard:** libp2p's swarm builder is a chained API — mid-chain `#[cfg]` is not legal, so the conditional `.with_quic()`/`.with_relay()` step must be split out via a `cfg_if` macro, `then_some`, or builder reconstruction.
  * **Upstream `rust-libp2p` ≥ 0.56.** Track when the next rust-libp2p release bumps `libp2p-tls`'s `rcgen = "^0.13"` → `"^0.14"`. Once available, arniko can re-resolve without any workspace patch. Verifiable trigger: `cargo update -p rcgen` succeeds without E0119, AND `cargo update -p time` rolls time forward to ≥ 0.3.36 cleanly.
  **D4 is ORTHOGONAL:** publishing `exo-bliss-net` to crates.io does not itself unlock A-4b (exo-bliss-net uses `default-features = false, features = ["local"]` and does not pull libp2p). Revisiting D4 would let downstream consumers pin arniko's `exo-bliss-net` semver and apply their own rcgen/libp2p-tls patches at the consumer layer, but is a separate piece of work from A-4b.
  **Acceptance (full):** `--features full` AND `--features networking` green from arniko's own tree. Verify via:
  ```
  cargo check -p arniko --features networking && cargo check -p arniko --features full
  ```
  Both should exit 0 once either unblock path lands. Currently red — `cargo check --features networking` fails with `rcgen v0.13.2 conflicting implementations of trait From<format_description::parse::format_item::HourBase> for type <HourBase as ModifierValue>::Type`. The `--features reactive`, `--features launch`, `default`, and `--features gpu` feature gates are green at HEAD `655efdc` (re-verifiable via `cargo check -p arniko`, `cargo check -p arniko --features reactive`, etc.). The live working tree may have unrelated dirty files (B-7 rest primitives from the prior session) that DO NOT change this gate status when checked against HEAD.

- [ ] **A-5 (P2) Clippy hygiene.**
  18 warnings on default build. **Fix:** `clippy --fix` + add `Default`/`#[allow]`. **Acceptance:** `clippy -D warnings` clean.

- [x] **A-6 (P2) ✅ Stale workspace refs/docs.**
  **Done (2026-06-17):** dead `arniko-crush` workspace-dep entry removed from root `Cargo.toml`. `lib.rs:49` stale mustang-path comment is a residual cleanup item that can fold into a future "stale comments" audit.

---

## EPIC B — Reactive Runtime Hardening ⭐ FOCUS TRACK

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
| **B-7: Missing production primitives** | **P2** | 🟡 Partial (`Show`/`Switch` landed in commit d79fa75). **Remaining** (gated by **D3**): `create_effect`, `create_resource` (depends on flush), `provide`/`inject`, error boundaries, `batch()`, programmatic flush, keyed lists. |

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

## EPIC C — Component Library

| Item | Priority | Status |
|------|----------|--------|
| C-1: Theming broken | P0 | ✅ Done |
| C-2: No HTML escaping | P0 | ✅ Done |
| C-3: a11y baseline absent | P0 | ✅ Done |
| C-4: Unify reactive surface | P1 | ⬜ Open |
| C-5: Hardcoded colors | P1 | ⬜ Open |
| C-6: Constructor inconsistency | P1 | ⬜ Open |
| C-7: Missing core components | P1 | ⬜ Open |
| C-8: Dead code + stale docs | P2 | ⬜ Open |

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

### C-8 (P2) — Dead code + stale docs

**Fix:**
- Delete orphaned `placeholder_components.rs`
- Update `.dejavue/context.md:40` "13 components" → actual count (~28)
- Reconcile `lib.rs:28` theme list (4) vs the real 6
- Add rustdoc `# Examples` to components

---

## EPIC D — Engine Robustness *(panic-driven; hotspot is bliss-dom)*

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
| D-9: Strip `dbg!`, audit casts, accesskit stubs | P2 | ⬜ Open |

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
| E-1: Reactive core unit tests | P0 | ⬜ Partial (18 reactive done, direct_mut pending) |
| E-2: Integration + engine tests | P0 | ⬜ Partial (25 integration done, engine pending) |
| E-3: Layout / pointer / paint / component tests | P1 | ⬜ Open |
| E-4: CI breadth (workspace, matrix, MSRV) | P1 | ⬜ Open |
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
| F-1: Git-pinned deps block crates.io | P0-if-pub | ⬜ Open |
| F-2: Cross-repo path deps | P0-if-pub | ⬜ Open |
| F-3: License compliance | P0 | ⬜ Open |
| F-4: Workspace package inheritance | P1 | ⬜ Open |
| F-5: Top-level README + cargo audit/deny | P1 | ⬜ Open |
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
