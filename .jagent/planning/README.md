# `.jagent/planning` — how this board works

| file | what it is |
|---|---|
| `RULES.md` | **Read first.** Standing discipline: verify-before-fix, one worktree per ticket, publishing traps. |
| `TASKS.md` | One line per ticket, grouped by priority. The index. |
| `STATE.md` | Where this backlog came from, what was actually verified, and the known-unknowns. |
| `tickets/ARNIKO-NNN-*.md` | The tickets themselves. |

## Ticket format

Header table (`ID`, `Priority`, `Status`, `Phase`, `Assignee`, `Dependencies`,
`Estimated effort`), then `## Problem`, `## Success criteria` (checkboxes),
`## Non-goals`, and — where a claim needs backing — `## Reproduction` or a
"how this was established" section.

`Status` is one of `Backlog` / `In progress` / `Done` / `Blocked`. A `Backlog`
status is **a claim, not a fact** — RULES §1 applies.

## Closing a ticket

Set `Status: Done`, add a `## Resolution` paragraph saying what actually happened,
and tick the matching `TASKS.md` line **in the same commit**. Never delete a ticket;
a wrong ticket gets a `## Resolution` explaining why it was wrong.

## Numbering

`ARNIKO-NNN`, monotonic, never reused. If you find a new problem while working a
ticket, file it as its own number rather than folding it in (RULES §1).
