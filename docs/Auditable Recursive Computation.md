# Canonical Prime Basis Architecture — Revision 5.0 (Final)

## A Rigorous Three-Layer Framework for Auditable Recursive Computation

---

## 0. Status of statements

This document distinguishes:

1. **Definitions** — notation and terminology.
2. **Theorems** — mathematical consequences with proofs or standard citations.
3. **Policy axioms and design hypotheses** — normative choices, engineering targets, and unverified empirical claims.

No ethical conclusion is derived from number theory. No empirical performance claim is presented as established. Prime canonicality is scoped to arithmetic domains. All thresholds are policy parameters.

**Change log from the provided evaluation document:**

- Corrected “non-associative” to **non-commutative** throughout.
- Removed adelic \(L^2(\mathbb{A}_{\mathbb{Q}})\) from Layer A. The adele ring is not a UFD and has zero divisors; it is unsuitable as a primary reference ring.
- Renamed “Meta-Prime Matrix” and “Pi-atom” to **Pi-Kernel Ambient Space** and **joint projector atom**.
- Clarified tensor-product MUB construction: it is **not** a true MUB set.
- Downgraded all hardware benchmarks to **unverified hypotheses**.
- Labeled density-gated cross-talk bound as **conjectural**.
- Replaced “Audit Functor” with **Certified Bridge Map** unless categories are defined.
- Scoped Zariski cohomology as trivial; étale requires explicit site and sheaf.
- Added quantization requirement for PWEH observables.
- Added rollback attestation requirement.
- Separated static and dynamic lawfulness criteria.
- Introduced **Arithmetic State Space** as a Dedekind domain or its ideals, not a free \(\mathbb{Z}\)-module.

---

## 1. Mathematical foundations

### 1.1 Hilbert space layer

Let \((\mathcal{H},\langle\cdot,\cdot\rangle)\) be a separable complex Hilbert space with norm \(\|\cdot\|\). Let \(\mathcal{B}(\mathcal{H})\) denote bounded operators. An evolution satisfies
\[
U(t,t)=I,\qquad U(t,r)U(r,s)=U(t,s).
\]

A **certified atomic family** \(\mathcal{A}=\{e_i\}_{i\in I}\) is orthonormal. Its projection is
\[
P_{\mathcal{A}}=\sum_{i\in I}e_i e_i^*,
\]
and the residual is
\[
R_{\mathcal{A}}(S)=(I-P_{\mathcal{A}})S.
\]
A state is \(\mathcal{A}\)-representable if \(R_{\mathcal{A}}(S)=0\), and \(\varepsilon\)-representable if \(\|R_{\mathcal{A}}(S)\|^2\le\varepsilon\).

### 1.2 Arithmetic reference layer

Let \(R\) be a **Dedekind domain** with fraction field \(K\). Primary cases:

- \(R=\mathbb{Z}\), \(K=\mathbb{Q}\).
- \(R=\mathcal{O}_K\), the ring of integers of a number field \(K\).

If \(R\) is a UFD, elements factor uniquely into irreducibles. If \(R\) is a general Dedekind domain, **nonzero fractional ideals** factor uniquely into prime ideals:
\[
\mathfrak{a}=\mathfrak{p}_1^{e_1}\cdots\mathfrak{p}_k^{e_k}.
\]

**Layer A is defined exclusively by the pair \((R,\mathcal{I}(R))\)**, where \(\mathcal{I}(R)\) is the monoid of nonzero fractional ideals under multiplication. Adelic \(L^2\) spaces are **not** part of Layer A. Adelic harmonic analysis, if used, belongs to an auxiliary analytic layer for spectral calculations.

**Audit map.** For \(a\in R\setminus\{0\}\),
\[
\alpha(a)=\sum_{\mathfrak{p}}v_{\mathfrak{p}}(a)\,e_{\mathfrak{p}},
\]
where \(v_{\mathfrak{p}}\) is the valuation at the prime ideal \(\mathfrak{p}\). This is multiplicative and injective on \(R^\times\)-classes.

---

## 2. Three-layer architecture

### 2.1 Layer A — Arithmetic Reference Space

**Definition 2.1.1.** Layer A is \((R,\mathcal{I}(R))\). The arithmetic reference invariant of an ideal \(\mathfrak{a}\) is its prime ideal factorization.

**Definition 2.1.2.** The **audit map** is
\[
\alpha:R\setminus\{0\}\to\bigoplus_{\mathfrak{p}}\mathbb{Z}\,e_{\mathfrak{p}},\qquad
\alpha(a)=\sum_{\mathfrak{p}}v_{\mathfrak{p}}(a)\,e_{\mathfrak{p}}.
\]

**Arithmetic invariants.** Unique ideal factorization, CRT isomorphisms, and Galois stability hold as exact algebraic properties.

**Failure mode.** A state with no well-defined ideal factorization, or with an unregistered ideal class, is structurally inadmissible.

### 2.2 Layer B — Hardware Atomic Coordinates

**Definition 2.2.1.** A hardware atomic family is
\[
\mathcal{H}_{\mathrm{hw}}=\{h_a\}_{a\in A}\subset\mathcal{H}
\]
with projectors \(\{P_a\}_{a\in A}\), recomposition \(\mathrm{rec}_{\mathrm{hw}}:\ell^2(A)\to\mathcal{H}\), and decomposition \(\mathrm{dec}_{\mathrm{hw}}:\mathcal{H}\to\ell^2(A)\).

**Definition 2.2.2 (Static lawfulness).** \(\mathcal{H}_{\mathrm{hw}}\) is statically lawful if:

