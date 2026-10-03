The verification pipeline layer is where the proof-to-code feedback loop is operationalized. The UOR Foundation's `atlas-embeddings` project demonstrates the architectural principle: a **dual assurance system** where computational tests catch implementation bugs and formal proofs catch logical errors. For LexLean, this means treating verification not as a final checkpoint but as a continuous, layered discipline that runs on every commit. Below is a detailed expansion of each element.

---

## 1. Property-Based Testing for Executable Legal Fragments

### Why property-based testing complements formal proofs

Formal proofs establish that a theorem holds for all inputs satisfying its hypotheses. But the executable fragments of a legal reasoning engine—parsers, derivation tree evaluators, certificate checkers—are *programs*, and programs can have bugs that formal statements do not capture: stack overflows on deep derivation trees, timeouts on large statutory inputs, or serialization failures when certificates cross system boundaries. Property-based testing fills this gap by generating random inputs and checking that executable properties hold across them.

Plausible is the Lean 4 framework for this. It integrates into the tactic framework, so a property can be stated and tested in the same language as the theorem it supports. For built-in types, Plausible handles them automatically; for custom types (like `LegalSource`, `DerivationTree`, or `Certificate`), the developer provides instances of `Repr`, `Shrinkable`, and `SampleableExt`.

### Concrete properties for LexLean

**Property 1: Derivation tree round-trip.** If a `DerivationTree` is serialized to a compact certificate format and then deserialized, the result should be structurally identical to the original. This catches bugs in the certificate encoder/decoder that a formal proof about the *tree* would not detect.

```lean
-- Property: serialize/deserialize is identity on derivation trees
example (dt : DerivationTree conclusion) :
    deserialize (serialize dt) = dt := by
  plausible
```

**Property 2: Norm extraction totality.** Every well-formed `LegalSource` should yield at least one `Norm`. This is a totality property: the extraction function should never panic or return an empty list for a syntactically valid source.

```lean
example (s : LegalSource) (h : WellFormed s) :
    (extractNorms s).length > 0 := by
  plausible
```

**Property 3: Certificate checking is deterministic.** Two invocations of the certificate checker on the same certificate and context should produce identical results. This catches non-determinism introduced by caching, mutable state, or hash-order dependence.

```lean
example (c : Certificate) (ctx : Context) :
    checkCertificate c ctx = checkCertificate c ctx := by
  rfl  -- trivially true in pure Lean, but the executable analogue
       -- may use IO and needs testing
```

### The shrinker as a debugging asset

Plausible's `Shrinkable` instance is not merely a testing convenience; it is a **counterexample minimizer**. When a property fails, Plausible shrinks the input to a minimal counterexample. For LexLean, this means that when a derivation tree fails a property, the shrinker can reduce it to the smallest tree that still exhibits the failure—often a two-node tree with a single inference rule. This makes debugging dramatically more efficient than examining a large random input.

### Integration with the proof pipeline

Property-based tests should run in the same CI job as `lake build`. The `#eval Plausible.Testable.check` command compiles and runs the test; if it finds a counterexample, the CI job fails. The test is *not* a proof—it is a computational complement to the proof, catching bugs that formal verification was never designed to catch.

---

## 2. Differential Testing Against a Reference Implementation

### The architecture of differential testing

Differential testing compares two implementations of the same specification: the **oracle** (the Lean 4 model) and the **system under test** (the production implementation, typically in Python, Rust, or TypeScript). Random inputs are generated, both implementations are executed, and outputs are compared. Divergences identify either bugs in the production implementation or underspecification in the Lean model.

The UOR Foundation's approach to this is exemplified by the `verified-ledger` reference architecture: a simplified formal model is developed in Lean 4, invariants are proven, and from this model a differential fuzzing framework is constructed that tests the target system against the proven oracle. The Resonate documentation describes a similar three-layer correctness story: an executable formal specification in Lean 4, differential random testing against an independent in-memory oracle, and deterministic simulation testing of the SDK.

### LexLean-specific differential testing design

For LexLean, the oracle is the Lean 4 formalization of legal reasoning. The system under test is the production legal reasoning engine (whether written in Rust for performance or Python for ecosystem integration). The differential testing harness would:

1. **Generate random legal scenarios**: Create synthetic statutes, regulations, and fact patterns from a grammar that respects the well-formedness constraints of the Lean model.
2. **Execute both implementations**: Run the Lean model (via `native_decide` or a compiled executable) and the production implementation on the same input.
3. **Compare conclusions and derivation trees**: The comparison must be structural, not just at the level of final conclusions. If the Lean model produces a derivation tree that applies rule A then rule B, and the production implementation applies rule B then rule A, the conclusions may agree but the derivations differ—and that difference may matter for auditability.
4. **Report divergences with minimal repros**: The harness should shrink the input to the smallest scenario that produces a divergence, mirroring Plausible's shrinking behavior.

