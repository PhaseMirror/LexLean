# ADR-PML-058: LexLean Atlas "self-contained" claim does not bind the external native provenance plane

## Status
Proposed

## Axis (Phase Mirror tension class)
risk claimed vs risk owned

## Owner (multi-agent lever)
`the-examiner`

## Dissonance Score
- Impact = severity (4) x blast radius (10) = **40**
- Tractability = **5.0**
- **Score = 200.0** (LexLean audit, rank 1 of 6)

## Context (stated intent vs implementation)

### Stated intent (documents)
- `packages/LexLean/README.md` ("Duo / UOR Atlas" release block): the Atlas
  modules are "generated from one closed-lexicon source"; "no independently
  authored Atlas implementation exists"; the one-time conversion was "the
  result of a programmatic conversion … recorded in `MIGRATION.md`".
- `packages/LexLean/CONFORMANCE.md` (row `VR-19`): "The native Atlas source
  graph is self-contained: every generated Atlas module publicly depends only
  on Init and the generated graph, its only backend-support import is Lean,
  and no independently authored Atlas implementation exists."
- `packages/LexLean/language/uor/atlas-registers.toml` header: "The
  declaration graph in `examples/uor-atlas/src/Atlas.lex.tex` owns every live
  row. This file records only non-declaration dispositions." `S43` "carries
  the completed formalization's true integer-uniqueness statement".
- `packages/LexLean/language/uor/atlas/entries/atlas-s43.toml`
  (`kind = "document"`, `module = "Atlas"`, `component = "UorAtlas.Scales.S43"`).

### Implementation reality (LexLean corpus)
- `examples/uor-atlas/MIGRATION.md` records that the *one-time* migration
  closed at commit `f972285fab4564c1a4231fe3d6efdb776b5becc7`. At that
  checkpoint there were **two trees external to the release**:
  1. the "independently authored Lean implementation", git tree
     `71fc050796816009ea665ceb87d6687967947906` (71 files, 68 Lean files);
  2. the "native source tree", git tree
     `79c321296059eaa4142fcbf3e98965f7f5866e61` (66 source modules).
  Neither tree is present in the repository today; they survive only as
  content-addressed identities inside `MIGRATION.md`.
- The exhaustive `conformance_vr_19` comparison ran once, at that commit,
  over 1,019.88 seconds, and converted 5,577 source declaration records
  (5,519 native environment declarations + 58 private compiler records kept
  as hashed provenance). No rerun occurs in the gate `just vv`.
- The live denotations of the S-number label plane (`atlas-s43.toml` →
  `UorAtlas.Scales.S43`) point at `Atlas.lex.tex`, the *generated* closed-core
  form, not at any reachable native declaration.

### Contradiction (productive)
`VR-19`'s locality clause ("publicly depends only on Init and the generated
graph") constrains the **import** plane of the generated modules. The
**content** plane — where "the native Atlas source graph" is also claimed as
the owner of every live register row — is held by definitional runs produced
by the same exporter at one frozen commit. The phrase "the native Atlas
source graph is self-contained" therefore reads as a statement that the
*native* graph is repository-contained, when in fact the native graph is
absent and the graph that is contained is the byte-equal but divergent-copy
generated core.

### Hidden assumptions
- **Lossless-once ⇒ lossless-always**: byte-equality verified at migration
  commit `f972285...` is projected forward onto every subsequent commit, yet
  the register file itself participates in the compiler-semantics digest
  (`atlas-registers.toml` header) and every compiler-semantics change from
  1.0→1.1 plumbing silently renews the claim without a native re-check.
- **No joint-failure fixed point**: a bug shared by the exporter and the
  golden oracle (a fixed-point miscompilation) is invisible to a same-origin
  byte comparison; the only breakers would be a re-import of the external
  trees or an independently authored cross-check.
- **Hash identity = auditability**: the frozen git-tree SHAs in `MIGRATION.md`
  establish *what* was compared, but nothing in the monorepo re-derives or
  re-imports that provenance; the trees are unreachable artifacts.