1. **Atomicity.** Each \(h_a\) is irreducible or spans a minimal invariant subspace.
2. **Orthogonality or frame bounds.** Either \(\langle h_a,h_b\rangle=\delta_{ab}\), or there exist \(0<A\le B<\infty\) with
   \[
   A\|x\|^2\le\sum_a\|P_a x\|^2\le B\|x\|^2.
   \]
3. **Lossless recomposition.** \(\mathrm{rec}_{\mathrm{hw}}\circ\mathrm{dec}_{\mathrm{hw}}=I\) on the certified subspace, or \(\|I-\mathrm{rec}_{\mathrm{hw}}\circ\mathrm{dec}_{\mathrm{hw}}\|\le\delta_{\mathrm{rec}}\).
4. **Certified bridge.** There exists an auditable bridge map \(U\) (see Section 2.3).
5. **Audit registration.** The family definition, \(U\), frame bounds, and certificates are committed to an append-only hash-chained log.

**Definition 2.2.3 (Dynamic lawfulness).** An evolution \(F:\mathcal{H}\to\mathcal{H}\) is dynamically lawful if:

6. **Residual boundedness.** \(\|(I-\mathrm{rec}_{\mathrm{hw}}\circ\mathrm{dec}_{\mathrm{hw}})F^t(x)\|^2\le\varepsilon\) on the certified window.
7. **Contraction on the violation cone.** \(F\) is nonexpansive on \(\mathcal{H}\) and strictly contractive on the complement of the certified subspace.
8. **Deterministic trace replay.** The evolution is deterministically replayable from the hash-chained log.

**Remark 2.2.4.** Static criteria certify the basis and bridge. Dynamic criteria certify the evolution.

### 2.3 Layer C — Certified Bridge and Audit Layer

**Definition 2.3.1 (Bridge bifurcation).** The bridge map splits into two distinct operators:

- **Arithmetic bridge:**
  \[
  U_{\mathrm{arith}}:\mathcal{H}_{\mathrm{arith}}\to(R,\mathcal{I}(R)),
  \]
  defined strictly on arithmetic subcategories. It extracts valuations and ideal classes.

- **Labeling bridge:**
  \[
  U_{\mathrm{label}}:\mathcal{H}_{\mathrm{hw}}\to\mathcal{P},
  \]
  assigning prime indices to orthonormal basis vectors as an auditable coordinate naming convention. This is not an arithmetic invariant.

**Definition 2.3.2 (Extraction map).** For arithmetic states,
\[
\mathrm{Ext}:\mathcal{H}_{\mathrm{arith}}\to R,
\]
defined only on the arithmetic subcategory. For continuous states where \(\mathrm{Ext}\) is undefined, audit verification defaults to Prime-Weighted Execution Hashing (PWEH) and cryptographic trace logs.

**Definition 2.3.3.** The **certified bridge** is a pair \((U,\mathcal{V})\), where \(U\) is either \(U_{\mathrm{arith}}\) or \(U_{\mathrm{label}}\), and \(\mathcal{V}\) verifies the certificates in Definitions 2.2.2 and 2.2.3.

**Remark 2.3.4.** Layer C is a **certified bridge map**, not a functor, unless explicit categories and mapping rules are defined.

---

## 3. Core theorems

### Theorem 3.1 (Basis independence)

Any two complete orthonormal families in a separable Hilbert space are related by a unitary \(U\) with \(U^*U=UU^*=I\).

### Theorem 3.2 (Residual drift under expansive violation)

Let \(\mathcal{H}=\mathcal{H}_{\mathrm{cert}}\oplus\mathcal{H}_{\mathrm{viol}}\). If \(T\) satisfies
\[
\|T^t x\|\ge e^{\mu_{\min}t}\|x\|\quad\forall x\in\mathcal{H}_{\mathrm{viol}},\ t\ge0,
\]
with \(\mu_{\min}>0\), then
\[
\|P_{\mathrm{viol}}x(t)\|\ge e^{\mu_{\min}t}\|P_{\mathrm{viol}}x(0)\|.
\]

### Theorem 3.3 (Product basis completeness)

Let \(\{P_{i,\pi_i}\}_{\pi_i}\) be commuting, complete, orthogonal families on tensor factors \(\mathcal{H}_i\). Then \(R_\pi=\bigotimes_i P_{i,\pi_i}\) satisfy
\[
\sum_\pi R_\pi=I,\qquad R_\pi R_{\pi'}=\delta_{\pi\pi'}R_\pi.
\]

### Theorem 3.4 (Damped proximal contraction)

Let \(\mathrm{Prox}_\pi\) be nonexpansive and let \(\sup_t\|M_t\|_{\mathrm{op}}<1\). Then
\[
c_\pi(t+1)=(1-\alpha_\pi)c_\pi(t)+\alpha_\pi\,\mathrm{Prox}_\pi(u_\pi(t))
\]
is a strict contraction with a unique fixed point \(c^*\) and exponential convergence.

---

## 4. Pi-Kernel Ambient Space

### 4.1 Definition

The **Pi-Kernel Ambient Space** is
\[
\mathcal{H}=\mathcal{H}_{\mathrm{RNS}}\otimes\mathcal{H}_{\mathrm{sym}}\otimes\mathcal{H}_{\mathrm{spec}}\otimes\mathcal{H}_{\mathrm{wav}}\otimes\mathcal{H}_{\mathrm{alg}}\otimes\mathcal{H}_{\mathrm{qudit}},
\]
with joint projectors
\[
R_\pi=E_a\otimes P_\rho\otimes\Pi_\Omega\otimes W_{j,k}\otimes Z_\beta\otimes B_\mu.
\]

