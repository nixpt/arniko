# RULES — standing discipline for anyone working this backlog

Not suggestions. Every agent touching `.jagent/planning/` follows them.

## 1. Verify before you fix

**A ticket's `Backlog` status is a claim, not a fact.** Before spending effort:

1. Re-run its `## Reproduction` / `## Success criteria` verbatim against current
   `main` (pull first, unmodified tree).
2. If it no longer reproduces, or is already satisfied: set `Status: Done`, add a
   one-paragraph `## Resolution`, and tick the `TASKS.md` line. Do **not** silently
   delete the ticket.
3. If it does reproduce: fix it.

Recursive: a NEW bug found while working a ticket gets its **own** ticket, not a
silent fold-in, so it gets its own verify cycle later.

## 2. One worktree/branch per ticket

`git worktree add -b agent/<name>/ARNIKO-NNN …`. Never commit to `main` directly.

⚠️ `git worktree add -b` silently sets the new branch to **track `origin/main`** — a
bare `git push` from it targets main. Always push explicitly:
`git push -u origin agent/<name>/ARNIKO-NNN`.

## 3. This repo does not build from a git worktree

Peer path-deps (`../crush-ast`, `../crush-workspace/polydex`) resolve one directory
short from `.claude/worktrees/<name>/`, and workspace inheritance then fails with
confusing errors like `` `workspace.dependencies` was not defined ``. Symlink shims
work but are fragile. **Build and test from a checkout at `projects/` depth**, or
from the main checkout. See ARNIKO-006.

## 4. Publishing is not a plain `cargo publish`

Three crates publish under **different names** than their local package name, and
that mapping is not yet recorded in any manifest. Publishing without reading
ARNIKO-001 will either fail (name taken by a stranger) or, worse, silently point
consumers at somebody else's crate (ARNIKO-002).

## 5. crates.io publishes are irrevocable

`cargo yank` blocks new dependents resolving to a version. It does **not** remove the
source tarball — anyone can still download it anonymously, forever. Treat every
publish as permanent disclosure. See ARNIKO-003.

## 6. Don't "helpfully" de-vendor the Blitz forks

`accesskit_xplat`, `anyrender_vello`, `debug_timer`, `stylo_taffy` intentionally carry
upstream names while being local forks. Replacing them with upstream is a real
behaviour change, not cleanup — it is ARNIKO-004, and it needs a decision first.
