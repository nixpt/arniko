# ARNIKO-005 — Renamed crates use two different prefixes (`arniko-` vs `bliss-`)

| Field | Value |
|-------|-------|
| **ID** | ARNIKO-005 |
| **Priority** | P3 |
| **Status** | Backlog |
| **Phase** | Publishing hygiene |
| **Assignee** | — |
| **Dependencies** | ARNIKO-001 |
| **Estimated effort** | XS to decide |

## Problem

The three crates renamed at publish time did not follow one convention:

| local | published | prefix used |
|---|---|---|
| `bliss` | `arniko-bliss` | `arniko-` |
| `mustang` | `arniko-mustang` | `arniko-` |
| `stylo_taffy` | **`bliss-stylo-taffy`** | `bliss-` |

Two of three took `arniko-`; the third took `bliss-`. Since eight *un*-renamed crates
already carry a natural `bliss-` prefix (`bliss-dom`, `bliss-html`, `bliss-net`,
`bliss-paint`, `bliss-shell`, `bliss-traits`), `bliss-stylo-taffy` reads as though it
were an original Bliss crate rather than a renamed fork — while `arniko-bliss` reads as
though `bliss` were subordinate to `arniko`, which is backwards from how the code is
layered.

Low severity: nothing is broken, and existing published names cannot be reclaimed
anyway. It matters for the **next** crate that needs a rename, of which there are up
to four candidates in ARNIKO-004.

## Success criteria

- [ ] A one-line rule recorded in `PROJECT.md`, e.g. *"a crate renamed only to dodge a
      taken name takes the `arniko-` prefix; `bliss-` is reserved for crates that are
      genuinely part of the Bliss layer."*
- [ ] ARNIKO-004's namespaced option, if taken, follows that rule.
- [ ] Existing published names are left alone and noted as grandfathered.

## Non-goals

- Re-publishing or deprecating `bliss-stylo-taffy` to fix its prefix. Not worth the
  churn for a cosmetic inconsistency, and it would strand existing dependants.
