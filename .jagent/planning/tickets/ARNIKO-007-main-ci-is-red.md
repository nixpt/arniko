# ARNIKO-007 — `main` CI is red and has been for at least three runs

| Field | Value |
|-------|-------|
| **ID** | ARNIKO-007 |
| **Priority** | P1 |
| **Status** | Backlog |
| **Phase** | Repo hygiene |
| **Assignee** | — |
| **Dependencies** | — |
| **Estimated effort** | M |

## Problem

The CI workflow fails on `main`, and has for its last three recorded runs
(`f358c1c`, `fb8de9d`, `472c27f`). Two jobs fail and everything downstream is
**skipped**, so `main` currently has **no passing build, test, or clippy signal at
all**:

```
fmt + clippy                              failure
cargo audit (F-5)                         failure
test (arniko + bliss-dom + mustang)       skipped
check --workspace (E-4)                   skipped
build arniko + mustang                    skipped
check (${{ matrix.os }}) (E-4)            skipped
```

### `cargo audit (F-5)` — 4 unfixed advisories

```
error: 4 vulnerabilities found!
  RUSTSEC-2026-0204
  RUSTSEC-2026-0194
  RUSTSEC-2026-0195
  RUSTSEC-2026-0185
```
plus warnings incl. `RUSTSEC-2025-0141`, `RUSTSEC-2024-0436`, `RUSTSEC-2026-0206`,
`RUSTSEC-2026-0192` (*"Unsoundness in `Error::downcast_mut()`"*).

### `fmt + clippy` — fails separately

Not diagnosed here; run it locally to see.

## Why this matters beyond the obvious

A permanently-red `main` means **every** PR is red, so red stops carrying
information. Any real regression a contributor introduces is indistinguishable from
the standing failure — which is how a genuine break ships unnoticed. It also makes
`RULES.md` §1 ("verify against current main") much harder to follow honestly.

This ticket was filed *because* it caused exactly that confusion: PR #13 is a
**documentation-only** change (13 markdown files, zero source) and its CI went red.
Establishing that the failure was pre-existing required diffing job-for-job against
`main`'s run at the same base commit.

## Reproduction

```bash
gh run list --branch main --limit 3
gh run view <id> --json jobs --jq '.jobs[] | "\(.name): \(.conclusion)"'
gh run view <id> --log-failed | grep -E "error:|RUSTSEC"
```

## Success criteria

- [ ] `cargo audit` passes, or each of the 4 errors is explicitly accepted with a
      written justification in `.cargo/audit.toml` (khukuri does this — see its
      `chore(audit): accept RUSTSEC-…` commits for the pattern).
- [ ] `cargo fmt --check` and `cargo clippy` pass on `main`.
- [ ] The downstream `test` / `check --workspace` / `build` jobs actually **run** on
      `main`, so there is a real green baseline.
- [ ] A green run recorded on `main`.

## Non-goals

- Fixing the advisories by upgrading if the fix belongs upstream in the vendored
  Blitz forks — that overlaps ARNIKO-004 and should be sequenced with it.

## Notes

Not caused by, and not fixable within, PR #13. Filed from that PR so it is visible
rather than rediscovered by the next person whose docs change goes red.