Only the RNS and qudit factors have intrinsic prime structure. The correct name is **Pi-Kernel Ambient Space** or **Composite Tensor-Product Architecture**.

### 4.2 Commutativity requirement

For joint projectors to be complete and orthogonal, factor families must commute. If hardware operations do not commute, one must restrict to a commuting subalgebra, define a non-commutative product basis with explicit ordering data, or quotient by the non-commutative ideal and track ordering in the audit layer.

### 4.3 Damped proximal updates

State coefficients evolve by
\[
c_\pi(t+1)=(1-\alpha_\pi)c_\pi(t)+\alpha_\pi\,\mathrm{Prox}_\pi(u_\pi(t)),
\]
with \(\alpha_\pi\in(0,1)\) and \(\mathrm{Prox}_\pi\) nonexpansive.

**Stability monitors:**

- **SlopeUB:** infinity-norm Lipschitz envelope.
- **GapLB:** \(1-\mathrm{SlopeUB}\).

If \(\mathrm{GapLB}>0\), the global update is a strict Banach contraction.

### 4.4 Conjectural cross-talk bound

A **conjectural cross-talk stability bound** is proposed:
\[
\varepsilon_{\mathrm{cross}}<\frac{1-\rho}{\sum_\pi w_\pi}.
\]
If validated via spectral norm bounds, it would establish Banach fixed-point convergence across coupled channels. Until then, it is a hypothesis, not a theorem.

---

## 5. Cryptographic governance

### 5.1 Prime-Weighted Execution Hashing (PWEH)

At step \(t\),
\[
S_{\mathrm{int}}(t)=\mathrm{Hash}_{\mathrm{PQC}}\!\left(S_{\mathrm{int}}(t-1)\,\|\,p_{i_t}\,\|\,\widehat{N}_{p_{i_t}}(t)\,\|\,M(t)\right),
\]
where \(p_{i_t}\) is the active prime index, \(\widehat{N}_{p}(t)\) is a quantized multiplicity-weighted norm observable, \(M(t)\) is governance metadata, and \(\mathrm{Hash}_{\mathrm{PQC}}\) is a post-quantum hash.

**Security basis.** Operator composition is **associative but non-commutative**:
\[
(AB)C=A(BC),\qquad AB\ne BA\ \text{in general}.
\]
Order sensitivity derives from non-commutativity.

### 5.2 Bounded saturation and Lipschitz-tied quantization

Continuous observables must be quantized before hashing. Define
\[
Q(x)=\min\!\left(M_{\max},\ \left\lfloor \Delta_q^{-1}x+\tfrac12\right\rfloor \Delta_q\right),
\]
where \(M_{\max}>0\) is a saturation ceiling and \(\Delta_q>0\) is a quantization step.

The step \(\Delta_q\) is chosen relative to the system Lipschitz margin \(L_T\) so that for any valid execution, if \(\|x-x'\|<\delta\), then \(Q(x)=Q(x')\). This prevents small continuous perturbations from altering discrete hash-chain outputs during valid executions.

### 5.3 Execution lock and rollback

Rejection triggers fail-closed halt and rollback to the last certified snapshot. Snapshots must be cryptographically attested. On halt, the runtime emits a signed rollback receipt containing the fault hash, active policy manifold \(M(t)\), state residual \(R_{\mathcal{A}}(S)\), and target rollback checkpoint ID.

---

## 6. Sheaf cohomology: correct scoping

### 6.1 Zariski site

For a quasi-coherent sheaf \(\mathcal{M}\) on \(\operatorname{Spec} R\) with \(R\) a Dedekind domain, Serre’s theorem gives
\[
H^i(\operatorname{Spec} R,\mathcal{M})=0\quad\forall i>0.
\]
Vanishing checks on the Zariski site are automatic and provide no information.

### 6.2 Étale site

For non-trivial checks, specify:

- the site (étale, fppf, etc.),
- the sheaf \(\mathcal{M}\),
- the degree range,
- and comparison theorems.

A meaningful statement is
\[
H^i_{\text{ét}}(\operatorname{Spec} R,\mathcal{M})=0\quad\text{for }1\le i\le N,
\]
with \(\mathcal{M}\) explicitly defined.

---

## 7. Physical realization and empirical status

### 7.1 Qudit realization

For a \(d\)-level qudit with \(d=p_1^{a_1}\cdots p_k^{a_k}\), tensor-product basis constructions use
\[
\text{Basis}(d)=\bigotimes_i\text{Basis}(p_i^{a_i}).
\]
If each factor is a complete set of MUBs, the tensor product yields a **structured tensor-product frame**, not a complete MUB set in dimension \(d\).

**Terminology correction.** \(\mathrm{MUB}(2)\otimes\mathrm{MUB}(5)\) yields \(18\) orthonormal bases in \(d=10\). Pairwise overlaps are \(1/\sqrt{2}\), \(1/\sqrt{5}\), or \(1/\sqrt{10}\), not uniformly \(1/\sqrt{10}\). These are **not** mutually unbiased bases in \(d=10\). They are a CRT-aligned tensor-product frame.

### 7.2 Physical benchmarks

All hardware figures — Strontium-87 Raman scattering rates, coherence times, forgetting rates, quantum noise reductions, semantic drift rates, compliance overhead reductions — are **unverified design hypotheses**. They require:

- controlled experiments on standard datasets,
- reproducible protocols with open code and seeds,
- statistical significance tests,
- peer review.

### 7.3 Density-gated cross-talk bound (conjectural)

The bound
\[
\varepsilon_{\mathrm{cross}}<\frac{1-\rho}{\sum_\pi w_\pi}
\]
is a small-gain condition. A rigorous derivation would proceed from the Lindblad master equation, linearize around the target state, bound cross-channel coupling in terms of \(\gamma_{ij}\), and apply the small-gain theorem. Until then, it is a **conjecture**.

