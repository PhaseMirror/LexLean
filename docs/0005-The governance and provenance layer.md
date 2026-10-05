The governance and provenance layer is where the formalization's claims become **independently verifiable** — not merely trusted because the project says so, but because an external auditor can trace every theorem back to its legal source, every proof artifact back to its content hash, and every certified claim back to its corresponding Lean module. The Ouroboros Substrate unifies "Lean 4 kernel proofs, a running agentic substrate, an observability/security graft, and a sovereign-AI provenance chain". For LexLean, this means building three interconnected mechanisms: a **content-addressed ledger** that anchors every proof artifact, a **machine-readable citation layer** that links theorems to their legal sources, and a **public certificate registry** that lists all externally verifiable claims.

---

## 1. Hash Every Proof Artifact and Record It in a Content-Addressed Ledger

### Why content addressing is the foundation of provenance

A content-addressed system identifies each artifact by a cryptographic hash of its contents rather than by a mutable name or path. This means that the identity of a proof artifact is **intrinsic** to the artifact itself: if the source changes by a single byte, the hash changes, and the provenance chain records a different object. The `eigenius` architecture demonstrates this principle concretely: "The proof elaborates against a specific Lean environment E (a pinned, content-addressed resource)" and "At every step, every artifact is content-addressed and tracked as a resource". An auditor walks from a verdict to the proof term, payload, and chain class, re-fetching every byte from the content-addressed store.

The `uuidna` project provides a production example: a "content-addressed identity system, honest by construction" with "a Lean 4 theorem ledger — 2499 distinct theorems under 2582 keys" where every theorem is "proven by decide, sorry-free, no Mathlib, axiom-free against the bare leanprover/lean4 kernel". The system folds to a single order-invariant receipt, and the deposited metadata is *generated from the ledger it describes* — the archive states the ledger that exists rather than one that has moved on.

### What a LexLean proof-artifact ledger should contain

A LexLean content-addressed ledger would record four classes of artifacts:

**A. Source artifacts.** Every `.lean` file in the LexLean repository, hashed at the exact commit that produced the proof. The hash of the source file is the anchor for the proof's origin.

**B. Compiled artifacts.** The `.olean` files produced by `lake build`. These are the kernel-checked outputs that the Lean runtime consumes. The `eigenius` pattern treats "the proof term's content hash, combined with the environment hash and the FFI hash" as "the cache key for verification results". For LexLean, the key would be `hash(proof_term) + hash(lean_environment) + hash(lexlean_dependencies)`.

**C. Audit artifacts.** The `#print axioms` output for each theorem, the proof-health report, and the skeleton registry snapshot. These are the **evidence** that the proof meets the project's governance policy.

**D. Certificate artifacts.** The serialized derivation trees, the legal source citations, and the certificate registry entries (described below).

### The provenance chain structure

The ledger should be structured as an **append-only chain**, where each entry references the previous entry by hash. This is the WORM (write-once-read-many) pattern: entries cannot be modified after they are written, and the chain is tamper-evident. The `SNAPKITTYWEST` framework demonstrates this with "cryptographically sealed artifact provenance using append-only SHA-256 WORM chains". The `eigenius` trace tree is described as *being* the provenance chain: "To find what inputs produced an output, walk the tree from root to leaves".

For LexLean, the provenance chain would be:

```
Entry_N = {
  hash: SHA256(Entry_{N-1}.hash || artifact_hash || timestamp || metadata),
  artifact_hash: SHA256(artifact_bytes),
  artifact_type: "source" | "olean" | "axiom_audit" | "certificate",
  legal_source: URI,  // machine-readable citation (see section 2)
  theorem_name: Name,  // if applicable
  axiom_profile: List Name,  // if applicable
  registry_entry: Name  // skeleton registry entry, if applicable
}
```

### Integration with the verification pipeline

The content-addressed ledger integrates directly with the verification pipeline's CI gates. On every commit:

