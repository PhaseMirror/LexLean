import LexLeanCertification.Positions
set_option linter.unusedSimpArgs false
namespace LexLeanCertification
open LexLeanTarget.TargetSyntax LexLeanTarget.TargetSemantics LexLeanPreservation LexLeanTarget
open Certification (Fact Premise Verdict Certified Obligation)

theorem scanLower_none {strict bound att} : ∀ (facts : List Fact) (pos : Nat),
    Certification.scanLower strict bound att facts pos = none →
    ∀ i f, Certification.factAt facts i = some f →
      Certification.lowerPosition strict bound (Gnaf.natIn (pos + i) att) f (pos + i) = none
  | [], _, _, i, f, h => by simp [Certification.factAt] at h
  | x :: fs, pos, hscan, i, f, h => by
      simp only [Certification.scanLower] at hscan
      cases hl : Certification.lowerPosition strict bound (Gnaf.natIn pos att) x pos with
      | some o => rw [hl] at hscan; simp at hscan
      | none =>
        rw [hl] at hscan
        simp only [] at hscan
        cases i with
        | zero =>
          simp only [Certification.factAt] at h
          cases h; simpa using hl
        | succ i =>
          simp only [Certification.factAt] at h
          have := scanLower_none fs (pos + 1) hscan i f h
          have e : pos + 1 + i = pos + (i + 1) := by omega
          rw [e] at this; exact this

theorem lowerVerdict_inv {strict bound att facts x}
    (h : Certification.lowerVerdict strict bound att facts = .certified x) :
    Certification.ascending att 0 = true ∧ Certification.allBelow att facts.length = true ∧
    Certification.scanLower strict bound att facts 0 = none ∧ att ≠ [] ∧
    x = (if strict = true then Certified.argmin att bound else Certified.optimum att bound) := by
  unfold Certification.lowerVerdict at h
  have hlen : Certification.LexLeanRuntime.length facts = facts.length := rfl
  rw [hlen] at h
  by_cases h1 : Certification.ascending att 0 = true
  · by_cases h2 : Certification.allBelow att facts.length = true
    · simp only [h1, h2, Bool.and_self, Bool.not_true, Bool.false_eq_true, if_false] at h
      cases hs : Certification.scanLower strict bound att facts 0 with
      | some o => rw [hs] at h; simp at h
      | none =>
        rw [hs] at h
        simp only [] at h
        cases att with
        | nil => simp at h
        | cons a rest =>
          simp only [] at h
          refine ⟨h1, h2, rfl, List.cons_ne_nil _ _, ?_⟩
          cases strict
          · simp only [Bool.false_eq_true, if_false, Verdict.certified.injEq] at h ⊢; exact h.symm
          · simp only [if_true, Verdict.certified.injEq] at h ⊢; exact h.symm
    · rw [Bool.not_eq_true] at h2
      simp only [h1, Bool.true_and, h2, Bool.not_false, if_true] at h
      cases h
  · rw [Bool.not_eq_true] at h1
    simp only [h1, Bool.false_and, Bool.not_false, if_true] at h
    cases h

theorem selectorAt_of_lt : ∀ (U : List Gnaf.Selector) (k : Nat), k < U.length →
    ∃ s, Certification.selectorAt U k = some s
  | [], k, h => by simp at h
  | x :: U, 0, _ => ⟨x, by simp [Certification.selectorAt]⟩
  | x :: U, k+1, h => by
      obtain ⟨s, hs⟩ := selectorAt_of_lt U k (by simp at h; omega)
      exact ⟨s, by simpa [Certification.selectorAt] using hs⟩

/-- The semantic content of a certified lower bound, over the universe `U`. -/
structure LowerSem (req : Gnaf.Request) (g : Gnaf.Grammar) (U : List Gnaf.Selector) (strict : Bool)
    (bound : Nat) (attainers : List Nat) : Prop where
  nonempty : attainers ≠ []
  attained : ∀ F', req.machine.fuel ≤ F' → ∀ k, k ∈ attainers →
    ∃ s z, Certification.selectorAt U k = some s ∧ statusAt req g s F' = .admitted bound z
  bounded : ∀ F', req.machine.fuel ≤ F' → ∀ k s c z, Certification.selectorAt U k = some s →
    statusAt req g s F' = .admitted c z → bound ≤ c
  exact : strict = true → ∀ F', req.machine.fuel ≤ F' → ∀ k s c z, Certification.selectorAt U k = some s →
    statusAt req g s F' = .admitted c z → c = bound → k ∈ attainers

