# TASKS — arniko backlog

One line per ticket. Detail lives in `tickets/`. Tick a line **only** together with
setting that ticket's `Status: Done` + a `## Resolution` paragraph (RULES §1).

## P0 — silent correctness

- [ ] **ARNIKO-002** — `path` + `version` on a vendored-fork name silently ships
      `nicoburns`' crate to registry consumers instead of our fork. No warning.
      Live blast radius today is zero; becomes real on the next publish.

## P1 — blocks publishing / needs a captain decision

- [ ] **ARNIKO-001** — three crates publish under a different name than their manifest
      (`bliss`→`arniko-bliss`, `mustang`→`arniko-mustang`,
      `stylo_taffy`→`bliss-stylo-taffy`) and nothing in the repo records it. A plain
      `cargo publish` from the current tree does the wrong thing.
- [ ] **ARNIKO-003** — repo is private, 11 crates (~609 KB of source) are public and
      **irrevocable**. Captain to confirm which was intended.

## P2 — dependency strategy

- [ ] **ARNIKO-004** — vendored Blitz forks drift: published arniko depends on
      *upstream* `anyrender_vello ^0.7` while local builds use the fork, so local
      patches never reach registry consumers. Upstream is now 0.14.0.
- [ ] **ARNIKO-006** — consumers can't build against arniko from a git worktree
      (relative path-deps miss by one level). Registry deps verified to work —
      503 packages, 0 path deps. Blocked on ARNIKO-002.

## P3 — cosmetic

- [ ] **ARNIKO-005** — renamed crates use two prefixes (`arniko-` vs `bliss-`).
      Matters for the *next* rename, not the existing ones.

---

## Suggested order

`ARNIKO-003` (decision, unblocks nothing but cheap) → `ARNIKO-001` (records the map)
→ `ARNIKO-002` (closes the silent-substitution hazard) → `ARNIKO-004` (fork-vs-upstream
call) → `ARNIKO-006` (migrate consumers) → `ARNIKO-005` (tidy).

ARNIKO-002 and ARNIKO-006 are deliberately ordered that way: 006 is the migration that
would *expose* 002.
