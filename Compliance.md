# Compliance evidence: the gap to this specification

**Status: non-normative.** This file carries no conformance claim and is not
registered in `model/ids.toml`. SPEC.md §1 holds that a behavior not authorized
by the specification is not part of LexLean, so nothing here is a promise. It
records (a) which parts of a compliance-evidence design this repository
already implements, (b) which it does not, and (c) four corrections that change
the design if the gap is ever closed.

The normative statement of what this repository does is SPEC.md; the normative
register is `CONFORMANCE.md`, generated from `model/*.toml`.

## 1. The substrate layer is already built

A compliance-evidence design needs four objects: a standard, a requirement
within it, an evidence artifact that discharges a requirement, and an
assessment deriving discharge from evidence. Those are not new here. They are
the language-1.2 model semantics:

| Design role | This repository | Registered |
|---|---|---|
| `ComplianceStandard` | a versioned lexicon package under an exact lock | `GL-01`..`GL-18` |
| `Requirement` | a contract: preconditions, postcondition, output invariant | `MD-05` |
| `Evidence` | a member of the closed claim set, discharged by a statement-exact theorem | `MD-06` |
| `Assessment` | the derivation, with every claim its evidence leaves open becoming a runtime check through a sound validator | `MD-07` |
| composition across standards | sequence, fan-out, product, branch, scan composites | `MD-08` |
| production disposition of contracts and evidence | contracts and evidence erased, realization table complete | `MD-11` |

So a regulatory framework is a glossary package, not a new inductive type, and
the authoring surface is `.lex.tex` under the closed grammar of §15, not a new
`STANDARD`/`REQUIRE`/`EVIDENCE`/`DISCHARGE` syntax. The remaining substrate
work is the packages and one verified checker per evidence type, not a compiler.

The governance and replay layers are likewise already present:

| Design role | This repository | Registered |
|---|---|---|
| tamper-evident ledger, RFC 6962 domain separation, inclusion and consistency proofs | SPEC.md §33.5, §33.9 | `LG-08`..`LG-11`, `LP-*` |
| detached authorship | §33.3 | `LG-04`, `LG-05` |
| external time | §33.4 | `LG-06`, `LG-07` |
| replay façade a third party can run alone | §33.8 single self-contained HTML; §23 `lexlean lexeme verify` | `LG-12`, `LG-14`, `LG-15` |
| the tool's own specification as the first entry | §33.9, §33.10 | `LG-16` |
| axiom-dependency audit as the trust report | §5.4, §22 | `VR-*` |
| trusted computing base | §5.1 | — |

Note the boundary in the last two rows. §5.1 defines the TCB of *a LexLean
verification attestation* — the compiler semantics, the pinned toolchain, the
kernel, `leanchecker`, the locked workspace, the operating system. It is not a
statement about any device, and §33.3's hardware key binding (`device: "piv"`,
a PIV slot, a PIN policy) is a statement about *where a signing key lives*, not
a measurement of what firmware ran.

## 2. What the compliance framing claims and this specification does not authorize

Each row is a capability with no row in the §31 register, no scenario, no
`conformance_<id>` test, and no gate. Under R1 and §1 none of them is part of
LexLean today.

| Claimed capability | Where claimed | State here |
|---|---|---|
| FRE 902(13)/(14) certification packet; a qualified-person certification template | former `Inventorship.md`, `Compliance.md` §"Court-Admissibility" | absent; no occurrence of "Federal Rules", "902", or "qualified person" in SPEC.md |
| threat model, adversary class, device TCB | both files | absent as an adversarial model; §5.1 covers only the attestation TCB |
| TPM measured boot, boot-time attestation, DRTM, remote attestation | `Compliance.md` §"Observability & Security" | absent; no occurrence of "TPM" in SPEC.md |
| five separate executables `lexledger`, `lexverify`, `lexcomply`, `lexattest`, `lexpacket`; a separate `lexlean-verify` binary | both files | conflicts with §2.3 and §23, which define one crate and one executable, `lexlean`, with subcommands; AGENTS.md records that LexLean 1.0 ships exactly one crate |
| a WASM build | both files | absent; §33.8 specifies a single self-contained HTML document with no backend and no network request |
| property-based testing of checkers | `Compliance.md` §"Verification Pipeline" | absent as a registered technique; §28 does not name it |
| starter glossary packages for FCC Part 15, FDA 21 CFR 11, GDPR Arts. 5/32, NIST SP 800-53/171, ISO 27001, CPSC, FTC, state consumer protection | both files | absent; the `GL-*` packages shipped are the language's own builtins |
| the "2499 theorems, one order-invariant receipt" pattern | former `Inventorship.md` | absent; §34.0 records that no receipt-hash interoperability with the observed compiler was established |
| a `STANDARD`/`REQUIRE`/`EVIDENCE`/`DISCHARGE` surface syntax | `Compliance.md` §"Tooling" | conflicts with §15, whose grammar is closed |

