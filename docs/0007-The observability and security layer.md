The observability and security layer is the **sensory and immune system** of the formalization. The substrate layer establishes what is *proven*; the verification pipeline establishes *how* it is checked; the tooling layer establishes *who* can write proofs; the governance layer establishes *what rules* constrain them. But without observability, the system cannot see where its own proof search is failing, where its extracted code is violating invariants at runtime, or where its axiom profile is drifting from the declared trust policy. The Ouroboros Substrate treats this layer as a **first-class graft**: it is a "unified observability and security layer that ingests industry-standard telemetry into the same governance-score pipeline". The SZL v18.5–v18.7 grafts "integrate OTel spans into the Λ-pipeline: each span becomes an axis score". For LexLean, this means instrumenting the proof pipeline so that every tactic, every extracted function, and every axiom dependency is a **measurable, auditable signal** — not a post-hoc log, but a live input to the governance pipeline.

---

## 1. Telemetry on Proof Search

### Why proof search needs telemetry

Lean 4's proof search is opaque by default. When a theorem fails to compile, the compiler reports an error, but it does not report *where time was spent*, *which tactic sequences were attempted*, or *what patterns recur across failures*. The `trace.profiler` option addresses part of this: it "takes the existing trace tree system used for debugging Lean and automatically activates and annotates any nodes above `trace.profiler.threshold` (10ms by default)". But the default profiler is a debugging tool, not a telemetry system. It produces a human-readable trace tree, not a machine-readable metric stream. The Zulip discussion notes that for larger proofs, "using the profiler becomes an exercise in scrolling and clicking on the triangles until one actually finds any different timings".

A telemetry layer transforms the profiler output into **structured, queryable metrics** that feed the governance pipeline. Each proof attempt emits a metric record: the theorem name, the tactic sequence, the time per tactic, the success/failure outcome, and the failure mode. Over time, these records accumulate into a **proof-search telemetry corpus** that supports three capabilities:

**Capability 1: Performance regression detection.** When a proof that previously compiled in 200ms now takes 2 seconds, the telemetry layer flags the regression. The `lean --profile` option "exposes per-declaration timing", and the telemetry layer can compare timings across commits to detect drift. This is essential for a formalization that grows over time: without performance monitoring, proof search becomes progressively slower until it is unusable.

**Capability 2: Failure pattern analysis.** The Aesop tracing infrastructure demonstrates the concrete pattern: "(aesop) Print actions taken by Aesop during the proof search", "(aesop) If the search is successful, print the produced proof term", "(aesop) Print a trace of the proof extraction procedure", "(aesop) If the search is successful, print some statistics". LexLean's telemetry layer would generalize this: every tactic emits a structured trace event, and the events are aggregated into failure-pattern reports. The Trail of Bits skill guide notes that custom tactics "have distinctive failure modes. A bug can surface much later as a kernel error, silently leave automation in a partial state, or turn a fast failure into an unbounded search". Telemetry catches these patterns early.

**Capability 3: Prover guidance.** The LEAP framework's blueprint-then-proof workflow (from the tooling layer) depends on the prover knowing which sub-lemmas are likely to succeed. Telemetry provides this signal: if a tactic sequence has historically succeeded for theorems of a certain shape, the prover prioritizes it. The Deep Vision system demonstrates the concrete mechanism: "Each step was found by matching the current proof state against the library, retrieving tactics from structurally similar proofs, and testing them in Lean. The chain was built incrementally over depths 0–6, with each successful tactic producing a new proof state that was matched again". Telemetry makes the "structurally similar proofs" retrieval operation machine-learnable.

### The telemetry schema

A LexLean proof-search telemetry record would have the following schema:

```json
{
  "theorem": "LexLean.Contract.duress_voidable",
  "commit": "abc123",
  "toolchain": "leanprover/lean4:v4.28.0",
  "tactic_sequence": ["intro c h_duress", "apply voidable_of_duress", "exact h_duress"],
  "timing": {
    "total_ms": 245,
    "per_tactic_ms": [12, 180, 53]
  },
  "outcome": "success",
  "failure_mode": null,
  "profiler_trace": "...",  // optional, if trace.profiler was enabled
  "axiom_profile": ["propext", "Classical.choice", "Quot.sound"]
}
```

