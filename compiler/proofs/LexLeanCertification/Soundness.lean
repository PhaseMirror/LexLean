import LexLeanCertification.Check
set_option linter.unusedSimpArgs false
namespace LexLeanCertification
open LexLeanTarget.TargetSyntax LexLeanTarget.TargetSemantics LexLeanPreservation LexLeanTarget
open Certification (Fact Premise Verdict Certified Obligation Bridge Certificate)

/-- A system of the independent universe: every well-formed selector of the grammar is a position of its
expansion (`Gnaf.expandComplete`), so a bound over the expansion's positions is a bound over `U_sys`. -/
theorem universe_position (g : Gnaf.Grammar) (s : Gnaf.Selector) (hwf : Gnaf.wellFormed g s = true) :
    ∃ k, Certification.selectorAt (Gnaf.expand g) k = some s := by
  apply selectorAt_of_selectorIn
  rw [Gnaf.expandComplete]; exact hwf

/-- Theorem 11.1 (lower-bound attainment), as the checker certifies it: a certified optimum or argmin
bounds the cost of every admitted system of the request's grammar universe, at the request's fuel and at
every larger one, and a system attains the bound. -/
theorem lower_bound_attainment (req : Gnaf.Request) (cert : Certificate) (ids : List Nat) (bound : Nat)
    (h : Certification.check req cert = .certified (.optimum ids bound) ∨
         Certification.check req cert = .certified (.argmin ids bound)) :
    ∃ g, req.carrier = .grammar g ∧
      (∃ s, Gnaf.wellFormed g s = true ∧ ∃ z, statusAt req g s req.machine.fuel = .admitted bound z) ∧
      ∀ F', req.machine.fuel ≤ F' → ∀ s, Gnaf.wellFormed g s = true → ∀ c z,
        statusAt req g s F' = .admitted c z → bound ≤ c := by
  have hsem : ∃ g, req.carrier = .grammar g ∧ req.domain ≠ [] ∧
      ∃ strict, LowerSem req g (Gnaf.expand g) strict bound ids := by
    rcases h with h | h
    · obtain ⟨g, hc, hD, hs⟩ := check_sound req cert _ h
      exact ⟨g, hc, hD, false, hs⟩
    · obtain ⟨g, hc, hD, hs⟩ := check_sound req cert _ h
      exact ⟨g, hc, hD, true, hs⟩
  obtain ⟨g, hc, hD, strict, hs⟩ := hsem
  refine ⟨g, hc, ?_, ?_⟩
  · obtain ⟨k, hk⟩ : ∃ k, k ∈ ids := by
      cases hids : ids with
      | nil => exact absurd hids hs.nonempty
      | cons k rest => exact ⟨k, by simp⟩
    obtain ⟨s, z, hsel, hadm⟩ := hs.attained req.machine.fuel (Nat.le_refl _) k hk
    refine ⟨s, ?_, z, hadm⟩
    rw [← Gnaf.expandComplete]; exact selectorIn_of_selectorAt _ _ _ hsel
  · intro F' hle s hwf c z hadm
    obtain ⟨k, hk⟩ := universe_position g s hwf
    exact hs.bounded F' hle k s c z hk hadm

/-- The Pareto analogue: a certified frontier is exactly the frontier of the admitted systems of the grammar
universe at the request's fuel and every larger one. -/
theorem frontier_complete (req : Gnaf.Request) (cert : Certificate) (ms : List Nat)
    (h : Certification.check req cert = .certified (.frontier ms)) :
    ∃ g, req.carrier = .grammar g ∧ FrontierSem req g (Gnaf.expand g) ms ∧
      ∀ F', req.machine.fuel ≤ F' → ∀ s, Gnaf.wellFormed g s = true → ∀ c z,
        statusAt req g s F' = .admitted c z →
        (∃ k, Certification.selectorAt (Gnaf.expand g) k = some s ∧ k ∈ ms) ∨
        (∃ m sm cm zm, m ∈ ms ∧ Certification.selectorAt (Gnaf.expand g) m = some sm ∧
          statusAt req g sm F' = .admitted cm zm ∧ Sdom (cm, zm) (c, z)) := by
  obtain ⟨g, hc, hD, hs⟩ := check_sound req cert _ h
  refine ⟨g, hc, hs, ?_⟩
  intro F' hle s hwf c z hadm
  obtain ⟨k, hk⟩ := universe_position g s hwf
  rcases hs.complete F' hle k s c z hk hadm with hm | hd
  · exact Or.inl ⟨k, hk, hm⟩
  · exact Or.inr hd

/-- No theorem makes normality optimality: a normality certificate is never certified. -/
theorem normality_never_certifies (req : Gnaf.Request) (cert : Certificate) (normal : Nat)
    (hroute : cert.route = .normality normal) (x : Certified) :
    Certification.check req cert ≠ .certified x := by
  intro h
  obtain ⟨g, hc, hD, hs⟩ := check_sound req cert x h
  -- re-run the proof of `check_sound` at the route: the normality branch has no certified verdict
  unfold Certification.check at h
  split at h; · simp at h
  cases hval : Gnaf.validate req with
  | some r => rw [hval] at h; simp at h
  | none =>
    rw [hval] at h
    dsimp only at h
    rw [hc] at h
    dsimp only at h
    split at h; · simp at h
    split at h; · simp at h
    split at h; · simp at h
    split at h; · simp at h
    split at h; · simp at h
    split at h; · simp at h
    try dsimp only at h
    rw [hroute] at h
    dsimp only at h
    exact normalityVerdict_not_certified _ _ _ _ _ _ h

/-- A certified optimum is GNAF-normal (Theorem 11.7); normality is necessary and not sufficient. -/
theorem certified_optimum_is_normal (req : Gnaf.Request) (cert : Certificate) (ids : List Nat) (bound : Nat)
    (h : Certification.check req cert = .certified (.optimum ids bound) ∨
         Certification.check req cert = .certified (.argmin ids bound)) :
    ∃ g, req.carrier = .grammar g ∧ ∀ k ∈ ids, ∀ s, Certification.selectorAt (Gnaf.expand g) k = some s →
      Certification.isNormal req g s = true := by
  rcases h with h | h
  · obtain ⟨g, hc, hD, hs⟩ := check_sound req cert _ h
    exact ⟨g, hc, optimum_is_normal req g hD false bound ids hs⟩
  · obtain ⟨g, hc, hD, hs⟩ := check_sound req cert _ h
    exact ⟨g, hc, optimum_is_normal req g hD true bound ids hs⟩
end LexLeanCertification