---

## 8. Governance and threshold setting

### 8.1 Policy parameters

Thresholds \(\varepsilon\), \(\tau_6\), \(\tau_{24}\), \(\theta_{\max}\), \(\Lambda_H\), \(\Lambda_\alpha\), \(\beta_{\max}\), \(\Delta_q\), \(M_{\max}\), and \(\Delta_{\mathrm{drift}}\) are **policy parameters**. They are set by an accountable process with documented rationale, stability margin analysis, audit cost analysis, and domain-specific risk assessment.

### 8.2 Separation of powers

- **Technical Registrar:** verifies mathematical certificates.
- **Ethical Tribunal:** interprets policy axioms, sets thresholds, hears appeals.
- **Independent Auditor:** checks hash chains, deterministic replay, separation of powers.

### 8.3 Ethical principles as policy axioms

Neutrality, recursive truth, universal beneficence, silence clause, and wisdom of the unplugged are **normative axioms**, not mathematical consequences. They are enforced by governance.

---

## 9. Ten Xi-Critiques (revised)

| ID | Requirement | Criterion | Tooling |
|----|-------------|-----------|---------|
| Ξ₀ | Decomposability | \(\|R\|^2\le\varepsilon\) | Decomposition engine |
| Ξ₁ | Contractivity | Spectral radius \(<1\) on violation cone | Spectral analysis |
| Ξ₂ | Entropy budget | \(\Delta H\le\Lambda_H\) | Information accounting |
| Ξ₃ | Symmetry invariance | Invariance under declared \(G\) | Representation witnesses |
| Ξ₄ | Proof obligations | \(\{P\}U\{Q\}\) discharged | Weakest precondition |
| Ξ₅ | Abstract interpretation | Sound inductive invariants | Lattice fixpoint solvers |
| Ξ₆ | Temporal compliance | LTL/CTL specs satisfied | Model checking |
| Ξ₇ | Runtime enforceability | Security automata constructed | Monitor synthesis |
| Ξ₈ | Cryptographic integrity | Valid signature and Merkle inclusion | Hash-chain verifiers |
| Ξ₉ | Deterministic replay | Replay reproduces state exactly | Trace reproduction |

Failure of any critique blocks admission.

---

## 10. Constitutional articles (revised)

> **Article I — Arithmetic Reference.** Layer A is a Dedekind domain \(R\) and its monoid of nonzero fractional ideals \(\mathcal{I}(R)\). Prime ideal factorization is canonical here.
>
> **Article II — Runtime Autonomy.** Computation may execute in any statically and dynamically lawful hardware atomic family.
>
> **Article III — Certified Bridge.** A hardware family is lawful only if it carries either an arithmetic bridge \(U_{\mathrm{arith}}\) or a labeling bridge \(U_{\mathrm{label}}\), recorded in an append-only cryptographic ledger.
>
> **Article IV — Residual Bounding.** \(\|R\|^2\le\varepsilon\). Violations trigger PROJECT, FREEZE, or ROLLBACK.
>
> **Article V — Separation of Concerns.** The choice of \(\mathcal{H}_{\mathrm{hw}}\) is an engineering decision. The bridge \(U\) is the legal interface. The arithmetic reference \(R\) is the legal memory.
>
> **Article VI — Cryptographic Integrity.** Observables entering the hash chain must be quantized with bounded saturation and Lipschitz-tied step size. Snapshots must be cryptographically attested.
>
> **Article VII — Empirical Honesty.** Performance claims are hypotheses until validated by reproducible, peer-reviewed experiments.
>
> **Article VIII — Governance.** Thresholds and ethical axioms are policy, set by an accountable process with separation of powers.
>
> **Article IX — Terminology.** “Meta-Prime Matrix” is retired. The composite basis is the “Pi-Kernel Ambient Space” or “Composite Tensor-Product Architecture.”
>
> **Article X — Cohomology.** Vanishing checks on the Zariski site are automatic. Non-trivial checks require explicit site, sheaf, degree range, and comparison theorems.

---

## 11. Summary

> **Basis-agnostic at runtime, arithmetic-referential at audit (where arithmetic structure exists), policy-enforced at governance, empirically honest, and mathematically scoped.**

Revision 5.0 resolves the remaining imprecisions:

- Layer A is purely Dedekind domain and ideals.
- The bridge splits into arithmetic and labeling maps.
- Static and dynamic criteria are correctly separated.
- The cross-talk bound is explicitly conjectural.
- The extraction map \(\mathrm{Ext}\) is defined only on arithmetic states.
- PWEH quantization is bounded and tied to the Lipschitz margin.

This is a defensible foundation for further theoretical and engineering work.

# Analysis of Revision 5.0 (Final)

This document is a substantial and well-organized specification. It correctly incorporates nearly all of the corrections identified in the prior meta-evaluation. The epistemological rescoping is clear, the mathematical foundations are properly scoped, and the separation between static and dynamic lawfulness is largely correct. However, a few remaining issues need to be addressed before the document can be considered fully rigorous.

---

## 1. Corrections successfully incorporated

The document correctly implements:

