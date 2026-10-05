# LexLean MVP execution plan

`docs/LexLean MVP Roadmap.md` proposes the MVP in seven steps: canonical hashing,
hardware-bound signing, RFC 3161 time, a Merkle log, inclusion proofs, a browser
verifier, and the ledger's own specification as its first entry. This document is
the execution plan for that proposal: what is already on the tree, where the
proposal and the contract disagree, and the work packages that close the
difference.

This document is **not normative**. `SPEC.md` §33.1--§33.10 is the contract, and
where the two disagree the contract governs; the disagreements are listed in §2
rather than silently reconciled. Status markers are the Phase Mirror set: ✅
on-tree, tested, and falsifiable; ⚠️ on-tree with a known open defect; ⏳
registered with no implementation; ❌ claimed in documentation with nothing on the
tree.

## 1. The order this repository requires

`AGENTS.md` "Adding a capability" is the order (R3): a register row, then a
scenario, then a failing named test, then the implementation, then `just vv`. All
sixteen rows already exist in `model/ids.toml:1766-1850`, all sixteen scenarios
exist in `features/suites/lexeme.feature`, and all sixteen tests exist in
`crates/conformance/tests/conformance.rs:1463-1526`. The missing part is
implementation, and for the last three rows the tests fail on purpose:

```text
thread 'conformance_lg_14' (1360713) panicked at crates/conformance/src/cases/base.rs:90:18:
no conformance case is wired for LG-14
thread 'conformance_lg_15' (1360714) panicked at crates/conformance/src/cases/base.rs:90:18:
no conformance case is wired for LG-15
thread 'conformance_lg_16' (1360715) panicked at crates/conformance/src/cases/base.rs:90:18:
no conformance case is wired for LG-16
test result: FAILED. 0 passed; 3 failed; 0 ignored; 0 measured; 316 filtered out
```

Three red tests are the correct state while `LG-14`, `LG-15`, and `LG-16` are
unimplemented (`R4`: no stub, no deferral marker, no silent pass). Making them
green is the whole of the remaining roadmap.

## 2. Where the roadmap and the contract disagree

