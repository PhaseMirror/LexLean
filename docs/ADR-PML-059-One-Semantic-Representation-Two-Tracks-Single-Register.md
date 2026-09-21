# ADR-PML-059: LexLean "one semantic representation" claim spans two language tracks under a single unqualified register

## Status
Proposed

## Axis (Phase Mirror tension class)
intent vs operating incentives

## Owner (multi-agent lever)
`the-publisher`

## Dissonance Score
- Impact = severity (3) x blast radius (8) = **24**
- Tractability = **7.0**
- **Score = 168.0** (LexLean audit, rank 4 of 6)

## Context (stated intent vs implementation)

### Stated intent (documents)
- `packages/LexLean/README.md:3`: "A closed-lexicon LaTeX-to-Lean 4 compiler
  whose canonical document and prose-free Lean program are generated from one
  semantic representation."
- `packages/LexLean/README.md:9`: "one typed intermediate representation
  generates both the canonical LaTeX document and the prose-free Lean 4
  program."
- `packages/LexLean/CONFORMANCE.md` presents a single §31-style register of
  223 claims, headers shared across languages, plus the row table with one
  `suite` column; README:129: "All 223 registered conformance IDs are
  implemented and pass".

### Implementation reality (LexLean corpus)
- Two language versions are normative: `lexlean-language/1.0` and
  `lexlean-language/1.1`. They load **different canonical data** through the
  same loader:
  - `crates/lexlean/src/lexicon/mod.rs:171-178` — `load_bootstrap_for`
    selects `language/bootstrap.toml` for `"1.0"` vs
    `language/bootstrap-1.1.toml` for `"1.1"`.
  - `language/semantics.toml:7-9` vs `language/semantics-1.1.toml:7-11`:
    `lean_backend = "3"` (1.0) vs `"9"` (1.1); `latex_backend = "2"` vs `"3"`;
    `semantic_ir` defined only in 1.1 (`"2"`); `proof_lowering = "1"` vs `"2"`.
  - `language/std/nat-1.1` adds `beq`/`ble`/`blt` entries importing `bool`;
  - SPEC §17.11 adds a *second canonical content channel* per module —
    `\semanticdata{...}` source data for `semanticmodule`s — with its own
    fixed-schema sequences, distinct from language-1.0's pure closed-lexicon
    source.
- The §31 register has **no language facet**: no `id` row records whether it
  governs 1.0, 1.1, or both. Yet whole suites are language-specific:
  `SM-16…SM-22` ("Language 1.1 …", "language-1.1 semantic snapshot"),
  `DF-11` ("Language 1.1 checks and lowers …"), and `CL-20` (a 1.1 create/verify
  CLI behavior) are verified as 1.1-only statements, while `RP/CF/PF/...`
  suites apply structurally to both.

### Contradiction (productive)
"One semantic representation" is literally true only inside a track: the IR
of a 1.0 module and the IR of a 1.1 `semanticmodule` differ in schema, in
backend majors, and in proof lowering. Publishing "223 IDs pass" without a
language qualifier makes every claim a four-way conjunction — true in 1.0,
true in 1.1, and by construction of neither. The productivity is that this is
fixable as *data*, not as a redesign: the compiler already routes per-track
by version; only the public accounting does not.

### Hidden assumptions
- **Track-uniformity imputed**: readers assume a claim that holds for 1.1
  (the richer language) also holds for 1.0, or that a 1.0-verified claim is
  re-verified in 1.1. Neither is stated; conformance tests are per-track but
  the register rows do not say which track they attest.
- **Vocabulary identity across tracks**: `std/nat-1.1` extending `bool`
  usage means "nat" in 1.0 and "nat" in 1.1 are different semantic modules;
  the unqualified register does not record this split, so an operator reading
  a `LN-`/`TX-` claim cannot know which nat it governed.
- **Budget claims are per-track too**: the 1.1-only fixed budgets
  (maxRecDepth/maxHeartbeats, SPEC §17.11) are not annotated as 1.1-only in
  any public claim row.

### Manifested boundary
A single normative register claims to be a bijection over claim IDs while its
rows simultaneously serve two languages whose semantics differ at the module
level. Leaked (unmanifested): no — manifested by this ADR.

## Decision (the lever)
Add a **language facet** to the claim register and make the validator enforce
it:

1. **Schema**: `model/ids.toml` gains a per-row `language = "1.0" | "1.1" |
   "both"` facet (schema bumped to `lexlean/ids/2` with a migration note);
   `validate-spec-links`/`model` gates enforce that `both` rows hold under
   both track serializations and that every row's suite/backends exist in its
   declared language(s).
2. **Publication**: `CONFORMANCE.md`'s register table gains a `Language`
   column, and README:129 is reworded to the three-way accounting
   (e.g. "223 claim IDs: 197 1.0 + 202 1.1 + 176 both; all pass"), killing
   the unqualified "223 pass".
3. **Track claim**: amend the "one semantic representation" phrasing in
   README:3/:9 to "one semantic representation *per language track*, sharing
   one closed-core backend" — with SPEC §17.11 explicitly declared the second
   canonical source channel.

## Consequences
- **Positive**: register readers learn which language a claim certifies; the
  1.0/1.1 divergence stops being a hidden assumption; `SM-16…22`/`DF-11`/
  `CL-20` become visibly 1.1-specific statements.
- **Negative / Constraints**: the register schema change must stay
  back-compatible with existing attestations (byte-compat rules, SPEC §30);
  README claim-count rewording risks a one-time operator-facing diff.
- **Verification Strategy**: the `model` gate rejects any row lacking a valid
  facet; per-track conformance rerun validates the three-way counts.

## Metrics (resolution is confirmed when)
- Zero `id` rows without a `language` facet; validator rejects facet-less
  rows.
- Conformance output prints the per-language counts and README:129 uses the
  three-way phrase.
- `CONFORMANCE.md` table includes the `Language` column; `SM-16…22`, `DF-11`,
  `CL-20` show `1.1`.

## Actionable Levers
1. Extend `model/ids.toml` rows + schema version; add migration note.
2. Enforce the facet in `model`/`validate-spec-links` gates (see `just vv`).
3. Regenerate `CONFORMANCE.md`; reword README:3/9/129.
4. Cross-check §31 bijection per language in the state manifest
   (`lexlean` accounting block).

## Links
- LexLean: `README.md:3,9,129`, `CONFORMANCE.md` (`SM-16…22`, `DF-11`,
  `CL-20`), `model/ids.toml`, `crates/lexlean/src/lexicon/mod.rs:171-178`,
  `language/bootstrap*.toml`, `language/semantics*.toml`, SPEC §17.11, §31
- Foundry: `docs/adr/ADR-0119.md` (register discipline),
  `docs/adr/proposed/ADR-PML-062-Release-Plane-Artifact-Versus-Release-Criterion.md`
  (consumes the claim accounting)
- Loop index: `docs/adr/proposed/ADR-Plan-LexLean-Phase-Mirror-Loop.md`