- **Non-commutativity vs. non-associativity.** The terminology is corrected throughout.
- **Dedekind domain reference.** Layer A is defined exclusively as \((R, \mathcal{I}(R))\). Adelic \(L^2\) is excluded from Layer A and re-scoped to an auxiliary analytic layer.
- **Terminology.** “Meta-Prime Matrix” and “Pi-atom” are replaced by “Pi-Kernel Ambient Space” and “joint projector atom.”
- **Qudit basis qualification.** Tensor-product constructions are explicitly stated **not** to be true MUB sets in composite dimension.
- **Empirical benchmarks.** All performance claims are downgraded to unverified design hypotheses.
- **Cross-talk bound.** Labeled conjectural.
- **Categorical scoping.** “Audit Functor” is replaced by “Certified Bridge Map.”
- **Cohomological scoping.** Zariski cohomology is trivial; non-trivial checks require étale site and sheaf.
- **Observable quantization.** PWEH now mandates quantization.
- **Rollback attestation.** Signed rollback receipts are required.
- **Static vs. dynamic lawfulness.** The criteria are separated.
- **Failure modes.** Each layer has documented failure modes.

These changes resolve the major overclaims from earlier versions.

---

## 2. Remaining substantive issues

### 2.1 Quantization formula lacks saturation and Lipschitz-tied step size

The document defines
\[
Q(x(t)) = \left\lfloor \frac{x(t)}{\Delta_q} + \frac12 \right\rfloor \Delta_q,
\]
with \(\Delta_q\) a “fixed precision quantization step.” This is a round-to-nearest-multiple operation. Two problems remain:

1. **No saturation ceiling.** The norm \(\|A_p T(t)\|_{\mathrm{mult}}\) is unbounded in general. The floor operation requires a bounded range or an explicit saturation rule \(M_{\max}\).
2. **No Lipschitz-tied step size.** If \(\Delta_q\) is too small, arbitrarily small continuous perturbations can flip the quantized value and break deterministic replay. If \(\Delta_q\) is too large, the hash chain loses resolution and may fail to detect meaningful state changes.

In the prior analysis, the recommendation was:
\[
Q(x) = \min\!\left(M_{\max},\ \left\lfloor \Delta_q^{-1}x + \tfrac12\right\rfloor \Delta_q\right),
\]
with \(\Delta_q\) chosen relative to the system Lipschitz margin \(L_T\) so that for any valid execution, if \(\|x-x'\|<\delta\), then \(Q(x)=Q(x')\).

**Correction:** Add the saturation ceiling \(M_{\max}\) and specify how \(\Delta_q\) is tied to the Lipschitz margin.

### 2.2 “Residual Boundedness” is misplaced in the static criteria

The document lists static lawfulness criteria as:

1. Atomicity
2. Frame Bounds
3. Lossless Recomposition
4. Certified Bridge Map
5. **Residual Boundedness**
6. Audit Registration

Criterion 5 is stated as
\[
\|R_{\mathcal{A}}(S)\|^2 \le \varepsilon.
\]
This is a condition on a **state** \(S\), not on the hardware atomic family \(\mathcal{H}_{\mathrm{hw}}\). It is therefore a dynamic criterion, not a static one.

In the prior analysis, the recommendation was:

- **Static:** Atomicity, Frame/Orthogonality Bounds, Lossless Recomposition, Certified Bridge, Audit Registration.
- **Dynamic:** Residual Drift Boundedness, Contraction on Violation Cones, Deterministic Trace Replay.

**Correction:** Move “Residual Boundedness” to the dynamic criteria. The static criteria should describe the basis; the dynamic criteria should describe the evolution.

### 2.3 “Thirteen corrections” vs. ten-row table

The text states:

> “Revision 5.0 (Final) incorporates thirteen fundamental structural corrections and formal refinements from baseline specifications…”

The accompanying table contains ten rows. There is a mismatch. Either the number should be ten, or three additional corrections should be added to the table.

**Correction:** Reconcile the count. If there are thirteen, list them all. If there are ten, correct the number.

### 2.4 Speculative hardware sections

The sections on the Universal Atomic Calculator (87Sr) and the Linux kernel ABI are speculative engineering proposals. They are clearly labeled as mappings and integrations, but they are not part of the core mathematical framework. In a final specification, these would be better placed in an appendix or separate document, with the main text focusing on the three-layer architecture and its mathematical guarantees.

**Suggestion:** Move the hardware realization and OS integration sections to an appendix, or clearly mark them as “application notes” distinct from the normative specification.

---

## 3. Minor issues

### 3.1 Linearity of the evolution operator in Theorem 2

Theorem 2 (Residual Drift Exponentiation) refers to a “time-evolution operator \(T\)” and a “minimal Lyapunov exponent \(\mu_{\min}>0\).” For a general nonlinear operator, the Lyapunov exponent is not simply the log of the spectral radius. The theorem is standard for bounded linear operators. The document should specify that \(T\) is linear (or at least that the Lyapunov exponent is defined in the standard way for the class of operators considered).

### 3.2 The “GapLB” definition

The document defines
\[
\mathrm{SlopeUB} = \|A\|_\infty,\qquad A = \mathrm{diag}(1-\alpha) + \mathrm{diag}(\alpha)|K|,
\]
and
\[
\mathrm{GapLB} = 1 - \mathrm{SlopeUB}.
\]
This is a concrete and reasonable choice. However, the document should clarify that \(\|A\|_\infty<1\) is a **sufficient** condition for contraction, not a necessary one. The spectral radius could be less than one even when the infinity norm is greater than one. This is a minor precision issue.

### 3.3 “Audit map” injectivity modulo units

The document states that \(\alpha\) is “injective on \(R^*\)-classes (modulo units).” This is correct: \(\alpha(a)=\alpha(b)\) iff \(a\) and \(b\) generate the same principal ideal, i.e., differ by a unit. The phrasing could be tightened to “injective on principal ideals.”

---

## 4. What is now rigorous

The following parts of Revision 5.0 are mathematically sound:

