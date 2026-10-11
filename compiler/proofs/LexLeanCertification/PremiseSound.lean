import LexLeanCertification.Facts
set_option linter.unusedSimpArgs false
namespace LexLeanCertification
open LexLeanTarget.TargetSyntax LexLeanTarget.TargetSemantics LexLeanPreservation LexLeanTarget
open Certification (Fact Premise)

theorem allAgree_sublist_le {sys ref F F'} (hle : F ≤ F') {D S : List Value} {c x : Nat}
    (hS : S.Sublist D) (hx : AllAgree sys ref F S x) (hD : AllAgree sys ref F' D c) : x ≤ c := by
  obtain ⟨y, hy, hay⟩ := allAgree_sublist hD hS
  have := allAgree_unique (allAgree_stable hle hx) hay
  omega

theorem premiseFact_sound (req : Gnaf.Request) (g : Gnaf.Grammar) (U : List Gnaf.Selector)
    (s : Gnaf.Selector) (premise : Premise) (hD : req.domain ≠ []) :
    FactSound req g U s (Certification.premiseFact req g U s premise) := by
  cases premise with
  | excluded position =>
    unfold Certification.premiseFact
    dsimp only
    cases hv : Certification.valueAt req.domain position with
    | none => exact trivial
    | some argument =>
      dsimp only
      cases hs : Gnaf.statusOn (Gnaf.realize g s) req.reference req.machine.fuel [argument] with
      | admitted _ _ => exact trivial
      | unresolved => exact trivial
      | inadmissible =>
        intro F' hle c z h
        obtain ⟨a, ha, hd⟩ := inadmissible_witness _ hs
        have ha' : a ∈ req.domain := by
          have : a = argument := by simpa using ha
          rw [this]; exact valueAt_mem _ _ _ hv
        obtain ⟨steps, hs', _, _⟩ := statusAt_admitted h
        exact disagree_not_admitted hle ha' hd _ _ hs'
  | paysAtLeast positions =>
    unfold Certification.premiseFact
    dsimp only
    split
    · cases hs : Gnaf.statusOn (Gnaf.realize g s) req.reference req.machine.fuel
          (Certification.pickFrom req.domain 0 positions) with
      | inadmissible => exact trivial
      | unresolved => exact trivial
      | admitted steps size =>
        obtain ⟨_, hall⟩ := allAgree_of_admitted _ hs
        intro F' hle c' z' h
        obtain ⟨steps', hs', rfl, rfl⟩ := statusAt_admitted h
        obtain ⟨_, hall'⟩ := allAgree_of_admitted _ hs'
        have := allAgree_sublist_le hle (pickFrom_sublist req.domain 0 positions) hall hall'
        exact ⟨by omega, rfl⟩
    · exact trivial
  | measured =>
    unfold Certification.premiseFact
    dsimp only
    cases hm : Gnaf.status req g s with
    | admitted c z => exact measured_sound req g s c z hm
    | inadmissible => exact measured_inadmissible req g s hm
    | unresolved => exact trivial
  | reduced schema =>
    unfold Certification.premiseFact
    dsimp only
    cases hrw : Certification.rewrite schema s with
    | none => exact trivial
    | some target =>
      dsimp only
      cases hp : Certification.positionOf U target 0 with
      | none => exact trivial
      | some tp =>
        dsimp only
        split
        · rename_i hc
          simp only [Certification.improves, hrw, Bool.and_eq_true] at hc
          obtain ⟨k, hk, hat⟩ := positionOf_at U target 0 tp hp
          have hk' : tp = k := by omega
          subst hk'
          intro F' hle c z h
          obtain ⟨c', z', hq, hlt, hz⟩ := schema_cheaper req g s target schema hrw hc.1
            (Nat.ble_eq.mp hc.2.1) (Nat.ble_eq.mp hc.2.2) hD F' c z h
          exact ⟨target, c', z', hat, hq, hlt, hz⟩
        · exact trivial
end LexLeanCertification
