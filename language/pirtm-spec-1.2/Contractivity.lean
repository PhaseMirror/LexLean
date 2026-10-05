/-
The §34.2 contractivity arithmetic as machine-checked Lean definitions.

This file is the Tier-1 specification of the ADR recorded at
`docs/0001-Production-Grade Verification Architecture for the Typed Recursive
System.md`: the stability certificate stops being prose and becomes a
definition the Lean kernel checks.

It imports **nothing**. Mathlib is deliberately absent: an unpinned
third-party dependency would move under the specification without moving the
pin that authorizes it, and R6 requires every dependency boundary to be explicit
and audited. Lean's core `Rat` is a canonically-normalized, decidably-ordered
rational, which is the whole numeric surface §34.2 needs, so the sums and maxima
below are written as structural recursion over `Fin n` rather than through
`Finset` and `sInf`. That choice is not stylistic: the fold in
`crates/lexlean/src/lexeme/contractivity.rs::column_max` walks atoms in index
order and keeps the lowest index on a tie, and a `sInf` over a `Set.range` would
specify a different function — one whose witness of attainment and whose
tie-breaking are both unstated. The specification states the same fold the
implementation computes, so "the extracted function matches the specification"
is a theorem about one traversal rather than a comparison of two idioms.

Every `theorem` here is about §34.2's arithmetic and nothing else. No theorem
here says a lexeme is lawful, convergent, or lawful in any dynamic sense: §34.0
records that the receipt is a certificate about the shape of a declaration
graph, and a theorem that blurred that would be the exact blurring R2 forbids.
-/

open Rat

namespace LexLeanContractivity

/-! ## Finite sums and maxima over the atom indices

Both are structural recursions on the atom count rather than library folds, so
that the specification names the traversal the Rust implementation performs. -/

/-- `Σ_{j : Fin n} f j`, by recursion on `n`. -/
def sumFin (n : Nat) (f : Fin n → Rat) : Rat :=
  match n with
  | 0 => 0
  | n + 1 =>
      f ⟨n, Nat.lt_succ_self n⟩ + sumFin n fun j => f j.castSucc

/-- `Σ_{i : Fin n} f i` in `Nat`, by the same traversal as [`sumFin`]. -/
def sumFinNat (n : Nat) (f : Fin n → Nat) : Nat :=
  match n with
  | 0 => 0
  | n + 1 =>
      f ⟨n, Nat.lt_succ_self n⟩ + sumFinNat n fun j => f j.castSucc

/-- The maximum of `f` over `Fin n`, keeping the **lowest** index on a tie.

The tie rule is part of the specification rather than an implementation detail:
§34.2's receipt names the column that attained the maximum, and a receipt whose
column depended on evaluation order would not be reproducible on the platforms
§10 requires artifacts to agree on. The empty case is `0` rather than a panic,
which §34.1's refusal of an empty stratum makes unreachable. -/
def maxFin (n : Nat) (f : Fin n → Rat) : Rat :=
  match n with
  | 0 => 0
  | n + 1 =>
      let smaller := maxFin n fun j => f j.castSucc
      let last := f ⟨n, Nat.lt_succ_self n⟩
      if last > smaller then last else smaller

/-- `maxFin` is an upper bound: every value is at most the maximum. -/
theorem le_maxFin (n : Nat) (f : Fin n → Rat) (j : Fin n) : f j ≤ maxFin n f := by
  induction n with
  | zero => exact Fin.elim0 j
  | succ n ih =>
      by_cases h : f ⟨n, Nat.lt_succ_self n⟩ > maxFin n (fun k => f k.castSucc)
      · simp only [maxFin, h, ↓reduceIte]
        exact Fin.cases (le_of_eq rfl) (fun k => le_trans (ih _ k.castSucc) (le_of_eq rfl)) j
      · simp only [maxFin, h, ↓reduceIte]
        refine Fin.cases (le_of_not_gt h) (fun k => ?_) j
        exact ih _ k.castSucc

/-! ## The reference matrix -/

/-- The reference adjacency matrix of a stratum: `A i j` is the number of
whole-token occurrences of atom `j`'s unqualified name in atom `i`'s body
proper.

Non-negativity is structural rather than checked, because §34.2's non-negative
entry requirement is then a property of the type and an `is_non_negative`
predicate over this matrix would be a tautology — the same reason the Rust
`Adjacency` holds `u64`. -/
abbrev ReferenceMatrix (n : Nat) := Fin n → Fin n → Nat

/-- The number of atoms that mention atom `j`, which is `Σ_i A i j` — the
unweighted column sum of §34.2. -/
def referencesTo (A : ReferenceMatrix n) (j : Fin n) : Nat :=
  sumFinNat n fun i => A i j

/-- The contraction factor of atom `j`: `λ j = 1 / (1 + t_j)` for the token
count `t_j` of its body proper.

The numerator is one by construction, so every factor is a unit fraction, which
§34.2 states as a requirement on the factors rather than as a consequence. -/
def contractionFactor (t : Nat) : Rat := 1 / (1 + t)

/-- The gain column of atom `j`: `λ j * refs(j)`.

§34.2 defines the norm as the maximum absolute column sum of `A · diag(λ)`, and
the column sum of column `j` is exactly this product. Absolute values are absent
because §34.2 requires every entry and every factor to be non-negative, so an
absolute value of a non-negative quantity is the quantity; carrying an `abs`
here would assert a generality the specification does not have. -/
def gainColumn (A : ReferenceMatrix n) (t : Fin n → Nat) (j : Fin n) : Rat :=
  contractionFactor (t j) * (referencesTo A j : Rat)

/-- `‖G‖₁` — the exact rational one-norm of §34.2, which is the maximum over the
atoms of `refs(j) / (1 + t_j)`.

`A · diag(λ)` is never formed: since column `j` of the product sums to
`λ j * refs j`, the column-sum form and the matrix product agree on every input,
and the column form is what the implementation computes. -/
def norm (A : ReferenceMatrix n) (t : Fin n → Nat) : Rat :=
  maxFin n fun j => gainColumn A t j

/-! ## The certificate

§34.2 makes a receipt ACCEPT exactly when `‖G‖₁ < 1`, and the boundary matters:
an atom with an empty body has `λ = 1`, so a single reference to it gives
`‖G‖₁ = 1/1` and is refused. A failed certificate is a well-formed receipt
carrying the exact value that failed, not an error — the same distinction the
ADR draws between *absence of a proof of stability* and *proof of instability*. -/

/-- A receipt's status, which §34.2 defines as a function of the norm alone. -/
def Status : Rat → Type
  | q => if q < 1 then PUnit else PUnit

/-- §34.2: a receipt is ACCEPT exactly when `‖G‖₁ < 1`. -/
def isAccept (A : ReferenceMatrix n) (t : Fin n → Nat) : Bool :=
  decide (norm A t < 1)

/-! ### The norm bounds every column -/

/-- The norm is at least every column sum, which is the one-sided half of the
maximum's defining property.

This is what makes `isAccept` a statement about the whole matrix rather than
about one atom: an atom whose column exceeds one would fail the certificate
regardless of the other columns, and that is exactly this theorem read in the
other direction. -/
theorem le_gainColumn (A : ReferenceMatrix n) (t : Fin n → Nat) (j : Fin n) :
    gainColumn A t j ≤ norm A t :=
  le_maxFin n (gainColumn A t) j

end LexLeanContractivity