- **Hilbert space mechanics.** Definitions of \(P_{\mathcal{A}}\), \(R_{\mathcal{A}}\), and \(\varepsilon\)-representability are correct.
- **Arithmetic reference.** Dedekind domains and unique factorization of ideals are the right algebraic setting.
- **Theorems 1–4.** Basis independence, residual drift, product basis completeness, and damped proximal contraction are correctly stated (with the caveat in 3.1 above).
- **Pi-Kernel Ambient Space.** The product basis with commuting projectors is coherent.
- **PWEH security basis.** Reliance on non-commutativity and post-quantum hashing is correct.
- **Cohomology scoping.** Zariski is trivial; étale requires explicit data.
- **Empirical honesty.** Performance claims are hypotheses.
- **Static/dynamic separation.** Largely correct, with the one placement issue in 2.2.

---

## 5. Summary of recommended final refinements

To produce a fully rigorous Revision 5.0 (Final):

1. **Add saturation ceiling \(M_{\max}\)** and **Lipschitz-tied \(\Delta_q\)** to the PWEH quantization formula.
2. **Move “Residual Boundedness”** from static to dynamic criteria.
3. **Reconcile the correction count** (thirteen vs. ten).
4. **Move speculative hardware sections** to an appendix or mark as application notes.
5. **Specify linearity** of \(T\) in Theorem 2.
6. **Clarify** that \(\|A\|_\infty<1\) is sufficient but not necessary for contraction.

With these changes, the document would be fully consistent and defensible.

---

# Analysis of the Word Deserializer Fix and Fifth Defect

This report is another strong iteration. It correctly fixes the deserializer, finds a fifth defect that the earlier review anticipated but did not enumerate, and is honest about a test that was initially worthless. It also clearly separates engineering work from governance decisions. Below is an assessment.

---

## 1. What is fixed and correct

### Word deserializer
- Replaced placeholder with a real parser for the four `Display` shapes.
- Added 6 unit tests and 2 property tests (256 generated words).
- Round‑trip coverage is now meaningful.
- **Assessment:** Correct. This is the minimum bar for a codec.

### Primality validation decision
The report deliberately does **not** validate primality in the parser, with the rationale:

> “pm‑core keeps `Atom(u64)` unvalidated so L0_07/L0_08 stay reachable, and rejecting here would make ill‑formed words un‑representable and push the check to the wrong layer.”

This is a defensible layering argument:

- If the parser rejects non‑prime atoms, then malformed inputs cannot be constructed, and tests for rejection logic cannot run.
- The check belongs in a validation layer, not in the codec.
- Keeping `Atom(u64)` unvalidated preserves reachability of the validation rules L0_07 and L0_08.

**Assessment:** Correct, with one caveat. A common alternative is to introduce two types:

- `RawAtom(u64)` — produced by the parser, no primality guarantee.
- `Atom(u64)` — constructed only via a validator that enforces primality.

This makes the invariant explicit in the type system. The current design leaves the invariant implicit, which is workable but relies on convention. If the codebase is small and disciplined, the current approach is fine. If it grows, consider the two‑type split.

### Fifth defect — NoOpRedactor
This is the most important finding in the report. The CLI read the real nonce from the environment, then discarded it and passed `NoOpRedactor`, which:

- MACs with a hardcoded literal.
- Reports `redaction_hits: 0` unconditionally.

**Impact:** Every report attested nothing about the operator’s secret while appearing to be MACed. It was security theater: the system produced the appearance of redaction and integrity without either.

**Fix:**
- `NonceRedactor` MACs with the configured nonce.
- Counts entropy hits rather than reporting zero.
- A missing nonce is now fatal (exit 64), not a warning.

**Assessment:** Correct and important. This is a class of bug worth naming: **attestation that attests nothing**. Any system that produces a MAC, signature, or hash should have a test that verifies the MAC changes when the secret changes, and fails when the secret is missing.

### Meta-lesson about the test
The report flags that the first version of the assertion was worthless because the test nonce was the same string as the `NoOp` literal, so it compared a value to itself and passed. It only became meaningful once the test nonce was distinct.

**Assessment:** This is exactly the kind of self‑critique that should appear in a status report. It also suggests a general rule:

> Any test that verifies a MAC, signature, or hash must use a secret distinct from any default or placeholder value. Otherwise the test may pass trivially.

### Exit code taxonomy
We now have:

- **Exit 2 = Block** (policy violation, from earlier report).
- **Exit 64 = missing nonce** (EX_USAGE, configuration error).

This is a reasonable taxonomy. It should be documented explicitly so that callers and operators know what each code means.

---

## 2. Feature flag decision — engage

The report says:

> “I did not gate dual‑representation behind a feature flag as the review suggested. The word codec is now correct and tested, so the flag would be guarding against a problem that no longer exists.”

Two responses:

1. **The decision is defensible.** If the codec is small, correct, and tested, a feature flag adds maintenance burden without proportional benefit. Flags are not free; they fragment test matrices and can hide stale code paths.
2. **The reasoning is slightly incomplete.** Feature flags are not only about guarding against past bugs. They are also about:
   - **Rollout safety.** If the codec causes a performance regression or exposes a corner case in production, a flag lets you disable it without a rollback.
   - **Rollback capability.** If downstream consumers depend on the previous behavior, a flag gives an operational escape hatch.

If the codec is a pure improvement with no behavioral ambiguity, the flag is unnecessary. If it changes observable behavior, consider whether an escape hatch is worth the cost. The report’s conclusion may be correct; the rationale should acknowledge the broader role of flags.

---

## 3. ADR‑009B — open items

The report correctly separates three open items in ADR‑009B §4:

