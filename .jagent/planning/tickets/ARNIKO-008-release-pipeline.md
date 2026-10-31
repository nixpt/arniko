# ARNIKO-008 — releases skip crates; arniko-crush can't be installed

**Status:** In review
**Filed:** 2026-10-31, from a crates.io audit of the bliss/arniko crates (with bliss-engine BLISS-PIPE-1)

## Problem

- **0.3.1 went out for `arniko` only.** The workspace version moved to 0.3.1 for all three crates,
  but crates.io has `exo-bliss-net` and `arniko-crush` at 0.3.0: there is no publish workflow, the
  release was done by hand, and nothing checks that every crate goes out together.
- **`arniko-crush` 0.3.0 can't be installed** (nixpt/arniko#27): it requires `crush-lang-sdk ^0.2.0`,
  whose only match (0.2.0) is yanked. `main` requires 0.3.9, which is five crush-ast releases old.
- CI installs system packages with `awalsh128/cache-apt-pkgs-action@latest`, which restored a cached
  set without fontconfig's pkg-config file in bliss-engine and turned all of its CI red from
  2026-10-12 (bliss-engine BLISS-PIPE-1). It is green here today only by luck of the cache.

## Done when

- [x] `arniko-crush` depends on `crush-lang-sdk` 0.8.0 (the current crush-ast release):
      `cargo test -p arniko-crush --locked` 29 + 1 pass.
- [x] Workspace version 0.3.2 for all three crates.
- [x] `.github/workflows/publish.yml` (from bliss-engine's): tag-triggered, Trusted Publishing,
      publishes `exo-bliss-net` → `arniko` → `arniko-crush`, refuses mixed versions, skips what's
      already published.
- [x] CI installs apt packages with plain `apt-get` (+ `pkg-config`).
- [ ] CI green on the PR.
- [ ] Trusted Publishing configured on crates.io for the three crates (repo nixpt/arniko, workflow
      publish.yml) — owner.
- [ ] Released: tag `v0.3.2`; then yank `arniko-crush` 0.3.0 (it can't resolve).
