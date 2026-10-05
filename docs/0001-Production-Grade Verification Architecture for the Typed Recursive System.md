# ADR-PM-AGENT-021: Production-Grade Verification Architecture for the Typed Recursive System

## Status

**Proposed** — filed at `docs/adr/proposed/ADR-PM-AGENT-021-lean4-kani-verification.md`

Depends on: ADR-005 (Callable Meta-Check Protocol), the corrected Theorem 3.2 and Theorem 3.4 from the Revision 5.0 review cycle, and the four-way architectural split (arithmetic reference / runtime operator / stability certificate / authorization gate).

Supersedes: nothing. This ADR is a new decision, not a revision. The `PM-AGENT-005` collision noted in the packaging thread is resolved by renumbering to `021`; the sub-letter convention (`009A`/`009B`) splits a single decision into parts and is not applicable here.

---

## Context

The typed recursive system has reached the point where its mathematical core is small enough to specify and its implementation is small enough to verify. The corrected formulations are:

**Theorem 3.2 (Residual drift, corrected).** The original statement assumed the violating subspace was invariant under \(T\), which fails in general. The corrected form:
\[
\mathcal{H}=\mathcal{H}_c\oplus\mathcal{H}_v,\qquad T\mathcal{H}_v\subseteq\mathcal{H}_v,
\]
and
\[
\|T^t v\|\ge e^{\mu t}\|v\|\quad\forall v\in\mathcal{H}_v,\ t\ge0,\ \mu>0.
\]
Then \(P_vT^tv=T^tv\) and the exponential lower bound holds on the violating component.

**Theorem 3.4 (Damped proximal contraction, corrected).** The original statement assumed \(\sup_t\|M_t\|_{\mathrm{op}}<1\) implied contraction. It does not, without a uniform relaxation condition \(0<\alpha_{\min}\le\alpha_\pi\le\alpha_{\max}<1\). The printable certificate is:
\[
q = 1-\alpha_{\min} + \alpha_{\max}M < 1,\qquad M=\sup_t\|M_t\|_{\mathrm{op}}.
\]
This is sufficient, not necessary. A failed certificate is not proof of instability; it is absence of a proof of stability.

**The architectural split.** The system separates four objects that earlier formulations conflated:

| Object | Type | Function |
|---|---|---|
| \((R,\mathcal{I}(R))\) | Dedekind arithmetic structure | canonical arithmetic reference (optional) |
| \(F_t\) | state evolution | runtime computation |
| \(J_t\), \(\rho(J_t)\) | gain matrix, scalar | stability certificate |
| authorization gate | predicate / state machine | admission decision |

Prime structure is an optional parameterization of the typed recursive system \(z_{t+1}=F_t(z_t;\xi_t)\) with scalar \(\xi_t\). It is not foundational. The stability and audit properties do not depend on it. The null hypothesis \(\xi_t=0\) must be a first-class case in any reference implementation.

**The verification gap.** The system currently has:
- A mathematical specification small enough to formalize.
- A Rust implementation with `decide`, `prismpm verify`, and the authorization gate.
- Kani harnesses for bounded model checking of individual kernels.
- No machine-checked link between the specification and the implementation.

This ADR decides how to close that gap.

---

## Decision

We adopt a **three-tier verification architecture** with explicit ownership at each tier.

### Tier 1 — Lean 4: mathematical specification and proofs

Lean 4 with Mathlib4 is the authoritative home for:
- The corrected Theorem 3.2 and Theorem 3.4 as machine-checked lemmas.
- The spectral-radius certificate \(q = 1-\alpha_{\min}+\alpha_{\max}M < 1\) as a definition and a theorem.
- The invariance condition \(T\mathcal{H}_v\subseteq\mathcal{H}_v\) as a hypothesis carried explicitly.
- The typed recursive system \(z_{t+1}=F_t(z_t;\xi_t)\) as a specification, with \(\xi_t\) scalar.

Lean proofs are checked by the Lean kernel. No AI prover output is trusted without kernel acceptance.

**Extraction pipeline.** Rust code is lifted into Lean 4 using **Charon → Aeneas** for functions Aeneas supports, and **Hax** for dynamic-dispatch shapes Aeneas does not yet support. Aeneas models functions as pure Lean functions wrapped in a `Result` monad (success, panic, non-termination). Memory safety is not re-proved; Rust's borrow checker provides it. The obligation is that the extracted Lean function matches the mathematical specification.

**Toolchain pinning.** Lean 4 and Mathlib4 versions are pinned in `lean-toolchain` and `lakefile.lean`. Toolchain drift across Charon, Aeneas, Hax, and Mathlib4 is a known engineering gap and a recurring source of breakage. The pin is part of the verification contract.

### Tier 2 — Kani: bounded model checking of Rust kernels

