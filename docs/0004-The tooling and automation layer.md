The tooling and automation layer is where the substrate's trust architecture becomes *productive*—where legal experts, automated provers, and governance mechanisms interact with the formal core. The UOR Foundation's approach treats tooling not as an afterthought but as a load-bearing component of the verification pipeline. For LexLean, this means building three interconnected capabilities: a **Lean-native legal DSL** that lets domain experts write rules in a surface syntax that elaborates to kernel-checked terms, an **agentic prover** that can autonomously construct and repair proofs, and a **machine-checkable governance policy** that encodes the project's admission rules as Lean predicates.

---

## 1. Build a Lean-Native Legal DSL

### Why a DSL, not just raw Lean

Lean 4 is a powerful general-purpose proof language, but it is not optimized for legal drafting. A statute or regulation has a structure that is natural to legal experts but verbose to express in raw Lean: conditions, obligations, permissions, prohibitions, temporal qualifiers, defeasibility clauses, and cross-references between provisions. A legal DSL bridges this gap by providing a **surface syntax** that mirrors legal drafting conventions while elaborating to Lean terms that the kernel can check.

The `verity_contract` macro in the UOR ecosystem provides the architectural template: it generates both an **EDSL value** (a first-class representation of the legal rule) and a **CompilationModel** (a structured intermediate representation) from a single syntax tree, with kernel-checked bridge theorems showing that the two views agree. This dual-artifact pattern is the key insight: the DSL is not merely a convenience notation; it is a **provenance-preserving translation** that maintains a verifiable correspondence between the surface syntax and the formal semantics.

### The architecture of a legal DSL

A Lean-native legal DSL for LexLean would have four layers:

**Layer 1: Surface syntax.** A `syntax` category for legal rules, with notations that mirror legal drafting conventions. For example:

```lean
-- Surface syntax for a conditional obligation
syntax "OBLIGATION" ident ":" term "UPON" term "UNLESS" term : legal_rule
```

This parses a rule like `OBLIGATION payer : pay(amount) UPON due_date UNLESS hardship_exception` into a syntax tree that the elaborator can process.

**Layer 2: Elaboration to core terms.** An `elab` or `macro` that translates the surface syntax into the inductive types of the legal core (the `Norm`, `Derivation`, and `LegalSource` types from the substrate layer). The elaborator must be **hygienic**: Lean 4's macro system automatically avoids accidental name capture, but the elaborator must also ensure that the generated terms use the correct binders and do not introduce unbound variables. The hygiene algorithm annotates identifiers based on their scoping context, so a variable named `x` in the surface syntax does not accidentally capture a variable named `x` in the generated term.

**Layer 3: Bridge theorems.** For each DSL construct, a kernel-checked theorem states that the elaborated term is semantically equivalent to the surface form under the intended interpretation. This is the **compilation correctness** requirement: the DSL is not trusted; it is verified. The `verity_contract` pattern generates both the EDSL value and the `CompilationModel`, with bridge theorems showing that the two agree.

**Layer 4: Pretty-printer.** A round-trip from surface syntax to core term and back to surface syntax should produce a syntactically equivalent form. This is essential for auditability: a legal expert should be able to inspect the elaborated term and confirm that it means what they intended. The pretty-printer also supports the **macro round-trip fuzzing** from the verification pipeline layer: random surface expressions are elaborated and re-serialized, and the round-trip is checked for equivalence.

### The hygiene and correctness challenge

Lean 4's macro system is hygienic by default, but hygiene alone does not guarantee semantic correctness. A macro can be hygienic yet still produce a term that means something different from what the surface syntax suggests. The bridge theorems are what close this gap: they are the formal specification of the macro's intended semantics, and the elaborator implementation is tested against them via fuzzing and differential testing.