### 3.1 k-anonymity release gate
- Aggregate still returns unsatisfied buckets with a flag.
- Quarantine needs a defined consumer.
- It is a denial of service on solo‑maintainer deployments that will never reach 10 orgs.
- **This is a governance call, not an engineering one.**

**Assessment:** Correct framing. The engineering is straightforward; the policy is not. Options:

- Make k‑anonymity a per‑deployment policy parameter. Default to off for solo‑maintainer deployments; require it for multi‑tenant or regulated deployments.
- Define the consumer of quarantined buckets explicitly. If no one consumes them, they are dead weight.
- Document the trade‑off: privacy vs. availability.

### 3.2 Schema semantic diff
- No registry exists.
- The earlier schema/report mismatch is exactly the bug this would have caught.
- A hash check would not have found it, because the pair differed in every field name.
- Current defence is a per‑crate test, which will not catch third‑party drift.

**Assessment:** Correct. A semantic diff requires:

- A registry of expected schema versions.
- A comparison of field names, types, and constraints, not just hashes.
- A policy for what constitutes a breaking change.

Without a registry, third‑party consumers can drift undetected. This is a real gap.

### 3.3 DynamoDB TTL index
- Requirements are met in memory only.
- The sweep is O(n) and needs the index before any real cadence.

**Assessment:** Correct. An in‑memory sweep is fine for tests but will not scale. The index should be designed before the sweep runs on a schedule.

---

## 4. Remaining risks

### 4.1 The NoOpRedactor pattern may exist elsewhere
The `NoOpRedactor` was not malicious; it was a placeholder that looked like a real implementation. Similar placeholders may exist elsewhere in the codebase. A search for `NoOp`, `Stub`, `Placeholder`, `Dummy`, and `Fake` is warranted.

### 4.2 Entropy‑based hit counting is not the same as redaction
`NonceRedactor` counts entropy hits. This is a heuristic, not a guarantee. High‑entropy strings may be false positives; low‑entropy secrets may be missed. The count should be understood as a diagnostic, not as proof of redaction.

### 4.3 Exit code 64 for missing nonce
Exit 64 is `EX_USAGE` on most Unix systems. That is reasonable for a configuration error. But the earlier report used exit 2 for policy blocks. Ensure the taxonomy is documented and stable, because scripts and operators will depend on it.

### 4.4 Property tests for the codec
256 generated words is a good start. Consider:

- Round‑trip properties for all four `Display` shapes.
- Malformed input properties (does the parser fail gracefully?).
- Size limits (does the parser reject oversized inputs?).

### 4.5 Deliberate non‑validation of primality
The rationale is sound, but it should be documented in the code as a comment or in an ADR. Otherwise a future maintainer may “fix” it by adding validation, breaking L0_07/L0_08.

---

## 5. Recommendations

1. **Document the exit code taxonomy.** Exit 2 = policy block; exit 64 = configuration error; exit 0 = success. Make it stable.

2. **Search for other NoOp/Stub placeholders.** The fifth defect suggests a pattern. Audit the codebase for similar placeholder implementations, especially in security‑critical paths.

3. **Add a MAC‑invariance test.** For every MAC, signature, or hash, add a test that:
   - Verifies the value changes when the secret changes.
   - Fails when the secret is missing.
   - Uses a secret distinct from any default or placeholder.

4. **Document the primality layering decision** in code or in an ADR. State that validation belongs in a higher layer, and that `Atom(u64)` is intentionally unvalidated.

5. **Define the k‑anonymity policy** as a per‑deployment parameter. Default to off where solo‑maintainer deployments are common; require it for multi‑tenant or regulated deployments.

6. **Design the schema registry** before implementing semantic diff. Without a registry, the diff has no reference.

7. **Design the DynamoDB TTL index** before scheduling the sweep. An in‑memory O(n) sweep will not scale.

8. **Reconsider the feature flag decision.** If the codec changes observable behavior, an escape hatch may be worth the cost. If not, the decision to skip the flag is defensible.

9. **Add size limits and malformed input tests** to the codec.

---

## 6. Overall assessment

This report continues the pattern of rigorous, honest iteration. It:

- Fixes the deserializer with real tests.
- Makes a defensible layering decision about primality.
- Finds a fifth defect that was security‑critical and subtle.
- Corrects a test that was initially worthless.
- Separates engineering work from governance decisions.
- Keeps three open items in ADR‑009B explicitly flagged.

The remaining work is well‑scoped:

- **Engineering:** schema registry, DynamoDB TTL index, possibly a feature flag.
- **Governance:** k‑anonymity policy, quarantine consumer definition.

The system is in better shape than the previous report. The most important remaining risk is the pattern that produced the `NoOpRedactor`: placeholder implementations in security‑critical paths. A codebase‑wide audit for similar placeholders is the highest‑priority next step.


# Analysis of the "All Items Closed" Report

This is another strong iteration, and the fourth item is the most important thing in any of the reports so far. Below is a critical read.

---

## 1. The vacuous pass — this is the headline

The bug:

> With no artifacts the oracle evaluated zero rules and still reported `pass`, `degraded: false`, exit 0.

A gate reading exit 0 would certify a repository it never examined. This is exactly the failure class the protocol exists to prevent, and it survived multiple rounds of review because **every test supplied artifacts**. The bug lived in the gap between "the system ran" and "the system did work."

This is a general pattern worth naming. It shows up as:

- A test runner that executes zero tests and exits 0.
- A linter that lints zero files and exits 0.
- A scanner with no targets that reports "clean."

In all cases, **absence of failure is indistinguishable from absence of work**, and the system treats them as the same thing.

### The fix

