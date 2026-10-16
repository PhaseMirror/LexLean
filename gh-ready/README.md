# gh-ready — credential-transfer package (2026-10-16)

Everything short of the GitHub credential wall is done. This directory is the
wall crossing: each file is paste-ready, each action below is one `gh` call.

## Ownership statement (adopted definition)

> Owned means an actor with credentials has accepted the next commit or
> comment.

Under that definition:

| Action | Owner |
| --- | --- |
| Branches `issue-115-pd08`, `issue-116-pd09` (committed, gates run, failing tests verified) | done here |
| Paste-ready PR bodies, claim comments, decision/blocked nudges, escalations, deferral record, §31 protocol | done here |
| Opening the two draft PRs, posting every comment, pushing the branches | **credential holder (you)** — not accepted here; verified impossible in this environment (no `gh`, no token, no private key; anonymous API is read-only) |
| Merging, review requests, maintainer decisions | maintainers |

## Files

| File | Posts as / acts as |
| --- | --- |
| `lexlean-branches.bundle` | git bundle: `main` + `issue-115-pd08` (4fcb5b4f6) + `issue-116-pd09` (9f16effae) + `pr-82-rebased` (5d0f4377d) — fetch instructions in `run-when-authed.sh` step 0; `git bundle verify` passes ("complete history") |
| `pr-115-body.md` | body of draft PR for `issue-115-pd08` (#115, PD-08) |
| `pr-116-body.md` | body of draft PR for `issue-116-pd09` (#116, PD-09) |
| `claim-115.md` | comment claiming #115, register-first lane |
| `claim-116.md` | comment claiming #116, register-first lane |
| `nudge-decisions.md` | 4 comments: #107 (D2), #108 (D3), #109 (D4), #110 (D7) — resolved / delegate / defer by 2026-10-23 |
| `nudge-blocked.md` | Template A → #119–#125; Template B → #127 — same window |
| `nudge-68.md` | PR comment on #68: merge / close-with-supersession / reassign by 2026-10-19 |
| `escalation-91-92.md` | checkpoint comments on #91 and #92 (dated draft or reassignment by 2026-10-23) |
| `deferral-111.md` | #111 deferral record with reopen trigger + protocol pointer |
| `protocol-31-rebase.md` | §31 collision protocol (also referenced by the #111 deferral) |
| `run-when-authed.sh` | ordered `gh` sequence for all of the above |

## Receipt and start condition

- **Reachable path (canonical)**: `LexLean/gh-ready/` inside the Multiplicity
  working directory (this directory). The lever's literal `/workspace` does
  not exist on this machine; the workspace root is the Multiplicity tree, and
  everything here is on the user's own mount — no pull step. The repo itself
  is restored beside this directory at `LexLean/` with all twelve branches
  (worktree at `main` 96b382dfb; `issue-115-pd08`, `issue-116-pd09` and
  `pr-82-rebased` carry the session's commits).
- **Receipt**: 13 entries rendered with sizes and SHA-256 on 2026-10-16;
  byte-diff against the sandbox origin reported identical; bundle verified.
- **Start condition for the 1-day horizon**: begins when the credential holder
  opens this directory and confirms receipt. Until then the horizons below do
  not run — an inaccessible package is no longer a possible state.

## Horizons (from the lever table)

- 1 day (2026-10-17): draft PRs open; claim comments present; decision
  comments on #107–#110 and blocked-lane comments on #119–#125/#127 posted.
- 3 days (2026-10-19): PR #68 status comment posted (maintainer's answer due).
- 7 days (2026-10-23): decision window closes; escalation checkpoints on
  #91/#92 fire; #111 protocol adoption reviewed.

## If you do not act

No alternative path exists from this environment: without credentials no
comment, PR, or push can be created, and silently substituting local commits
for public records would break the very claim discipline the levers test.
The fallback recorded in the artifacts themselves is conservative: lanes stay
blocked, #106's plan stays accurate, and the branch evidence is no longer
sandbox-bound — `lexlean-branches.bundle` in this directory carries all three
branch tips with full history, so the commits survive this session either way.
