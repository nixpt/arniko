# Decisions

## arniko-crush: variadic argc for optional args
- **Date:** 2026-06-15
- **Decision:** Use `argc: None` in HostCapSpec for capabilities with optional arguments (button, card, alert, badge, input)
- **Rationale:** Allows programs to call with fewer args; the host capability parses based on args.len(). Fixed argc would require multiple cap names or padding with Null.
- **Tradeoff:** Slightly more complex parsing in cap implementations; no compile-time arity checking.

## arniko-crush: no modification to crush-lang-sdk
- **Date:** 2026-06-15
- **Decision:** arniko-crush exposes its own `register()` function instead of adding a method to HostCapsBuilder
- **Rationale:** Avoids circular dependency; keeps crush-lang-sdk dependency-light. Users call `arniko_crush::register(&mut host_caps)` after building HostCaps.
- **Tradeoff:** Slightly more verbose setup; cleaner separation of concerns.

## capsule-ui: embedded CSS vs runtime fetch
- **Date:** 2026-06-15
- **Decision:** Embed minimal arniko CSS directly in CrushMarkup component rather than fetching at runtime
- **Rationale:** No network dependency; works offline; SSR-compatible. CSS is ~2KB minified.
- **Tradeoff:** Duplicates arniko CSS (could drift if arniko updates). Production apps should load full arniko stylesheet via `arniko.styles` capability and disable `includeStyles`.

## 2026-06-17T16:02:06-05:00 — [CONSTITUTIONAL] A-1: drop arniko-crush from workspace; collateral cleanup of dropping_references lints and dead into_handles

Reason:
arniko-crush/Cargo.toml:15 points at a non-existent ../../../crush-ast/crates/crush-lang-sdk. The spec listed two fix paths: restore/rename the crate, or [workspace] exclude + drop the member. cargo check --workspace before the change failed with E0599 in caps.rs:37 (Value type unresolved). After dropping the member AND its workspace-dep entry, cargo check --workspace is green and cargo check -p arniko --features reactive is green (the six A-2/A-3 errors predicted by the spec were already resolved by the B-4..B-7 commits). Collateral cleanups done in the same recompile: (i) handle_pointerdown TextInput arm — replaced drop(font_ctx) with inner scope-block so the parking_lot MutexGuard is released naturally; (ii) handle_click label arm — removed drop(target_node) since it was dropping a reference (no-op, borrow ends at end of if-let arm anyway); (iii) Scope::into_handles — removed; its doc said Reactor::park_scope would use it, but park_scope just pushes the whole Scope onto parked_scopes: Vec<Scope>, keeping bindings alive via Scope::drop.

Rejected alternatives:
- **Stub crush-lang-sdk here**: would require creating a sister crate with the Value/HostCap/HostCapSpec/HostCaps types — out of scope for this branch. Wait for upstream crush-ast to publish crush-lang-sdk: no timeline, would block every cargo invocation from this repo in the meantime.


## 2026-06-17T16:38:19-05:00 — D-2 phase 1 + D-3: graceful-handle panic surfaces in bliss-dom engine

Reason:
Pointer/events hot path + style setters in bliss-dom previously panicked on degenerate input (text-node hit target, stale NodeId in style setters, math overflow from a hypotetical pre-epoch SystemTime, empty file lists). Phase 1 of D-2 (P0) hardened: set_style_property/remove_style_property in document.rs migrated to get_node_mut + if let Some else { return }; pointer.rs keys sites either silent-fallback with debug_assert or let Some else { return } so production never crashes on attacker HTML/CSS. D-2b (mutator.rs 22-site sweep) and D-2c (document.rs deep_clone_node + sibling setters) intentionally deferred — high-risk API surface change requires Option result-type propagation review, commit kept reviewable.


## 2026-06-17T16:50:59-05:00 — D-2b: migrate mutator.rs panic surfaces to safe get_node_mut

