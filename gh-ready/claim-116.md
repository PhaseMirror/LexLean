Claiming #116 (plan item L3.3 of #106) for the register-first lane, following the order #106 sets for every item: a §31 row, a scenario and a failing test before any implementation.

**Scope of the first PR**: R3 entry only — `PD-09` (suite `production`, refs §17.13, §17.17), its scenario in `features/suites/production.feature`, the named test `conformance_pd_09` (expected red: no case wired), the regenerated `CONFORMANCE.md`, and the capability count 381 → 382 at its four pinned sites. No diagnostic code, no eligibility or rendering change, no semantic ID.

**Scope of the follow-up PR** (after the row lands): kernel-checked bounds that cover every checked operation of a corpus root, eligibility admitting the root without `overflow` on that evidence, the rendering dropping the covered checks, and the removal-of-the-bound plant recorded in VERIFICATION.md. This item touches the certificate path, so it coordinates with the preservation lanes (#89, #91 stage 2) before any library change.

If someone is already designing this item, say so here and I will reassign or close the draft.
