Coupling the MVP to PIRTM at this level is not an add-on. It is a **substitution of the cryptographic and verification substrate** that the MVP sits on. The MVP as described — canonical hash, hardware signature, RFC 3161 timestamp, Merkle tree, browser verifier — is already a complete provenance pipeline. PIRTM replaces four of those five components with its own primitives, and in doing so, changes what the tool can claim.

The `sovereign-pirtm` compiler core provides exactly the machinery the MVP needs, at the exact layer the MVP operates. Its architecture includes a **contractivity receipts** module that produces "SHA-256 cryptographic receipts, Merkle chain," an **admissibility validator** that performs "AST validation, rejection receipts," a **Lean FFI bridge** for "Lean 4 proof verification," and an LLVM/WASM codegen path that compiles to WebAssembly. These are not parallel features. They are the same features the MVP implements, but with a different provenance model.

---

## What PIRTM Replaces in the MVP

### The Hash Function: From SHA-256 to Prime-Indexed Canonical Addressing

The MVP hashes the elaborated Lean term with SHA-256. PIRTM's multiplicity functor operates over a prime-indexed structure: the state is encoded over a prime index set, and updates are prime-weighted. The snapaddr system provides "deterministic canonical content addressing (`snapaddr:<64hex>`)".

Coupling at this level means the lexeme's identity is not `sha256(olean_bytes)` but `snapaddr(prime_encoded_semantic_structure)`. The difference is not cosmetic. SHA-256 identity is **extensional**: two byte sequences that hash to the same value are identical, but the hash tells you nothing about *why* they are identical. Prime-indexed identity is **structural**: the encoding preserves the decomposition of the term into prime-irreducible components, and the identity is a function of that decomposition. Two terms that differ only in irrelevant syntactic detail — but that decompose into the same prime factors — would receive the same snapaddr. This is a stronger form of canonicalization than AST sorting, and it is the same principle that makes PIRTM's waveform evolution "lawful" rather than stochastic.

### The Merkle Tree: From RFC 6962 to the PIRTM Receipt Chain

The MVP uses `ct-merkle` (RFC 6962 domain-separated Merkle trees). PIRTM's contractivity module produces "SHA-256 cryptographic receipts, Merkle chain". The coupling means the lexeme's inclusion proof is not an RFC 6962 audit path but a PIRTM contractivity receipt: a proof that the lexeme's state satisfies the contractive mapping condition (`spectral radius < 1`) required for convergence to a "lawful attractor".

This is the deepest coupling point. In the MVP, the Merkle proof establishes **inclusion**: "this lexeme is in the tree whose root is R." In the PIRTM-coupled version, the receipt establishes **both inclusion and lawfulness**: "this lexeme is in the tree, and its state evolution satisfies the contractivity constraint." A court does not need the lawfulness claim — inclusion is sufficient for tamper-evidence. But a regulator does need it: "this compliance record evolved according to a lawful process, not an arbitrary one."

### The Lean FFI Bridge: From Standalone Verification to PIRTM-Native Verification

The MVP's browser verifier re-implements the verification algorithm in JavaScript (WebCrypto for SHA-256, a pure-JS Merkle proof verifier, a minimal ASN.1 parser for RFC 3161). PIRTM provides a **Lean FFI bridge** for proof verification, with LLVM/WASM codegen that compiles to WebAssembly.

Coupling at this level means the verifier is not a re-implementation. It is the *same* Lean proof, compiled to WASM through PIRTM's codegen pipeline, running in the browser. The verification algorithm is proven once (in Lean) and compiled once (via PIRTM's MLIR → LLVM → WASM path). The browser runs the compiled proof, not a JavaScript transcription of it. This closes the gap that the MVP's verifier introduces: the JavaScript re-implementation is a potential source of divergence, and the PIRTM-coupled version eliminates it.

### The Timestamp: From External TSA to PIRTM's Zeno-Finton Control