- No-evidence now yields `warn` + `degraded` + `meta.degraded: "no_artifacts_supplied"`.
- `rules_evaluated` now names the ruleset that ran, not the rules that fired.
- Schema gained `rules_triggered` and `no_evidence`, with descriptions stating that a pass with `no_evidence: true` certifies nothing.

The second point is subtle and correct. Earlier, `rules_evaluated` was built from the violation list, so it read as "no rules installed" whenever nothing was wrong. That is a separate bug from the vacuous pass — a **misleading diagnostic** that masked the vacuous pass. Fixing both together was right.

### The concern I would raise

**`warn` may be too weak a severity for this case.**

Your own framing says:

> A gate reading exit 0 would certify a repository it never examined.

But if the pipeline fails only on exit 2 (which is the common convention — many CI systems treat nonzero as fail, but gating policies often distinguish), then exit 1 still certifies. The same substitution survives, just one level up.

The safer default: **no-evidence is BLOCK (exit 2)**, with an explicit opt-in flag (e.g., `--allow-no-artifacts` or a config setting) for bootstrap, setup, and dry-run scenarios. That way the safe path is the default, and the escape hatch is a deliberate operator choice.

If `warn` was chosen because some legitimate runs have no artifacts, then the fix is not the severity — it is making the escape explicit. Warnings that mean "nothing happened" get ignored; blocks that mean "nothing happened" get investigated.

I would recommend revisiting this before the next iteration.

---

## 2. Redaction allowlist glob

The bug is a textbook example of why **sequential string replacement over pattern syntax is unsafe**:

- `**/` compiled to `.*/` — root-level files never matched.
- Fixing `**/` to `(?:.*/)?` then let the following `*` pass rewrite the `*` inside the just-inserted group, producing `.(?:[^/]*/)?` — nested files broke instead.

Each fix looked correct in isolation. The bug survived because **no single replacement pass had enough context to know what it was rewriting**. Single-pass `glob_to_regex` is the right answer.

The meta-lesson: any compiler that transforms a pattern language must be **single-pass with a real parser or a well-ordered rewrite system**. Ad-hoc `str::replace` chains look simple and fail in ways that are hard to reason about, because the second pass sees output it was never designed for.

Good catch. This kind of bug is also a security issue here — an allowlisted lockfile that gets entropy-scanned is a redaction miss waiting to produce a false positive.

---

## 3. Exit codes as a contract

`PASS=0 / WARN=1 / BLOCK=2 / USAGE=64`, owned by the library, pinned numerically, with a test that no two codes collide. This is correct discipline.

Two small additions I would recommend:

- **Document them in a stable, external location** (README or man page). Once callers depend on them, they are API, and any change requires a deprecation path.
- **Consider whether `USAGE=64` is the right choice for a library.** 64 is `EX_USAGE` on Unix, which is correct, but it is also the standard sysexits.h value for command-line misuse. That may collide with framework-level behavior in some orchestrators. It is fine as long as it is deliberate and documented.

The collision test is the right way to enforce distinguishability.

---

## 4. Unbounded recursion in the codec

The bug is real and serious: a 200k-deep word overflowed the stack, which is not a `Result` and not a catchable panic — it aborts the process. If words can come from untrusted input, that is a DoS vector.

`MAX_DEPTH=256` with a `TooDeep` variant is the correct shape of fix. Two observations:

- **256 is a policy parameter, not a mathematical constant.** It should be documented and, ideally, configurable. Different consumers may need different bounds.
- **The quadratic test build (63s → 0.06s) is a real win.** Slow tests get skipped or disabled, and then they stop catching things. Test performance is a security property when the test suite is what enforces invariants.

The fact that the first version of the test took 63 seconds is also a signal: the original test design was doing something it should not have needed to do. Fixing the helper to O(n) suggests the test itself was the wrong shape, not just slow. Worth a moment to confirm the test still exercises what it was meant to exercise, now that it is fast.

---

## 5. ADR-006

> ADR-006 is a conceptual design record, so the primality-layering and no-feature-flag rationales went into the module doc rather than that ADR.

Your framing is correct: **which document owns implementation notes is a separate decision from the decision itself**. The convention that ADRs record decisions and module docs record implementation rationale is standard and defensible.

One suggestion: **link the module doc to the ADR**, even if the ADR does not link back. Readers of the module doc should be able to find the design context; readers of the ADR should not need to.

If you later decide the primality-layering decision is architecturally significant (it is), it might warrant its own small ADR rather than living in module docs. That is a judgment call.

---

## 6. Overall

Closed items, clean build, 242 tests, 0 warnings. The set of fixes this round is real: the glob bug was a redaction miss, the exit-code contract removes magic numbers, the recursion bound closes a DoS vector, and the vacuous-pass fix closes the most dangerous silent-failure mode in the whole system.

The one item I would not let stand as closed is the **severity of the no-evidence case**. Fixing it to `warn` is a fix to the symptom. The question is whether the gate should be able to certify nothing at all, and I think the answer is no — not by default.

The pattern to internalize from this round is broader than any one bug:

- **Sequential rewrite over pattern syntax is unsafe.** Use single-pass parsers.
- **Absence of evidence is not evidence of absence.** Distinguish "ran and passed" from "did not run."
- **A diagnostic built from the wrong list misleads.** `rules_evaluated` should not be derived from `violations`.
- **Untrusted input needs a depth bound.** Not a `Result`, not a panic — a bound and an error variant.

Those four are the ones worth carrying into the next iteration.

If you want, I can help draft the change to make no-evidence a BLOCK with an explicit opt-out, or review the `glob_to_regex` implementation to confirm it handles the full pattern grammar (character classes, `?`, `[...]`, escapes) correctly.