# PM-LL-MVP-001 — Phase mirror of the Space Bunny Alpha retirement

Status: receipt, not a contract. Recommendations only. No pull request into
UOR-Foundation/LexLean.
Date: 2026-10-05
Retiring session: Space Bunny Alpha (retired)
Published anchor: `UOR-Foundation/LexLean` `main`
`629f76b2f76659d91c5a47ba666f531b0795a456`, committed 2026-10-04T05:01:36Z.
Local `main`, `origin/main`, and `HEAD` all resolve to that commit, so the
retiring session's work exists nowhere but this working tree.
Receipt copy of record: `artifacts/PM-LL-MVP-001_Space_Bunny_Alpha_retirement.md`.

## 1. The retirement

Space Bunny Alpha is retired rather than merged. Its output is a working tree,
not a ref: `docs/LexLean MVP Plan.md`, `lexeme/corpus/`,
`crates/conformance/src/cases/base.rs`, and `crates/conformance/src/cases/lexeme.rs`
are all untracked or uncommitted. Nothing it reported is on a published ref, so
nothing it reported is established until it is re-run and published. §4 and §5
below are the re-runs.

## 2. What governs

SPEC.md §33 governs the lexeme ledger. `docs/LexLean MVP Roadmap.md` is the stale
source. Its superseded recipes, which must not be re-implemented:

1. `.olean` hashing under the identifier `olean-v3`
2. the `ct-merkle` `MemoryBackedTree`
3. `merkletreejs` plus a WebCrypto bundle for the browser verifier
4. the `freetsa` crate CLI, with `openssl ts -verify` as the acceptance step
5. the six-binary split (`lexhash`, `lexsign`, `lexstamp`, `lexappend`, `lexverify`)
6. YubiKey through `signatory`/`yubihsm` PKCS#11
7. `LexSpec.lean` as one file, rather than §33.9's `lexeme/lean/LexSpec/` with
   its five named theorem families
8. git-push root publication, and the "~1,700 LOC, one week" manifest as the
   definition of done

(The first receipt named six of these under the heading "eight". The list is
corrected here because a receipt that miscounts its own items is not evidence
about them.)

## 3. Published state, measured 2026-10-05

| Claim | Measured |
| --- | --- |
| `main` is the published anchor | `git rev-parse main origin/main HEAD` all `629f76b2…` |
| the anchor's description | subject "Implement verification logic for LexLean entries and introduce OSCAL profile documentation"; there is no `PhaseMirror` tag (`git tag -l` → `v0.1.0`) |
| `docs/LexLean MVP Plan.md` at that ref | absent |
| `lexeme/` at that ref | absent, 0 entries |
| `conformance_lg_14` at that ref | 0 files |
| §33.7 at that ref | `SPEC.md:6625` — "`heads.json`  the published tree heads, each signed by the log key" |
| `crates/lexlean/src/lexeme/ledger.rs` at that ref | **published**, and its `TreeHead` (`:57-65`) has `tree_size`, `root_hash`, `consistency_proof` and no signature field |
| `LLG1009` at that ref | **published**, and its statement already ends "…or a tree head is not signed by the log key" (`model/errors.toml`) |
| lexeme rows in the published register | 16 (`model/ids.toml`, `suite = "lexeme"`) |
| published `conformance.rs` | 291 `conformance_` tests, **0** mentions of `LG-` |
| published `crates/conformance/src/cases/` | 25 files; no `base.rs`, no `lexeme.rs` |
| published `features/suites/lexeme.feature` | absent |

Three corrections to the first receipt follow from that table.

1. The `LLG1009` registration is **published**, not a local artifact of the
   retiring session. The local `model/errors.toml` diff adds only `LLG1011`–
   `LLG1015`, the §34 stratification codes. What is published is a split between
   the contract and the struct, not a split introduced locally.
2. The divergence at the published ref is larger than `TreeHead`. Sixteen
   registered capabilities have no scenario, no test, and no case body at
   `629f76b`, while §33 and `LLG1001`–`LLG1010` are published normative. By the
   mechanism `every_id_has_a_scenario_and_a_test`, whose own failure is recorded
   at `VERIFICATION.md:2051-2063`, that ref cannot pass the meta-gate. This is
   derived from the file absences above and that record; it was not executed
   here, and no build of `629f76b` was run.
3. The anchor's description is the commit subject above, not "PhaseMirror".

## 4. Re-runs of what the first receipt deferred

