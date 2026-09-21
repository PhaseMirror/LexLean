# ADR-PML-062: LexLean release plane publishes "223 pass" via 0.3.0 artifacts while SPEC §30 refuses any release before 1.0.0

## Status
Proposed

## Axis (Phase Mirror tension class)
risk claimed vs risk owned

## Owner (multi-agent lever)
`the-publisher`

## Dissonance Score
- Impact = severity (3) x blast radius (7) = **21**
- Tractability = **7.0**
- **Score = 147.0** (LexLean audit, rank 6 of 6)

## Context (stated intent vs implementation)

### Stated intent (documents)
- `packages/LexLean/README.md:26-27`:
  ```
  docker pull ghcr.io/afflom/lexlean:0.3.0
  docker run --rm -v "$PWD:/work" ghcr.io/afflom/lexlean:0.3.0 verify
  ```
- `packages/LexLean/README.md:129`: "All 223 registered conformance IDs are
  implemented and pass; `just vv` runs clean from a checkout with the pinned
  toolchain installed."
- `packages/LexLean/CONFORMANCE.md` (`RP-12`): "A release is refused unless
  the complete release criterion and all required artifacts are satisfied."
- SPEC §30: the release criterion is the full gate; README:126 `just release
  # vv, then the §30 release criterion; refused until 1.0.0`.

### Implementation reality (LexLean corpus)
- The published image tag `ghcr.io/afflom/lexlean:0.3.0` — and its sibling
  0.3.x binaries — ARE distributed and ARE advertised as the shortest path
  to "a run that can actually verify" (README docker verification block),
  i.e. the operator-facing route to *verified* modules.
- Crate metadata states version `0.3.0` while SPEC §2.3 fixes `0.1.0` as the
  initial implementation version and `1.0.0` as the candidate-release
  version; `just release` refuses to run the §30 release criterion until
  1.0.0.
- No claims in the documentation or image metadata state *which* of the 223
  claim IDs the 0.3.0 tag encodes or whether the image was built from a tree
  whose `just vv` result is sealed.

### Contradiction (productive)
The *release claim* is refused (`RP-12`, "release refused until 1.0.0") while
a *published, pullable, advertised artifact* named `0.3.0` exists and is the
recommended verification path. The word "release" is doing contradictory
duty: the §30 machinery refuses to bless it, yet public operators are sent to
it as the blessed entry point. The productive question is not "is 0.3.0 a
release?" but "what does a pullable, advertised artifact certify?" — and the
repo has one crisp answer available: the pinned-tree attestation recorded at
build time.

### Hidden assumptions
- **Tag = claim completeness**: pulling `:0.3.0` and trusting README:129's
  "all 223 pass" silently assumes the image bytes equal a clean-checkout
  `just vv` pass, with a tolerably short code-to-image gap.
- **Immutable-tag discipline**: a mutable `:0.3.0` tag (or a rebuilt image
  under the same tag) would change *which* claims the artifact certifies with
  no digest change visible to the operator.
- **Attestation travels**: nothing in the image upload guarantees the
  attestation record (toolchain+digests) ships alongside the image the way it
  ships in the tree; an operator verifying from the image has no attestation
  for the image.

### Manifested boundary
Artifact-availability ("pullable, advertised") and claim-legitimacy ("§30
release criterion") are two different planes with one name "0.3.0". Leaked
(unmanifested): no — manifested by this ADR.

## Decision (the lever)
Make the published artifact self-describing and decouple "advertised
artifact" from "release":

1. **Artifact ledger**: for every pushed tag, commit a
   `release/{tag}.attestation.json` next to the README containing: source
   commit, `just vv` result link, exact claim-ID set active at that commit
   (the three-way `language` accounting from ADR-PML-059), and toolchain +
   budget fields (from ADR-PML-061). Publishing without the ledger entry is a
   gate failure for the container workflow.
2. **Wording**: README docker block says *exactly* "pre-1.0.0 build artifact;
   certifies the claims listed in `release/0.3.0.attestation.json`, not the
   §30 release criterion" and points README:129's "223 pass" at the tree
   state, not the image.
3. **Tag discipline**: switch the advertised tag to an immutable, commit-level
   tag (`:0.3.0-<commit>` or digest reference) and keep `:0.3.0` as a mutable
   convenience alias documented as such.

## Consequences
- **Positive**: public elaboration of what an artifact certifies; no
  operator can mistake a tree-state claim for an image claim; §30's refusal
  stays logically consistent with the presence of published binaries.
- **Negative / Constraints**: container workflow change (tag schemas ripple to
  docs/CI); one-time README diff affects copied operator instructions.
- **Verification Strategy**: a `just artifact-ledger` recipe compares the
  ledger's claim set + toolchain fields to the pushed tag's manifest; the
  §31 language-accounting block (ADR-PML-059) supplies the ID set.

## Metrics (resolution is confirmed when)
- `release/{tag}.attestation.json` exists for every advertised tag; CI
  refuses a push without it.
- README docker block + README:129 use the scoped phrasing of Decision (2).
- An operator can reproduce the ledger entry from the published tree at the
  recorded commit (no hidden state).

## Actionable Levers
1. Add the container-workflow ledger gate + `just artifact-ledger`.
2. Reword README docker block and README:129 claim scoping.
3. Migrate advertised tag to immutable commit-level tags; document the
   mutable alias.
4. Cross-link to ADR-PML-059 (claim accounting) and ADR-PML-061 (budget
   fields in the ledger schema).

## Links
- LexLean: `README.md:26-27,126,129`, `CONFORMANCE.md` (`RP-12`), SPEC §2.3,
  §30; container workflow + image metadata
- Foundry: `docs/adr/proposed/ADR-PML-059-One-Semantic-Representation-Two-Tracks-Single-Register.md`,
  `docs/adr/proposed/ADR-PML-061-Verification-Evidence-Forward-Portability.md`
- Loop index: `docs/adr/proposed/ADR-Plan-LexLean-Phase-Mirror-Loop.md`