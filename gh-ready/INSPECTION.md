# gh-ready inspection record

Date: 2026-10-16 (afternoon, post-restore)
Inspector: sandbox agent (not the credential holder — see Precision answers)

## Scope

Full internal inspection of `LexLean/gh-ready/` (13 entries) after the
restore into the workspace tree. Six checks, all executed, all passing:

1. **Inventory** — 13 files; sizes match the 08:54 receipt listing
   (bundle 94,012,813 B; all others byte-for-byte as listed).
2. **SHA-256** — 11/13 hashes identical to the 08:54 receipt. The two
   expected diffs: `README.md` and `run-when-authed.sh`, both edited
   after that receipt (canonical-path update; sandbox-path removal).
   Bundle hash unchanged: `00f17ebea29cc5f676bccd4997c7c0ac30add064a465fc0a0fd5b6f5079988ce`.
3. **Runner + bundle** — `bash -n run-when-authed.sh` clean;
   `git bundle verify` reports complete history.
4. **Stale paths** — `grep -rn '/tmp/opencode'` over the package: zero
   hits. Every path is workspace-relative or uses `$(dirname "$0")`.
5. **Cross-references** — PR bodies and runner both name branches
   `issue-115-pd08` / `issue-116-pd09`; date fields cluster on the
   adopted horizons 2026-10-17 / -19 / -23 with no strays.
6. **Bundle ↔ repo tips** — `git bundle list-heads` tips equal the
   restored repo's branch tips exactly:
   `issue-115-pd08` 4fcb5b4f617ab5834e3b4647c308572495ef553e,
   `issue-116-pd09` 9f16effae879ff9dca53f340704016c286b4f2b4,
   `pr-82-rebased` 5d0f4377d284dd788ff40cd99004e04447d090c6.

## Precision answers (recorded)

- **Is inspection complete?** Yes — by this agent. This record is the
  evidence. It is **not** the credential holder's inspection.
- **Remaining blocker to execution:** GitHub credentials (no `gh`, no
  token, no deploy key — verified this session). The credential holder
  must (a) inspect this directory and (b) run `run-when-authed.sh` or
  the equivalent `gh` calls. Neither (a) nor (b) can be performed or
  substituted from this environment.
- **1-day horizon:** has **no measured start**. It starts on the
  credential holder's inspection of this directory, not on this record.

## Location binding (2026-10-16, post-restore audit)

The package exists at **exactly one** reachable path:
`/media/citizen/b361d448-7c51-413a-aa23-9515cb626930/home/citizen/Multiplicity/LexLean/gh-ready/`.
Evidence: `find` over the workspace (depth 3) returns no other `gh-ready*`;
`grep -rn 'artifacts/gh-ready'` over the package returns zero path claims
(the only `artifacts` word-hits are ordinary English in README's fallback
paragraph, a verbatim issue quotation in `pr-115-body.md`, and blob content
inside the bundle — none are location claims). The pre-restore
`artifacts/gh-ready/` placement was deleted; the sandbox backup at
`/tmp/opencode/gh-ready/` is outside the workspace and unreachable to the
credential holder. The stale `artifacts/` references existed only in the
agent's chat session summaries, not in any artifact; this section closes
that dissonance on-tree.

## Open decision (credential holder)

`gh-ready/` is untracked in the LexLean repo. Recommendation:
**keep local-only** — the package is transfer tooling, not repository
content; committing it into any PR branch would ship tooling text as
code review material and dirty the one-branch-per-issue discipline.
No action required to accept the recommendation; it stands as default
unless overruled.
