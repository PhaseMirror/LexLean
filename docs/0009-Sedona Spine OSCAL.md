Treating Sedona Spine as an OSCAL profile means treating its FFI closure enforcement primitives as a **catalog of controls**, and using OSCAL's profile resolution machinery to select, tailor, and operationalize them for a specific compliance context. The coupling is not a metaphor — it is a structural identity. Sedona Spine already enforces closure invariants (single-crossing, RAII guards, phase tags) at the compiler level. OSCAL profiles already provide a deterministic, predictable process for reducing and tailoring a catalog into a system-specific baseline. The integration makes the compiler's enforcement primitives **first-class compliance controls**, and makes the OSCAL resolution pipeline **kernel-checked**.

---

## The Mapping: Sedona Spine Primitives as OSCAL Controls

Sedona Spine's primitives are the lowest layer of the PIRTM compiler: "FFI Closure Enforcement" with "single-crossing / RAII guards / Phase tags". These are not abstract policy statements; they are **enforceable constraints on the compiler's output**. An OSCAL catalog entry is a control: a statement of what must hold, with parameters, assessment objectives, and evidence requirements.

The mapping is natural because both are **machine-checkable predicates**:

| Sedona Spine primitive | OSCAL control statement | Assessment objective |
|---|---|---|
| **Single-crossing** | "The FFI boundary is crossed exactly once per closure invocation." | Count crossings; assert count = 1. |
| **RAII guards** | "Every acquired resource is released on all exit paths." | Static analysis; no leak in the closure body. |
| **Phase tags** | "Each FFI call occurs in the correct phase (init, compute, finalize)." | Phase-tag check at compile time. |
| **Contractivity condition** | "The closure's state evolution satisfies `κ(t) = κ₀ · e^(-αt)` with `α > 0`." | Spectral radius of the recurrence `< 1`. |
| **Lean FFI bridge** | "The closure's proof term is verified by the Lean kernel before codegen." | `#print axioms` on the extracted proof. |

Each primitive becomes a **control** in the Sedona Spine catalog:

```json
{
  "control": {
    "id": "sedona-spine.single-crossing",
    "title": "FFI Single-Crossing Invariant",
    "params": [
      { "id": "closure-id", "label": "Closure identifier" }
    ],
    "parts": [
      {
        "name": "statement",
        "prose": "The FFI boundary is crossed exactly once for closure {insert: param, closure-id}."
      },
      {
        "name": "assessment-objective",
        "prose": "Count FFI crossings in the closure body; assert exactly 1."
      },
      {
        "name": "assessment-method",
        "prose": "Static analysis of the MLIR dialect; PIRTM admissibility validator."
      }
    ]
  }
}
```

---

## The OSCAL Profile as the Selector and Tailor

The OSCAL profile is where the compliance context enters. A profile does three things: **imports** controls from one or more catalogs, **selects** a subset (by ID, by wildcard matching, or by explicit inclusion), and **modifies** the selected controls (setting parameters, amending statements, adding guidance).

For a Sedona Spine–based system, the profile answers: *which FFI invariants apply to this deployment, and with what parameters?*

```json
{
  "profile": {
    "uuid": "prof-radio-firmware-v1",
    "metadata": { "title": "Sedona Spine Profile for Radio Firmware" },
    "imports": [
      {
        "href": "sedona-spine-catalog.json",
        "include-controls": [
          { "with-ids": ["sedona-spine.single-crossing", "sedona-spine.raii-guards"] }
        ],
        "exclude-controls": [
          { "with-ids": ["sedona-spine.phase-tags"] }
        ]
      }
    ],
    "modify": {
      "set-parameters": [
        {
          "param-id": "closure-id",
          "values": ["radio.init", "radio.tx", "radio.rx"]
        }
      ],
      "alterations": [
        {
          "control-id": "sedona-spine.contractivity",
          "adds": [
            {
              "position": "ending",
              "parts": [
                {
                  "name": "guidance",
                  "prose": "For radio firmware, α must be ≥ 0.05 to ensure bounded convergence within the FCC dwell-time window."
                }
              ]
            }
          ]
        }
      ]
    }
  }
}
```

