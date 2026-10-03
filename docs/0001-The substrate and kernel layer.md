The substrate and kernel layer is the load-bearing foundation for a formalization project like LexLean. It is not merely where definitions live; it is where the **trust surface** of the entire system is established. The UOR Foundation's approach treats this layer with architectural discipline: the Lean 4 kernel is the single source of truth, and every claim about legal reasoning must ultimately reduce to kernel-checked terms. Expanding this layer means making that trust surface **explicit, auditable, and machine-enforceable**.

Below is a detailed expansion of each element.

---

## 🧱 1. Audit the Axiom Surface

### Why the axiom surface is the first object of scrutiny

In Lean 4, the kernel itself assumes exactly three logical axioms: `propext`, `Classical.choice`, and `Quot.sound`. Every theorem in Mathlib and every major formalization project runs on top of these. When a project introduces its *own* axioms—whether through `axiom` declarations, `opaque` definitions, or `sorry` placeholders—it is effectively extending the trust surface. The UOR Foundation's "zero custom axioms" commitment means that no theorem in the core development should depend on anything beyond the kernel's three axioms.

For LexLean, the axiom surface is the set of all logical assumptions that legal claims ultimately rest upon. If a theorem about statutory interpretation depends on an axiom that says "this statute is valid," then the theorem is only as trustworthy as that axiom. The audit makes this dependency visible and governable.

### What a rigorous axiom audit entails

**Step 1: Enumerate the axiom frontier.** For every public theorem in LexLean, run `#print axioms` and record the transitive closure of axioms it depends on. The UOR Foundation's Stage-9 auditor approach works from the *compiled artifact* rather than the source prose: it asks what the compiled theorem actually depends on, not what the documentation claims. This distinction matters because a theorem may appear axiom-free in its source file but import a dependency that carries an axiom.

**Step 2: Classify each axiom.** Axioms fall into distinct categories, and each category requires a different governance response:

| Category | Example | Governance response |
|---|---|---|
| **Kernel axioms** | `propext`, `Classical.choice`, `Quot.sound` | Permitted by default; no justification needed |
| **Mathlib axioms** | Axioms inherited from dependencies | Document the dependency chain; justify why the dependency is necessary |
| **LexLean-specific axioms** | Legal primitives (e.g., "a contract is a legal object") | Isolate; document; minimize; ideally replace with inductive definitions |
| **Sorry-ax** | `sorryAx` introduced by `sorry` | Forbidden in core modules; permitted only in designated skeleton/stub modules |

**Step 3: Automate the audit in CI.** The audit must run on every commit, not as a manual review. The `ai-safety-formalization-atlas` project demonstrates a concrete pattern: a Python script generates a `#print axioms` harness, runs `lake env lean` on it, and asserts that each named declaration depends only on the permitted kernel axioms. LexLean should adopt a similar gate:

```
# Pseudocode for a LexLean axiom gate
for each public declaration D in LexLean:
    axioms_D = run("#print axioms " ++ D.name)
    for axiom in axioms_D:
        if axiom ∉ permitted_kernel_axioms and axiom ∉ documented_lexlean_axioms:
            FAIL("Undeclared axiom dependency: " + axiom + " in " + D.name)
```

**Step 4: Maintain an axiom ledger.** The audit produces a machine-readable artifact: a ledger mapping every public declaration to its axiom dependencies. This ledger becomes a first-class governance document. When a new axiom is proposed, the ledger shows exactly which theorems would be affected. When an axiom is eliminated (by replacing it with an inductive definition), the ledger records the change.

### The opaque-witness pattern as an axiom-containment strategy

The UOR Foundation's `enforcement` module provides a concrete pattern for introducing *controlled* assumptions without polluting the axiom surface. It defines `Datum`, `Validated`, `Derivation`, and `FreeRank` as **sealed types with private fields** that prove a value passed through a specific computational boundary. Prism code can consume these values but cannot fabricate them. The key property is that the *only way* to obtain a `Validated<T>` is through the reduction evaluator or the two-phase minting boundary—not by construction.

LexLean can mirror this pattern for legal primitives:

```lean
-- A sealed witness that a legal text has been parsed through the LexLean parser
structure ParsedLegalText where
  private mk ::
  text : String
  -- no public constructor

-- A sealed witness that a rule has been type-checked against the legal ontology
structure ValidatedRule where
  private mk ::
  rule : LegalRule
  -- no public constructor
```

The advantage is architectural: rather than declaring `axiom legal_text_is_valid : ...`, LexLean can provide a **builder function** that returns a sealed witness. The builder function's *implementation* may use axioms internally, but those axioms are isolated to a single, auditable module. Downstream theorems that consume the sealed witness do not inherit the builder's axioms in their `#print axioms` output, because the witness is an opaque value, not a logical assumption.