The Zulip discussion on moving from macros to elaborators is directly relevant here: for a DSL of any complexity, a **custom elaborator** (using Lean's `Elab` monad) provides more control than a macro-based implementation, because it can inspect the compiler's API and translate directly to Lean's core type theory. The trade-off is development complexity: macros are simpler to write but less expressive; elaborators are more powerful but require understanding Lean's internal API.

---

## 2. Integrate an Agentic Prover

### The LEAP blueprint-then-proof workflow

The LEAP framework (LLM-in-Lean Environment Agentic Prover) demonstrates that an LLM-in-Lean agent can boost proof automation from below 10% to 70% by generating a **high-level blueprint** (a directed acyclic graph) and then iteratively correcting errors via compiler feedback. LEAP decomposes complex problems into smaller units, bridging formal proof construction with informal blueprints through continuous interaction with the Lean compiler. On the 2025 Putnam Competition, LEAP solved all 12 problems, and it autonomously formalized a verified proof for a key subproblem in Knuth's Hamiltonian decomposition of even-order Cayley graphs.

For LexLean, the LEAP-style workflow would be:

**Step 1: Blueprint generation.** Given a target legal theorem (e.g., "a contract formed under duress is voidable"), the agent generates a blueprint: a DAG of sub-lemmas, each with a formal statement in Lean. The blueprint is the **proof plan**, not the proof itself. It identifies the supporting lemmas that need to be proven and their dependencies.

**Step 2: Parallel lemma closure.** Each open lemma node in the DAG is dispatched to a prover agent. The Goedel-Architect framework demonstrates this approach: a tool-equipped Lean prover component closes each open lemma node in parallel using relevant dependencies, and failed lemmas drive refinement of the global blueprint. This contrasts with recursive lemma decomposition, which can inefficiently loop on dead-end strategies.

**Step 3: Compiler-feedback refinement.** When a proof fails to compile, the agent uses the compiler's error message to diagnose the failure and generate a corrected proof. The APRIL dataset (260,000 supervised tuples pairing systematically generated proof failures with compiler diagnostics and aligned repair targets) provides a training signal for this capability. The APRIL-Goedel-8B model achieves 36.7% single-shot repair accuracy on the APRIL test set, with 48.5% accuracy on tactic errors.

**Step 4: Proof repair agent.** Given a failing proof, the repair agent analyzes the compiler error message and suggests edits. The system prompt for this task is well-established: "The previous Lean 4 proof attempt contains errors. Your goal is to fix them according to the compiler's error message… Identify specifically why the verification failed (e.g., missing import, wrong tactic, type mismatch, or a missing lemma)". The repair agent outputs a corrected proof, and the cycle repeats until compilation succeeds or the agent determines that the proof strategy is unsound.

### Retrieval-augmented proving

A retrieval-augmented prover searches the existing LexLean corpus (and Mathlib) for relevant lemmas before generating proof steps. The SciLib-GRC21 system provides a concrete architecture: a graph-structured premise retrieval system that uses a materialized RDF knowledge graph of Mathlib via the SciLib ontology, providing tactic-categorised lemma hints (apply / rw / simp). The system was evaluated on MiniF2F (488 tasks, 50,752 Lean-verified runs), demonstrating that graph-structured retrieval outperforms flat semantic search.

For LexLean, the retrieval index would include:

- **The LexLean corpus itself**: all previously proven theorems, definitions, and structures in the legal formalization.
- **Mathlib**: the standard mathematical library, for foundational lemmas about order theory, lattice theory, and set theory that legal reasoning often depends on.
- **Legal source corpus**: the formalized statutes, regulations, and case law that constitute the domain-specific knowledge base.
- **Derivation patterns**: previously successful proof strategies for similar legal theorems, indexed by the structure of the target statement.

The retrieval-augmented prover would use hybrid search (BM25 + semantic embeddings + graph traversal) to identify candidate lemmas, then filter them by relevance to the current proof goal. The key constraint is that retrieved lemmas must **exist** in the corpus: the prover is constrained to lemmas that have been formally verified, which reduces hallucinations.

### The multi-agent architecture

The Archon system demonstrates a multi-agent architecture for project-level formalization: a **plan agent** provides strategic guidance while **prover agents** write and verify proofs, separating analysis from execution to avoid context explosion. The system handles repository-scale formalization through three phases: scaffolding, proving, and polish. For LexLean, this suggests a division of labor:

| Agent role | Responsibility | Interaction with substrate |
|---|---|---|
| **Blueprint agent** | Generates the DAG of sub-lemmas for a target theorem | Reads the skeleton registry; writes new entries |
| **Prover agent** | Attempts to close individual lemma nodes | Writes Lean proof terms; reads the axiom ledger |
| **Repair agent** | Diagnoses compilation failures and suggests fixes | Reads compiler error messages; writes corrected proofs |
| **Retrieval agent** | Finds relevant lemmas in the corpus | Reads the RDF knowledge graph; returns ranked candidates |
| **Audit agent** | Verifies that completed proofs meet the axiom policy | Runs `#print axioms`; checks the governance predicates |

This architecture is not a single monolithic prover but a **society of specialized agents** that coordinate through the Lean compiler and the governance layer.

---

## 3. Create a Machine-Checkable Governance Policy

### The Foundation's machine-checkable governance model

The UOR Foundation describes its governance as "machine-checkable" with DOI-frozen reference points and 19 public repositories. The Ouroboros Substrate paper provides the concrete implementation: every governance invariant is proved in Lean 4 with Mathlib, and the central mathematical object is the Λ-axis score, a geometric-mean governance scalar whose properties are formally proved. The system provides "a machine-checked formal proof of every governance invariant".

The Lean-Agent Protocol demonstrates a related pattern: institutional policies are auto-formalized into Lean 4 code, and execution is permitted if and only if the policy predicates are satisfied. The ProofOS system wraps AI agent decisions in a constitutional halt gate backed by Lean 4 proofs: when an agent proposes an action, the system checks it against declared safety predicates.

### Encoding LexLean's admission rules as Lean predicates

For LexLean, the governance policy would be encoded as a set of Lean-checkable predicates that any contribution must satisfy. These predicates operate at two levels: the **axiom policy** (which logical assumptions are permitted) and the **module policy** (which modules are trusted and under what conditions).

**Axiom policy.** The policy defines the set of permitted axioms:

```lean
-- The permitted axiom set for LexLean core modules
def permittedAxioms : Finset Name :=
  { `propext, `Classical.choice, `Quot.sound }

