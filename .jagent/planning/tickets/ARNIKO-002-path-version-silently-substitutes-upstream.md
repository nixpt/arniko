# ARNIKO-002 — `path` + `version` on a vendored-fork name silently ships a stranger's crate

| Field | Value |
|-------|-------|
| **ID** | ARNIKO-002 |
| **Priority** | P0 |
| **Status** | Backlog |
| **Phase** | Publishing hygiene |
| **Assignee** | — |
| **Dependencies** | ARNIKO-001 |
| **Estimated effort** | S to diagnose, M to decide |

## Problem

The root `Cargo.toml` declares vendored forks with **both** a `path` and a `version`:

```toml
stylo_taffy   = { path = "crates/stylo_taffy",  version = "0.2.99", default-features = false }
debug_timer   = { path = "crates/debug_timer",  version = "0.1.3" }
```

For a normal crate that dual form is correct and idiomatic — `path` wins in-workspace,
`version` is what gets recorded for published consumers. **It is dangerous here**,
because `debug_timer` and `stylo_taffy` on crates.io are **not ours** — both belong to
`nicoburns` (dioxuslabs/blitz).

So the two builds diverge silently:

| | resolves to |
|---|---|
| local build | `crates/debug_timer` — our fork |
| any crates.io consumer | **`nicoburns`' `debug_timer` 0.1.3** |

Same name, same version number, **different code**, and **no warning of any kind**.
Cargo is behaving exactly as documented; the hazard is entirely in the name collision.

This is the highest-severity item in this backlog because it fails silently and in the
direction that reaches other people's machines, not ours.

## Reproduction

1. `grep -n "debug_timer\|stylo_taffy" Cargo.toml` — observe the dual `path` + `version`.
2. `curl -s https://crates.io/api/v1/crates/debug_timer | jq '.crate.repository'`
   → `https://github.com/dioxuslabs/blitz` (not ours).
3. Same for `stylo_taffy` → `nicoburns`, currently `0.3.0-beta.1`.
4. Any published crate carrying that dependency line resolves to the upstream crate.

## Why it has not bitten yet

The published `arniko 0.2.99` does **not** list `debug_timer` or `stylo_taffy` among
its direct dependencies — its normal deps are `anyhow`, `parking_lot`, `serde`,
`serde_json`, `url`, and the optional `anyrender_vello`, `arniko-bliss`,
`arniko-mustang`, `bliss-dom`, `exo-bliss-net`, `keyboard-types`, `winit`. So the
live blast radius today is zero. It becomes real the moment a crate that *does* carry
those lines is published.

## Success criteria

- [ ] Every dual `path` + `version` dependency in the workspace is audited against
      crates.io ownership; any whose bare name is not ours is fixed.
- [ ] Fix is one of, decided per crate and written down:
      - publish the fork under a namespaced name (as ARNIKO-001 does), then depend on
        it via `package = "…"`; **or**
      - drop the `version` key so the dep is path-only and the crate is explicitly
        unpublishable; **or**
      - genuinely adopt upstream and delete the fork (that is ARNIKO-004, not this).
- [ ] A note in `RULES.md` warns against re-adding a bare `version` to a vendored name.
- [ ] `cargo publish --dry-run` on the affected crates resolves only to crates we own.

## Non-goals

- Deciding fork-vs-upstream. This ticket only ensures we never *silently* ship
  upstream while believing we shipped the fork.

## Notes

Discovered while evaluating whether consumers could move from path-deps to crates.io
versions. The evaluation itself came out positive — see ARNIKO-006 — but this hazard
must be closed first, because that migration is exactly what would expose it.
