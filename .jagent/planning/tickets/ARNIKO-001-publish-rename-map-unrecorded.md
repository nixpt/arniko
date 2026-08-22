# ARNIKO-001 — Three crates publish under a different name than their manifest says

| Field | Value |
|-------|-------|
| **ID** | ARNIKO-001 |
| **Priority** | P1 |
| **Status** | Backlog |
| **Phase** | Publishing hygiene |
| **Assignee** | — |
| **Dependencies** | — |
| **Estimated effort** | S |

## Problem

Three crates in this workspace are published to crates.io under a **different name**
than the `package.name` in their own `Cargo.toml`, because the natural name was
already taken by an unrelated project:

| local `crates/…` | `package.name` today | published on crates.io as | bare name is owned by |
|---|---|---|---|
| `bliss` | `bliss` | **`arniko-bliss`** | `ajmwagar/bliss` 0.1.3 (2019) — *"Ignorance is bliss! Ignore your .gitignore"* |
| `mustang` | `mustang` | **`arniko-mustang`** | `sunfishcode/mustang` — *"Rust programs written entirely in Rust"* |
| `stylo_taffy` | `stylo_taffy` | **`bliss-stylo-taffy`** | `nicoburns` (dioxuslabs/blitz) |

**Nothing in the repository records this.** No manifest under `crates/` mentions
`arniko-bliss`, `arniko-mustang`, or `bliss-stylo-taffy`. Whoever published `0.2.99`
renamed out-of-band.

Two consequences:

1. **A plain `cargo publish` from the current tree does the wrong thing.** It tries to
   claim `bliss` / `mustang` / `stylo_taffy`, all of which belong to other people, and
   fails — or, if a version ever resolves, points consumers at a stranger's crate
   (that failure mode is ARNIKO-002).
2. **Nobody can tell which local crate a published one came from** without diffing
   descriptions, which is how this map was reconstructed in the first place.

## How this was established

Listed every crate owned by `nixpt` on crates.io (31 total, **11 from `nixpt/arniko`**),
then matched each back to a local directory by `description` + `version`. The three
above match by description exactly:

- local `crates/bliss` — *"High-level APIs for rendering HTML with Bliss"* = `arniko-bliss` 0.2.99
- local `crates/mustang` — *"Mustang: GPU-Accelerated Effect Compositor for Exosphere"* = `arniko-mustang` 0.2.99
- local `crates/stylo_taffy` — *"Interop crate for the stylo and taffy crates"* = `bliss-stylo-taffy` 0.2.99

Unchanged names, for completeness (all `nixpt`, all 0.2.99): `arniko`, `arniko-crush`,
`bliss-dom`, `bliss-html`, `bliss-net`, `bliss-paint`, `bliss-shell`, `bliss-traits`.
`exo-bliss-net` is also published from this repo but has **no matching directory under
`crates/`** — it is a distinct crate from `bliss-net`, not a rename of it.

## Success criteria

- [ ] Each renamed crate records its crates.io identity in its own manifest:
      ```toml
      [package]
      name = "arniko-bliss"   # crates.io identity
      [lib]
      name = "bliss"          # so `use bliss::…` is unchanged everywhere
      ```
- [ ] Internal dependants use the rename form so the local key stays readable:
      `bliss = { package = "arniko-bliss", version = "0.2.99", path = "../bliss" }`
- [ ] `cargo publish --dry-run` succeeds for all three without manual edits.
- [ ] `cargo check --workspace` still passes — no `use` statement anywhere had to change.
- [ ] The mapping table above lands in `README.md` or `PROJECT.md`.

## Non-goals

- Renaming the *local directories*. `crates/bliss` can stay `crates/bliss`.
- Trying to acquire the bare `bliss` / `mustang` / `stylo_taffy` names.
- Resolving the prefix inconsistency — that is ARNIKO-005.

## Notes

Consumers are unaffected either way: they write `arniko = "0.2.99"` and never name
the renamed crates directly. This ticket is about making the repo self-describing so
the next publish does not depend on someone remembering.