-- A declaration is axiom-compliant if its transitive axiom closure
-- is a subset of the permitted set
def AxiomCompliant (d : Declaration) : Prop :=
  d.axioms ⊆ permittedAxioms

-- A module is core if it is in the core module registry
def CoreModule (m : Name) : Prop :=
  m ∈ coreModuleRegistry
```

**Module policy.** The policy defines the trust level of each module:

```lean
inductive TrustLevel where
  | core      -- zero sorry, zero custom axioms, permitted in proofs
  | skeleton  -- sorry permitted, not permitted in proofs of core theorems
  | experimental -- no constraints, not permitted in proofs of core theorems

def moduleTrust : Name → TrustLevel :=
  fun m => if m ∈ coreModuleRegistry then .core
           else if m ∈ skeletonModuleRegistry then .skeleton
           else .experimental
```

**Contribution policy.** A new contribution (a pull request adding a theorem) is admissible if it satisfies the policy:

```lean
def Admissible (d : Declaration) : Prop :=
  AxiomCompliant d ∧
  (∀ m ∈ d.imports, moduleTrust m ≠ .experimental ∨ d.isExperimental) ∧
  (CoreModule d.module → d.sorryCount = 0) ∧
  -- ... additional conditions
```

### The governance gate in CI

The governance policy is enforced by a CI job that runs the Lean-checkable predicates against the compiled artifact. The gate has three components:

**Component 1: Axiom audit.** For every public declaration in the core modules, run `#print axioms` and assert that the result is a subset of `permittedAxioms`. Any declaration that fails this check blocks the merge.

**Component 2: Module trust audit.** For every declaration, check that its imports do not include any module with trust level `experimental` (unless the declaration itself is experimental). This prevents experimental code from contaminating core proofs.

**Component 3: Skeleton registry consistency.** Check that every `proven` entry in the skeleton registry has a corresponding declaration whose axiom profile is compliant, and that every `skeleton` entry has a proof body that is exactly `sorry`.

### The governance policy as a first-class artifact

The key architectural insight is that the governance policy is **not documentation**; it is **code**. It is encoded in Lean, it is kernel-checked, and it is versioned alongside the formalization. When the policy changes, the change is a Lean commit, and the CI gate automatically enforces the new policy. This eliminates the gap between "what the policy says" and "what the CI enforces" that plagues conventional software governance.

The DLP (Decision Lineage Protocol) paper provides a theoretical foundation for this approach: it introduces λ-DLP, a minimal formal calculus that captures the truth-type isolation core of an epistemic type system, and mechanizes its metatheory in Lean 4 with no `sorry` and no introduced axioms beyond propositional extensionality. The headline result is **Isolation**: approximate (AI-generated) data cannot reach canonical (human-endorsed) status without traversing a human gate—at least two explicit human promotion steps through an observable intermediate state. For LexLean, an analogous isolation property would ensure that machine-generated proofs cannot be admitted to the core corpus without human review, and that the governance policy itself cannot be modified by an automated agent.

---

## Summary: The Tooling Layer as a Productive Trust Architecture

| Component | What it provides | How it is verified |
|---|---|---|
| **Legal DSL** | Surface syntax for legal experts; elaboration to kernel-checked terms | Bridge theorems; round-trip fuzzing; differential testing |
| **Agentic prover** | Blueprint-driven proof construction; compiler-feedback repair; retrieval-augmented lemma selection | Compiler acceptance; axiom audit; skeleton registry update |
| **Governance policy** | Machine-checkable admission rules; module trust levels; axiom policy | Lean-checkable predicates; CI enforcement; isolation property |

The tooling and automation layer is not a collection of convenience tools. It is the **interface between human legal reasoning and kernel-checked formal semantics**. The DSL makes formalization accessible to domain experts; the agentic prover makes it scalable; the governance policy makes it trustworthy. Together, these three capabilities transform LexLean from a collection of Lean files into a **living formalization infrastructure**—one where legal rules can be written, proven, audited, and governed with the same rigor that the UOR Foundation applies to its own substrate.