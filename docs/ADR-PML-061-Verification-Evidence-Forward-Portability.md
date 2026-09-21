# ADR-PML-061: LexLean verification evidence is not forward-portable (pinned axiom forms, fixed budgets)

## Status
Proposed

## Axis (Phase Mirror tension class)
control desired vs available

## Owner (multi-agent lever)
`the-examiner`

## Dissonance Score
- Impact = severity (3) x blast radius (9) = **27**
- Tractability = **6.0**
- **Score = 162.0** (LexLean audit, rank 5 of 6)

## Context (stated intent vs implementation)

### Stated intent (documents)
- `packages/LexLean/README.md` (verification block): verification is
  "reproducible across different absolute directories" (`R10` determinism;
  `AR-08`/`AR-13`/`EX-06`), and the produced attestation is
  content-addressed and portable.
- `packages/LexLean/CONFORMANCE.md` (`AR-14`): platform-bound attestation bytes
  vs platform-independent artifact distinction exists.
- `packages/LexLean/SPEC.md` §17.11: fixed generated backend options —
  `maxRecDepth = 100000`, `maxHeartbeats = 1000000000` — "fixed generated
  backend options, not source-supplied commands or request overrides; neither
  may be zero (unlimited)".

### Implementation reality (LexLean corpus)
- SPEC §22.5: the axiom parser "accepts only the pinned exact output forms"
  (`VR-10`), namely the *4.32.1* normalized payload — verbatim text:
  `'<name>' does not depend on any axioms`. Any future Lean whose
  `# print axioms` output changes even cosmetically makes **every existing
  attestation unreadable** by the same compiler-instrument — evidence stops
  being evidence under the newest toolchain of the same product line.
- SPEC §17.11 (line 2391): "Exhausting a finite Lean budget still fails
  verification; the budgets do not assert that every resource-bounded source
  can be verified on every host." So a `verified` status is a statement about
  *this host, this budget profile, this toolchain* — yet the public surface
  (`docker run … verify`) presents verification as a portable yes/no.
- SPEC §8.2: attestations hash the `lean`/`lake`/`leanchecker` executables
  used, and the pinned Lean tag resolves to `f054605…`; the *budget profile*
  and the *parser grammar* are not hashed into the attestation.

### Contradiction (productive)
Determinism (`R10`) governs the *artifact bytes*; verification *success* is
governed by host resources and budget constants. The product is honest about
each claim separately but the consumer reading "verified, content-addressed,
portable, reproducible" cannot distinguish the two. And the portability story
reaches a dead end by design: the only reader (the pinned axiom parser) is
shut to the very future toolchains the product line is expected to move into.
The productive reading: LexLean's evidence is a *snapshot credential*, not a
*portable credential* — and that is fine only if the snapshot is fully
described (toolchain + budgets + parser grammar + digests).

### Hidden assumptions
- **Forward readability implied**: an attestation created today is expected to
  be re-checkable tomorrow, while §22.5 guarantees exactly the opposite for
  any payload drift.
- **Budget stability across the line**: the fixed constants are assumed to be
  equal across host classes (memory/CPU) and across compiler patch series,
  unrecorded and thus unverifiable in an attestation.
- **Unit-cost stability**: `maxHeartbeats` is measured in "thousands of small
  allocations" whose cost varies by compiler version, so two toolchains that
  both "respect the budget" can still differ on whether a module *passes*,
  without any digit in the attestation changing to explain it.

### Manifested boundary
"Verified" is attained under a resource profile; attested evidence is
readable only under one pinned grammar; neither profile nor grammar is part
of the attestation record. Leaked (unmanifested): no — manifested by this
ADR.

## Decision (the lever)
Turn the snapshot into a *fully described* snapshot and scope the portability
phony claims:

1. **Attest the program too**: add to every attestation record: the resolved
   Lean commit (`f054605…` already pin), the effective `maxRecDepth`/
   `maxHeartbeats` used, the axiom-parser grammar revision (SPEC §22.5 + a
   synthetic `AXE` claim id), so two runs that could differ have their
   differing parameters in the record.
2. **Portability criteria, normative**: publish the conditions under which an
   attestation may be imported at a different budget profile — a *budget
   superset* rule (a higher-budget host may re-run; a lower-budget host must
   refuse and say why) — so "verified" is no longer unbounded.
3. **Forward-proofing of the parser**: decouple the axiom normalizer from the
   4.32.1-only exact strings by documenting the *entry-point* over which the
   grammar holds (`does not depend on any axioms` family) and versioning the
   accepted forms in the register, so a future Lean upgrade first produces a
   `VR-10` *delta attestation* instead of silent unreadability.

## Consequences
- **Positive**: attestations differ visibly when budgets/toolchain differ;
  consumers get an explicit import rule instead of a silent re-run; the
  4.32.1→4.34 plane change (see ADR-PML-057) becomes an auditable delta
  rather than an opaque break.
- **Negative / Constraints**: attestation payload growth must obey §30
  byte-compat rules (append-only fields); a "budget superset" rule is a
  semantics decision that must be ratified by the conformance gate
  (`model`/`CB-12`-style suite).
- **Verification Strategy**: a drill that (a) runs a module at two different
  budgets and shows the attestations differ in the budget field, and (b) runs
  a future-simulated `# print axioms` form and shows the parser emits a
  `VR-10` delta rather than silent accept/reject.

## Metrics (resolution is confirmed when)
- Every attestation includes toolchain commit + budgets + parser grammar
  fields.
- The budget-superset import rule is normative text in SPEC §8.x/§30 and
  white-box tested by the conformance gate.
- A `VR-10` delta attestation for a foreign grammar form exists in the repo
  (generated in the portability drill).

## Actionable Levers
1. Extend attestation schema with `budgets`, `grammar_rev` (append-only).
2. Author the budget-superset rule + refusal message in SPEC.
3. Version the §22.5 accepted forms in the register; add the drill as a
   `just` recipe (`verify-portability`).
4. Integrate with ADR-PML-057's `toolchain-plane` ledger.

## Links
- LexLean: `SPEC.md` §8.2, §17.11 (2391), §22.5, §30, §31;
  `CONFORMANCE.md` (`VR-10`, `AR-08`, `AR-13`, `AR-14`, `EX-06`); `README.md`
  (determinism block, docker verify)
- Foundry: `docs/adr/proposed/ADR-PML-057-Monorepo-Lean-Toolchain-Plane-Unbound.md`
- Loop index: `docs/adr/proposed/ADR-Plan-LexLean-Phase-Mirror-Loop.md`