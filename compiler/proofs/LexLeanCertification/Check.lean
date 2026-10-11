import LexLeanCertification.Normality
set_option linter.unusedSimpArgs false
namespace LexLeanCertification
open LexLeanTarget.TargetSyntax LexLeanTarget.TargetSemantics LexLeanPreservation LexLeanTarget
open Certification (Fact Premise Verdict Certified Obligation Bridge Certificate)

theorem validate_domain {req : Gnaf.Request} (h : Gnaf.validate req = none) : req.domain ≠ [] := by
  intro hd
  simp [Gnaf.validate, hd] at h

/-- What a certified result says about the universe of the request's grammar. -/
def CertifiedSem (req : Gnaf.Request) (g : Gnaf.Grammar) : Certified → Prop
  | .optimum ids b => LowerSem req g (Gnaf.expand g) false b ids
  | .argmin ids b => LowerSem req g (Gnaf.expand g) true b ids
  | .frontier ms => FrontierSem req g (Gnaf.expand g) ms

theorem sem_of_strict {req g} (strict : Bool) (ids : List Nat) (b : Nat)
    (hsem : LowerSem req g (Gnaf.expand g) strict b ids) :
    CertifiedSem req g (if strict = true then Certified.argmin ids b else Certified.optimum ids b) := by
  cases strict <;> simpa [CertifiedSem] using hsem

theorem reductionVerdict_sound (req : Gnaf.Request) (g : Gnaf.Grammar) (U : List Gnaf.Selector)
    (facts : List Fact) (hf : FactsSound req g U facts) (shape : Nat) (x : Certified)
    (h : Certification.reductionVerdict shape facts = .certified x) :
    (∃ strict ids b, x = (if strict = true then Certified.argmin ids b else Certified.optimum ids b) ∧
        LowerSem req g U strict b ids) ∨ (∃ ms, x = Certified.frontier ms ∧ FrontierSem req g U ms) := by
  unfold Certification.reductionVerdict at h
  cases hs : Certification.scanReduction facts 0 with
  | some o => rw [hs] at h; simp at h
  | none =>
    rw [hs] at h
    dsimp only at h
    by_cases h3 : Nat.blt shape 3 = true
    · simp only [h3, if_true] at h
      cases hm : Gnaf.argmin Nat (Gnaf.scalarRows (Certification.factRows facts 0)) with
      | none => rw [hm] at h; simp at h
      | some found =>
        rw [hm] at h
        dsimp only at h
        obtain ⟨hx, hsem⟩ := lowerVerdict_sound req g U facts hf _ _ _ x h
        exact Or.inl ⟨_, _, _, hx, hsem⟩
    · simp only [h3, Bool.false_eq_true, if_false] at h
      cases hr : Certification.factRows facts 0 with
      | nil => rw [hr] at h; simp at h
      | cons row rest =>
        rw [hr] at h
        dsimp only at h
        obtain ⟨hx, hsem⟩ := frontierVerdict_sound req g U facts hf _ x h
        exact Or.inr ⟨_, hx, hsem⟩

theorem check_sound (req : Gnaf.Request) (cert : Certificate) (x : Certified)
    (h : Certification.check req cert = .certified x) :
    ∃ g, req.carrier = .grammar g ∧ req.domain ≠ [] ∧ CertifiedSem req g x := by
  unfold Certification.check at h
  split at h
  · simp at h
  · cases hval : Gnaf.validate req with
    | some r => rw [hval] at h; simp at h
    | none =>
      rw [hval] at h
      dsimp only at h
      have hD := validate_domain hval
      cases hcar : req.carrier with
      | grammar g =>
        rw [hcar] at h
        dsimp only at h
        refine ⟨g, rfl, hD, ?_⟩
        split at h; · simp at h
        split at h; · simp at h
        split at h; · simp at h
        split at h; · simp at h
        split at h; · simp at h
        split at h; · simp at h
        try dsimp only at h
        have hf := factsOf_sound req g (Gnaf.expand g) hD (Gnaf.expand g) cert.premises
        generalize Certification.factsOf req g (Gnaf.expand g) (Gnaf.expand g) cert.premises = facts at h hf
        cases hroute : cert.route with
        | lowerBound bound attainers =>
          rw [hroute] at h
          dsimp only at h
          split at h
          · obtain ⟨hx, hsem⟩ := lowerVerdict_sound req g _ facts hf _ _ _ x h
            rw [hx]; exact sem_of_strict _ _ _ hsem
          · simp at h
        | frontierCover members =>
          rw [hroute] at h
          dsimp only at h
          split at h
          · obtain ⟨hx, hsem⟩ := frontierVerdict_sound req g _ facts hf _ x h
            subst hx; exact hsem
          · simp at h
        | reduction =>
          rw [hroute] at h
          dsimp only at h
          rcases reductionVerdict_sound req g _ facts hf _ x h with ⟨strict, ids, b, hx, hsem⟩ | ⟨ms, hx, hsem⟩
          · rw [hx]; exact sem_of_strict _ _ _ hsem
          · subst hx; exact hsem
        | representation normal bridge =>
          rw [hroute] at h
          dsimp only at h
          split at h
          · obtain ⟨b, ns, cost, _, _, hx, _, hsem⟩ := representationVerdict_sound req g _ facts hf normal bridge x h
            subst hx; exact hsem
          · simp at h
        | normality normal =>
          rw [hroute] at h
          dsimp only at h
          exact absurd h (normalityVerdict_not_certified _ _ _ _ _ _)
      | internalPlans => rw [hcar] at h; simp at h
      | optimizerOutput => rw [hcar] at h; simp at h
      | discovered _ => rw [hcar] at h; simp at h
      | cached => rw [hcar] at h; simp at h
end LexLeanCertification
