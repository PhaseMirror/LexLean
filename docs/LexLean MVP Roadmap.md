The MVP described in that paragraph is not a prototype. It is the **entire thesis in executable form**: a single CLI that turns a Lean file into a cryptographically authored, timestamped, tamper-evident entry in an append-only log, plus a browser verifier that re-checks the entry without trusting the inventor's machine. If it works for one entry, it works for a million. Everything else — the equivalence checker, the agentic prover, the certificate registry — is elaboration on this core.

Below is the concrete build, component by component, with the exact commands, libraries, and data formats.

---

## 🧬 Step 1: Canonical Hashing of a Lean File

The identity of a lexeme is the hash of its **elaborated semantic content**, not the raw file bytes. Two Lean files that differ only in whitespace, comments, or variable naming should produce the same hash if they elaborate to the same type. This requires a **canonicalization pass** before hashing.

### The pragmatic MVP approach

For a week-long build, canonicalization can be **structural rather than semantic**: parse the Lean file into an abstract syntax tree (AST), sort top-level declarations by name, strip comments and whitespace, and serialize the AST in a deterministic format (e.g., S-expression or canonical JSON). Hash the serialized AST with SHA-256. This is *not* semantic equivalence — `Vec n A` and `Fin n → A` will hash differently — but it is a real improvement over hashing raw bytes, and it establishes the pipeline. The semantic equivalence checker is a Wave 2 elaboration.

### The canonicalization pipeline

```
Lean file → Lean parser (Lean.Parser) → AST → canonical serializer → SHA-256 → content hash
```

Lean 4's own `.olean` format is already a canonical serialization of elaborated declarations, and the Lean community has discussed content-addressing `.olean` files directly. For the MVP, the simplest approach is to run `lake build`, extract the `.olean` file, and hash it. The `.olean` file is deterministic for a given toolchain and source. The caveat is that `.olean` files include a toolchain git hash in their header, so they are only comparable across identical toolchains. The MVP should record the toolchain version alongside the hash, so that a verifier knows which environment the hash corresponds to.

### The canonical hash record

```json
{
  "source_path": "Lex/Inventions/Radio.lean",
  "olean_hash": "sha256:9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08",
  "toolchain": "leanprover/lean4:v4.28.0",
  "mathlib_commit": "a1b2c3d4",
  "canonicalization": "olean-v3",
  "declarations": ["Radio.modulation", "Radio.frequency_hopping"]
}
```

---

## 🔐 Step 2: Signing with a Hardware Key

The signature binds the content hash to a private key, and the private key is bound to a physical device. The hardware key requirement is what makes "cryptographic authorship" meaningful: the signature cannot be produced without physical possession of the device and knowledge of the PIN.

### Hardware key options

| Device | Interface | Ed25519 support | CLI tool |
|---|---|---|---|
| **YubiKey 5 / 5C / Bio** | PIV / PKCS#11 | Yes (YubiKey 5.7+) | `yubico-piv-tool`, `ykcs11` |
| **Solo 2 / Tap** | FIDO2 / PKCS#11 | Yes (`ed25519-sk`) | `ssh-keygen`, `ykcs11` |
| **Tillitis TKey** | USB | Yes (Ed25519 app) | `tkey-ssh-agent` |

For the MVP, the **YubiKey 5** is the pragmatic choice: it is widely available, has a stable CLI (`yubico-piv-tool`), and supports Ed25519 in slot 9c (digital signature) on firmware 5.7+. The PIV tool exposes the key via PKCS#11, so any PKCS#11-aware signing library (including the Rust `signatory` crate's `yubihsm` provider) can use it without extracting the private key.

### The signing command

```bash
# Generate an Ed25519 key in slot 9c (one-time setup)
yubico-piv-tool -a generate -s 9c -A ED25519 -o public.pem

# Sign a content hash
yubico-piv-tool -a sign-data -s 9c -A ED25519 \
    --hash=SHA256 \
    -i content_hash.bin \
    -o signature.bin \
    --pin=123456
```

The signature is **detached**: it signs the 32-byte SHA-256 hash, not the full file. This is standard practice for Ed25519 over large payloads and keeps the signature fixed at 64 bytes.

### The key identity record

```json
{
  "device": "YubiKey 5C",
  "slot": "9c",
  "algorithm": "Ed25519",
  "public_key": "ed25519:MCowBQYDK2VwAyEA...",
  "attestation": "yubico-attestation:...",   // optional, proves key is hardware-backed
  "pin_policy": "always"
}
```

---

## ⏱️ Step 3: FreeTSA RFC 3161 Timestamping

The timestamp establishes that the signed hash **existed at a specific moment**, independently of the inventor's clock. FreeTSA is a free, public RFC 3161 Time Stamp Authority that has been operational since 2008 and is widely used in open-source projects.

### The timestamping pipeline

The `freetsa` Rust crate provides both a library and a CLI for acquiring timestamps from freetsa.org. The CLI accepts a file or a hash, produces a `.tsq` (timestamp query) and a `.tsr` (timestamp response), and the `.tsr` can be verified with OpenSSL.