---

## 📐 2. Formalize Operational Semantics as Inductive Types

### Why inductive types, not axioms

An axiom asserts a proposition without proof. An inductive type *defines* a proposition by specifying its constructors. In Lean 4, inductive types are kernel-checked: the kernel verifies that each constructor is well-formed and that the type is strictly positive (which guarantees consistency). This means that an inductive definition cannot introduce inconsistency, whereas an axiom can.

For legal reasoning, this distinction is foundational. Consider the claim that "a contract requires offer and acceptance." As an axiom:

```lean
axiom contract_requires_offer_and_acceptance :
  ∀ (c : Contract), HasOffer c ∧ HasAcceptance c
```

This axiom could be inconsistent with other axioms, and the kernel cannot detect the inconsistency. As an inductive definition:

```lean
inductive ValidContract : Contract → Prop where
  | mk (c : Contract) (h_offer : HasOffer c) (h_accept : HasAcceptance c) :
      ValidContract c
```

The inductive definition *constructs* evidence for `ValidContract`. Any theorem that needs to know "this contract is valid" must supply a term of type `ValidContract c`, which in turn requires actual evidence of offer and acceptance. The proof obligation is **explicit and irreducible**.

### The UOR Foundation's inductive substrate

The UOR Framework's kernel is built on the ring `Z/(2^n)Z` and the dihedral group `D_{2^n}`, which are defined as mathematical structures rather than assumed as axioms. The `CriticalIdentityProof` demonstrates that `neg(bnot(x)) = succ(x)` for all ring elements—a theorem derived from the inductive structure of the ring, not assumed. This is the model LexLean should follow: define the *structure* of legal reasoning as an inductive family, and derive properties from that structure.

### A concrete inductive design for LexLean

A legal reasoning engine needs at minimum:

**A. The source layer.** Statutory text, regulations, and case law are represented as abstract syntax trees. This is an inductive type:

```lean
inductive LegalSource : Type where
  | statute : StatuteId → Section → LegalSource
  | regulation : RegulationId → Section → LegalSource
  | case : CaseId → Holding → LegalSource
  | derived : LegalSource → DerivationRule → LegalSource
```

**B. The norm layer.** Each source yields norms (obligations, permissions, prohibitions). Norms are inductive:

```lean
inductive Norm : LegalSource → Type where
  | obligation : Party → Action → Condition → Norm s
  | permission : Party → Action → Condition → Norm s
  | prohibition : Party → Action → Condition → Norm s
  | power : Party → LegalEffect → Condition → Norm s
```

**C. The derivation layer.** Legal conclusions are derived from norms through inference rules. The derivation relation is inductive:

```lean
inductive Derives : Norm s → LegalConclusion → Prop where
  | modus_ponens : Derives n₁ (n₂ → c) → Derives n₂ n₂ → Derives n₁ c
  | statutory_interpretation : ...
  | precedent_application : ...
  | defeasible_override : ...  -- for conflicting norms
```

**D. The certificate layer.** Every derivation produces a certificate that can be independently checked. This mirrors the UOR `Certify<I>` trait:

```lean
structure Certificate where
  conclusion : LegalConclusion
  derivation : DerivationTree conclusion
  -- derivation tree is a first-class artifact, not a proof term
```

The crucial property is that `DerivationTree` is a **data structure**, not a proof. It can be serialized, audited, and replayed by an independent verifier. The Lean proof then shows that any valid `DerivationTree` implies the corresponding conclusion. This separation—between the *computational* derivation and the *logical* proof—is what makes the system both executable and formally verifiable.

### The payoff: kernel as single source of truth

When operational semantics are inductive, the kernel becomes the arbiter of every legal claim. A disputed interpretation is not resolved by appealing to external authority; it is resolved by exhibiting a derivation tree that the kernel accepts. The `Validated<T>` witness pattern then becomes a *machine-checkable certificate* that a particular derivation has been kernel-checked.

---

## 📋 3. Add a "Skeleton Registry"

### What a skeleton registry is

A skeleton registry is a **machine-checkable catalog of proof obligations**. For each target theorem in LexLean, the registry records:

- The theorem's **name** and **target module**
- Its **statement** (the Lean type it purports to prove)
- Its **verification status**: `proven`, `skeleton` (statement only, proof body is `sorry`), or `open` (statement not yet written)
- Any **dependencies** on other registry entries
- The **axiom profile** of the proven theorem (once completed)