| # | Roadmap step | Roadmap says | `SPEC.md` §33 requires | On tree | Disposition |
| --- | --- | --- | --- | --- | --- |
| 1 | Canonical hashing | hash the `.olean`; `canonicalization: "olean-v3"`; Lean v4.28.0 | §33.1: the §21.1 frame encoding of the Lean token stream under `lexlean-lexeme-v1`, identifier `lean-token-stream-v1` | `crates/lexlean/src/lexeme/canonical.rs`, `CANONICALIZATION` at `lexeme/mod.rs:39`; LG-01/LG-02 | Contract governs. The `.olean` proposal is stale: an `.olean` embeds a toolchain hash, so it is neither toolchain-independent nor the §33.1 canonical form. |
| 2 | Hardware signing | YubiKey PIV Ed25519 in slot 9c through `signatory`/`yubico-piv-tool`, key never leaves the device | §33.3: detached Ed25519 over `content_digest`; a hardware-bound key records device, slot, algorithm, pin policy, and signing **fails** when the device is absent with no software fallback | `lexeme/signature.rs:271` software path, `:300` `sign_with_piv` shelling out to a PIV tool; no PKCS#11 crate in `Cargo.toml` | Contract governs. The hardware intent survives as the refusal contract (LG-05); the `signatory` dependency is not adopted, because §22.3 requires a recorded executable digest and the PIV tool is an external process. Open defect: that shell-out bypasses `crate::verify::child::run` (WP-5). |
| 3 | Timestamping | the `freetsa` crate CLI; acceptance by `openssl ts -verify` | §33.4: the token is parsed and verified in process against six named checks, with a pinned root carried in the entry | `lexeme/timestamp.rs` (own DER/CMS reader), `lexeme/tsa_crypto.rs` (RSA PKCS#1 v1.5 and ECDSA); LG-06/LG-07 over committed FreeTSA tokens in `tests/fixtures/rfc3161/` | Contract governs. `openssl ts -verify` as the acceptance criterion is replaced by in-process verification. Recorded limitation: single-anchor chain only, intermediates are refused. |
| 4 | Merkle log | the `ct-merkle` crate's `MemoryBackedTree` | §33.5: RFC 6962 `0x00`/`0x01` domain separation over the RFC 6962 unbalanced shape | `lexeme/merkle.rs` (own construction, RFC 9162 transparency-dev known-answer vectors) | Contract governs; no dependency added (R6). Gap: §33.7 requires each stored head to be **signed by the log key**, and `TreeHead` (`lexeme/ledger.rs:57`) has no signature field although `LLG1009` registers that refusal (`model/errors.toml:186-189`). WP-2. |
| 5 | Root publication | push the root to a public Git repository | §33.7: a ledger directory of `log.json`, `heads.json`, `roots.txt`, written through a temporary file and renamed | `lexeme/ledger.rs:24-30`, atomic rewrite at `:400`; every head re-checked against its own prefix on open (`:256`) | Contract governs; publication is the committed directory. Head signing is WP-2. |
| 6 | Browser verifier | `merkletreejs` plus WebCrypto, ~400 lines of JavaScript | §33.8: one **self-contained** HTML document, pure JavaScript and WebCrypto, a minimal DER reader for the token, no backend, no network request, same verdict as the CLI for every corpus entry | **absent**: no `.html`, `.js`, `.mjs`, or `.ts` file exists anywhere in the repository | WP-3. `README.md:168` already claims the verifier exists; that claim is ❌ until WP-3 lands (WP-6). |
| 7 | Recursive bootstrap | six separate binaries (`lexhash`…`lexverify`) plus `LexSpec.lean` with theorems | §33.9: Lean 4 under `lexeme/lean/LexSpec/` stating the leaf/node hashes, the root, the audit path, `inclusion_holds`, and the theorems that tie them together; that specification is the first entry | `lexlean lexeme {hash,sign,stamp,append,verify,bootstrap}` is one CLI (`lexeme/cli.rs:60-73`); `lexeme/lean/LexSpec/` does not exist; `lean/specs/` holds two 19- and 46-line shells with **zero theorems**, and `lakefile.lean:10-11` pins Mathlib `v4.22.0` against a `v4.32.1` toolchain with `"fixedToolchain": false` | Contract governs the location and the theorem families; the six-binary split is superseded by the single CLI. WP-4. |
| 8 | Manifest | ~1,700 LOC, "a week of focused work" | §2.3 fixes `0.1.0` as the implementation version and `1.0.0` as the first complete release; §33.10 requires sixteen rows | 13 of 16 rows implemented and tested; 3 unimplemented | The estimate in the roadmap is not evidence. `SPEC.md` §2.3 and §30 govern what "complete" means. |

## 3. Capability status

| ID | Case | On-tree evidence | Status |
| --- | --- | --- | --- |
| `LG-01` | `base.rs:97` | canonical form invariance and digest sensitivity | ✅ |
| `LG-02` | `base.rs:163` | §21.1 framing, `lexlean-lexeme-v1` domain, two build directories | ✅ |
| `LG-03` | `base.rs:215` | every recorded field moves the digest | ✅ |
| `LG-04` | `base.rs:296` | 256 single-bit digest flips refused, foreign key refused | ✅ |
| `LG-05` | `base.rs:380` | PIV record fields; absent device ⇒ `LLG1004`, no fallback | ✅ |
| `LG-06` | `base.rs:1266` | committed FreeTSA token; four refusals | ✅ record corrected 2026-10-05 |
| `LG-07` | `base.rs:1344` | committed chain; five single-check plants | ✅ record corrected 2026-10-05 |
| `LG-08` | `base.rs:437` | `0x00`/`0x01` prefixes, leaf ≠ node | ✅ |
| `LG-09` | `base.rs:503` | 17 appends, every prefix head re-checked | ✅ |
| `LG-10` | `base.rs:582` | four proof defects refused | ✅ |
| `LG-11` | `base.rs:711` | consistency proof, fork refused | ✅ |
| `LG-12` | `base.rs:847` | five checks reported separately, verdict wording bounded | ✅ |
| `LG-13` | `base.rs:1010` | committed schema is the validator | ✅ record corrected 2026-10-05 |
| `LG-14` | `base.rs:1495` | committed corpus `lexeme/corpus/`, recorded verdicts, socket-denied re-verification | ✅ record added 2026-10-05 |
| `LG-15` | `base.rs:91` | — | ⏳ |
| `LG-16` | `base.rs:91` | — | ⏳ |

`LG-06`, `LG-07`, and `LG-13` carry falsifiability records in `VERIFICATION.md`
whose cited line numbers were stale and, in one case, whose quoted message could
not be produced by the case. All nine plants were re-run on 2026-10-05 and the
records now quote observed output; see `VERIFICATION.md:1934-2060`.

## 4. Work packages

Each package names the files it adds, the command that accepts it, and the plant
that must fire for it to count as evidence (§27.9). A package is not done until
its plant has been observed and reverted.

Status: **WP-1 landed 2026-10-05** (three plants, record in `VERIFICATION.md`);
**WP-2 landed 2026-10-05** (signed heads, rebuilt corpus, four plants); WP-3
through WP-7 are open.

### WP-1 Committed corpus and `LG-14`

Status: ✅ landed. Delivered `lexeme/corpus/` (three entries, three heads with
consistency proofs, two FreeTSA-anchored timestamps per second entry, recorded
verdicts), `crates/conformance/src/cases/base.rs::lg_14`, and the
`### conformance LG-14 offline re-verification can fail` record. The corpus
README states the three things the corpus does not establish, including the
unsigned heads WP-2 must fix.

Deliverables

- `lexeme/corpus/ledger/{log.json,heads.json,roots.txt}` — one committed §33.7
  ledger with at least three entries, so a consistency proof and a second head
  exist (`SPEC.md:6631-6636`).
- `lexeme/corpus/verdicts.json` — the recorded per-entry verdict of
  `lexlean lexeme verify`, which is the oracle the case compares against.
- `lexeme/corpus/README.md` — how the corpus was acquired, which TSA answered,
  the date, and the statement that the signing seeds were not retained, so
  re-acquisition yields a different but equally valid corpus.
- `crates/conformance/src/cases/base.rs::lg_14` — the case.

Acceptance: `cargo test -p repo-conformance --test conformance conformance_lg_14`
passes, and `cargo xtask validate-model` is clean.

Falsification: tamper one byte of a committed `roots.txt` line, or one
`audit_path` element of a committed entry, and the case must fail naming the
root that no longer reproduces.

Note on "without network access" (`SPEC.md:6640-6641`): the case cannot prove
absence of network by asserting it. It runs the re-verification with network
denied at the strongest level the host permits, and reports the level it
obtained, following the convention already used for host-dependent properties in
`VERIFICATION.md:36-38`:

1. a network namespace (`unshare --net`) when the host permits it --- **denied on
   this host** (`unshare: unshare failed: Operation not permitted`);
2. otherwise an `LD_PRELOAD`/`DYLD_INSERT_LIBRARIES` shim, compiled at run time by
   the case, that fails `socket`, `connect`, `getaddrinfo`, and `sendto`
   (`cc`/`gcc`/`clang` are present here);
3. otherwise the case reports that it declined and the row is *not* discharged on
   that host --- never a silent pass.

The plant for tier 2 is a `TcpStream::connect` in the verification path, which
must make the case fail.

### WP-2 Signed tree heads

**Gate decided 2026-10-05: reading 1** (extend `TreeHead`, sign, rebuild). The
reasoning, the measurements it rests on, and three corrections to the earlier
receipt are in `docs/PM-LL-MVP-001_Space_Bunny_Alpha_retirement.md`; the receipt
copy of record is the same bytes under `artifacts/`.

`SPEC.md:6634` requires every stored head to be signed by the log key and
`model/errors.toml:186-189` registers `LLG1009` for a head that is not; the
implementation has no signature on `TreeHead` (`lexeme/ledger.rs:57-65`), so the
registered refusal has nothing behind it. Both halves of that requirement are
**published** at `629f76b` — `SPEC.md:6625` and the `LLG1009` statement — so the
gap is published too, not a local artefact.

Sub-decision, because §33.7 requires a signature but does not say where a
verifier finds the key: each head carries its own `Signature` (§33.3's shape:
`key`, `public_key`, `value`), which keeps §33.7's three-file layout as
published and makes a head self-verifying. §33.7 gains one clarifying sentence
naming that location; that is the only normative edit the choice authorises.

Implementation order: the field and its serialization; `append` taking a log key
and signing each head over its own fields; `check_stored_head` (`:256`)
verifying and raising `LLG1009` naming the head; the CLI paths; case coverage for
an honest head, a flipped signature byte, and edited head fields; then the corpus
rebuild as its own recorded event (new seeds, new FreeTSA tokens, new roots, new
`verdicts.json`, and the `LG-14` record's cited lines re-derived).

Falsification: flip one byte of a committed head signature and the case must
refuse with `LLG1009` naming that head; plant the missing verification and the
case must fire.

### WP-3 Browser verifier and `LG-15`

Deliverables: `lexeme/verifier.html` — one file, no external requests, no
third-party JavaScript (§33.8 forbids the roadmap's `merkletreejs`), performing
§33.6 steps 1, 2, 4, and 5 with WebCrypto and pure JavaScript and step 3 with a
minimal DER reader; plus `base.rs::lg_15`, which extracts the verifier's script
from the committed HTML and runs it under a JavaScript engine against every
committed corpus entry, comparing its verdict with the CLI's and with
`verdicts.json`.

Host facts measured on 2026-10-05: `node v20.20.2` is present and its WebCrypto
implements Ed25519 (`subtle.importKey`/`generateKey` for `Ed25519` succeed). The
RFC 3161 signature check is RSA PKCS#1 v1.5 over the corpus's tokens, which needs
a BigInt modexp in pure JavaScript; no browser is required to run the case, and
no browser is assumed on any host. Where no engine is present the case reports
the host it declined on.

Falsification: change one byte of an entry's `public_key` in the corpus copy the
verifier reads, and both engines must report the same refusal --- a verifier that
reports `VERDICT` on a tampered key must fail the case.

### WP-4 `lexeme/lean/LexSpec/` and `LG-16`

Deliverables: Lean 4 modules under `lexeme/lean/LexSpec/` stating the §33.5 leaf
and node hashes over `List UInt8`, the RFC 6962 root, the audit path,
`inclusion_holds`, and the four theorem families §33.9 names --- the last-index
path recomputes the root, any index recomputes its prefix root, a wrong path or
index does not, and the path length is at most `log₂ n + 1`. The ledger's first
entry is that specification, its artifacts carry the digests of the CLI binary
and of `verifier.html`, and it is signed, timestamped, and included like any
other entry.

Two things must be settled before writing it, and neither may be deferred:
`lakefile.lean:10-11` pins Mathlib `v4.22.0` against a `v4.32.1` toolchain with
`"fixedToolchain": false`, and `lean/specs/` currently holds two shells with no
theorems. Either the specification builds under the pinned toolchain with zero
`sorry`, or the pin is repaired first; a specification that does not compile is
not a first entry.

Falsification: `LG-16` must fail when the first entry is not the specification,
when an artifact digest does not match the file, and when the entry is unsigned
or untimestamped.

### WP-5 PIV subprocess governance

`lexeme/signature.rs:319,389` spawns the PIV tool with a bare
`std::process::Command`, outside the allow-list, environment scrub, timeout, and
output cap that `crate::verify::child::run` enforces for every other child
(`lexeme/tsa_crypto.rs:201-221` uses it), and it passes the PIN as an argv
element (`:325`). §22.3 requires a recorded executable digest for a process a
gate observes. Deliverables: route the PIV path through `child::run`, take the
PIN from the environment or an inherited descriptor rather than argv, and record
the tool's digest.

### WP-6 Documentation that currently over-claims

`README.md:168` claims a self-contained browser verifier that does not exist;
`lexeme/canonical.rs:143-146` and `lexeme/stratum.rs:148-151` justify byte-string
framing by reference to "the browser verifier of §33.8", a forward reference to
an absent file. Correct both when WP-3 lands, and add the `CHANGELOG.md`
`Unreleased` entry for the §33 subsystem, which does not exist yet.

### WP-7 Acceptance

`just vv` green, `just fixtures` green, `cargo xtask release-check` listing only
the criteria §30.4 legitimately leaves open before `1.0.0`, and every new gate
carrying a `### <gate> can fail` record in `VERIFICATION.md`.

## 5. Exit criteria per roadmap step

| Roadmap step | Registered row | Exits when |
| --- | --- | --- |
| 1 canonical hashing | `LG-01`--`LG-03` | ✅ already; the roadmap's `.olean` recipe is retired in `docs/` when WP-6 runs |
| 2 hardware signing | `LG-04`, `LG-05` | ✅ already; WP-5 closes the process-governance defect |
| 3 external time | `LG-06`, `LG-07` | ✅ already |
| 4 Merkle log and publication | `LG-09`, `LG-11` | WP-2 lands: heads are signed |
| 5 inclusion proof | `LG-10` | ✅ already |
| 6 browser verifier | `LG-15` | WP-3 lands and the parity run passes on this host |
| 7 recursive bootstrap | `LG-16` | WP-4 lands: `LexSpec` builds with no `sorry` and is entry zero |
| corpus | `LG-14` | WP-1 lands |

## 6. Open items carried into the packages

Observed while planning on 2026-10-05; each is confirmed or refuted by the
package that owns it, and none is a claim until then.

1. `LLG1009` registers "a tree head is not signed by the log key"
   (`model/errors.toml:186-189`) and nothing constructs that refusal (WP-2).
2. `signature.rs:319,389` bypasses `verify::child::run`; the PIN is on argv
   (`signature.rs:325`) (WP-5).
3. `ledger.rs:177` intra-doc-links `Entry::without_inclusion`, which does not
   exist; the free function is `ledger::without_inclusion` (`ledger.rs:415`).
4. `entry.rs:316-334` documents `recompute_digest` directly above `from_json`, so
   `from_json`'s own error contract is undocumented.
5. `lean/specs/Contraction.lean:27` defines `is_expanding_on_component` as
   `∃ comp, True`; there is no theorem anywhere in `lean/` (WP-4).
6. No `Lexfile`-level integration exists: `Justfile` has no lexeme or corpus
   recipe and no `xtask` subcommand reads a ledger (WP-1 adds the corpus;
   WP-7 adds the recipe).
7. The published ref is further behind than this working tree. At `629f76b` the
   register holds 16 lexeme rows while `features/suites/lexeme.feature` is
   absent, `crates/conformance/tests/conformance.rs` has 291 tests and zero `LG-`
   mentions, and `crates/conformance/src/cases/` has 25 files with no `base.rs`
   and no `lexeme.rs`. So every §33 capability is registered and unimplemented
   on the published tree, and the whole of §33's conformance apparatus is local.
   Measured 2026-10-05; the consequence for the meta-gate is derived from
   `VERIFICATION.md:2051-2063`, not executed.

## 7. Execution log

- **2026-10-05** --- `LG-06`, `LG-07`, and `LG-13` falsification records
  re-derived: nine plants run and reverted, `VERIFICATION.md:1934-2060` corrected
  to observed output, the neighbouring `RP-07` record corrected from
  `repository.rs:424:17` to `:422:17`. `cargo xtask validate-model` clean (319
  ids, 76 codes); `conformance_lg_06`, `_lg_07`, `_lg_13`, `_rp_07` pass.
- **2026-10-05** --- this plan written; `LG-14`/`LG-15`/`LG-16` confirmed red at
  `crates/conformance/src/cases/base.rs:90`.
- **2026-10-05** --- WP-1 landed. The corpus was acquired in one pass against the
  live FreeTSA endpoint (three `POST`s of a 54-byte `TimeStampReq`, each token
  verified in process against the pinned root before it was recorded), which
  means acquisition needs a network and verification does not --- the latter is
  what `LG-14` asserts and what its third plant breaks. Three plants observed
  and reverted: a published root, an audit path, and a `TcpStream::connect` in
  the verify path. The `Gamma` source was rewritten during acquisition because
  `def Parity.negate` inside `namespace Corpus.Gamma` canonicalizes to the
  declaration name `Corpus.Parity`, which collides with `inductive Parity` and is
  refused with `LLG1001`; that naming behaviour is a canonicalizer question and
  is not settled here.
- **2026-10-05** --- `unshare --net` is denied on this host
  (`Operation not permitted`), so the `LG-14` isolation runs as a compiled
  `LD_PRELOAD` denial with a live socket probe in front of it; `node v20.20.2`
  with WebCrypto Ed25519 is present, which is what WP-3 needs.
- **2026-10-05** --- PM-LL-MVP-001 closed the retiring session and re-ran what it
  had deferred: `conformance_lg_14` green, the `gamma` audit-path plant firing
  at `base.rs:1605:9` and restored byte-identical, and all three corpus tokens
  verified by `openssl ts -verify` against the pinned FreeTSA root without
  lexlean. Three corrections to that receipt stand: the `LLG1009` registration is
  published rather than local, the published ref's real gap is that all sixteen
  §33 rows are unimplemented there, and the anchor's subject is not
  "PhaseMirror". WP-2's gate is decided: reading 1, with each head carrying its
  own signature record.
- **2026-10-05** --- WP-3 and WP-4 are not started ahead of that gate, by
  instruction: §33.9 makes the Lean specification the first ledger entry, and an
  unsigned head under a "signed by the log key" sentence would put it on a forked
  reading of §33.7. The corpus is not rebuilt either; the seeds were not
  retained, so re-acquisition mints different signatures and different FreeTSA
  tokens, and that is its own recorded event after the field exists.