The profile is **declarative**: it says what to include and how to modify it, not how to resolve it. Resolution is a separate, deterministic process.

---

## The Resolution Pipeline: Profile → Resolved Catalog

OSCAL profile resolution consumes a profile and the catalogs it references, and produces a **new tailored catalog**. The resolver processes elements in a specific order: process imports (recursively for profile chains), apply inclusions and exclusions, apply modifications. The result is deterministic: the same profile and catalog produce the same resolved catalog regardless of the tool used.

For the Sedona Spine integration, the resolved catalog is the **effective set of FFI invariants** for the deployment. It is not a policy document; it is the compiler's enforcement configuration. The resolver output includes:

- The selected controls (single-crossing, RAII guards, etc.).
- The parameter values (closure IDs, α threshold).
- The amended statements (guidance for radio firmware).
- The assessment objectives (how each control is checked).

```bash
# Resolve the profile to a catalog
oscal-cli profile resolve sedona-radio-profile.json --to sedona-radio-resolved.json

# Validate the resolved catalog
oscal-cli catalog validate sedona-radio-resolved.json
```

The resolved catalog is then consumed by the PIRTM compiler: the compiler reads the resolved controls and **configures the Sedona Spine enforcement passes** accordingly. If the profile excludes phase-tags, the compiler omits the phase-tag check. If the profile sets α ≥ 0.05, the compiler uses that threshold in the Zeno-Finton control.

---

## The Lean Formalization of Resolution and Invariants

The deterministic resolution process is itself a **Lean-checkable function**. The OSCAL Profile Resolution specification defines the normative requirements; encoding them in Lean makes the resolver **kernel-checked**.

```lean
-- The resolution relation: profile × catalog → resolved catalog
inductive Resolves : Profile → Catalog → ResolvedCatalog → Prop where
  | imports : ResolvesImports p c r → Resolves p c r
  | inclusions : ...
  | modifications : ...
  | composition : Resolves p c₁ r₁ → Resolves p r₁ r₂ → Resolves p c₁ r₂

-- Determinism theorem: the resolver is a function
theorem resolution_deterministic
    (p : Profile) (c : Catalog) (r₁ r₂ : ResolvedCatalog)
    (h₁ : Resolves p c r₁) (h₂ : Resolves p c r₂) :
    r₁ = r₂ := by
  -- proof that the resolution relation is functional
  ...
```

The Sedona Spine invariants are **theorems about the compiled output**:

```lean
-- The single-crossing invariant holds for every compiled closure
theorem single_crossing_holds (cl : Closure) (h : Compiles cl) :
    countCrossings cl.ffiBody = 1 := by
  ...

-- The contractivity condition holds for every lawful closure
theorem contractivity_holds (cl : Closure) (h : Compiles cl) :
    spectralRadius cl.recurrence < 1 := by
  ...
```

These theorems are the **evidence** that the resolved catalog's controls are satisfied. The assessment objectives from the OSCAL controls become Lean proof obligations; the proofs are the assessment results.

---

## The OSCAL Lifecycle with Sedona Spine as the Baseline

OSCAL defines a full lifecycle: catalog → profile → resolved catalog → System Security Plan (SSP) → Assessment Plan (SAP) → Assessment Results (SAR). With Sedona Spine as the catalog, the lifecycle becomes:

| OSCAL artifact | Sedona Spine interpretation |
|---|---|
| **Catalog** | The complete set of FFI invariants (single-crossing, RAII, phase tags, contractivity, Lean FFI). |
| **Profile** | The deployment-specific selection and tailoring (which invariants, what parameters). |
| **Resolved catalog** | The effective enforcement configuration for the compiler. |
| **SSP** | The system description: the closures compiled, the hardware, the TCB. |
| **SAP** | The assessment plan: which Lean theorems to prove, which proofs to check. |
| **SAR** | The assessment results: the `#print axioms` output, the contractivity receipts, the provenance chain. |

The SAR is where the formalization meets the compliance claim. Each control's assessment objective is discharged by a Lean proof; the proof's axiom profile is audited; the audit receipt is recorded in the provenance chain.