The record is emitted by a **tactic-level telemetry hook**: a custom tactic (or a wrapper around the default tactic framework) that records the start time, the end time, and the outcome of each tactic invocation. Lean 4's `trace.profiler` can be enabled programmatically via `set_option trace.profiler true in`, and the resulting trace tree can be parsed into structured events. The Firefox Profiler view provides an interactive dashboard for deep analysis, and the `-Dtrace.profiler.output=profile.json` option enables machine-readable export.

### Integration with the governance pipeline

The Ouroboros Substrate's observability graft "ingests industry-standard telemetry into the same governance-score pipeline". For LexLean, the telemetry pipeline would feed into three governance signals:

**Signal 1: Proof-health score.** A composite metric that combines the skeleton registry's `proven` fraction with the telemetry's success rate and median proof time. A declining proof-health score triggers a CI warning; a score below a threshold triggers a CI failure.

**Signal 2: Axiom-drift alarm.** If the telemetry layer detects that a theorem's axiom profile has changed (e.g., a previously axiom-free theorem now depends on `native_decide`), it emits an alarm. The per-front axiom policy is "machine-enforced, not documentary". The telemetry layer makes the enforcement *continuous*: every commit, not just every release, is checked against the policy.

**Signal 3: Failure-mode report.** A weekly (or per-commit) report that aggregates failure modes across all proof attempts. The report identifies the top five failure patterns (e.g., "missing lemma about order theory", "tactic timeout on large derivation trees") and their frequency. This is the input to the proof-repair agent (from the tooling layer): the agent prioritizes repairs by the frequency of the failure mode.

---

## 2. Runtime Monitors for Extracted Executable Code

### The gap between static proof and runtime behavior

Lean's guarantees are "static and specification-relative… but they do not by themselves detect emergent runtime faults outside the model (e.g., I/O failures, environment misconfiguration). Such behaviors must either be modeled and proved, or mitigated with production safeguards (monitoring, sandboxing, input validation)". For LexLean, this gap is concrete: a derivation checker may be proven correct for all well-formed derivation trees, but if the extracted executable code receives a malformed certificate (a truncated JSON payload, a tree with a cycle, a hash that does not match the content), the runtime behavior is undefined by the proof.

The runtime monitor layer fills this gap. It is a **production safeguard** that checks legal invariants at runtime, independent of the static proof. The Lean-Agent Protocol demonstrates the architectural pattern: "Synchronous runtime interception" where "the `lean-worker` service runs the Lean 4 type-checker against pre-compiled `.olean` binaries. Verification is synchronous, binary (proved/refuted), and produces a formal proof trace on every decision". For LexLean, the runtime monitor would be a **thin layer** around the extracted executable code that checks a small set of critical invariants on every call.

### The invariant set

Which legal invariants should be monitored at runtime? The answer depends on the extracted code's trust boundary. If the derivation checker is extracted to WASM for browser use (from the integration layer), the runtime monitor must check the invariants that the WASM boundary could violate:

**Invariant 1: Certificate well-formedness.** The deserialized certificate must be a well-formed `DerivationTree`: no cycles, no missing children, all node labels in the registered rule set. This is a **structural invariant** that the static proof assumes but the runtime environment cannot guarantee.

**Invariant 2: Citation freshness.** Every legal citation in the certificate must reference a provision whose `text_hash` matches the current hash of the provision. This is a **temporal invariant** that the static proof cannot capture (because the legal source changes over time).

**Invariant 3: Axiom compliance.** The certificate's proof term must have an axiom profile that is a subset of the permitted set. This is a **policy invariant** that the runtime monitor checks before accepting the certificate.

**Invariant 4: Resource bounds.** The certificate's derivation depth and the number of inference steps must not exceed pre-configured bounds (e.g., max depth 1000, max steps 10,000). This is a **safety invariant** that prevents denial-of-service attacks on the runtime checker.

### The monitor as a Lean-checkable predicate

The most rigorous approach is to express the runtime monitors as **Lean-checkable predicates** that are compiled to executable code and embedded in the extracted artifact. This is the pattern demonstrated by the hardware monitor in SC-NeuroCore: "Formal verification of the six safety invariants that SC-NeuroCore's hardware monitor (`safety_monitor.sv`) enforces at runtime. Twenty-one invariants are proved directly in Lean 4 pure core… the SystemVerilog monitor is the runtime enforcer". The Lean proof establishes that the monitor's logic is correct; the extracted code (SystemVerilog, or in LexLean's case, WASM) is the runtime enforcer.

