# Phase mirror: LexLean PIRTM session, 2026-10-04

## Object identity

There are **two** candidate remotes and they disagree. Both probed directly.

| Remote | `refs/heads/main` | Holds `629f76b2`? |
| --- | --- | --- |
| `UOR-Foundation/LexLean` (this repo's `origin`) | `629f76b2f76659d91c5a47ba666f531b0795a456` | yes |
| `PhaseMirror/The-Foundry` (not configured here) | `c71874104b63a7c656b6d3c9299c71a2e4c653a7` | **no** |

- `git ls-remote origin refs/heads/main` → `629f76b2…`, exit 0.
- `git ls-remote https://github.com/PhaseMirror/The-Foundry refs/heads/main`
  → `c7187410…`, exit 0.

An earlier revision of this register said "a fetchable origin object" with no
remote named. That was ambiguous, and the ambiguity is what let it read as false
against `PhaseMirror/The-Foundry`. The claim is withdrawn as unqualified: it
holds only for `UOR-Foundation/LexLean`. It does **not** hold for the remote this
session is audited against, and which remote is authoritative is not mine to
settle.

### Binding half, independent of remote

| Field | Value |
| --- | --- |
| `git rev-parse HEAD` | `629f76b2f76659d91c5a47ba666f531b0795a456` |
| Staged | nothing — index empty |
| Six modules | all untracked (`??`) |
| **SHA containing the six modules** | **none, on either remote** |
| Reflog SHA for session tree | none |

The SHA does not carry the work. This does not depend on which remote is
authoritative: `629f76b2` contains none of the six modules. **A1 unmet.**

Six modules, all untracked:

- `crates/lexlean/src/lexeme/rational.rs`
- `crates/lexlean/src/lexeme/stratum.rs`
- `crates/lexlean/src/lexeme/contractivity.rs`
- `crates/lexlean/src/lexeme/zeno.rs`
- `crates/lexlean/src/lexeme/pirtm.rs`
- `crates/conformance/src/cases/lexeme.rs`

### Where this register lives

`artifacts/phase-mirror-lexlean-pirtm-session-2026-10-04.md`, on device `78`,
reachable at both `/home/citizen/Multiplicity/LexLean/artifacts/…` and
`/media/citizen/b361d448-…/home/citizen/Multiplicity/LexLean/artifacts/…` —
same device, so a bind mount or alias, not a copy. No separate cloud Drive is
mounted here; if "Drive" means another volume, it is not present and this file
is not on it.

## Adoption

Nothing here authorizes adoption. The six modules are in no SHA that either
remote can fetch.

## Status language

`just bdd` is **red** and is a separate gate. The pass subset is: `fmt-check`,
`model`, `spec-links`, `lint`, `features`, `deny`, `cargo test -p lexlean --lib`
(137), LP-01..LP-12 (12). `complete` is not claimed.

## Claims, each with its binding

| Claim | Bound by | Verdict |
| --- | --- | --- |
| `body_proper` amendment to §34.2 | fixture `head_inclusive_scan_forces_every_diagonal_to_one_and_body_proper_does_not` + `the_empty_body_boundary_is_reachable_only_under_the_body_proper_scan` | **proposed** — fixtures are local to an unpushed tree |
| Snapaddr SHA-256 under `lexlean-pirtm-v1` | `SnapAddr` frames, LP-03 | pass |
| Prime index is a stratum label | LP-01, LP-02 | pass |
| Receipt hash under `lexlean-contractivity-v1`, not PIRTM `seal_hash` | `the_receipt_hash_is_lexleans_and_does_not_claim_to_be_the_compilers`, refusal 3 below | pass |
| Continued-fraction comparator | build check + 90,000-point cross-multiplication grid | build check only; **the grid is not an oracle** |
| `MAX_LEAF_INDEX = 126` | `an_index_past_the_exact_range_is_refused_rather_than_rounded` names 126, 127, and the `i128` overflow that makes 127 first | pass |
| "Self-verifying frames" | `the_receipt_names_every_refusal_it_makes`, three named refusals | pass |

Substituting the prime index for either digest is refused.

### The two fixtures the amendment was proposed on

Both compute **both** scan definitions over the same real lexeme, via
`Scan::{HeadInclusive, BodyProper}`; `Scan::BodyProper` is normative,
`Scan::HeadInclusive` is retained solely so the amendment is falsifiable.

1. **Forced diagonal.** On a four-atom source, head-inclusive makes every
   `A[i][i] >= 1` because each head carries its own name; body-proper collapses
   three of the four to `0` and leaves the one genuine self-mention at `1`.
   Off-diagonal entries are identical under both scans, so the disagreement is
   exactly the forced diagonal.
2. **Reachability.** `axiom bare` lexes to a 2-token body and a **0-token body
   proper**. So `t = 0`, `λ = 1`, one reference, `‖G‖₁ = 1/1`, refused — the
   boundary §34.2 states is reachable, and reachable *only* under body-proper.
   Head-inclusive gives that atom `t = 2`, `λ = 1/3`, norm `2/3`, and
   **accepts** a lexeme §34.2 refuses. The amendment changes a real verdict.

Precision correction found while building these: the head-inclusive diagonal is
forced **nonzero**, not equal to 1 (`delta` reaches 2 — head plus body). SPEC
§34.2 now says "at least 1". §34.2 also now defines `A[i][j]` as a count of
whole-token occurrences, matching the implementation, which previously
contradicted its own "is 1 when ... and 0 otherwise" wording.

## LG-01..LG-16

Left panicking. Correct under R4: no deferral marker, no stub, no false pass.
These are §33 base-layer cases. **Owner is §33, not this layer.** Not stubbed,
not weakened, not reported as passing.

## Defect candidate

Real if §33.1 body is head-inclusive. Under the pre-amendment text the
empty-body route to `λ = 1` is unreachable (`t >= 2` always, head included), so
the stated `‖G‖₁ = 1/1` boundary has no witness. That was a fixture, not a
wording pass, and fixture 2 above is that witness.

## Working-tree incident

`git checkout crates/lexlean/src/lexeme/entry.rs` was used to inspect a
pre-existing doc-comment bug. It reverted the uncommitted `Entry.pirtm` wiring.
Caught, redone, re-verified. It is recorded here because for a window the
working tree was not the intended object, and no reflog SHA exists for it.