Kani proves bounded safety and functional properties of Rust kernels via `#[kani::proof]` harnesses. It operates at the MIR level and integrates with `cargo kani` identically to `cargo test`.

**Scope.** Kani owns:
- Panic-freedom of the authorization gate and the stability certificate computation.
- Integer-overflow absence in the \(\rho(J_t)\) and \(q\) computations.
- Unsafe-block soundness, if any unsafe code enters the system.
- Bounded functional correctness of individual kernels: "for all inputs within bounds, the kernel returns the expected value."

**What Kani does not own.** A green Kani run means "proved within stated harness, assumptions, and bounds" — never "whole system proved." Unbounded properties and whole-system invariants belong in Lean, not Kani.

**Production scale.** Kani runs in production CI on the Rust standard library with over 16,000 harnesses verified per code change. The harness count is not the concern; harness coverage and harness maintenance are.

### Tier 3 — Rust: runtime, audit, authorization

The Rust implementation owns:
- The execution of \(F_t\) with the chosen parameterization \(\xi_t\).
- Prime-Weighted Execution Hashing (PWEH) with quantized observables.
- The append-only ledger and rollback attestation.
- The authorization gate as an independent predicate.

Rust does not own the stability proof. It owns the computation that the proof is about.

### The interface between tiers

The verification architecture requires three explicit interfaces:

**Specification interface.** Each Lean specification of a Rust function names the function, the file, and the extraction tool (Aeneas or Hax). A `specs/` directory maps Lean module → Rust module. The map is a file, not a convention.

**Harness interface.** Each Kani harness names the property it proves and the Lean theorem it corresponds to (if any). A `#[kani::proof_for_contract(...)]` harness with a contract is preferred over a bare `#[kani::proof]`.

**Certificate interface.** The runtime prints \(q\), \(M\), \(\alpha_{\min}\), and \(\alpha_{\max}\) on every execution. If \(q\ge 1\), the certificate fails. The runtime reports the failure as a warning or a block depending on policy — the same distinction the no-evidence fix required: a failed certificate is not proof of instability, it is absence of a proof of stability.

### No-evidence discipline

The no-evidence rule from the earlier session applies at every tier:
- A Lean build that produces no theorems and exits 0 certifies nothing.
- A Kani run with zero harnesses executed certifies nothing.
- A runtime execution with no evidence supplied certifies nothing.

Each tier must distinguish "ran and passed" from "did not run." Lean CI must assert that the expected theorem count is non-zero. Kani CI must assert that the expected harness count is non-zero. The runtime must report `no_evidence: true` and degrade accordingly.

---

## Consequences

### Positive

**The specification becomes machine-checked.** The corrected Theorem 3.2 and Theorem 3.4, and the certificate \(q<1\), are no longer prose. They are Lean lemmas with proofs that the kernel accepts or rejects. A proof that the Lean kernel accepts is final; no reviewer needs to re-derive the algebra.

**The implementation-to-specification link is explicit.** Aeneas/Hax extraction produces Lean functions that are recognizably the same code as the Rust source. The proof obligation — "extracted function matches specification" — is a single theorem per function, not a free-floating assurance claim.

**Kani provides cheap bounded coverage where Lean is expensive.** Panic-freedom and overflow absence are proved per-kernel by Kani in CI, at production scale. These are exactly the properties Rust's type system does not cover and that fuzzing only samples.

**The null hypothesis is testable.** With \(\xi_t\) scalar and the specification in Lean, the question "does prime structure matter for stability?" becomes a theorem: does the stability certificate hold for \(\xi_t=0\)? If yes, the primes are decorative. If no, there is a result.

### Negative

**Toolchain maintenance is a real cost.** Lean 4, Mathlib4, Charon, Aeneas, Hax, and Kani must be version-pinned and kept in a working combination. Toolchain drift across these tools is documented friction. The pin is not optional, and upgrading it is a project, not a `cargo update`.

**Proof maintenance is a real cost.** A Lean proof that depends on a Rust function's extraction breaks when the function changes. The `specs/` map is the defence: the proof and the function are linked by name, so a rename or signature change is caught at build time, not at review time.

**AI provers are a temptation and a hazard.** The Rust-to-Lean pipeline has demonstrated that AI provers can close proof obligations, with every proof kernel-checked. The hazard is not unsoundness — kernel checking prevents that — it is proof debt: an AI-closed proof that no human understands is a proof that cannot be maintained. The policy: AI-closed proofs are accepted only if the proof term is readable or the obligation is discharged by `decide`/`native_decide` on a decidable proposition.

**Not every function is extractable.** Aeneas does not support all Rust patterns; dynamic dispatch requires Hax, and some shapes require neither. Functions that cannot be extracted are verified by Kani alone, with the limitation recorded. The `specs/` map must mark extraction failures explicitly, not silently.