Closing any of these is not a documentation change. Under R3 each one needs a
`model/ids.toml` row, a `features/suites/` scenario whose statement equals the
register statement after trimming, a failing `conformance_<id>` test, and the
implementation — in that order. A normative capability additionally needs a
SPEC.md §31 row, and `cargo xtask validate-spec-links` requires the two tables
to be bijective.

## 3. Four corrections to the framing

These matter independently of whether the gap above is ever closed, because
each one is a claim that would not survive contact with an adversary or an
opposing expert.

**Rule 902 authenticates; it does not establish reliability.** The 2017
Advisory Committee Note states that a certification under 902(13) or (14) "is
solely limited to authentication", that any hearsay exception "must be made
independently", and — in its own worked example — that certifying a computer
output "does not preclude an objection that the information produced is
unreliable; the authentication establishes only that the output came from the
computer." A packet built by this design would get an artifact into evidence.
It would not make the compliance conclusion admissible. The packet therefore
has two halves with different legal characters, and its format must keep them
apart.

**"Self-authenticating" still requires a qualified person.** Both subsections
require a certification "of a qualified person that complies with the
certification requirements of Rule 902(11) or (12)", plus 902(11) notice, and
the certification must "contain information that would be sufficient to
establish authenticity were that information provided by a witness at trial."
The consequence is concrete rather than rhetorical: the highest-value artifact
such a design produces is a pre-drafted certification, written as stand
testimony, with only the certifier's name and qualifications left blank.

**An inclusion proof carries no time.** A §33.5 audit path establishes that a
leaf is in a tree with a given root. Establishing *when* requires a published,
signed tree head and, for the general claim, the consistency proof of `LG-11`.
And the assumption actually needed for modification detection is second-preimage
resistance, not collision resistance; collision resistance is what the separate
identity claim needs. Naming the weaker assumption makes the claim stronger.
§33.5 already fixes the structure and `LG-11` already fixes fork detection, so
this is a matter of stating the theorem in the direction that carries weight —
`verify` accepts, therefore the record was committed at the stated time — and
of recording the assumption as a hypothesis rather than an axiom, so that the
axiom audit of §5.4 stays empty while the assumption set stays explicit.

**A free RFC 3161 timestamp is not a qualified one.** §33.4 puts a TSA on the
critical path and pins a root in the entry. That is a deliberate TCB choice and
it is defensible, but it is a trust dependency on an external party, and a
token from an unqualified TSA is not a qualified electronic timestamp under
Regulation (EU) 910/2014. Two consequences: a signed RFC 6962 checkpoint could
carry the time claim instead and remove the TSA from the critical path, and an
eIDAS-facing packet needs a qualified TSA regardless. Any change here is a §33
amendment, not a configuration.

One further point, smaller and cheap: a detached Ed25519 signature should be
verified strictly, rejecting the non-canonical encodings a permissive verifier
accepts, because a malleable signature makes "one record, one signature" false
in exactly the way an opposing expert will test.

## 4. Honest limits

These hold whether or not the gap in §2 is closed.

1. **No unhackable machine in the abstract.** The strongest available form is
   conditional: under a named threat model, a named TCB, and named cryptographic
   assumptions, no adversary in a named class violates a named invariant. §5.1
   already refuses the unqualified version for LexLean itself by enumerating
   what an attestation depends on, and that enumeration includes the operating
   system.
2. **The primitives are assumptions.** SHA-256 and Ed25519 guarantees are
   cited standards, not theorems this repository proves.
3. **The hardware is an assumption.** A hardware root of trust is not a proven
   one; its certification is external evidence for that assumption.

The strength of the approach is not that it proves the impossible. It is that
the assumptions are explicit, the TCB is enumerated, and the evidence is
independently re-checkable — which is what §33.6 already commits to when it
restricts the verdict to existence at a stated time, authorship by a stated
key, and integrity since, and forbids asserting novelty, validity, or
enforceability.

## 5. Retirement of `Inventorship.md`

`Inventorship.md` was removed. Its normative content is already implemented and
registered: RFC 6962 ledger with inclusion and consistency proofs, detached
authorship, external time, browser/CLI parity, and a Lean specification of the
Merkle construction are SPEC.md §33, registered as `LG-01`..`LG-16`; its own
"publish the tool's own specification as the first entry" step is `LG-16`. §34
is the layer above it.

What it added beyond that was a table of legal uses graded "Strong", "Medium",
and "Weak", and citations to 35 U.S.C. §102. Those are conformance-shaped
claims with no row in `model/ids.toml`, no scenario, and no test, so under R1
the model is not their source and under §1 they are not part of LexLean. They
also asserted a weaker design than §33 in two places: it put RFC 3161 on the
critical path without the qualified-timestamp qualification, and it proposed a
single "order-invariant receipt" where §33.1 fixes a structural canonicalization
and §34.0 records that receipt-hash interoperability was not established.

Recover it with `git show HEAD:Inventorship.md`.