1. **Hash all modified source files** and record them in the ledger.
2. **Compile the project** and hash the resulting `.olean` files.
3. **Run the axiom audit** and hash the `#print axioms` output for each public declaration.
4. **Generate the proof-health report** and hash it.
5. **Append a new ledger entry** that references the previous entry's hash.

The CI gate then verifies that the ledger is **consistent**: every `proven` theorem in the skeleton registry has a corresponding ledger entry with a compliant axiom profile, and every ledger entry's `previous_hash` matches the actual previous entry.

### The ledger as a public artifact

The ledger should be published as a **machine-readable JSON or CBOR artifact** alongside the Lean source. This makes the provenance chain independently verifiable: an auditor can download the ledger, re-compute the hashes from the source, and confirm that the chain is intact. The `uuidna` project demonstrates this with an MCP server exposing the ledger. For LexLean, a public ledger endpoint would allow any stakeholder — a regulator, a court, or a downstream consumer — to verify the provenance of a specific theorem without trusting the LexLean infrastructure.

---

## 2. Link Each Theorem to Its Legal Source via a Machine-Readable Citation Format

### The traceability gap in legal formalization

A formal proof of a legal theorem is only as trustworthy as the legal source it formalizes. If the theorem claims that "a contract formed under duress is voidable" but the underlying statute has been amended, the theorem is **stale** — it proves a proposition that no longer corresponds to the authoritative text. The current state of legal citation is inadequate for this task: "There is no cryptographically verifiable proof that a quoted decision text is actually the officially published one" and citations between decisions are "raw text strings (BGE 140 III 86)" that "each consuming application must resolve itself, with varying accuracy".

The `L3CV` framework addresses this with a three-level citation verification approach: syntactic (format), semantic (content), and contextual (authority). For LexLean, the citation layer must go further: it must provide a **machine-checkable link** from each theorem to the specific legal provision it formalizes, and it must detect when that provision changes.

### The URI-addressable legal norm model

The Legal Knowledge Graph Foundations paper provides the architectural blueprint: "Assign a unique URI to each legal norm, ensuring unambiguous identification. Use HTTP(S) URIs. Configure each URI to return machine-readable metadata about the identified resource". The ELI (European Legislation Identifier) standard demonstrates this with URIs like `http://www.legislation.gov.uk/ukpga/2014/2/section/7#section-7-2-b`, which resolve to metadata about the specific section. The Akoma Ntoso standard provides a similar global reference format: `/akn/uk/act/pga/2014/2#section-7-2-b`.

For LexLean, each formalized provision should have a **canonical URI** that:

- **Identifies the legal source** (statute, regulation, or case) by its official identifier.
- **Identifies the specific provision** within that source (section, subsection, paragraph).
- **Resolves to machine-readable metadata** about the provision, including its current text, its amendment history, and its effective date.

### The citation format

The citation format should be **machine-readable and human-auditable**. A JSON-LD representation would include:

```json
{
  "@context": "https://lexlean.org/citation/v1",
  "@type": "LegalProvisionCitation",
  "theorem": "LexLean.Contract.duress_voidable",
  "provision": {
    "uri": "https://legislation.gov.example/statute/2024/contract-act#section-12-3",
    "identifier": "Contract Act 2024, s 12(3)",
    "text_hash": "sha256:a1b2c3...",
    "effective_date": "2024-07-01",
    "amendment_status": "current"
  },
  "formalization": {
    "module": "LexLean.Contract",
    "lean_statement": "theorem duress_voidable (c : Contract) ...",
    "proof_hash": "sha256:d4e5f6...",
    "axiom_profile": ["propext", "Classical.choice", "Quot.sound"]
  },
  "provenance": {
    "ledger_entry": "sha256:789abc...",
    "registry_entry": "LexLean.SkeletonRegistry.duress_voidable",
    "verification_status": "proven"
  }
}
```

The `text_hash` field is the critical addition: it is the hash of the **exact text** of the legal provision at the time the theorem was formalized. If the provision is amended, the hash changes, and the citation becomes **stale**. A CI job can periodically re-resolve the URIs and compare the current `text_hash` against the recorded `text_hash`. If they differ, the theorem is flagged for review.

