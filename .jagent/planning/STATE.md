# Planning state — arniko

**Created:** 2026-08-22 (s471, vega) — first `.jagent/` board in this repo.
**Branch at creation:** `theme-adaptation-mactahoe` (active WIP, untouched by this board).
**Published:** 11 crates on crates.io at `0.2.99`, last published 2026-06-20.

## Where these tickets came from

They are **not** a planned roadmap. Every one was found while answering a different
question — *"can consumers depend on arniko via crates.io instead of relative paths?"*
— which came out of `khukuri` being unable to build from a git worktree.

That matters for how to read them: the backlog is **audit output, not a design**.
Nobody has decided what arniko *should* do about any of it. ARNIKO-003 and ARNIKO-004
are explicitly decisions, not implementations.

## What was actually verified (not assumed)

- **The full published crate set** — enumerated via crates.io's owner API: `nixpt`
  publishes 31 crates, 11 of them from `nixpt/arniko`. The local→published name map
  was reconstructed by matching `description` + `version`, not guessed.
- **Registry resolution works** — a throwaway crate depending on `arniko` (with
  khukuri-desktop's exact feature set) + `bliss-dom` + `bliss-traits` +
  `exo-bliss-net` locked **503 packages, 100% registry, 0 path deps**; `cargo fetch`
  downloaded all of them.
- **Anonymous source access** — all 11 `.crate` tarballs download from
  `static.crates.io` with **no credentials of any kind**.
- **Name ownership** — every crate name under `crates/` was checked against crates.io
  ownership. 8 ours, 6 owned by others (`ajmwagar`, `sunfishcode`, `nicoburns` ×4),
  1 unpublished.

## Known-unknowns

- Whether the vendored Blitz forks carry local patches that matter. Nobody diffed
  them against upstream; ARNIKO-004 assumes they might.
- Whether `exo-bliss-net` (published from this repo, no matching `crates/` dir) lives
  somewhere else in the workspace or is stale.
- Whether `tui-shell` is intended to be published at all.

## Not covered by this board

Rendering/UI work, the MacTahoe theme adaptation in flight on the current branch, and
anything about `cece-code`. This board is scoped to **publishing and dependency
hygiene** only.
