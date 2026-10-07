# ARNIKO-003 — Repo is private; its 11 crates are public and irrevocable. Decide which is intended.

| Field | Value |
|-------|-------|
| **ID** | ARNIKO-003 |
| **Priority** | P1 |
| **Status** | Backlog — needs a decision, not an implementation |
| **Phase** | Publishing hygiene |
| **Assignee** | captain |
| **Dependencies** | — |
| **Estimated effort** | XS to decide |

## Problem

`nixpt/arniko` is a **private** GitHub repository. Eleven of its crates are
**published on crates.io**, which means their full source is public.

Verified with **no credentials at all** — plain HTTP to `static.crates.io`, no cargo,
no `gh`, no GitHub session. Every one downloads anonymously:

| crate | tarball |
|---|---|
| `arniko` | 212,075 B |
| `bliss-dom` | 167,464 B |
| `bliss-shell` | 51,900 B |
| `bliss-paint` | 46,105 B |
| `arniko-bliss` | 35,289 B |
| `bliss-stylo-taffy` | 25,783 B |
| `bliss-html` | 19,057 B |
| `bliss-net` | 19,317 B |
| `exo-bliss-net` | 16,921 B |
| `bliss-traits` | 16,431 B |
| `arniko-crush` | 12,996 B |

**≈ 609 KB of source from a private repo, readable by anyone.**

And crates.io publishes are **irrevocable**: `cargo yank` stops *new* dependents from
resolving to a version, it does **not** remove the tarball. Nothing here can be
un-published.

The vendored Blitz forks — `accesskit_xplat`, `anyrender_vello`, `debug_timer`,
`tui-shell` — are **not** published and are therefore not exposed.

## The decision

Only the captain can make this call. Two coherent answers:

**(a) The repo is private for confidentiality.** Then that is already moot for
whatever lives in those 11 crates, and yanking will not undo it. The follow-up is a
review of what was disclosed, not a takedown — and a rule about what may be published
in future.

**(b) The repo is private only because it is unannounced.** Then there is no problem
at all, and the situation is actually *favourable*: a crates.io dependency needs no
credentials, whereas a git dependency on a private repo needs a PAT in CI. That makes
crates.io the strictly better distribution channel for consumers.

## Success criteria

- [ ] Captain records (a) or (b) in this ticket's `## Resolution`.
- [ ] If (a): list what was disclosed and state the go-forward publishing rule.
- [ ] If (b): consider making the repo public so the `repository` metadata resolves,
      or accept the broken links below.
- [ ] Either way, `RULES.md` §5 (publishes are permanent) stays.

## Non-goals

- Yanking anything. Yanking does not achieve confidentiality and breaks consumers.

## Side finding (cosmetic)

The published `repository` field points at `https://github.com/nixpt/arniko`, which is
a **404 for everyone outside the org**. docs.rs "Source" links are therefore broken
for all external readers. Harmless, but it looks like an abandoned crate.