### The proof-driven development pattern

The `proof-driven-development` pattern is the recommended workflow: prove the specification in Lean 4 first, translate the verified algorithm to the target language, and run differential tests to catch translation bugs. For LexLean, this means:

- **Prove** that the derivation relation is sound (any valid derivation tree implies the conclusion).
- **Translate** the derivation checker to the production language.
- **Test** that the translated checker agrees with the Lean checker on random inputs.

This pattern is especially valuable for legal reasoning because the translation from formal derivation rules to production code is where subtle bugs—off-by-one errors in rule indexing, incorrect handling of defeasible overrides, or missing edge cases in statutory interpretation—are most likely to occur.

---

## 3. Macro Round-Trip Fuzzing

### The trusted computing base problem

Lean 4's trusted computing base is larger than the kernel alone. It includes the C++ runtime, the elaborator, the compiler, and the extensible macro system. Macros are syntax transformers that operate before elaboration; they maintain hygiene via explicit scope tracking and allow for the introduction of custom notations and derived-language fragments. If LexLean introduces a legal DSL as a macro, that macro becomes part of the trusted computing base: a bug in the macro can produce incorrect terms that the kernel then accepts.

The UNSOUND 2026 paper on fuzzing Lean 4's trusted computing base demonstrates the stakes: over 105 million fuzzing executions with AFL++, AddressSanitizer, and UBSan discovered a heap buffer overflow in the Lean runtime affecting every version of Lean 4 to date. While this particular bug is in the runtime rather than in macros, it establishes the principle that the non-kernel components of Lean 4 are subject to bugs and must be tested.

### Macro round-trip fuzzing for a legal DSL

If LexLean introduces a macro-based DSL for legal rules (analogous to the `verity_contract` macro), the macro must be tested for **round-trip correctness**: parsing a surface syntax tree into a Lean term and then re-serializing it should produce a syntactically equivalent surface form. More importantly, the elaborated term must be *semantically correct*—it must mean what the legal expert intended.

The `verity_contract` macro testing approach (520 differential tests + fuzzing) provides a concrete template. For LexLean:

- **Generate random legal DSL expressions** from a grammar of the surface syntax.
- **Elaborate each expression** through the macro to obtain a Lean term.
- **Re-elaborate the term** as surface syntax (if a pretty-printer exists) and check round-trip equivalence.
- **Compare against a hand-written reference** for a subset of expressions to catch semantic drift.

### The expansion lemma approach

A more rigorous alternative to fuzzing alone is to **prove** that the macro is correct for a core fragment. This is the approach taken by `verity_contract`: it generates both the EDSL value and a `CompilationModel` from a single syntax tree, with kernel-checked bridge theorems showing that the two views agree. For LexLean, this would mean:

```lean
-- The macro elaborates surface syntax to a core term
-- The bridge theorem states that the elaborated term is
-- semantically equivalent to the surface form under the
-- intended interpretation
theorem legal_dsl_elaboration_correct (syn : LegalSyntax) :
    denote (elaborate syn) = surfaceDenote syn := by
  ...
```

Fuzzing then tests the *implementation* of the macro (the elaborator code) against this specification, catching cases where the elaborator produces a term that the bridge theorem does not cover.

---

## 4. CI/CD Gates for the Verification Pipeline

### The cold-build gate

A full `lake build` from a clean state (no cached `.olean` files) is the only reliable compilation test. Warm builds reuse stale objects and can mask broken imports, namespace collisions, dangling `open` statements after refactoring, and cross-file definition drift. Single-file `lake env lean` passes are necessary but not sufficient: a file can compile alone while its downstream consumers break.

For LexLean, every merge to `main` requires a cold build to pass. The CI configuration should:

- **Cache nothing** in the cold-build job (or cache only the Lake package dependencies, not the project's own `.olean` files).
- **Run `lake build --wfail`** to treat warnings as errors.
- **Assert that the build produces no `sorry` warnings**—though as the Ripple playbook notes, textual grep is only a coarse upper bound; the authoritative signal is `#print axioms`.

### The `#print axioms` gate

This is the load-bearing gate. For every deliverable theorem, the CI runs `#print axioms T` and asserts that the output lists exactly the permitted kernel axioms (`propext`, `Classical.choice`, `Quot.sound`). Any appearance of `sorryAx` or a custom axiom name fails the gate. This catches `sorry` and axiom escapes anywhere in the transitive import closure, not just in the file being reviewed.

The `unsorry` project's ADR-006 describes a layered gate whose authoritative check is an axiom-footprint audit, with a two-package split: a library with the zero-`sorry` bar, and a goals package where statements legitimately carry `sorry`. LexLean should adopt a similar split: a **core library** where `sorry` is absolutely forbidden and the axiom profile is audited, and a **skeleton package** where proof obligations are stated with `sorry` and tracked in the skeleton registry.

The invariant registry and `#print axioms` CI gate described in the Foundation's own verification spike provides the concrete implementation model: the `lean-build` CI job audits zero-`sorry` status, and the `#print axioms` gate audits the axiom set.

### The proof-health report

The proof-health report is a CI artifact that summarizes the state of the formalization on every commit. It should include:

| Metric | Definition | Target |
|---|---|---|
| **Theorem count** | Number of theorems with status `proven` in the skeleton registry | Monotonically increasing |
| **Axiom count** | Number of distinct axioms in the transitive closure of all proven theorems | Exactly 3 (kernel axioms) |
| **Sorry count** | Number of `sorry` tokens in the core library | Zero |
| **Coverage** | Fraction of skeleton registry entries with status `proven` | Increasing over time |
| **Dependency depth** | Maximum depth of the proof dependency graph | Stable or decreasing |

The report is generated by a script that parses the skeleton registry, runs `#print axioms` on each proven theorem, and emits a JSON artifact. Because the registry is Lean-encoded, the report is machine-checkable: the CI can verify that the reported counts match the actual state of the code.

### The wiki backlink gate

The `prism` repository mechanically validates wiki backlinks in CI: every public item must carry a backlink to the UOR-Framework wiki section that defines it, and a broken anchor fails the build. This is not merely a documentation practice; it is an **architectural traceability mechanism**. The wiki specifies the architecture; the code implements it; the backlink is the proof that the implementation corresponds to a specific architectural commitment.

For LexLean, the same pattern applies. Every public theorem, definition, and structure should carry a docstring with a backlink to:

- The **legal source** (statute, regulation, or case law) that motivates it.
- The **formal specification** (the wiki page or ADR) that describes its intended semantics.
- The **skeleton registry entry** that tracks its verification status.

The CI gate is straightforward: a script extracts all `wiki://` or `lexlean://` links from docstrings, resolves each against the target corpus, and fails if any link is broken. This catches two classes of errors: docstrings that reference legal sources that have been superseded or amended, and docstrings that reference formal specifications that have been changed without updating the code.

### Documentation coverage gate

Mathlib's `docBlame` linter requires that public declarations have docstrings. LexLean should extend this with a **five-block doc structure** analogous to the Prism pattern: every public item carries a docstring with (1) a natural-language description, (2) a formal statement, (3) a backlink to the legal source, (4) a backlink to the formal specification, and (5) a reference to the skeleton registry entry. The CI gate enforces that all five blocks are present and that the backlinks resolve.

### The non-vacuity check

A clean-3 theorem can still be vacuous if its hypotheses are unsatisfiable or its conclusion is trivially true. The Ripple playbook requires a concrete witness for every headline theorem: for a computability theorem, a numerical integration that matches the certified limit; for a universality theorem, a specific machine instance. For LexLean, this means that every theorem about legal reasoning should come with a **concrete example**: a specific statute, a specific fact pattern, and a specific derivation tree that the theorem applies to. The CI gate checks that the example exists and that it type-checks against the theorem's statement.

---

## Summary: The Verification Pipeline as a Continuous Discipline

| Gate | What it catches | Enforcement mechanism |
|---|---|---|
| **Property-based tests** | Runtime bugs in executable fragments | `#eval Plausible.Testable.check` in CI |
| **Differential tests** | Translation bugs and semantic drift | Random input generation + oracle comparison |
| **Macro round-trip fuzzing** | Elaborator bugs and DSL semantic errors | Grammar-based fuzzing + bridge theorems |
| **Cold-build gate** | Stale `.olean` masking broken imports | `lake build` from clean state, `--wfail` |
| **`#print axioms` gate** | Hidden `sorry` and undeclared axioms | Axiom footprint audit on every theorem |
| **Proof-health report** | Regression in proof coverage or axiom hygiene | Machine-readable artifact, CI-checked |
| **Wiki backlink gate** | Documentation–code drift | Link resolution against wiki source |
| **Doc coverage gate** | Undocumented public declarations | `docBlame` + five-block structure check |
| **Non-vacuity check** | Theorems that are technically true but meaningless | Concrete witness required for headline theorems |

The verification pipeline is not a collection of independent checks. It is a **layered assurance architecture**: computational tests catch bugs that proofs cannot see; proofs catch logical errors that tests cannot exhaust; CI gates enforce that neither layer is bypassed. The UOR Foundation's commitment to machine-checkable governance is realized in the verification pipeline: every claim about the system—whether a theorem, a docstring, or a wiki backlink—is mechanically validated before it reaches `main`.