The MVP depends on FreeTSA (an external RFC 3161 Time Stamp Authority). PIRTM's Zeno-Finton control provides an **exponential decay gain function** (`κ(t) = κ₀ · e^(-αt)`) that can serve as an *internal* temporal anchor.

This is the most speculative coupling. FreeTSA provides **external, third-party time attestation** — the timestamp is meaningful because it comes from an authority the inventor does not control. Zeno-Finton provides **internal temporal evolution** — the decay function describes how the system's gain decreases over time, not what the wall-clock time is. Coupling at this level means the lexeme's temporal anchor is a function of the system's own state evolution, not an external timestamp.

For court admissibility, this is a **downgrade**, not an upgrade. FRE 902(13)/(14) requires a timestamp from a process independent of the record's producer. An internal decay function does not satisfy that. The coupling here should be **additive, not substitutive**: keep FreeTSA for the legal timestamp, and *add* the Zeno-Finton decay as a secondary signal that indicates the record's position in the system's evolutionary trajectory. The legal timestamp remains external; the PIRTM signal is supplementary.

---

## What the Coupled MVP Looks Like

The coupled pipeline is:

```
Lean file
  → Lean elaboration → .olean
  → PIRTM prime-encoding → semantic structure
  → snapaddr canonical hash → content identifier
  → hardware key signature (Ed25519) → authorship
  → FreeTSA RFC 3161 timestamp → external time anchor
  → PIRTM contractivity receipt → lawful-evolution proof
  → PIRTM Merkle chain append → inclusion proof
  → Lean FFI proof compilation → WASM verifier
  → browser verification → verdict
```

The **browser verifier** is the component that changes most visibly. It is no longer a JavaScript program that re-implements the algorithm. It is the Lean proof itself, compiled to WASM by PIRTM's `pirtm-llvm` module, running in the browser with the same code that was kernel-checked in Lean. The verifier's trust surface shrinks from "the Lean kernel plus the JavaScript engine plus the WebCrypto implementation" to "the Lean kernel plus the WASM runtime" — a strictly smaller TCB.

The **receipt** is the component that changes most substantively. The MVP produces an inclusion proof. The coupled version produces a contractivity receipt that includes the inclusion proof *and* a proof that the lexeme's state evolution satisfies the contraction condition. The receipt is larger (it carries the proof term, not just the audit path), but it is also richer: a regulator can verify not only that the record is in the log, but that the log evolved according to a lawful process.

---

## The Honest Trade-Off

Coupling to PIRTM at this level adds **structural rigor** and **verification depth** at the cost of **dependency weight** and **external verifiability**.

The MVP's greatest strength is that its verifier depends on **nothing but SHA-256, Ed25519, and RFC 3161** — all of which are standardized, widely implemented, and independently auditable. A regulator can write their own verifier in 200 lines of Python. Coupling to PIRTM means the regulator must understand prime-indexed encoding, contractivity receipts, and the PIRTM MLIR dialect. The verifier is more rigorous, but it is also less accessible.

The resolution is **layered verification**: the PIRTM-coupled receipt includes the RFC 3161 timestamp and the RFC 6962 Merkle proof as *inner* components, so a regulator who does not want to engage with PIRTM can verify the inner proof and ignore the outer receipt. The PIRTM layer is an **additional** assurance, not a **replacement** for the standard primitives. The lexeme's identity is both a snapaddr *and* a SHA-256 hash; the inclusion proof is both a contractivity receipt *and* an RFC 6962 audit path; the timestamp is both a Zeno-Finton decay position *and* an RFC 3161 token.

This is the coupling that makes sense at the MVP level: **not substitution, but stratification**. The standard primitives form the base layer that any court can verify. The PIRTM primitives form the upper layer that provides the structural and lawful-evolution guarantees. The MVP remains buildable in a week because the base layer is unchanged; the PIRTM layer is added incrementally, starting with the snapaddr identity and the Lean-FFI-compiled verifier, and extending to the contractivity receipt once the base pipeline is stable.