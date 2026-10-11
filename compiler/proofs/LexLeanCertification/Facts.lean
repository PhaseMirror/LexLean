import LexLeanCertification.Schemas
set_option linter.unusedSimpArgs false
namespace LexLeanCertification
open LexLeanTarget.TargetSyntax LexLeanTarget.TargetSemantics LexLeanPreservation LexLeanTarget
open Certification (Fact Premise)

/-! Lists of the universe, the domain, the facts. -/

theorem natIn_iff : ∀ (l : List Nat) (k : Nat), Gnaf.natIn k l = true ↔ k ∈ l
  | [], k => by simp [Gnaf.natIn]
  | x :: l, k => by
      simp only [Gnaf.natIn, Bool.or_eq_true, List.mem_cons, natIn_iff l k]
      constructor
      · rintro (h | h)
        · exact Or.inl (Nat.eq_of_beq_eq_true h)
        · exact Or.inr h
      · rintro (h | h)
        · exact Or.inl (by simp [h])
        · exact Or.inr h

theorem selectorEq_eq : ∀ (l r : Gnaf.Selector), Gnaf.selectorEq l r = true → l = r
  | .fixed a, .fixed b, h => by
      simp only [Gnaf.selectorEq] at h; exact congrArg _ (Nat.eq_of_beq_eq_true h)
  | .fixed a, .dispatch _ _ _, h => by simp [Gnaf.selectorEq] at h
  | .dispatch _ _ _, .fixed b, h => by simp [Gnaf.selectorEq] at h
  | .dispatch a b c, .dispatch d e f, h => by
      simp only [Gnaf.selectorEq, Bool.and_eq_true] at h
      obtain ⟨h1, h2, h3⟩ := h
      rw [Nat.eq_of_beq_eq_true h1, Nat.eq_of_beq_eq_true h2, Nat.eq_of_beq_eq_true h3]

theorem selectorEq_refl : ∀ (s : Gnaf.Selector), Gnaf.selectorEq s s = true
  | .fixed a => by simp [Gnaf.selectorEq]
  | .dispatch a b c => by simp [Gnaf.selectorEq]

theorem positionOf_at : ∀ (U : List Gnaf.Selector) (q : Gnaf.Selector) (n p : Nat),
    Certification.positionOf U q n = some p → ∃ k, p = n + k ∧ Certification.selectorAt U k = some q
  | [], q, n, p, h => by simp [Certification.positionOf] at h
  | x :: U, q, n, p, h => by
      simp only [Certification.positionOf] at h
      by_cases hx : Gnaf.selectorEq q x = true
      · simp only [hx, if_true, Option.some.injEq] at h
        subst h
        exact ⟨0, rfl, by simp [Certification.selectorAt, selectorEq_eq _ _ hx]⟩
      · simp only [hx, if_false, Bool.not_eq_true] at h
        obtain ⟨k, hk, hat⟩ := positionOf_at U q (n+1) p h
        exact ⟨k+1, by omega, by simpa [Certification.selectorAt] using hat⟩

theorem selectorAt_of_selectorIn : ∀ (U : List Gnaf.Selector) (s : Gnaf.Selector),
    Gnaf.selectorIn s U = true → ∃ k, Certification.selectorAt U k = some s
  | [], s, h => by simp [Gnaf.selectorIn] at h
  | x :: U, s, h => by
      simp only [Gnaf.selectorIn, Bool.or_eq_true] at h
      rcases h with h | h
      · exact ⟨0, by simp [Certification.selectorAt, selectorEq_eq _ _ h]⟩
      · obtain ⟨k, hk⟩ := selectorAt_of_selectorIn U s h
        exact ⟨k+1, by simpa [Certification.selectorAt] using hk⟩

theorem selectorAt_lt : ∀ (U : List Gnaf.Selector) (k : Nat) (s : Gnaf.Selector),
    Certification.selectorAt U k = some s → k < U.length
  | [], k, s, h => by simp [Certification.selectorAt] at h
  | x :: U, 0, s, h => by simp
  | x :: U, k+1, s, h => by
      simp only [Certification.selectorAt] at h
      have := selectorAt_lt U k s h
      simp; omega

theorem valueAt_mem : ∀ (D : List Value) (k : Nat) (a : Value),
    Certification.valueAt D k = some a → a ∈ D
  | [], k, a, h => by simp [Certification.valueAt] at h
  | x :: D, 0, a, h => by simp [Certification.valueAt] at h; simp [h]
  | x :: D, k+1, a, h => by
      simp only [Certification.valueAt] at h
      exact List.mem_cons_of_mem _ (valueAt_mem D k a h)

