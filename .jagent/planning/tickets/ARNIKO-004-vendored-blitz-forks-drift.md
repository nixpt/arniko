# ARNIKO-004 — Vendored Blitz/Dioxus forks are drifting from upstream, and published arniko already bypasses them

| Field | Value |
|-------|-------|
| **ID** | ARNIKO-004 |
| **Priority** | P2 |
| **Status** | Backlog |
| **Phase** | Dependency strategy |
| **Assignee** | — |
| **Dependencies** | ARNIKO-002 |
| **Estimated effort** | L |

## Problem

`crates/` vendors four crates from the Blitz/Dioxus stack under their **upstream
names**: `accesskit_xplat`, `anyrender_vello`, `debug_timer`, `stylo_taffy`. All four
names belong to `nicoburns` on crates.io (dioxuslabs/blitz, dioxuslabs/anyrender).
These are deliberate forks, not name-squats.

Two problems have accumulated:

**1. Local and published arniko are not the same dependency graph.**

```
local build      →  crates/anyrender_vello        (our fork)
published 0.2.99 →  anyrender_vello ^0.7          (nicoburns' upstream)
```

The published manifest depends on upstream. So **any local patch in the vendored copy
never reaches a crates.io consumer** — `khukuri-desktop` or `cece-code` built from the
registry get different rendering code than the same commit built locally. Nobody is
told.

**2. The pin is far behind.**

`anyrender_vello` is pinned at `^0.7`; upstream is at **0.14.0** (updated 2026-08-16).
That is a wide gap in 0.x semver, i.e. several incompatible releases.

There is also a concrete symptom: building arniko through a symlink fails with
`` error inheriting `vello` from workspace root manifest's `workspace.dependencies` ``
— the vendored copy carries its own workspace-inheritance expectations that do not
survive being reached from outside its own tree. See ARNIKO-006.

## The decision to make

Per crate, pick one and record it:

- **Keep the fork** → then it must not carry a bare upstream name with a `version`
  key (ARNIKO-002), and the published manifest should point at the fork, not upstream,
  or the divergence above is permanent and silent.
- **Adopt upstream** → delete the vendored crate, depend on the real one, and take the
  0.7 → 0.14 migration.
- **Fork with a namespaced name** → e.g. `bliss-anyrender-vello` (available on
  crates.io as of 2026-08-22), published, so local and registry builds agree.

## Success criteria

- [ ] Each of the four has a recorded decision in this ticket.
- [ ] Local build and a from-crates.io build of the same commit resolve the **same**
      code for every one of the four (or the difference is documented deliberately).
- [ ] If upstream is adopted anywhere, `anyrender_vello` moves off `^0.7` and the
      workspace-inheritance symptom in ARNIKO-006 is re-tested.
- [ ] `cargo tree -d` shows no case of the fork and upstream both in the graph.

## Non-goals

- Doing the 0.7 → 0.14 migration inside this ticket. That is its own piece of work
  once the direction is chosen.

## Notes

Name availability checked 2026-08-22 for the namespaced option:
`bliss-anyrender-vello`, `bliss-accesskit-xplat`, `bliss-debug-timer`, and
`arniko-stylo-taffy` were all **free**. `bliss-stylo-taffy` is already taken — by us
(see ARNIKO-001).
