# ARNIKO-006 — Consumers cannot build against arniko from a git worktree; crates.io deps would fix it

| Field | Value |
|-------|-------|
| **ID** | ARNIKO-006 |
| **Priority** | P2 |
| **Status** | Backlog |
| **Phase** | Dependency strategy |
| **Assignee** | — |
| **Dependencies** | ARNIKO-002 |
| **Estimated effort** | S per consumer |

## Problem

Consumers depend on arniko by relative path:

```toml
# nixpt/Khukuri — crates/khukuri-desktop/Cargo.toml
arniko    = { path = "../../../arniko/crates/arniko", features = ["html","components","launch","reactive"] }
bliss-dom = { path = "../../../arniko/crates/bliss-dom" }
```

A git worktree sits one directory deeper than the main checkout, so every `../` chain
misses and the build dies before compiling anything. The errors are actively
misleading — the first one blames a crate the developer has never heard of:

```
failed to load manifest for dependency `arniko`
  error inheriting `vello` from workspace root manifest's `workspace.dependencies`
  `workspace.dependencies` was not defined
```

Symlink shims (`.claude/worktrees/arniko -> …`) partly work but are fragile — they fix
one layer and expose the next (`polydex`, then `crush-ast`), and cargo's
workspace-root walk resolves differently through a symlink. **The workflow this breaks
is the standard one-worktree-per-ticket discipline**, so it is hit constantly.

## The fix is available today

Verified 2026-08-22: **the whole arniko graph resolves from crates.io with zero path
deps.** A throwaway crate depending only on

```toml
arniko        = { version = "0.2.99", features = ["html","components","launch","reactive"] }
bliss-dom     = "0.2.99"
bliss-traits  = "0.2.99"
exo-bliss-net = "0.2.99"
```

locked **503 packages, 100% from the registry**, and `cargo fetch` downloaded all of
them. All four features `khukuri-desktop` needs (`html`, `components`, `launch`,
`reactive`) exist on the published `0.2.99`.

Because arniko's crates are public on crates.io even though the repo is private
(ARNIKO-003), a **registry dep needs no credentials** — unlike a git dep on the
private repo, which would need a PAT in CI. That makes crates.io the better channel.

## Recommended shape

Registry version by default; local sources become **opt-in** rather than mandatory:

```toml
# consumer Cargo.toml — committed
arniko = { version = "0.2.99", features = [...] }
```

```toml
# consumer .cargo/config.toml — gitignored, per-developer
[patch.crates-io]
arniko    = { path = "/workspace/projects/arniko/crates/arniko" }
bliss-dom = { path = "/workspace/projects/arniko/crates/bliss-dom" }
```

A *committed* `path` is what makes builds depth-sensitive; a `[patch]` override is
opt-in, box-local, and works from any directory.

## Success criteria

- [ ] `khukuri-desktop` builds from a fresh `git worktree add` with no symlink shims.
- [ ] The `[patch.crates-io]` recipe is documented for developers wanting local arniko.
- [ ] Round-trip verified: with the patch active, a local edit to `crates/arniko` is
      picked up by the consumer build.

## Blocked-on / caveat

**Do not do this before ARNIKO-002.** Moving a consumer to registry versions is
exactly the operation that would expose the silent fork→upstream substitution, and
it also silently swaps vendored `anyrender_vello` for upstream `^0.7` (ARNIKO-004).
The plumbing change is safe; the *behaviour* change underneath it is not yet decided.

Note for `nixpt/Khukuri` specifically: its **library** has a second, unrelated blocker
— `polydex` is unpublished and private, so khukuri-core cannot go registry-only until
that is published or switched to a git dep. `khukuri-desktop` can move independently.
