# JS/TS Support Study for Arniko via `bliss-core` `js_engine`

## Scope
Study how to add JavaScript/TypeScript support to Arniko using the existing `bliss-core` JS engine stack, with minimal disruption to Arniko’s current HTML/native/GPU pipeline.

## Current State (Repo Facts)
- Arniko is a UI framework that generates HTML strings + CSS and supports native/GPU rendering via Mustang. There is no JS/TS integration in Arniko today.
- `bliss-core` already contains a `js_engine` module with a Boa-backed implementation and feature-gated V8/SpiderMonkey backends.
- `bliss-core` has a `script_engine` that extracts `<script>` tags and executes them using the JS engine with a DOM bridge.
- `bliss-core`’s README references Bun, but the actual `js_engine` implementation uses Boa as the default backend.

Relevant paths:
- `crates/platform/sdk/arniko/README.md`
- `crates/apps/web/bliss-core/src/engines/js_engine.rs`
- `crates/apps/web/bliss-core/src/engines/v8_engine.rs`
- `crates/apps/web/bliss-core/src/engines/spidermonkey_engine.rs`
- `crates/apps/web/bliss-core/src/engines/script_engine.rs`

## What “JS/TS Support for Arniko” Can Mean
1. **JS/TS can attach behavior to Arniko-generated UI** (event handlers, DOM mutations).
2. **JS/TS can generate or update UI** (e.g., a component tree or HTML string).
3. **JS/TS can run with a predictable capability model** (Exosphere-style gating).
4. **TypeScript is supported** (either AOT transpiled or runtime support).

## JS Engine Options (Landscape)
These are general embedding options relevant to `bliss-core`/Arniko:
- **Boa**: Rust-native JS engine, embeddable, with an ECMAScript context/VM in `boa_engine`. citeturn0search4turn0search7
- **V8 via `rusty_v8`**: High-performance JS engine with a mature embedding story; Deno’s `deno_core` adds an op/event-loop model on top of rusty_v8. citeturn1search0turn0search10
- **SpiderMonkey via `mozjs`**: Rust bindings to the SpiderMonkey JavaScript engine. citeturn0search8
- **QuickJS**: Small, embeddable C engine with Rust bindings (`quickjs-rs`) and ES2020 support. citeturn0search3
- **Bun**: A full JS/TS runtime built on JavaScriptCore, with TypeScript support built-in. citeturn0search0turn0search1turn0search2

## Feasible Integration Paths

### Path A: “Arniko HTML + Bliss DOM + JS Engine”
**Idea:** Arniko continues to generate HTML+CSS, then `bliss-core` parses the HTML into its DOM, runs the JS engine for scripts and event handlers, and renders via Bliss.

**Pros**
- Minimal JS/TS API surface to define in Arniko.
- Leverages existing `bliss-core` DOM and script engine.
- Capability gating already conceptualized in `bliss-core`.

**Cons**
- Arniko’s native/GPU paths need a bridge to Bliss DOM or a parallel “render-from-HTML” pipeline.
- Requires consistent mapping between Arniko component IDs and Bliss DOM nodes for event routing.

**Implementation concept**
- Add an Arniko “render-to-bliss” adapter:
  - `ArnikoApp::render()` -> HTML string.
  - Feed HTML into `bliss-core` document parser.
  - Use `bliss-core` `ScriptEngine` to run `<script>` tags.
  - Route events from Bliss DOM back to Arniko or expose direct DOM handles.

### Path B: “Arniko Component Tree as a Scriptable Runtime”
**Idea:** Expose Arniko’s component graph directly to JS (no HTML in between).

**Pros**
- Cleaner programmatic API for JS/TS.
- Avoids HTML parsing.

**Cons**
- Requires a custom JS<->Rust FFI API surface.
- More work to integrate with `bliss-core` DOM model.
- Higher risk of mismatch with existing Bliss rendering assumptions.

### Path C: “JS-Driven UI + Arniko as a Rendering Target”
**Idea:** Use `bliss-core` DOM as the canonical model, and Arniko only provides styling/components/themes.