| Reported | Re-run 2026-10-05 |
| --- | --- |
| `conformance_lg_14` passes | passes: `1 passed; 0 failed; 318 filtered out` |
| the case is falsifiable | one byte of `gamma`'s `audit_path[0]` fails it at `crates/conformance/src/cases/base.rs:1605:9`; the file was restored byte-identical (md5 `aa79a55aded2a030720c24b0974d60dd`) and the case passed again |
| the three anchors are genuine FreeTSA tokens | confirmed **without** lexlean: each token was extracted from the committed entry, its query rebuilt by `lexlean lexeme stamp --request`, and `openssl ts -verify -in <tsr> -queryfile <tsq> -CAfile <root> -untrusted <leaf>` reported `Verification: OK` for all three, at `genTime` `20261005030258Z`, `20261005030525Z`, `20261005030526Z` |

The socket-denial half of the case was not re-run under the preload; it ran in
the same invocation as the assertion above, which is the case's own verdict. The
`tampered roots.txt` and `TcpStream::connect` plants were not repeated; they were
observed on 2026-10-05 and recorded at `VERIFICATION.md`.

## 5. The choice for WP-2: reading 1

Reading 1 is chosen: extend `TreeHead` with the signature §33.7 already
requires, sign the head with the log key, verify it on open, and rebuild the
corpus as a separate recorded event.

Why reading 2 is rejected:

- Both halves of the requirement are already published: the §33.7 sentence and
  the `LLG1009` statement. Only the struct and its construction lag. Narrowing
  them would amend published normative text and a published registered
  diagnostic so that an unimplemented struct becomes correct — the inversion
  this repository's discipline exists to prevent.
- §33.5's fork argument runs off published heads: two heads are consistent
  exactly when the proof recomputes. A head anyone may publish is not a head.
  Consistency proves consistency; only a signature says who published it, which
  is what makes "the root matches published log" (§33.6 step 5) a statement
  about a publisher rather than about a file.
- The requirement is not an artefact of one document: the roadmap's own tree
  state record carried `tree_head_signature`.

Sub-decision, because §33.7 requires a signature but never says where a verifier
finds the key: **each head carries its own signature record**, reusing §33.3's
`Signature` shape (`key`, `public_key`, `value`). That keeps §33.7's three-file
layout exactly as published and makes a head self-verifying. §33.7 needs one
clarifying sentence naming that location; this is the only normative edit the
choice authorises, and it clarifies an under-specified sentence rather than
weakening one.

What the choice explicitly does not authorise: a silent corpus rebuild. The
seeds were not retained, so re-acquisition mints different signatures and
different FreeTSA tokens. The rebuild follows the field, and is recorded as its
own event with its own plants.

## 6. Not started, by instruction

WP-3 (browser verifier, `LG-15`) and WP-4 (`lexeme/lean/LexSpec/`, `LG-16`) are
not started ahead of this choice. §33.9 makes the Lean specification the first
ledger entry; an unsigned head under a "signed by the log key" sentence would
put that first entry on a forked reading of §33.7.

## 7. Left alone, correctly

`def Parity.negate` inside a namespace canonicalizes to `Corpus.Parity` and
collides with `inductive Parity`, refused with `LLG1001`. A canonicalizer
question, not a corpus question; unchanged.

## 8. WP-2 implementation order, now unblocked

1. `TreeHead` gains a `Signature`, serialized in `heads.json`.
2. `Ledger::append` takes a log key and signs each head over its own fields.
3. `Ledger::check_stored_head` verifies each head's signature and raises
   `LLG1009` naming the head when it does not verify — the refusal the published
   registration already promises and nothing yet constructs.
4. `lexlean lexeme append` and `bootstrap` take the log key.
5. Case coverage: an honest head verifies; a flipped signature byte refuses with
   `LLG1009`; a head whose fields were edited refuses.
6. Corpus rebuild, recorded: new seeds, new tokens, new roots, new
   `verdicts.json`, and the `### conformance LG-14 …` record's cited lines
   re-derived.
7. WP-2 falsification: plant the missing verification and confirm the case fires.

## 9. Remaining packages

WP-2 (above), then WP-3 browser verifier and `LG-15` parity, WP-4
`lexeme/lean/LexSpec/` and the `LG-16` bootstrap entry, WP-5 route PIV signing
through `verify::child::run` with the PIN off argv, WP-6 fix the README
over-claim and add the `CHANGELOG.md` entry, WP-7 full `just vv` and report the
remaining failures.