theorem allBelow_lt : ∀ (l : List Nat) (n k : Nat), Certification.allBelow l n = true → k ∈ l → k < n
  | [], n, k, _, hk => by simp at hk
  | x :: l, n, k, h, hk => by
      simp only [Certification.allBelow, Bool.and_eq_true] at h
      rcases List.mem_cons.mp hk with rfl | hk
      · exact Nat.blt_eq.mp h.1
      · exact allBelow_lt l n k h.2 hk

/-- Allagree is functional. -/
theorem allAgree_unique {sys ref F} {D : List Value} {c c' : Nat}
    (h : AllAgree sys ref F D c) (h' : AllAgree sys ref F D c') : c = c' := by
  induction h generalizing c' with
  | nil => cases h'; rfl
  | cons ht _ ih =>
    cases h' with
    | cons ht' h2 =>
      rw [ht] at ht'; cases ht'
      rw [ih h2]

/-! Facts. -/

/-- What a fact says about a system, at the request's fuel and every larger one. -/
def FactSound (req : Gnaf.Request) (g : Gnaf.Grammar) (U : List Gnaf.Selector)
    (s : Gnaf.Selector) : Fact → Prop
  | .inadmissible => ∀ F', req.machine.fuel ≤ F' → ∀ c z, statusAt req g s F' ≠ .admitted c z
  | .admitted c z => ∀ F', req.machine.fuel ≤ F' → statusAt req g s F' = .admitted c z
  | .atLeast c z => ∀ F', req.machine.fuel ≤ F' → ∀ c' z',
      statusAt req g s F' = .admitted c' z' → c ≤ c' ∧ z' = z
  | .reduces p => ∀ F', req.machine.fuel ≤ F' → ∀ c z, statusAt req g s F' = .admitted c z →
      ∃ q c' z', Certification.selectorAt U p = some q ∧ statusAt req g q F' = .admitted c' z' ∧
        c' < c ∧ z' ≤ z
  | .unresolved => True
  | .unproven => True

theorem statusAt_admitted {req g s F c z} (h : statusAt req g s F = .admitted c z) :
    ∃ steps, Gnaf.statusOn (Gnaf.realize g s) req.reference F req.domain = .admitted steps 0 ∧
      c = steps + Certification.staticCharge req g s ∧ z = Certification.staticSize g s := by
  unfold statusAt at h
  cases hs : Gnaf.statusOn (Gnaf.realize g s) req.reference F req.domain with
  | admitted steps size =>
    rw [hs] at h; simp only [Gnaf.Status.admitted.injEq] at h
    obtain ⟨rfl, rfl⟩ := h
    exact ⟨steps, by rw [(allAgree_of_admitted _ hs).1], rfl, rfl⟩
  | inadmissible => rw [hs] at h; simp at h
  | unresolved => rw [hs] at h; simp at h

theorem statusAt_of_allAgree {req g s F steps} {D : List Value}
    (hD : D = req.domain)
    (h : AllAgree (Gnaf.realize g s) req.reference F D steps) :
    statusAt req g s F = .admitted (steps + Certification.staticCharge req g s) (Certification.staticSize g s) := by
  subst hD
  unfold statusAt
  rw [admitted_of_allAgree h]

theorem measured_sound (req g s) (c z : Nat) (hc : Gnaf.status req g s = .admitted c z) :
    ∀ F', req.machine.fuel ≤ F' → statusAt req g s F' = .admitted c z := by
  intro F' hle
  rw [status_eq] at hc
  obtain ⟨steps, hs, rfl, rfl⟩ := statusAt_admitted hc
  obtain ⟨_, hall⟩ := allAgree_of_admitted _ hs
  exact statusAt_of_allAgree rfl (allAgree_stable hle hall)

theorem measured_inadmissible (req g s) (hc : Gnaf.status req g s = .inadmissible) :
    ∀ F', req.machine.fuel ≤ F' → ∀ c z, statusAt req g s F' ≠ .admitted c z := by
  intro F' hle c z h
  rw [status_eq] at hc
  unfold statusAt at hc
  cases hs : Gnaf.statusOn (Gnaf.realize g s) req.reference req.machine.fuel req.domain with
  | admitted _ _ => rw [hs] at hc; simp at hc
  | unresolved => rw [hs] at hc; simp at hc
  | inadmissible =>
    obtain ⟨a, ha, hd⟩ := inadmissible_witness _ hs
    obtain ⟨steps, hs', _, _⟩ := statusAt_admitted h
    exact disagree_not_admitted hle ha hd _ _ hs'
end LexLeanCertification