### Manifested boundary
The S-number plane (S1–S85 symbols) is simultaneously live in LexLean's
register and, independently, in UOR/Foundations formal modules inside this
monorepo (`packages/Foundry`, e.g. `docs/adr/completed/ADR-0037-*
ATLAS_DERIVED_DOMINANCE_THEOREM.md` and `ADR-0078-*
UOR_ATLAS_POSITIVITY_COMPLETE_FORMALIZATION`). No cross-check ever compares
the two planes on a shared S-label. Leaked (unmanifested): no — this tension
is manifested by this ADR.

## Decision (the lever)
Bind the Atlas label plane by (a) making the external provenance re-checkable
and (b) sampling-cross-checking the two in-repo authorities on shared labels.

1. **Re-import gate (flagged)**: `just atlas-prov` re-derives the recorded
   comparison from the two recorded trees (`71fc050…`, `79c3212…`) when the
   user supplies the trees, and re-triangulates the byte-equality checkpoint
   (`conformance_vr_19`) before the next major-semantics bump (1.0 → 1.1 →
   1.2) is permitted.
2. **Shared-label cross-check**: extend the Foundry `Foundation`/UOR audit to
   compare a sampled, deterministic subset (start: `S4`, `S37`, `S38`, `S43`,
   and every label cited by an in-repo ADR) between the LexLean-generated
   `Atlas.lex.tex` core and the Foundry formal module with the same S-label.
   A mismatch or an unmatched label is recorded as `ATLAS-LABEL-CONFLICT` in
   the issues register (per ADR-0119's foreign-ID binding rule) rather than
   silently carried by either side.
3. **Clarify wording**: amend README/`atlas-registers.toml` to say the native
   graph is *recorded by tree identity and reproduced as byte-equal generated
   core*, not "self-contained in the repository".

## Consequences
- **Positive**: provenance for 200+ atlas entries becomes testable instead of
  a frozen hash; the S-number plane gains a second, independent witness inside
  the monorepo; future compiler-semantics bumps cannot silently re-seal an
  unverified claim.
- **Negative / Constraints**: the full 5,577-record re-derivation is
  expensive (≈1,020 s single run) — the levers therefore default to a *sampled*
  cross-check with a documented coverage budget; providing the external trees
  is a one-time secretarial task (no network fetch).
- **Verification Strategy**: `just atlas-prov` (identity re-derivation) and
  the Foundry sampler both exit non-zero on mismatch; `ATLAS-LABEL-CONFLICT`
  entries must resolve to zero before the next `just vv` read of the
  register.

## Metrics (resolution is confirmed when)
- The `S43` uniqueness statement provably held by the sampled cross-check
  (exact shared-label correspondence `S43 ↔ UorAtlas.Scales.S43 ↔
  Foundry:...·uniqueness`).
- Zero unmatched live labels after the sampler run (with the coverage budget
  recorded).
- `just vv` output shows the atlas provenance step (`atlas-prov`) as a listed
  gate, so its absence is itself a failure.

## Actionable Levers
1. Add `just atlas-prov` recipe (re-derive identity + sampled byte equality),
   documented in `VerificationVerification.md` and gated into `just vv`.
2. Add the Foundry-side sampler comparing shared S-labels against the
   LexLean-generated core; emit `ATLAS-LABEL-CONFLICT` records.
3. Rewrite the README Atlas block and the `atlas-registers.toml` header to
   the precise provenance phrasing of Decision (3).
4. Calibrate the sampling budget from re-derivation cost (target ≤5% of the
   full 1,019.88 s).

## Links
- LexLean: `examples/uor-atlas/MIGRATION.md`, `README.md`, `CONFORMANCE.md`
  (`VR-19`), `language/uor/atlas-registers.toml`,
  `language/uor/atlas/entries/atlas-s43.toml`
- Foundry: `docs/adr/completed/ADR-0037-ATLAS_DERIVED_DOMINANCE_THEOREM.md`,
  `docs/adr/completed/ADR-0078-UOR_ATLAS_POSITIVITY_COMPLETE_FORMALIZATION (1).md`,
  `docs/adr/ADR-0119.md` (foreign-ID binding rule)
- Loop index: `docs/adr/proposed/ADR-Plan-LexLean-Phase-Mirror-Loop.md`