Claiming #115 (plan item L3.2 of #106) for the register-first lane, following the order #106 sets for every item: a §31 row, a scenario and a failing test before any implementation.

**Scope of the first PR**: R3 entry only — `PD-08` (suite `production`, ref §18.4), its scenario in `features/suites/production.feature`, the named test `conformance_pd_08` (expected red: no case wired), the regenerated `CONFORMANCE.md`, and the capability count 381 → 382 at its four pinned sites. No diagnostic code, no lowering change, no semantic ID.

**Scope of the follow-up PR** (after the row lands): the lowering change itself — a content-addressed artifact is decoded once at build and appears as data in the lowered program, with the corpus-root acceptance check from the issue, and the planted per-call-decoding defect recorded in VERIFICATION.md.

If someone is already designing this item, say so here and I will reassign or close the draft.