### The stale-citation gate

The CI gate for stale citations operates as follows:

1. **Extract all citations** from theorem docstrings and the citation registry.
2. **Resolve each URI** to the current text of the provision.
3. **Compute the hash** of the current text.
4. **Compare** against the recorded `text_hash`.
5. **If different**: fail the CI job and flag the theorem for formalization review.

This gate ensures that LexLean's formalization remains **synchronized with the legal sources it claims to formalize**. Without it, the formalization could drift from the law, producing proofs that are technically correct but legally meaningless.

### The citation as a first-class Lean object

The most rigorous approach is to encode the citation as a **Lean structure** that is part of the theorem's type:

```lean
structure LegalCitation where
  uri : String
  identifier : String
  textHash : ByteArray
  effectiveDate : Date
  -- no public constructor; citations are minted by the citation registry

def duress_voidable
    (citation : LegalCitation)
    (h_valid : citation.uri = "https://legislation.gov.example/statute/2024/contract-act#section-12-3")
    (c : Contract) (h_duress : Duress c) :
    Voidable c := ...
```

This approach makes the citation **part of the proof obligation**: the theorem cannot be stated without supplying a citation, and the citation must resolve to the correct provision. The `LegalCitation` structure mirrors the sealed-witness pattern from the substrate layer: it has no public constructor, so citations can only be minted by the citation registry, which is itself subject to the stale-citation gate.

---

## 3. Publish a Public Certificate Registry

### What a certificate registry is

A certificate registry is a **public, machine-readable catalog of externally verifiable claims**. Each entry lists a claim (a theorem statement), its corresponding Lean module, its verification status, its axiom profile, and its provenance chain entry. The ZLT `LeanCore-Registry` provides the concrete template: it is "a registry of seven terminal Lean 4 / Mathlib certificates corresponding to the foundational layer of the Zero Leap Theory corpus", where each certificate is "sorry-free, axiom-free in its formal-core layer, and uses only the standard Lean/Mathlib trust base (propext, Classical.choice, Quot.sound)".

The ZLT registry includes a **dependency DAG**, a **certificate summaries** section, an **axiom hygiene report**, a **reproducibility protocol**, and **explicit claim boundaries**. Each of these components is essential for a public registry: the DAG shows how certificates depend on each other, the summaries describe what each certificate proves, the hygiene report documents the axiom profile, the reproducibility protocol explains how to independently verify the certificate, and the claim boundaries state what the certificate does *not* claim.

### The structure of a LexLean certificate registry

A LexLean certificate registry would be organized by **legal domain** rather than by module, because the registry's primary audience is legal stakeholders rather than Lean developers. The registry would have the following structure:

**Section 1: Machine preamble.** The specification name (`LexLean-Certificate-Registry`), version, DOI, series layer, and trust base. This mirrors the ZLT registry's machine preamble.

**Section 2: What "certified" means.** An explicit definition of the certification threshold: a theorem is certified if (a) it is `proven` in the skeleton registry, (b) its axiom profile is exactly the kernel axioms, (c) it has a valid legal citation with a current `text_hash`, and (d) its provenance chain entry is consistent. This is the operational threshold — the criteria that a claim must meet to appear in the registry.

**Section 3: Certificate registry.** The list of certified claims, organized by legal domain (contract law, statutory interpretation, constitutional law, etc.). Each entry includes:

| Field | Description |
|---|---|
| **Claim** | The natural-language statement of what is proven |
| **Theorem** | The Lean identifier (e.g., `LexLean.Contract.duress_voidable`) |
| **Module** | The Lean module containing the proof |
| **Legal source** | The machine-readable citation (URI + identifier + text hash) |
| **Axiom profile** | The transitive axiom closure (should be the kernel axioms) |
| **Provenance** | The ledger entry hash |
| **Dependencies** | Other registry entries this claim depends on |
| **Boundary** | What the claim does *not* assert |

