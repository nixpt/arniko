# arniko

Native Rust UI stack — the Bliss render/DOM layer plus the `arniko` application shell,
used by `cece-code` and `khukuri-desktop`.

## Identity

- **Repository:** `nixpt/arniko` — **PRIVATE**
- **Language:** Rust (workspace, `crates/*`)
- **Published:** 11 crates on crates.io at `0.2.99` (see `planning/tickets/ARNIKO-003`)
- **Consumers:** `khukuri-desktop` (`nixpt/Khukuri`), `cece-code`

## Shape

`crates/` holds three different kinds of crate, and the distinction matters more than
it looks — most of the open tickets exist because it was not written down anywhere:

1. **Original Bliss/arniko crates** — `arniko`, `arniko-crush`, `bliss`, `bliss-dom`,
   `bliss-html`, `bliss-net`, `bliss-paint`, `bliss-shell`, `bliss-traits`.
   Ours, published.
2. **Vendored forks of the Blitz/Dioxus stack**, kept under their *upstream* names —
   `accesskit_xplat`, `anyrender_vello`, `debug_timer`, `stylo_taffy`. On crates.io
   those names belong to `nicoburns` (dioxuslabs). Not published by us (except
   `stylo_taffy`, which publishes renamed — see ARNIKO-001).
3. **Unpublished local-only** — `tui-shell`, `mustang` (publishes renamed).

**The local crate name is not always the crates.io name.** Three crates are renamed at
publish time and nothing in the repo records it. That is ARNIKO-001, and it is the
first thing to read before publishing or migrating a consumer.

**Working this backlog?** Read `.jagent/planning/RULES.md` first.
