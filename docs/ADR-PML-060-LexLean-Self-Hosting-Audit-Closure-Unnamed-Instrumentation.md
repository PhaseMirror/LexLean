# ADR-PML-060: LexLean's claim/audit instrumentation is inside the TCB but omitted from SPEC §5.1 (self-hosting audit closure)

## Status
Proposed

## Axis (Phase Mirror tension class)
risk claimed vs risk owned

## Owner (multi-agent lever)
`the-examiner`

## Dissonance Score
- Impact = severity (4) x blast radius (12) = **48**
- Tractability = **4.0**
- **Score = 192.0** (LexLean audit, rank 2 of 6)

## Context (stated intent vs implementation)

### Stated intent (documents)
- `packages/LexLean/SPEC.md` §5.1 names the trusted computing base for a
  verification attestation as exactly seven items: compiler semantics, the
  pinned Lean toolchain, Lean's elaborator and kernel, `leanchecker`,
  the imported Lean workspace + locked deps, the OS/filesystem/process
  implementation, and any external PDF engine (bytes only).
- `packages/LexLean/VERIFICATION.md` (R1–R10): every claim is falsifiable;
  §27.9 tabulates the falsifiability record per gate.
- `packages/LexLean/model/ids.toml`: 223 registered `level="build"` claim IDs
  whose oracles are (per `CONFORMANCE.md`) "constructed here and validated
  against its oracle".

### Implementation reality (LexLean corpus)
- For a `build`-level ID, the oracle is (a) a committed byte-compared golden
  and (b) a conformance test that runs the shipped binary under test. The
  same artifact that makes the claim decides whether the claim passed:
  crate, golden bytes, conformance runner, `xtask` audits, model-registry
  parser, `just` recipes, and the built binaries are compiled from the same
  repository tree.
- The instrumentation layer — the conformance runner, `xtask` audits,
  `just` gate recipes, `rustc`/`cargo` versions, and the model-registry
  parsers that read `ids.toml`/`errors.toml`/registers — appears nowhere in
  the §5.1 TCB enumeration, yet it is the *deciding* component of every
  claim's acceptance.
- `leanchecker` is correctly *not* claimed as an independent checker
  (`SPEC.md` §4.2 refuses that claim): it is same-kernel replay. The only
  independent verifier in the dependency graph, therefore, is absent by
  design, so the honesty of the audit reduces to the toolchain pins and the
  falsifiability records — each itself produced by repo code.

### Contradiction (productive)
The TCB enumerates seven components, but the *auditor* — the thing that
reads the oracles and ratifies or rejects every §31 ID — is not among them.
The manifest-positive operation of `just vv` ("the same tree that ships is
the tree that judged itself") is a textbook closed-loop oracle. The
productivity is in the mitigation path: because every gate HAS a recorded
falsification test (a planted defect that must plausibly fire), the closure is
bounded by *defect-reachability*, not by trust in a hidden oracle.

### Hidden assumptions
- **No corruption applied twice**: an edit that simultaneously alters the
  produced JSON golden, the parser's acceptance rule, and the claim's
  falsification marker (a single interleaving edit) is undetectable by the
  self-hosted gate; nothing but external review breaks the fixed point.
- **Parser fidelity is perpetual**: the model-registry parser and `#print
  axioms` normalizer are trusted to be faithful forever, without a
  differential test against any second implementation.
- **Pinned-host sufficiency**: `rustc`/`cargo`/`just` are not version-pinned
  in the attestation, so two attestations for the same module can differ in
  the instrument's own code without a digest to explain it.

### Manifested boundary
`§5.1`'s "a verification attestation depends on:" list is incomplete relative
to the actual dependency set of a passing claim: the instrumentation stack is
outside the enumeration while being the effective decider. Leaked
(unmanifested): no — manifested by this ADR.

## Decision (the lever)
Make the auditor an *itemized, pinned* TCB member and add one externalizing
cross-check:

1. **Name it**: extend SPEC §5.1 (or a new §5.5 "Claim instrumentation") to
   enumerate: `rustc` + `cargo` commit hashes, the `just` recipe set, the
   conformance-runner and `xtask` audit source ids, and the model-registry
   parsers as TCB members, each recorded by commit/content hash in every
   attestation along with `lean`/`lake`/`leanchecker` SHAs (already recorded
   per §8.2).
2. **Pin it**: record `rustc`/`cargo`/`just` versions in the attestation
   payload (a new `instrumentation` field), so audit differences are
   attributable.
3. **Break the fixed point once per release**: the §30 release gate re-runs
   `just vv` from a clean cargo-published tarball (not the working tree) so
   `cargo`'s own registry/source fingerprinting sits between the artifact and
   its self-judgment; seal the result in the release attestation.

## Consequences
- **Positive**: the audit becomes auditable — attribution of which instrument
  version ratified which claim; release self-judgment loses its privileged
  access to the working tree.
- **Negative / Constraints**: a small attestation-payload extension must not
  break existing attestation-reading consumers (`§30` byte-compat rules);
  the clean-tarball gate adds CI wall time.
- **Verification Strategy**: a planted-defect drill — edit one golden + its
  parser acceptance rule + its falsification marker in one PR and show that
  (a) the current gates cannot see it, (b) the clean-tarball gate + external
  parser-differential test can; the drill result is recorded next to the
  §27.9 falsifiability table.

## Metrics (resolution is confirmed when)
- Attestations include the `instrumentation` (rustc/cargo/just/audit source
  ids) field with content hashes.
- The planted-defect drill is in the verification docs with a recorded run.
- `SPEC.md` §5.1 enumerates the instrumentation layer as an explicit member.

## Actionable Levers
1. Patch `SPEC.md` §5.1 (+ §5.5) and bump `LEXLEAN-SPEC-1` revision note.
2. Extend the attestation schema (backwards-compatible) with instrumentation
   digests; keep §30 portability rules intact.
3. Add the clean-tarball gate to the §30 release criterion.
4. Add the parser-differential test (second reader for `ids.toml`/`errors.toml`).

## Links
- LexLean: `SPEC.md` §4.2, §5.1, §8.2, §27.9, §30; `VERIFICATION.md` (R1–R10);
  `model/ids.toml`, `model/errors.toml`
- Foundry: `docs/adr/ADR-0113.md` (UCC audit closure baseline),
  `docs/adr/proposed/ADR-PML-060-*.md` siblings
- Loop index: `docs/adr/proposed/ADR-Plan-LexLean-Phase-Mirror-Loop.md`