The Ouroboros Substrate uses this concept: a Lean 4 skeleton is placed in `lean_skeletons/`, and theorems with `sorry` skeletons are marked `[skeleton — Lean Czar pending]`. The registry makes the proof obligation surface *explicit*: instead of a scattered collection of `sorry` statements, there is a single ledger of what remains to be proved.

### Why a registry is architecturally necessary

A formalization project of any scale accumulates proof obligations. Without a registry, the obligations are implicit: they exist as `sorry` placeholders scattered across modules, or as theorems that are stated in documentation but never formalized. The registry makes them **explicit, countable, and trackable**.

For LexLean, this is especially important because legal reasoning involves many **interdependent claims**. A theorem about statutory interpretation may depend on a theorem about the validity of the statute, which may depend on a theorem about the legislative process. The registry makes this dependency graph visible and machine-checkable.

### Concrete registry design

A skeleton registry can be implemented as a Lean file that is itself kernel-checked:

```lean
-- LexLean/SkeletonRegistry.lean

structure SkeletonEntry where
  name : Name
  module : Name
  statement : Expr
  status : SkeletonStatus
  dependencies : List Name

inductive SkeletonStatus where
  | proven
  | skeleton  -- statement exists, proof is sorry
  | open      -- statement not yet written

def lexleanRegistry : List SkeletonEntry := [
  { name := `LexLean.contract_requires_offer,
    module := `LexLean.Contract,
    statement := ...,  -- the Expr of the theorem statement
    status := .proven,
    dependencies := [] },

  { name := `LexLean.statutory_interpretation_sound,
    module := `LexLean.Statutory,
    statement := ...,
    status := .skeleton,
    dependencies := [`LexLean.contract_requires_offer] },

  { name := `LexLean.precedent_binding_force,
    module := `LexLean.Precedent,
    statement := ...,
    status := .open,
    dependencies := [`LexLean.statutory_interpretation_sound] }
]
```

Because this is a Lean definition, the registry is **kernel-checked**: the `Expr` values for statements are real Lean expressions, not strings. A CI job can then:

1. **Count** the entries by status and report progress over time.
2. **Verify** that every `proven` entry actually has a proof in the compiled artifact (no `sorryAx` in its axiom profile).
3. **Verify** that every `skeleton` entry has a statement that elaborates and a proof body that is exactly `sorry`.
4. **Check** that the dependency graph is acyclic.
5. **Generate** a human-readable report of the proof obligation surface.

### The registry as a governance artifact

The registry is not merely a development tool. It is a **governance artifact** that makes the project's verification claims machine-checkable. The UOR Foundation describes its governance as machine-checkable with DOI-frozen reference points. A skeleton registry extends this principle: the project's *proof obligations* become machine-checkable, not just its *proofs*.

This has several concrete benefits:

- **Progress tracking.** The registry provides an objective measure of formalization progress: "142 of 210 target theorems are proven."
- **Prioritization.** The dependency graph identifies which unproven theorems block the most downstream work.
- **Audit readiness.** External auditors can inspect the registry to understand what has been claimed, what has been proven, and what remains open.
- **Agent integration.** An automated prover (or an LLM-based agent) can consume the registry as a work queue: it reads the next `skeleton` entry, attempts a proof, and updates the status if successful.

### Integrating the registry with the axiom audit

The registry and the axiom audit are complementary. The registry defines **what must be proven**; the axiom audit defines **what is assumed**. Together, they form the complete trust surface:

```
Trust Surface = Proof Obligations (Registry) + Logical Assumptions (Axiom Ledger)
```

A theorem is fully trustworthy when it appears in the registry with status `proven` and its axiom profile contains only permitted kernel axioms. A theorem that is `proven` but depends on an undeclared axiom is *not* trustworthy, and the combination of the registry and the audit catches this.

---

## 🧭 Summary: The Substrate Layer as a Trust Architecture

| Element | What it establishes | How it is enforced |
|---|---|---|
| **Axiom audit** | The logical assumptions of the system | `#print axioms` CI gate; axiom ledger; sealed witness pattern |
| **Inductive semantics** | The meaning of legal constructs | Kernel-checked inductive definitions; no axiomatization of legal primitives |
| **Skeleton registry** | The proof obligation surface | Lean-encoded registry; CI checks; dependency graph |

The substrate layer is not a static collection of definitions. It is a **living trust architecture**: every commit either preserves or changes the trust surface, and the CI gates make that change visible. The UOR Foundation's commitment to zero custom axioms and zero `sorry` in core developments is not a stylistic preference—it is an architectural discipline that keeps the kernel as the single source of truth. LexLean's substrate layer should inherit that discipline, and the mechanisms above are how it becomes operational.