**Section 4: Dependency DAG.** A visual and machine-readable representation of how the certified claims depend on each other. This is essential for auditability: if a foundational claim is later found to be flawed, the DAG shows which downstream claims are affected.

**Section 5: Axiom hygiene report.** The `#print axioms` output for every certified theorem, with a machine-checkable assertion that the axiom profile is a subset of `{propext, Classical.choice, Quot.sound}`. The ZLT registry includes this as a separate section with "Axiom Check Logs".

**Section 6: Reproducibility protocol.** The exact commands and environment needed to independently reproduce the proofs. This should include the Lean toolchain version, the Mathlib commit, the LexLean commit, and the `lake build` command. The ZLT registry includes a "Reproducibility Protocol" section.

**Section 7: Claim boundaries.** For each certified claim, an explicit statement of what the proof does *not* establish. This is the most important section for legal stakeholders: a theorem that "a contract formed under duress is voidable" does *not* mean that a specific contract *was* formed under duress, nor does it mean that a court *must* void it. The boundary statement clarifies the scope of the formalization. The ZLT registry includes a "Claim Boundaries" section.

### The registry as a DOI-frozen artifact

The UOR Foundation describes its governance as "machine-checkable" with "DOI-frozen reference points". The LexLean certificate registry should follow this pattern: each version of the registry is assigned a **DOI** via Zenodo, making it a citable, immutable artifact. When the registry is updated, a new DOI is assigned, and the previous version remains accessible at its DOI. This provides **temporal traceability**: a legal stakeholder can cite the exact version of the registry that was current at a specific date.

The `atlas-embeddings` release process demonstrates the concrete pattern: Zenodo DOI integration, a `CHANGELOG.md`, a `RELEASE.md`, and a `citation` metadata file with the DOI. For LexLean, the release process would be:

1. **Freeze the ledger** at a specific commit.
2. **Generate the registry** from the ledger and the skeleton registry.
3. **Run the reproducibility protocol** to confirm that all proofs rebuild.
4. **Deposit the registry** on Zenodo and receive a DOI.
5. **Record the DOI** in the ledger and the registry metadata.

### The registry as a governance artifact

The certificate registry is not merely a documentation artifact. It is the **public interface** of LexLean's governance. Every claim in the registry is backed by a machine-checkable proof, a content-addressed provenance chain, a machine-readable legal citation, and an axiom hygiene report. When a stakeholder — a regulator, a court, a downstream developer — needs to verify a claim, they can:

1. **Look up the claim** in the registry.
2. **Download the Lean source** at the recorded commit.
3. **Re-run `lake build`** and `#print axioms` to confirm the proof.
4. **Resolve the legal citation** to confirm that the provision is current.
5. **Verify the provenance chain** by re-computing the hashes.

This is the operational meaning of "machine-checkable governance": every claim can be independently verified by anyone with access to the registry, the Lean source, and the legal source. The registry is the **single point of entry** for that verification.

---

## Summary: The Governance and Provenance Layer as a Verifiable Trust Architecture

| Component | What it establishes | How it is verified |
|---|---|---|
| **Content-addressed ledger** | The provenance of every proof artifact | Hash chain integrity; re-computation from source |
| **Machine-readable citations** | The link between theorems and legal sources | URI resolution; text-hash comparison; stale-citation gate |
| **Certificate registry** | The set of externally verifiable claims | DOI-frozen artifact; reproducibility protocol; axiom hygiene report |

The governance and provenance layer is the **public face** of LexLean's trust architecture. It answers the question that every legal stakeholder will ask: *"Why should I trust this proof?"* The answer is not "because we say so" but "because you can verify it yourself, using the ledger, the citation, and the registry." The Ouroboros Substrate's sovereign-AI provenance chain provides the model: every computation is linked to a dual-witness attestation chain, SCITT-compatible and SHA-256-anchored. LexLean's provenance chain should be equally rigorous: every legal theorem linked to a machine-readable citation, every proof artifact content-addressed, every certified claim listed in a public, DOI-frozen registry.