### Risks

**Lean CI wall-clock.** A full Mathlib4 build is hours; a prebuilt cache reduces incremental builds to minutes. CI must use a cached Mathlib4. A cold build is not a per-PR cost.

**Kani boundedness.** Kani is a bounded model checker. A proof that holds for inputs of size \(\le N\) does not hold for size \(N+1\). The bound must be documented per harness, and the choice of bound justified against the input domain.

**The reference implementation could become the specification.** If the Lean specification is derived *from* the Rust code rather than the code being verified *against* the specification, the proof is circular. The discipline: write the specification first, then extract, then prove. The `specs/` map records which came first.

---

## Implementation Plan

### Phase 1 — Specification (weeks 1–4)

- Formalize the corrected Theorem 3.2 and Theorem 3.4 in Lean 4 with Mathlib4.
- Define \(F_t\), \(J_t\), \(\rho(J_t)\), and the certificate \(q\) in Lean.
- Write the Lean specification for `decide` and the authorization gate **before** extraction.
- Pin `lean-toolchain` and the Mathlib4 revision.

### Phase 2 — Extraction (weeks 5–8)

- Run Charon + Aeneas on the Rust kernels that support it; run Hax on the rest.
- For each extracted function, write the theorem `extracted_fn = specification_fn`.
- Record extraction failures in the `specs/` map with the reason and the Kani fallback.

### Phase 3 — Kani harnesses (weeks 9–12)

- Write `#[kani::proof_for_contract(...)]` harnesses for panic-freedom and overflow absence in the certificate computation.
- Write bounded functional harnesses for kernels that could not be extracted.
- Assert the harness count is non-zero in CI.

### Phase 4 — CI integration (weeks 13–16)

- Lean build with cached Mathlib4; assert theorem count is non-zero.
- `cargo kani` in CI with cached toolchain; assert harness count is non-zero.
- Runtime prints \(q\) on every execution; failed certificate is a warn or block per policy.
- No-evidence rule applied at all three tiers.

### Phase 5 — Reference implementation and null hypothesis (weeks 17–20)

- Generate \(F_t\) for a finite-dimensional system with and without prime parameterization.
- Verify the predicted decay when \(q<1\); deliberately test \(q\ge1\) and confirm rejection.
- Center the null hypothesis: does \(\xi_t=0\) achieve the same stability and auditability?
- Publish the result as the first falsifiable test of the architecture's core claim.

---

## Open Cells

**The authorization interface is undefined.** "Authorization \(\perp\) research parameters" is a claim, not a specification. The gate needs: what state it reads, what policy it evaluates, what it emits, and its failure modes. This is the same shape as the exit-code contract from the earlier session — small, named, testable, with the consumer explicitly recorded, including the possibility of no consumer.

**The PWEH layer must not certify nothing.** The audit hash \(S_{t+1}=H(S_t\|p_{i_t}\|Q(N_t)\|M_t)\) can still produce a pass when \(N_t\) is empty or the observable is trivially zero. The no-evidence rule applies here too.

**The extraction boundary is not yet known.** Which functions extract cleanly, which need Hax, and which need Kani-only is an empirical question. The `specs/` map is the record; the first extraction run is the answer.

---

## Compliance with Existing Conventions

- **Document Status is authoritative for what was decided; directory is authoritative for where it lives.** This ADR is `Proposed` and lives in `proposed/`.
- **External references are bare identifiers if the corpus is external by design.** No external references are load-bearing in this ADR; all dependencies are in-tree or in the standard toolchain.
- **No-evidence discipline.** Applied at all three tiers, as specified above.
- **The one-line wrong close is refused.** The `should_block` vs. `block_recommended` cell remains open. This ADR does not touch it.

---

## Decision

**Adopt the three-tier verification architecture: Lean 4 for mathematical specification and proofs, Kani for bounded model checking of Rust kernels, Rust for runtime execution and audit, with explicit interfaces between tiers and the no-evidence discipline applied at each.**

The specification is written first. Extraction and harnesses follow. The null hypothesis is centered. The certificate prints. A failed certificate is not proof of instability. The reference implementation is the first falsifiable test.

The retreat from prime foundationalism is named: prime structure is an optional parameterization of a typed recursive system. The stability and audit properties do not depend on it. What survives is a verifiable, auditable, fail-closed recursive system with a machine-checked core.

---

**Filed by:** research clerk. **Owner:** portal clerk (runtime interfaces), research clerk (Lean specification), K2 (vacant — no standing over the verification contract). **Metric:** zero unextracted kernels without a documented reason; zero Kani harnesses without a bound; zero Lean theorems with `sorry`; the certificate \(q\) printed on every execution. **Horizon:** 7 days specification draft; 30 days extraction run; 90 days reference implementation with null hypothesis result.