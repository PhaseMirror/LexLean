import LexLeanCertification.PremiseSound
set_option linter.unusedSimpArgs false
namespace LexLeanCertification
open LexLeanTarget.TargetSyntax LexLeanTarget.TargetSemantics LexLeanPreservation LexLeanTarget
open Certification (Fact Premise)

/-- Pointwise relation between two lists of equal length. -/
inductive Zip {α β : Type} (R : α → β → Prop) : List α → List β → Prop where
  | nil : Zip R [] []
  | cons {a : α} {b : β} {as : List α} {bs : List β} : R a b → Zip R as bs → Zip R (a :: as) (b :: bs)

theorem Zip.length_eq {α β : Type} {R : α → β → Prop} {as : List α} {bs : List β}
    (h : Zip R as bs) : as.length = bs.length := by
  induction h with
  | nil => rfl
  | cons _ _ ih => simp [ih]

/-- Every system of the universe with the fact its premise established. -/
def FactsSound (req : Gnaf.Request) (g : Gnaf.Grammar) (U : List Gnaf.Selector)
    (facts : List Fact) : Prop :=
  Zip (fun s f => FactSound req g U s f) U facts

theorem factsOf_sound (req : Gnaf.Request) (g : Gnaf.Grammar) (all : List Gnaf.Selector)
    (hD : req.domain ≠ []) : ∀ (selectors : List Gnaf.Selector) (premises : List Premise),
    Zip (fun s f => FactSound req g all s f) selectors
      (Certification.factsOf req g all selectors premises)
  | [], _ => by simp only [Certification.factsOf]; exact Zip.nil
  | s :: rest, [] => by
      simp only [Certification.factsOf]
      exact Zip.cons (trivial : FactSound req g all s Fact.unproven)
        (factsOf_sound req g all hD rest [])
  | s :: rest, p :: ps => by
      simp only [Certification.factsOf]
      exact Zip.cons (premiseFact_sound req g all s p hD) (factsOf_sound req g all hD rest ps)

theorem forall₂_at {R : Gnaf.Selector → Fact → Prop} : ∀ {U : List Gnaf.Selector} {facts : List Fact},
    Zip R U facts → ∀ k,
      (∀ s, Certification.selectorAt U k = some s → ∃ f, Certification.factAt facts k = some f ∧ R s f) ∧
      (∀ f, Certification.factAt facts k = some f → ∃ s, Certification.selectorAt U k = some s ∧ R s f)
  | _, _, h, k => by
      induction h generalizing k with
      | nil => exact ⟨fun s h => by simp [Certification.selectorAt] at h,
          fun f h => by simp [Certification.factAt] at h⟩
      | @cons s f U fs hr hrest ih =>
      cases k with
      | zero =>
        refine ⟨fun s' h => ?_, fun f' h => ?_⟩
        · simp [Certification.selectorAt] at h; subst h; exact ⟨f, by simp [Certification.factAt], hr⟩
        · simp [Certification.factAt] at h; subst h; exact ⟨s, by simp [Certification.selectorAt], hr⟩
      | succ k =>
        obtain ⟨h1, h2⟩ := ih k
        exact ⟨fun s' h => by simpa [Certification.selectorAt, Certification.factAt] using h1 s' (by simpa [Certification.selectorAt] using h),
               fun f' h => by simpa [Certification.selectorAt, Certification.factAt] using h2 f' (by simpa [Certification.factAt] using h)⟩

theorem factAt_lt : ∀ (facts : List Fact) (k : Nat) (f : Fact), Certification.factAt facts k = some f → k < facts.length
  | [], k, f, h => by simp [Certification.factAt] at h
  | x :: fs, 0, f, h => by simp
  | x :: fs, k+1, f, h => by
      simp only [Certification.factAt] at h
      have := factAt_lt fs k f h
      simp; omega

/-- A sound fact at every position of a universe is a sound fact at the universe's members. -/
theorem fact_of_member {req g U facts} (hf : FactsSound req g U facts) {k s}
    (hs : Certification.selectorAt U k = some s) :
    ∃ f, Certification.factAt facts k = some f ∧ FactSound req g U s f :=
  (forall₂_at hf k).1 s hs

theorem selector_of_fact {req g U facts} (hf : FactsSound req g U facts) {k f}
    (hfact : Certification.factAt facts k = some f) :
    ∃ s, Certification.selectorAt U k = some s ∧ FactSound req g U s f :=
  (forall₂_at hf k).2 f hfact

/-- Lists of facts in step with the universe have its length. -/
theorem facts_length {req g U facts} (hf : FactsSound req g U facts) : facts.length = U.length :=
  (Zip.length_eq hf).symm
end LexLeanCertification