Reason:
crates/bliss-dom/src/mutator.rs contained 32+ direct-index / unwrap / expect / unreachable! panic surfaces reachable from public DocumentMutator API (refreshed subtrees, CSS engine callbacks, image/stylesheet loading). Per the user instruction: where the public API can be widened to Option/Result, propagate; otherwise use debug_assert! + safe fallback. attach_shadow returns 0 as documented sentinel (no external callers in arniko or exosphere trees confirmed via rg). add_children_to_parent is explicitly all-or-nothing: if parent_id or any child_id is stale, the function bails early without applying parent damage or restyling hints—previous code would have either panicked mid-iteration (worst case) or applied partial damage (inconsistent state). The reparent_children caller briefly flagged for transient stale IDs which would previously have panicked; now it silently no-ops with debug_assert message. D-2c (document.rs sibling setters + deep_clone_node recursion) intentionally still open as it requires API-surface choices. cargo check --workspace + cargo check -p arniko --features reactive + cargo check -p bliss-dom --tests all green (only pre-existing unused-import warnings for DocumentMutator in tests/* remain).


## 2026-06-17T17:05:19-05:00 — D-2c: document.rs deep_clone_node atomic + 4 sibling CSS/script setters

Reason:
crates/bliss-dom/src/document.rs had ~10 panic surfaces reachable from CSS engine + script delegates. D-2c hardened: deep_clone_node rewritten as atomic recursive clone with usize::MAX failure sentinel (eliminates collision with document root slot id and trait-cascades sensibly into html5ever TreeSink::clone_subtree); process_style_element / find_containing_shadow_root / reload_resource_by_href / add_stylesheet_for_node unwrap migrated to debug_assert + safe-fallback pattern (D-2 phase 1 + D-2b precedent). DocumentMutator::flush gained stale-id observability guards on title_node / style_nodes / form_nodes so flush-boundary staleness surfaces with location-specific messages. D-3, D-2c-followup (root_element widening at 5 caller sites), and A-4b still open; cargo check --workspace + reactive feature + bliss-dom tests all green (only the pre-existing 3 unused-import warnings remain).


## D-2c-followup: root_element widening (2026-06-17T23:05:45Z, commit d20ede9)

Widened `BaseDocument::root_element` to `Option<&Node>` to eliminate the chained-unwrap panic surface (`.first_element_child().unwrap().as_element().unwrap()` panicked any caller when the document had no element child). The new body matches the prior `try_root_element` impl. Decision: keep `try_root_element` as a thin alias rather than deleting — preserves the existing 1 caller (`document.rs:618` fallback focus id setter) without inline churn.

7 caller migrations across the engine + app paths:
- 5 sites use `.map(|r| r.id).unwrap_or(0)`: `resolve.rs:` (2), `events/driver.rs`, `arniko/reactive/app.rs`. These map the idiomatic id access through `Option`. `resolve.rs` migrations are guarded dead code (upstream `is_none` short-circuits before reaching `unwrap_or(0)`).
- 1 site uses `.map(|r| r.final_layout.size).unwrap_or_default()`: `scroll_viewport_by_has_changed` (degrades to zero-size content).
- 1 site uses `?`-propagation: `hit()` (None on no-root propagates to caller).
- 1 site uses guard-style `if let Some(root) = ` and skips style traversal: `stylo.rs::resolve_stylist`.
- 1 site uses guard-style `let Some(root_element) = ... else { return; }` and emits an empty scene: `bliss-paint/render.rs::paint_scene`.

Trade-off considered: introduce a single `BaseDocument::root_element_id(&self) -> usize` helper to collapse the 5 `.map(|r| r.id).unwrap_or(0)` sites. Deferred: keeping the pattern explicit per-site preserves debug context for each call against the no-root branch. Open as a refactor followup if the count grows.

EPIC D is mechanically closed — every P0 and P1 ticket done. Only remaining EPIC D work is the P2 cleanup ticket `D-9` (strip 12 `dbg!`, audit `layout/inline.rs` integer casts, no-op accesskit UIKit/WinRT `unimplemented!` adapters).

## D-2c-followup safety backstop (2026-06-17)

Two safety additions applied post-cargo-check on `d20ede9`:

1. **`debug_assert!` at the unwrap_or(0) boundary in `resolve.rs`** — directly after each `let root_node_id = ...` and `let root_element_id = taffy::NodeId::from(...)`. The upstream `if ... first_element_child().is_none() { return; }` guard is now LOAD-BEARING for the unwrap_or(0) safety: deleting it would silently propagate `0` to `propagate_damage_flags` / `flush_styles_to_layout` / `taffy::compute_root_layout(self, NodeId::from(0), ...)`, which would compute layout against the document root node (id 0, an empty layout block) producing garbage layout with no panic. The 1-line `debug_assert!(self.root_element().is_some(), ...)` catches a future refactor that breaks this invariant on day 1.

2. **`test_root_element_none_safety` regression test** (`crates/bliss-dom/src/tests/document.rs`) — constructs an empty `BaseDocument` (no HTML parsed, no element child), then exercises 8 migrated call sites: `try_root_element`, `root_element`, `hit`, `scroll_viewport_by_has_changed`, `resolve`, `scroll_node_by_has_changed`, `clear_focus`, `clear_hover`. Asserts no panic + sensible `None` / `false` return for each. Locks in the new empty-doc semantics as a regression surface for future changes.

Decisions NOT taken (deliberately):

- **No `debug_assert!` in `paint_scene()` / `resolve_stylist()`** — silent empty-frame / no-op resolve is the right semantic at the top-of-pipeline with valid upstream guards. Adding assertions would create tracing-subscriber noise in production for non-error conditions.
- **No `BaseDocument::root_element_id(&self) -> usize` helper** — only 4 sites take the `.map(|r| r.id).unwrap_or(0)` shape; a 5th uses `.final_layout.size`. Collapsing them removes the explicit "degrade to a sentinel id, not a panic" affordance the reader can grep. YAGNI deferred to a future refactor if the count grows.
