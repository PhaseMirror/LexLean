**The inventor's invention is a registry.** The registry's entries are formal specifications. A specification's identity is its *semantic content* — the type — not its prose description. Authorship is a signature over the content hash. Priority is a timestamp in an append-only log. Novelty is a semantic search against prior entries. And the whole thing is a package manager, because inventions are composite: a device depends on a radio module, which depends on a modulation scheme, which depends on a mathematical primitive. The patent system treats each as a discrete legal instrument; a package manager treats each as a *lexeme* in a shared vocabulary.

"LexLean" is the right name: a **lexicon** of inventions, formalized in **Lean**.

---

## 🔁 The Recursive Bootstrap

The tool's first entry should be **the tool itself**. Publish the spec of the registry to the registry. The type of the registry is the first lexeme; the implementation of the registry is the first term; the theorems about the registry (append-only, tamper-evident, sound) are the first properties; the signature is the inventor's key; the timestamp is the first moment of the ledger.

This is not a gimmick. It is the demonstration. If the tool can protect its own specification — the hardest case, because the spec is the thing doing the protecting — it can protect anything downstream. And it makes the tool's own provenance the first piece of evidence a court would see: the registry was authored by the inventor at time T, and every subsequent entry chains back to that first entry.

---

## ⚙️ The Mechanism: From Semantics to Authorship

The pipeline is a five-step transformation:

**Step 1 — Semantics to type.** The inventor writes the invention as a Lean type. This is the *claim*: what the invention does, abstracted from how. `Vec n A` and `Fin n → A` are different types expressing the same semantic content; a canonicalization pass (or an equivalence proof) is what makes them the same lexeme.

**Step 2 — Type to kernel check.** The type is elaborated and checked by the Lean kernel. If the type is malformed, the pipeline stops. This is the "math checked" part: the specification cannot be a vague wish; it must be a well-formed mathematical object.

**Step 3 — Content to hash.** The elaborated type is serialized canonically and hashed (SHA-256). The hash *is* the lexeme's identity. Two inventions with the same hash are the same invention.

**Step 4 — Hash to signature.** The inventor signs the hash with their private key. This is the "cryptographic authorship" part: the signature binds the lexeme to a key, and the key is bound to an identity (via a hardware key, a web-of-trust certificate, or a CA-issued identity certificate).

**Step 5 — Signature to ledger.** The signed hash, the timestamp (RFC 3161), the dependencies, and the properties are appended to a Merkle tree. The root is published. Inclusion is proved. The chain is tamper-evident.

The output is a **lexeme entry**:

```json
{
  "spec_hash": "sha256:...",
  "impl_hash": "sha256:...",
  "properties": ["sha256:...", "sha256:..."],
  "author_pubkey": "ed25519:...",
  "signature": "ed25519:...",
  "timestamp": "rfc3161:...",
  "dependencies": ["sha256:...", "sha256:..."],
  "abstraction": "lex:...",   // CPC-like classification
  "license": "lex:...",       // terms of use
  "inclusion_proof": [...],
  "root_hash": "sha256:..."
}
```

This is the artifact. Everything else is tooling around it.

---

## ⚖️ The Crux: Semantic Novelty

This is where the honest engineering lives, and where the tool either succeeds or becomes a very expensive git repository.

A patent examiner judges *novelty* (no prior art) and *non-obviousness* (not an obvious extension). The tool can check novelty against the registry. It cannot decide non-obviousness. The novelty check has four possible strengths:

| Strength | Identity criterion | Tractable? | Evidentiary weight |
|---|---|---|---|
| **Exact** | Same spec hash | Trivially | Strong (identical) |
| **Canonical** | Same canonical form | Requires a canonicalizer | Strong (equivalent) |
| **Property-subsuming** | New spec's properties imply an existing spec's properties | Requires proof search | Medium (functionally equivalent) |
| **Abstraction-matching** | Same classification code + overlapping properties | Requires a taxonomy | Weak (similar) |

The tool should implement **all four**, and report which level of novelty it established. A court does not need the tool to say "this is novel." It needs the tool to say "at the exact level, no match; at the canonical level, no match; at the property level, the closest prior art is X, and the distinguishing property is Y." That is a *machine-checkable prior-art report*, and it is far more rigorous than a patent examiner's prose.

The theoretical limit is real: full semantic equivalence of arbitrary types is undecidable. The tool must be *sound* (if it says "equivalent," they are equivalent) and *incomplete* (if it says "not equivalent," they might still be). This is the same trade-off Lean itself makes, and it is the right one.

---

## 📦 The Packaging Architecture

This is the part the inventor already understands, because it is Cargo:

| Cargo | LexLean | Purpose |
|---|---|---|
| `Cargo.toml` | `Lex.toml` | Declares the spec, its properties, dependencies, license |
| `Cargo.lock` | `Lex.lock` | Freezes exact spec hashes for a build |
| `crates.io` | `lex.io` | Public registry (or private, for trade-secret priority) |
| `cargo publish` | `lex publish` | Submits a signed entry to the ledger |
| `docs.rs` | `lex.rs` | Rendered documentation for each lexeme |
| `rustup` | `lexup` | Toolchain management |
| `cargo tree` | `lex tree` | Dependency graph visualization |
| `cargo audit` | `lex audit` | Checks for revoked keys, withdrawn entries, superseded specs |

The key structural insight: **inventions are composable, and the registry makes composition explicit**. A patent for a radio is separate from a patent for a modulation scheme; a lexeme for a radio depends on a lexeme for a modulation scheme, and the dependency graph is the lockfile. This is not a feature the patent system has. It is the reason the packaging metaphor is not cosmetic.

---

## 🏛️ The Legal Theory (Honest Version)

"Replace the patent system" is not something a tool can do — patents are statutory, and replacing them requires legislation. But the tool can do something *more useful* for a basement inventor:

| Legal use | Strength | Mechanism |
|---|---|---|
| **Defensive prior art** | Strong | Published entry makes the invention unpatentable by anyone else (35 U.S.C. § 102) |
| **Priority evidence** | Strong | RFC 3161 timestamp + append-only ledger establish date of conception |
| **Authorship evidence** | Strong | Signature + identity binding establish who |
| **Trade secret support** | Medium | Hash of a private spec proves you had it first without disclosing it |
| **Copyright registration** | Medium | The spec is a creative work; the hash is a registration equivalent |
| **Licensing contract** | Medium | The entry's terms are clickwrap; enforceability evolving |
| **Patent replacement** | Weak without legislation | Requires statutory recognition of the registry as a priority instrument |

The honest pitch to the inventor is: **this is a machine-checkable prior-art and authorship registry, which is more than defensive publication and cheaper than a patent, and which can complement a patent by doing the disclosure and priority work so the patent only needs to do the enforcement work.**

---

## 📜 The Court Packet

When the inventor needs to prove something in court, the packet is:

1. **The record** — the spec, the implementation, the properties.
2. **The hashes** — SHA-256 of each.
3. **The signature** — Ed25519 over the hashes.
4. **The timestamp** — RFC 3161 from an independent TSA.
5. **The inclusion proof** — Merkle path to the root.
6. **The root** — published to an append-only log with third-party witnesses.
7. **The novelty report** — machine-checkable prior-art search at all four levels.
8. **The re-verification instructions** — one command any third party can run.
9. **The plain-language explanation** — what this proves and what it assumes.

For the device-security claim, add: the threat model, the TCB, the invariants, the Lean proofs, the attestation chain, and the assumptions (crypto primitives, TPM correctness).

---

## 🧭 The Six Layers, Condensed for This Domain

| Layer | Reframed for the semantic registry |
|---|---|
| **Substrate** | Lexeme = Lean type; authorship = signature; ledger = Merkle tree. Zero axioms in the registry core. |
| **Verification** | The registry's own operations (append, verify, resolve) are Lean-proven. Property tests on the canonicalizer; differential tests against a reference implementation. |
| **Tooling** | `lex publish`, `lex verify`, `lex tree`, `lex audit`. A DSL for writing specs that non-Lean-experts can use. |
| **Governance** | Admission rules in Lean: which equivalence levels are trusted, which keys are authorized, which dependencies are permitted. Provenance hashing at every step. |
| **Integration** | WASM verifier in the browser; JSON-RPC for CI; replay façade so any third party can re-check without the inventor's toolchain. |
| **Observability** | Telemetry on proof search and equivalence-checking; runtime monitors on the verifier; security audit of the registry's own TCB. |

---

## ⚠️ Three Things It Cannot Do

1. **It cannot decide non-obviousness.** That is a judgment about the state of the art and the reasoning of a skilled person. The tool can report "the closest prior art is X; the distinguishing feature is Y," but the decision requires a human or a legal process.

2. **It cannot prove the crypto primitives are secure.** SHA-256 collision-resistance and Ed25519 unforgeability are assumptions, named explicitly and cited (FIPS 180-4, RFC 8032). The tool's proofs are conditional on them.

3. **It cannot prove the hardware is correct.** A TPM is a hardware root of trust, but unless the TPM itself is formally verified, the proof is conditional on its correctness. The honest claim is "conditional on the TCB behaving correctly."

The strength of the tool is not that it proves the impossible. It is that it makes the assumptions explicit, the TCB small, and the evidence independently verifiable. That is what a court needs, and it is what no patent system currently provides.

---

## The One Thing That Would Make It Real

Build the smallest possible version: a CLI that takes a Lean file, computes its canonical hash, signs it with a hardware key, submits it to FreeTSA for an RFC 3161 timestamp, appends it to a local Merkle tree, and prints the inclusion proof. Then build a browser verifier that takes the proof and re-checks it. Publish the tool's own spec as the first entry. That is a week of work, and it is the entire thesis in executable form. Everything else — the equivalence checker, the agentic prover, the WASM build, the certificate registry — is elaboration.