theorem lowerVerdict_sound (req : Gnaf.Request) (g : Gnaf.Grammar) (U : List Gnaf.Selector)
    (facts : List Fact) (hf : FactsSound req g U facts) (strict : Bool) (bound : Nat)
    (attainers : List Nat) (x : Certified)
    (h : Certification.lowerVerdict strict bound attainers facts = .certified x) :
    x = (if strict = true then Certified.argmin attainers bound else Certified.optimum attainers bound) ∧
    LowerSem req g U strict bound attainers := by
  obtain ⟨hasc, hbelow, hscan, hne, hx⟩ := lowerVerdict_inv h
  refine ⟨hx, ?_⟩
  have hnone := scanLower_none (strict := strict) (bound := bound) (att := attainers) facts 0 hscan
  have pos_ok : ∀ k f, Certification.factAt facts k = some f →
      Certification.lowerPosition strict bound (Gnaf.natIn k attainers) f k = none := by
    intro k f hk
    simpa using hnone k f hk
  -- every attainer's fact
  have att_fact : ∀ k, k ∈ attainers → ∃ s z, Certification.selectorAt U k = some s ∧
      ∃ f, Certification.factAt facts k = some f ∧ FactSound req g U s f ∧ f = Fact.admitted bound z := by
    intro k hk
    have hk' : k < U.length := by
      have := allBelow_lt attainers facts.length k hbelow hk
      rw [facts_length hf] at this; exact this
    obtain ⟨s, hsel⟩ := selectorAt_of_lt U k hk'
    obtain ⟨f, hfact, hsound⟩ := fact_of_member hf hsel
    have hok := pos_ok k f hfact
    have hin : Gnaf.natIn k attainers = true := (natIn_iff _ _).mpr hk
    cases f with
    | admitted c z =>
      simp only [Certification.lowerPosition, hin, if_true] at hok
      by_cases hc : Nat.beq c bound = true
      · have := Nat.eq_of_beq_eq_true hc
        subst this
        exact ⟨s, z, hsel, _, hfact, hsound, rfl⟩
      · simp [hc] at hok
        split at hok <;> simp at hok
    | inadmissible => simp [Certification.lowerPosition, hin] at hok
    | atLeast c z => simp [Certification.lowerPosition, hin] at hok
    | reduces p => simp [Certification.lowerPosition, hin] at hok
    | unresolved => simp [Certification.lowerPosition] at hok
    | unproven => simp [Certification.lowerPosition] at hok
  -- the bound holds for every admitted system, strict off the attainers
  have key : ∀ F', req.machine.fuel ≤ F' → ∀ c, ∀ k s z, Certification.selectorAt U k = some s →
      statusAt req g s F' = .admitted c z →
      bound ≤ c ∧ (strict = true → c = bound → Gnaf.natIn k attainers = true) := by
    intro F' hle c
    induction c using Nat.strongRecOn with
    | _ c ih =>
      intro k s z hsel hadm
      obtain ⟨f, hfact, hsound⟩ := fact_of_member hf hsel
      have hok := pos_ok k f hfact
      cases f with
      | inadmissible => exact absurd hadm (hsound F' hle c z)
      | admitted c z0 =>
        have h0 := hsound F' hle
        rw [hadm] at h0
        simp only [Gnaf.Status.admitted.injEq] at h0
        obtain ⟨rfl, rfl⟩ := h0
        by_cases hin : Gnaf.natIn k attainers = true
        · simp only [Certification.lowerPosition, hin, if_true] at hok
          by_cases hc : Nat.beq c bound = true
          · have := Nat.eq_of_beq_eq_true hc
            exact ⟨by omega, fun _ _ => hin⟩
          · simp [hc] at hok; split at hok <;> simp at hok
        · simp only [Bool.not_eq_true] at hin
          simp only [Certification.lowerPosition, hin, Bool.false_eq_true, if_false] at hok
          by_cases hlt : Nat.blt c bound = true
          · simp only [hlt, if_true] at hok; simp at hok
          · simp only [hlt, if_false] at hok
            have hle' : bound ≤ c := by
              rcases Nat.lt_or_ge c bound with hc | hc
              · exact absurd (Nat.blt_eq.mpr hc) hlt
              · exact hc
            refine ⟨hle', fun hst heq => ?_⟩
            subst heq
            simp [hst] at hok
      | atLeast c0 z0 =>
        obtain ⟨hcle, _⟩ := hsound F' hle c z hadm
        have hnin : Gnaf.natIn k attainers = false := by
          cases hin : Gnaf.natIn k attainers
          · rfl
          · simp [Certification.lowerPosition, hin] at hok
        simp only [Certification.lowerPosition, hnin, Bool.false_eq_true, if_false] at hok
        cases hst : strict
        · simp only [hst, Bool.false_eq_true, if_false] at hok
          have : bound ≤ c0 := by
            rcases Nat.lt_or_ge c0 bound with hc | hc
            · simp [Nat.ble_eq] at hok; omega
            · exact hc
          exact ⟨by omega, fun h => absurd h (by simp)⟩
        · simp only [hst, if_true] at hok
          have : bound < c0 := by
            rcases Nat.lt_or_ge bound c0 with hc | hc
            · exact hc
            · simp [Nat.blt_eq] at hok; omega
          exact ⟨by omega, fun _ heq => by omega⟩
      | reduces p =>
        obtain ⟨q, c', z', hq, hqadm, hlt, _⟩ := hsound F' hle c z hadm
        have hnin : Gnaf.natIn k attainers = false := by
          cases hin : Gnaf.natIn k attainers
          · rfl
          · simp [Certification.lowerPosition, hin] at hok
        have := ih c' hlt p q z' hq hqadm
        exact ⟨by omega, fun _ heq => by omega⟩
      | unresolved => simp [Certification.lowerPosition] at hok
      | unproven => simp [Certification.lowerPosition] at hok
  refine ⟨hne, ?_, ?_, ?_⟩
  · intro F' hle k hk
    obtain ⟨s, z, hsel, f, hfact, hsound, rfl⟩ := att_fact k hk
    exact ⟨s, z, hsel, hsound F' hle⟩
  · intro F' hle k s c z hsel hadm
    exact (key F' hle c k s z hsel hadm).1
  · intro hst F' hle k s c z hsel hadm heq
    exact (natIn_iff _ _).mp ((key F' hle c k s z hsel hadm).2 hst heq)
end LexLeanCertification