```bash
# Install the FreeTSA client
cargo install freetsa

# Timestamp the signature (or the content hash)
freetsa timestamp file \
    --data signature.bin \
    --reply-out signature.tsr \
    --query-out signature.tsq

# Verify the timestamp against FreeTSA's certificates
openssl ts -verify \
    -in signature.tsr \
    -queryfile signature.tsq \
    -CAfile cacert.pem \
    -untrusted tsa.crt
```

The `.tsr` is a **signed timestamp token** that binds the hash of `signature.bin` to a UTC time. It is a self-contained DER-encoded object that can be stored alongside the lexeme entry and verified by any third party with OpenSSL. The token includes FreeTSA's signature, so verification does not require contacting FreeTSA again — the verification is offline and deterministic.

### The timestamp record

```json
{
  "tsa": "https://freetsa.org/tsr",
  "timestamp": "2026-10-03T14:32:00Z",
  "tsr_hash": "sha256:...",
  "tsq_hash": "sha256:...",
  "tsa_certificate": "freetsa-tsa.crt",
  "verification": "openssl-ts-verify"
}
```

---

## 🌳 Step 4: Appending to a Merkle Tree

The Merkle tree is the **tamper-evident log**. Each lexeme entry is a leaf; the root hash is the digest of the entire log. Adding a new entry changes the root in a way that is computationally infeasible to forge without recomputing the entire tree.

### The tree implementation

For the MVP, use the `ct-merkle` Rust crate, which implements the Certificate Transparency Merkle tree (RFC 6962) with inclusion and consistency proofs. It is an append-only memory-backed tree that also supports building proofs when the full tree does not fit in memory — the standard CT pattern.

```rust
use ct_merkle::{MemoryBackedTree, Sha256};

let mut tree = MemoryBackedTree::<Sha256, LexemeEntry>::new();
tree.push(entry);  // each lexeme is a leaf
let root = tree.root();  // publish this root
```

The leaf hash is `SHA-256(0x00 || entry_bytes)` and the internal node hash is `SHA-256(0x01 || left || right)`. The domain separation (0x00 for leaves, 0x01 for internal nodes) is what prevents second-preimage attacks on the tree structure. This is the RFC 6962 construction and is the same tree used by Certificate Transparency, Sigstore's Rekor, and Google's Trillian.

### The root publication

After appending the entry, the root hash is **published to an independent append-only log**. For the MVP, the simplest publication mechanism is a **public Git repository** that the inventor pushes to after each append. The commit hash of the push is itself a timestamped, signed record. A more robust mechanism is a public transparency log (Sigstore Rekor, or a custom log running Trillian), but Git is sufficient for a week-long build and is independently auditable.

### The tree state record

```json
{
  "tree_size": 1,
  "root_hash": "sha256:...",
  "root_publication": "https://github.com/inventor/lexledger/commit/abc123",
  "log_id": "sha256:...",
  "tree_head_signature": "ed25519:...",
  "consistency_proof": null  // first entry, no predecessor
}
```

---

## 📜 Step 5: Printing the Inclusion Proof

The inclusion proof is the **O(log n) path** from the leaf to the root. For a tree with `n` leaves, the proof is `log₂(n)` sibling hashes. For a tree with 1,000,000 entries, the proof is 20 hashes — 640 bytes. This is the artifact that travels with the lexeme entry: it lets any verifier recompute the root from the leaf and confirm that the leaf is in the published tree.

### Generating the proof

```rust
let leaf_index = tree.len() - 1;
let proof = tree.prove_inclusion(leaf_index);
```

The `InclusionProof` is a `Vec<u8>` of concatenated sibling hashes. The proof is serialized as a compact binary or JSON array:

```json
{
  "leaf_hash": "sha256:...",
  "leaf_index": 0,
  "tree_size": 1,
  "audit_path": [],
  "root_hash": "sha256:..."
}
```

For the first entry, the audit path is empty: the leaf hash *is* the root hash. For subsequent entries, the audit path is a list of sibling hashes ordered from leaf to root.

---

## ✅ Step 6: Browser Verifier

The browser verifier is the **court-facing artifact**. It accepts a lexeme entry and its proof, recomputes the hashes, and reports pass/fail — all client-side, with no backend, no trust in the inventor's machine, and no installation.

### The verification algorithm

```
1. Recompute the leaf hash: SHA-256(0x00 || entry_bytes)
2. Recompute the root from the leaf hash and the audit path:
   for each sibling in audit_path:
       if index is even: node = SHA-256(0x01 || node || sibling)
       else:             node = SHA-256(0x01 || sibling || node)
       index = index / 2
3. Assert that the recomputed root equals the published root_hash
4. Verify the Ed25519 signature over the content hash using the public key
5. Verify the RFC 3161 timestamp using the embedded TSR and FreeTSA's certificate
6. Report the combined verdict
```

