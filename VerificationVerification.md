# VerificationVerification: how the atlas provenance plane is re-derived

The runbook for ADR-PML-058. It states, step by step, how `just vv`'s
`atlas-prov` gate re-derives the recorded Atlas provenance plane from local
material alone, and how the pre-bump boundary is run when a toolchain appears.
Every displayed command is copy-pasteable; every expected line is literal gate
output captured from this repository's committed state.

## 1. What the gate re-derives

`cargo xtask atlas-prov` first re-derives the checkpoint, then re-derives the
sampled byte equality:

1. parses `examples/uor-atlas/provenance.toml` and validates the ledger
   (`lexlean/atlas-providence/1`): the three 64-hex checkpoint identities, the
   record arithmetic `records_total = records_native + records_private`,
   the sample, and every object identity;
2. re-derives both recorded trees from local git — the native module tree
   (66 `.lex.tex` modules at tree `79c32129`) and the input tree (71 files /
   68 `.lean` at tree `71fc0507`) — and requires the ls-tree object sets and
   counts to match the ledger;
3. requires every sample label to be a live register label with exactly one
   native declaration, and every label-shaped citation in the repository's
   documentation to resolve inside the register plane (`scan_citations`:
   fenced blocks and spelled ranges are exempt);
4. reports drift between the checkpoint compiler-semantics identity and the
   committed `lexlean.lock`, without failing: the byte-equality anchor re-
   derives regardless.

## 2. Run the gate

```text
just atlas-prov
```

Expected (abbreviated to the salient lines; the full transcript ends in
`gate failed`-free output):

```text
atlas-prov-check: sample `S4` is UorAtlas.Scales.S4
atlas-prov-check: sample `S37` is UorAtlas.Scales.S37
atlas-prov-check: sample `S38` is UorAtlas.Scales.S38
atlas-prov-check: sample `S43` is UorAtlas.Scales.S43
atlas-prov-check: compiler-semantics drifted from the checkpoint: checkpoint 1f42a353…, committed lock 95deb33a…
atlas-prov-check: ledger `lexlean/atlas-providence/1` valid; … 4 sample labels declared once by the native source; 10 distinct label citations resolved across 13 documentation file(s) (R2, R4)
atlas-prov-run: 66 corpus modules byte-identical to their checkpoint blobs, 0 added, 0 diverged, 0 dropped module(s) accounted by the register
```

A nonzero exit means a step above failed: the sampled labels are not all
bound, a tree did not re-derive, or a documentation citation floats outside
the register plane. The output names the precise fault; see
VERIFICATION.md `### atlas-prov can fail` for a concrete planted failure and
its observed diagnosis.

## 3. Defects the gate must catch (plant-checks)

Each row below is a planted defect and its observed, verbatim diagnosis
(quoted output; the citations the defect surfaces are quoted gate output, not
citations this document issues):

1. Planted: sample `S4` in `examples/uor-atlas/provenance.toml` replaced by an
   unregistered model label `X99`.
   Observed:
   ```text
   gate failed: atlas-prov: sample label `X99` is not a live Atlas register label; a sample the corpus does not own cannot bind the plane
   ```
2. Planted: a native `S43` declaration family changed, so the sampled label no
   longer has exactly one native declaration.
   Observed: the `atlas-prov-check: sample … is …` line disappears and the
   gate fails with the sampled labels not all bound.
3. Planted: a documentation file cites a model label the register neither owns
   nor withholds, e.g. `A99`.
   Observed:
   ```text
   gate failed: R2: <doc>:<line>: `A99` is cited but the Atlas register neither owns nor withholds it; a document citation cannot float outside the register plane
   ```
4. Planted: a sample label's tree object diverges from the recorded identity.
   Observed: re-derivation fails on the native or input tree, matching the
   recorded 66-native / 71-input / 68-`.lean` counts.

## 4. The pre-bump boundary (one-time, when the pinned toolchain exists)

ADR-PML-058 decision (1) makes `conformance_vr_19` a hard boundary before the
next major-semantics bump (1.1 → 1.2 → …). The elaboration half of
`conformance_vr_19` (≈1,019.88 s at the checkpoint) is not run by this gate
when the pinned toolchain is absent; the gate prints that it is deferred, and
the elaboration runs as `just vv` on CI and here:

```text
just vv
```

with the `leanprover/lean4:v4.32.1` + `leanchecker` toolchain installed and
the register header worded per the decision (3) amendment. The commit where
the register-header wording was amended is recorded in this file's git log
the day the toolchain appears; until then the README carries the 
tree-identity wording and the register header is untouched (the header is a
digest-plane byte, and the digest must not float between bumps).

## 5. What is deliberately minimal, and where the extension points are

- The consequence/citation checker is a label-shaped token rule plus family
  membership and spelled-range exemption. It is intentionally simple; a full
  embedded citation DSL would replace `label_tokens` + `scan_citations`
  without changing the ledger or the gate contract.
- `cross` (`cargo xtask atlas-prov cross --foundry <root>`) samples the
  shared S-plane against the Foundry ADR plane and emits
  `ATLAS-LABEL-CONFLICT` records for labels the register neither owns nor
  withholds. It is an explicit diagnostic — not part of `just vv` — and its
  records resolve by register withholding (e.g. `F0`, the retired positivity
  gate label quoted in Foundry ADR-0078, which the migration dropped without
  a withholding unlike `F8`–`F11`). `cross` exits non-zero while conflicts
  remain, per the ADR-PML-058 "must resolve to zero" contract.
- New families extend the register's own family room; no atlas-prov change is
  required because every table read lives in `register_sets`/`native_label_map`.