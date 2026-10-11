import LexLeanCertification.Representation
set_option linter.unusedSimpArgs false
namespace LexLeanCertification
open LexLeanTarget.TargetSyntax LexLeanTarget.TargetSemantics LexLeanPreservation LexLeanTarget
open Certification (Fact Premise Verdict Certified Obligation Bridge)

/-- Local normality is never an optimality result: the normality route's verdict is never certified. -/
theorem normalityVerdict_not_certified (req : Gnaf.Request) (g : Gnaf.Grammar) (selectors : List Gnaf.Selector)
    (normal : Nat) (facts : List Fact) (x : Certified) :
    Certification.normalityVerdict req g selectors normal facts ≠ .certified x := by
  intro h
  unfold Certification.normalityVerdict at h
  cases hf : Certification.factAt facts normal with
  | none => rw [hf] at h; simp at h
  | some f =>
    rw [hf] at h
    cases f with
    | admitted c z =>
      dsimp only at h
      cases hs : Certification.selectorAt selectors normal with
      | none => rw [hs] at h; simp at h
      | some sel =>
        rw [hs] at h
        dsimp only at h
        split at h <;> simp at h
    | inadmissible => simp at h
    | atLeast _ _ => simp at h
    | reduces _ => simp at h
    | unresolved => simp at h
    | unproven => simp at h

/-- A rewriting of a well-formed system is well-formed. -/
theorem wellFormed_rewrite (g : Gnaf.Grammar) (schema : Certification.Schema) (s q : Gnaf.Selector)
    (hwf : Gnaf.wellFormed g s = true) (hrw : Certification.rewrite schema s = some q) :
    Gnaf.wellFormed g q = true := by
  cases s with
  | fixed p => cases schema <;> simp [Certification.rewrite] at hrw
  | dispatch t sm lg =>
    simp only [Gnaf.wellFormed, Bool.and_eq_true] at hwf
    obtain ⟨_, hsm, hlg⟩ := hwf
    cases schema with
    | collapse =>
      simp only [Certification.rewrite] at hrw
      by_cases hb : Nat.beq sm lg = true
      · simp only [hb, if_true, Option.some.injEq] at hrw; subst hrw
        simp only [Gnaf.wellFormed]; exact hsm
      · simp [hb] at hrw
    | specializeSmall =>
      simp only [Certification.rewrite, Option.some.injEq] at hrw; subst hrw
      simp only [Gnaf.wellFormed]; exact hsm
    | specializeLarge =>
      simp only [Certification.rewrite, Option.some.injEq] at hrw; subst hrw
      simp only [Gnaf.wellFormed]; exact hlg

theorem selectorIn_of_selectorAt : ∀ (U : List Gnaf.Selector) (k : Nat) (s : Gnaf.Selector),
    Certification.selectorAt U k = some s → Gnaf.selectorIn s U = true
  | [], k, s, h => by simp [Certification.selectorAt] at h
  | x :: U, 0, s, h => by
      simp only [Certification.selectorAt, Option.some.injEq] at h
      subst h
      simp [Gnaf.selectorIn, selectorEq_refl]
  | x :: U, k+1, s, h => by
      simp only [Certification.selectorAt] at h
      simp [Gnaf.selectorIn, selectorIn_of_selectorAt U k s h]

/-- Theorem 11.7: a certified optimum is GNAF-normal; the converse is false (the normality route and the
negative fixtures). -/
theorem optimum_is_normal (req : Gnaf.Request) (g : Gnaf.Grammar) (hD : req.domain ≠ [])
    (strict : Bool) (bound : Nat) (attainers : List Nat)
    (hsem : LowerSem req g (Gnaf.expand g) strict bound attainers) :
    ∀ k ∈ attainers, ∀ s, Certification.selectorAt (Gnaf.expand g) k = some s →
      Certification.isNormal req g s = true := by
  intro k hk s hsel
  obtain ⟨s', z, hsel', hadm⟩ := hsem.attained req.machine.fuel (Nat.le_refl _) k hk
  rw [hsel] at hsel'; cases hsel'
  have hwf : Gnaf.wellFormed g s = true := by
    rw [← Gnaf.expandComplete]; exact selectorIn_of_selectorAt _ _ _ hsel
  have no_improve : ∀ schema, Certification.improves req g s schema = false := by
    intro schema
    by_cases himp : Certification.improves req g s schema = true
    · exfalso
      unfold Certification.improves at himp
      cases hrw : Certification.rewrite schema s with
      | none => rw [hrw] at himp; simp at himp
      | some q =>
        rw [hrw] at himp
        simp only [Bool.and_eq_true] at himp
        have hwq := wellFormed_rewrite g schema s q hwf hrw
        obtain ⟨kq, hkq⟩ := selectorAt_of_selectorIn _ q (by rw [Gnaf.expandComplete]; exact hwq)
        obtain ⟨c', z', hq, hlt, _⟩ := schema_cheaper req g s q schema hrw himp.1
          (Nat.ble_eq.mp himp.2.1) (Nat.ble_eq.mp himp.2.2) hD req.machine.fuel bound z hadm
        have := hsem.bounded req.machine.fuel (Nat.le_refl _) kq q c' z' hkq hq
        omega
    · simpa using himp
  unfold Certification.isNormal
  simp [no_improve]
end LexLeanCertification