Steps 1–3 use the browser's `WebCrypto` API for SHA-256 and a pure-JavaScript Merkle proof verifier. The `merkletreejs` library provides this directly: `tree.verify(proof, leaf, root)` returns `true` or `false`. Steps 4–5 use `WebCrypto` for Ed25519 verification and a minimal ASN.1 parser for the RFC 3161 token. The entire verifier is under 500 lines of JavaScript and runs in any modern browser.

### The verifier UI

The UI should be brutally simple, because its audience is a regulator, a judge, or a peer inventor — not a developer:

```
┌─────────────────────────────────────────────┐
│  LEXLEAN VERIFIER                           │
├─────────────────────────────────────────────┤
│  [ Drop lexeme entry JSON here ]            │
│                                             │
│  ✓ Content hash matches                     │
│  ✓ Signature valid (ed25519:MCowBQYD...)    │
│  ✓ Timestamp valid (2026-10-03 14:32 UTC)   │
│  ✓ Merkle inclusion proof valid             │
│  ✓ Root matches published log               │
│                                             │
│  VERDICT: This entry existed at the         │
│  stated time, was authored by the           │
│  holder of the stated key, and has not      │
│  been modified since.                       │
└─────────────────────────────────────────────┘
```

The verdict language is deliberately precise. It does **not** say "this invention is novel" or "this is legally valid." It says exactly what the cryptography proves: existence at a time, authorship by a key, and integrity since. Those are the three claims a court actually evaluates.

---

## 🪞 Step 7: The Recursive Bootstrap

The first entry in the ledger is the tool's own specification. This is the demonstration that the tool can protect the hardest case: the thing doing the protecting.

### What the first entry contains

- **The Lean specification** of the lexeme format: the `LexemeEntry` structure, its fields, and its invariants.
- **The Lean specification** of the Merkle tree: the leaf hash function, the internal node hash function, and the root computation.
- **The Lean specification** of the verification algorithm: the six-step procedure above, stated as a Lean function.
- **Theorems** about the verification algorithm: e.g., "if `verify(entry, proof, root) = true`, then `entry` is a leaf of the tree with root `root`."
- **The source code** of the CLI and the browser verifier.
- **The content hash** of each artifact.
- **The signature** over the combined hash.
- **The FreeTSA timestamp**.
- **The Merkle inclusion proof** (trivial for the first entry, but structurally present).

### The bootstrap command

```bash
# Build the tool's own spec
cd lexlean && lake build

# Compute the canonical hash
lexhash --source . --output content_hash.bin

# Sign with the hardware key
lexsign --hash content_hash.bin --output signature.bin

# Timestamp with FreeTSA
lexstamp --data signature.bin --output signature.tsr

# Append to the tree
lexappend --entry LexemeEntry.json --tree ./ledger --output proof.json

# Publish the root
git commit -m "Bootstrap: LexLean v0.1.0" && git push
```

After this command sequence, the tool is **self-describing and self-protecting**. The specification of the tool is the first lexeme. The proof that the tool works is the first inclusion proof. The authorship of the tool is the first signature. The priority of the tool is the first timestamp. Anyone who downloads the ledger and the verifier can confirm all four claims without trusting the inventor.

---

## 📦 The Complete MVP Manifest

| Component | Language | Dependency | LOC (est.) |
|---|---|---|---|
| **lexhash** | Rust | `sha2`, `lean` parser | 200 |
| **lexsign** | Rust | `signatory`, `yubico-piv-tool` | 150 |
| **lexstamp** | Rust | `freetsa` | 100 |
| **lexappend** | Rust | `ct-merkle` | 250 |
| **lexverify** (CLI) | Rust | `ct-merkle`, `ed25519-dalek`, `openssl` | 300 |
| **lexverify** (browser) | JavaScript | `merkletreejs`, `WebCrypto` | 400 |
| **LexSpec.lean** | Lean 4 | (none) | 300 |
| **Total** | | | **~1,700** |

This is a week of focused work for one person. The Rust CLI is the bulk of the effort; the Lean spec is straightforward; the browser verifier is a single HTML file with inline JavaScript.

---

## ⚠️ What the MVP Does Not Do (and Why That Is Fine)

The MVP does **not** implement semantic equivalence checking. It does **not** implement an agentic prover. It does **not** implement a compliance DSL. It does **not** implement runtime attestation. It does **not** implement a certificate registry. It does **not** implement a content-addressed store beyond the local Merkle tree.

What it *does* implement is the **irreducible core**: a signed, timestamped, tamper-evident record of a formal specification, with a browser-based verifier that any third party can run. If this core works, every other layer is a composition of it. If this core does not work — if the hashing is not canonical, if the signature is not hardware-bound, if the timestamp is not verifiable, if the Merkle proof is not sound — no amount of elaboration will fix it.

The MVP is not a prototype. It is the **thesis in executable form**. Build it, publish its own spec as the first entry, and the tool has already made its first legal argument: *"This specification existed at this time, was authored by this key, and has not been modified."* That is the entire foundation of the registry, and it is achievable in seven days.