**Pros**
- Single DOM model.
- JS integration is “first-class” in Bliss.

**Cons**
- Arniko becomes a styling/skin layer rather than the main UI API.
- Requires non-trivial refactors.

## TypeScript Support Options
1. **AOT TS -> JS** (recommended): Use `tsc`, `esbuild`, or `swc` in the build pipeline to produce JS bundles that `bliss-core` executes.
2. **Runtime TS**: Use a runtime that can execute TS directly (e.g., Bun). This is more operationally complex and less embeddable as a library. citeturn0search0turn0search1turn0search2

## Recommended Direction (Pragmatic)
**Path A + AOT TS->JS**
- Keep Arniko stable.
- Add a “bliss runtime adapter” that passes Arniko HTML into `bliss-core` and executes JS via its existing `js_engine`.
- Provide a small JS/TS API surface for event binding and DOM mutation (use the DOM bridge already present in `bliss-core`).
- Provide a build step for TS (no runtime TS execution initially).

## Crush-Primary Option (Polyglot-First)
If Crush is the primary scripting layer, JS/TS support can be treated as an embedded capability inside Crush (e.g., JS/TS blocks executed via the Crush runtime). In this model:
- **Arniko remains minimal** and only generates HTML/CSS.
- **Mustang/Bliss hosts the Crush runtime**, not a standalone JS engine.
- **Crush becomes the default scripting entrypoint**, and JS/TS is accessed through Crush blocks.

**Implications**
- JS/TS becomes an implementation detail of Crush rather than a top-level engine choice.
- The integration surface shifts from “JS engine + DOM bridge” to “Crush runtime + DOM bridge”.
- Capability gating and polyglot dispatch can live in one place (Crush runtime), reducing duplicated security policy.

**What needs to be defined**
- A **Crush DOM/Events API** (language-agnostic) that JS/TS blocks call through.
- A **bridge contract** between Crush and Bliss DOM (query, mutate, subscribe, dispatch).
- A **runtime policy** for which languages are enabled per capsule (JS/TS optional).

**Tradeoffs**
- Pro: Unified scripting model with built-in polyglot support.
- Pro: Cleaner capability story (one runtime gate).
- Con: Requires solid DOM/event APIs in Crush to avoid leaking engine details.
- Con: Debugging becomes multi-layered (Crush + embedded JS/TS).

## Key Risks / Open Issues
- **DOM Bridge Completeness**: `bliss-core`’s DOM bridge appears partial (e.g., `querySelector`, textContent, event handling). Needs extension for real UI behavior.
- **JS Engine Maturity**: V8/SpiderMonkey have heavier build and maintenance costs due to native dependencies and build tooling. citeturn1search1turn0search8
- **Performance**: HTML parsing + DOM layout + JS execution for every UI update may be expensive.
- **Security/Capabilities**: JS needs strict capability gating for DOM, network, filesystem, etc. (align with Exosphere’s model).
- **Threading**: JS engines often require thread affinity; `bliss-core` currently uses async flows and may require careful scheduling.

## Proposed Milestones (No Code Yet)
1. **Spike**: Wire Arniko HTML -> Bliss DOM -> JS Engine execution with a minimal example.
2. **Event Bridge**: Map Arniko-generated IDs to DOM nodes; dispatch click/input events to JS handlers.
3. **TS Build Step**: Add a sample `ts` -> `js` pipeline for capsules.
4. **Capability Gate**: Formalize a permission model for JS execution in Arniko-backed UIs.
5. **Performance Profiling**: Measure render latency and JS execution overhead.

## Concrete Next Step (If You Want Me to Implement)
I can draft a small integration design doc and a minimal test harness in `crates/apps/web/bliss-core/examples/` that:
- Takes Arniko HTML,
- Parses it into Bliss DOM,
- Executes a script that modifies the DOM,
- Renders the updated output.

If you want that, tell me which engine backend to target first: `boa` (default), `v8`, or `spidermonkey`.