For LexLean, this means:

```lean
-- The runtime monitor predicate
def MonitorInvariant (cert : Certificate) (ctx : Context) : Bool :=
  wellFormed cert &&
  citationsFresh cert ctx &&
  axiomCompliant cert &&
  resourceBounded cert

-- The proof that the monitor accepts only valid certificates
theorem monitor_sound (cert : Certificate) (ctx : Context) :
    MonitorInvariant cert ctx = true → ValidCertificate cert ctx := ...
```

The monitor is **kernel-checked to be sound**: if it returns `true`, the certificate is valid. But the monitor is also **executable**: it runs in the extracted code and catches invalid certificates before they reach the kernel. This is the dual-assurance pattern from the verification pipeline layer: the proof guarantees soundness, and the monitor guarantees enforcement.

### The monitor's role in the provenance chain

Every runtime monitor decision (accept or reject) emits a **provenance chain entry**: a hash of the certificate, the monitor's verdict, the monitor's version, and the timestamp. The `eigenius` pattern treats "the proof term's content hash, combined with the environment hash and the FFI hash" as the cache key for verification results. For LexLean, the runtime monitor's entry would be:

```json
{
  "artifact_hash": "sha256:...",
  "monitor_version": "LexLean.Monitor.v1",
  "verdict": "accepted",
  "invariants_checked": ["wellFormed", "citationsFresh", "axiomCompliant", "resourceBounded"],
  "failed_invariant": null,
  "timestamp": "2026-10-03T14:32:00Z"
}
```

This entry is appended to the provenance chain, making the runtime monitor's decision **auditable** independently of the monitor itself. A regulator can inspect the chain and see that a specific certificate was checked against a specific monitor version at a specific time.

---

## 3. Security Audit Module

### The axiom audit as a security primitive

The security audit module is the **enforcement mechanism** for LexLean's governance policy. The `unsorry` project's `AxiomAudit/Main.lean` provides the concrete template: it defines "Axioms every mathlib-based proof may rely on. Anything else fails the audit" and uses "Read-only environment monad sufficient for `Lean.collectAxioms`" to run the audit. The ADR-006 describes the layered gate: "the authoritative check is an axiom-footprint audit… The whitelist {propext, Classical.choice, Quot.sound}… The audit reports every declaration's footprint regardless of pass/fail, as a CI artifact and PR comment, so reliance on classical choice (for example) is visible per proof, not just policy-permitted".

For LexLean, the security audit module would be a **standalone Lean executable** (or a script that invokes Lean) that performs four checks:

**Check 1: Per-declaration axiom footprint.** For every public declaration in the core modules, run `#print axioms` and record the transitive axiom closure. The audit reports *every* declaration's footprint, not just the failures, as a CI artifact. This is the "machine-enforced, not documentary" axiom policy.

**Check 2: Trust-policy compliance.** Compare each declaration's axiom footprint against the declared trust policy. The policy has two levels: the **global policy** (permitted axioms: `propext`, `Classical.choice`, `Quot.sound`) and the **module-level policy** (core modules must have zero `sorry`; skeleton modules may have `sorry` but cannot be imported by core modules). The audit fails if any declaration violates the policy.

**Check 3: Sorry detection.** Detect `sorryAx` in the axiom footprint, which indicates the presence of a `sorry`. The audit distinguishes between `sorry` in skeleton modules (permitted, tracked in the registry) and `sorry` in core modules (forbidden). The Ripple playbook notes that "textual grep is only a coarse upper bound; the authoritative signal is `#print axioms`".

**Check 4: Drift detection.** Compare the current axiom footprint against the **previous audit's footprint**. If a declaration's axiom profile has changed (e.g., it now depends on a new axiom), the audit flags the change as drift. This is the "automated drift guard" mentioned in the UOR Foundation's development notes: "the model-code correspondence itself is established by audit plus an automated drift guard (Section 14), not by machine proof".

### The audit module's architecture

The audit module would have three components:

**Component 1: The collection harness.** A Lean file that imports all public modules and runs `Lean.collectAxioms` on each declaration. The `collectAxioms` function is the kernel-level API for extracting the axiom dependencies of a declaration. The harness emits a JSON artifact:

```json
{
  "declaration": "LexLean.Contract.duress_voidable",
  "module": "LexLean.Contract",
  "axioms": ["propext", "Classical.choice", "Quot.sound"],
  "sorry_count": 0,
  "policy_compliant": true,
  "previous_axioms": ["propext", "Classical.choice", "Quot.sound"],
  "drift": false
}
```

**Component 2: The policy checker.** A Python or Lean script that reads the JSON artifact and compares it against the policy specification. The policy specification is itself a Lean file (from the governance layer): `permittedAxioms`, `coreModuleRegistry`, `skeletonModuleRegistry`. The checker asserts that every declaration's axiom set is a subset of `permittedAxioms` and that no core-module declaration imports a skeleton module.

**Component 3: The receipt emitter.** For every audit run, the module emits a **receipt** that is appended to the provenance chain. The receipt includes the commit hash, the toolchain version, the audit artifact hash, and the verdict. The `uuidna` project's "single order-invariant receipt" is the model: the receipt is a compact, content-addressed summary that can be independently verified.

### The `AuditP90.lean` pattern

The search results reference a `P90` certificate that "satisfies the operational criteria of Level 5B as a certificate post-cutoff" and "carries no Mathlib bridge layer". The `AuditP90.lean` module would be a **security audit specialization** for a specific trust level: Level 5B, which requires zero custom axioms and zero `sorry` in the core, but permits `native_decide` in designated computation-only modules.

For LexLean, the audit module would have **trust-level specializations**:

| Trust level | Permitted axioms | `sorry` permitted | `native_decide` permitted |
|---|---|---|---|
| **Core (Level 5A)** | `propext`, `Classical.choice`, `Quot.sound` | No | No |
| **Computation (Level 5B)** | `propext`, `Classical.choice`, `Quot.sound` | No | Yes, in designated modules |
| **Skeleton (Level 4)** | `propext`, `Classical.choice`, `Quot.sound`, `sorryAx` | Yes | Yes |
| **Experimental (Level 3)** | Any | Yes | Yes |

The audit module reads the trust level from the module's docstring (or a separate trust registry) and applies the corresponding policy. The per-front axiom policy from the ZLT registry demonstrates the concrete mechanism: a table that maps each front (module group) to its allowed axioms and its `native_decide` permissions.

### Integration with the provenance chain

The security audit module's output — the JSON artifact and the receipt — is **content-addressed** and **appended to the provenance chain**. This makes the audit independently verifiable: a downstream consumer can download the audit artifact, re-run the audit against the pinned toolchain, and confirm that the receipt matches. The `EG-VAR` library demonstrates the concrete workflow: "Any third party holding this bundle and a Lean 4 toolchain can re-run the audit by cloning the EG-VAR library at the pinned commit, running `lake build`, then running `python -m runtime.verify_axioms <claim_module>.lean` (which invokes the kernel)".

For LexLean, the replayable audit command would be:

```bash
lake env lean --run LexLean/Audit/Replay.lean --module LexLean.Contract --level core
```

The command outputs the axiom footprint, the policy verdict, and a hash of the receipt. The hash is compared against the provenance chain entry, and the audit is accepted if the hashes match.

---

## Summary: The Observability and Security Layer as a Live Trust Signal

| Component | What it observes | How it feeds governance |
|---|---|---|
| **Proof-search telemetry** | Tactic timing, success/failure patterns, axiom drift | Proof-health score; failure-mode reports; prover guidance |
| **Runtime monitors** | Certificate well-formedness, citation freshness, axiom compliance, resource bounds | Provenance chain entries; independent auditability |
| **Security audit module** | Per-declaration axiom footprint, policy compliance, drift | CI artifact; provenance receipt; replayable audit command |

The observability and security layer is not a monitoring afterthought. It is the **live trust signal** of the formalization. The Ouroboros Substrate's observability graft integrates telemetry into the governance-score pipeline. LexLean's observability layer should do the same: every tactic timing, every monitor decision, and every axiom audit is a **measurable input** to the project's governance. Without it, the formalization is a black box: it either compiles or it does not. With it, the formalization is a **living system** whose performance, security, and trust posture are continuously visible — and continuously auditable.