---

## The Provenance Chain for OSCAL Artifacts

Every OSCAL artifact — catalog, profile, resolved catalog, SSP, SAP, SAR — is **content-addressed, signed, and timestamped**. The pipeline from the earlier MVP applies directly:

1. **Canonicalize** the OSCAL JSON (RFC 8785 JSON Canonicalization Scheme).
2. **Hash** with SHA-256.
3. **Sign** with the hardware key (Ed25519).
4. **Timestamp** with FreeTSA (RFC 3161).
5. **Append** to the PIRTM Merkle chain.
6. **Publish** the root.

The OSCAL artifact's identity is its hash. The profile's hash is referenced by the resolved catalog; the resolved catalog's hash is referenced by the SSP; the SSP's hash is referenced by the SAR. The chain of hashes is the **chain of custody** for the compliance claim.

```json
{
  "oscal_artifact": "sedona-radio-profile.json",
  "canonical_hash": "sha256:...",
  "signature": "ed25519:...",
  "timestamp": "rfc3161:...",
  "merkle_leaf": "sha256:...",
  "merkle_root": "sha256:...",
  "inclusion_proof": ["sha256:...", "sha256:..."]
}
```

---

## The PIRTM Coupling: Snapaddr Identity and Contractivity Receipts

The Sedona Spine layer sits between the PIRTM MLIR dialect and the Zeno-Finton control. Coupling OSCAL to this layer means:

- **Snapaddr identity for OSCAL artifacts.** The resolved catalog's identity is `snapaddr(canonical_oscal_json)`, not `sha256(raw_json)`. The prime-indexed encoding preserves the structural decomposition of the control tree.
- **Contractivity receipts for assessments.** The SAR includes a PIRTM contractivity receipt attesting that the system's state evolution (the compilation pipeline, the proof search) satisfies the contraction condition. This is the **lawful-process** guarantee that a regulator needs: not just "the record is in the log," but "the log evolved lawfully."
- **Lean FFI–compiled verifier.** The browser verifier for OSCAL artifacts is the Lean proof of the resolution algorithm, compiled to WASM via PIRTM's `pirtm-llvm` module. It does not re-implement resolution in JavaScript; it runs the kernel-checked resolver.

The receipt is **stratified**: the inner layer is the RFC 3161 timestamp and the RFC 6962 Merkle proof (verifiable by any standard tool); the outer layer is the PIRTM contractivity receipt and the snapaddr identity (verifiable by the PIRTM verifier). A court verifies the inner layer; a regulator verifies both.

---

## The Integration in One Command

```bash
# 1. Resolve the OSCAL profile against the Sedona Spine catalog
oscal-cli profile resolve sedona-radio-profile.json --to resolved.json

# 2. Extract the resolved controls as PIRTM compiler configuration
pir extract-controls resolved.json --output compiler-config.json

# 3. Compile the closures with Sedona Spine enforcement
pir compile radio.pirtm --config compiler-config.json --emit-wasm radio.wasm

# 4. Generate the assessment results (Lean proofs + axiom audits)
pir assess radio.wasm --catalog resolved.json --output sar.json

# 5. Sign, timestamp, and append to the provenance chain
lexledger append sar.json --key yubikey:9c --tsa freetsa --tree ./ledger
```

The output is a **compliance packet**: the resolved catalog, the compiler configuration, the WASM binary, the SAR, the Lean proofs, the axiom audits, the signature, the timestamp, and the inclusion proof. Any third party can verify the packet without installing PIRTM, without trusting the inventor, and without understanding Lean. The OSCAL artifacts are machine-readable; the cryptographic evidence is standard; the Lean proofs are the backing assurance.

The coupling is not a new system. It is the **same pipeline** from the MVP, with two substitutions: the identity function is snapaddr instead of SHA-256, and the inclusion proof carries a contractivity receipt. Everything else — canonical hashing, hardware signing, FreeTSA timestamping, Merkle inclusion, browser verification — remains unchanged. The Sedona Spine catalog provides the control vocabulary; the OSCAL profile provides the tailoring; the PIRTM receipt provides the lawful-evolution guarantee.