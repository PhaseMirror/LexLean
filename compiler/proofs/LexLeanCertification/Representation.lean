import LexLeanCertification.Frontier
set_option linter.unusedSimpArgs false
namespace LexLeanCertification
open LexLeanTarget.TargetSyntax LexLeanTarget.TargetSemantics LexLeanPreservation LexLeanTarget
open Certification (Fact Premise Verdict Certified Obligation Bridge)

theorem natAt_sizesOf (g : Gnaf.Grammar) : ∀ (U : List Gnaf.Selector) (k v : Nat),
    Certification.natAt (Certification.sizesOf g U) k = some v ↔
      ∃ s, Certification.selectorAt U k = some s ∧ v = Certification.staticSize g s
  | [], k, v => by simp [Certification.sizesOf, Certification.natAt, Certification.selectorAt]
  | x :: U, 0, v => by
      simp [Certification.sizesOf, Certification.natAt, Certification.selectorAt, eq_comm]
  | x :: U, k+1, v => by
      simp only [Certification.sizesOf, Certification.natAt, Certification.selectorAt]
      exact natAt_sizesOf g U k v

theorem firstSmaller_none : ∀ (sizes : List Nat) (limit pos : Nat),
    Certification.firstSmaller sizes limit pos = none → ∀ i v, Certification.natAt sizes i = some v → limit ≤ v
  | [], _, _, _, i, v, h => by simp [Certification.natAt] at h
  | x :: xs, limit, pos, hs, i, v, h => by
      simp only [Certification.firstSmaller] at hs
      by_cases hx : Nat.blt x limit = true
      · simp [hx] at hs
      · have hx' : Nat.blt x limit = false := by
          cases h : Nat.blt x limit
          · rfl
          · exact absurd h hx
        simp only [hx', Bool.false_eq_true, if_false] at hs
        cases i with
        | zero =>
          simp only [Certification.natAt, Option.some.injEq] at h
          subst h
          exact Nat.not_lt.mp (fun hlt => hx (Nat.blt_eq.mpr hlt))
        | succ i =>
          simp only [Certification.natAt] at h
          exact firstSmaller_none xs limit (pos+1) hs i v h

theorem scanBridge_none {b : Bridge} : ∀ (facts : List Fact) (sizes : List Nat) (pos : Nat),
    Certification.scanBridge b facts sizes pos = none →
    ∀ i f, Certification.factAt facts i = some f →
      ∃ v, Certification.natAt sizes i = some v ∧ Certification.bridgePosition b f v (pos + i) = none
  | [], _, _, _, i, f, h => by simp [Certification.factAt] at h
  | x :: fs, sizes, pos, hscan, i, f, h => by
      cases sizes with
      | nil => simp [Certification.scanBridge] at hscan
      | cons v vs =>
        simp only [Certification.scanBridge] at hscan
        cases hl : Certification.bridgePosition b x v pos with
        | some o => rw [hl] at hscan; simp at hscan
        | none =>
          rw [hl] at hscan
          simp only [] at hscan
          cases i with
          | zero =>
            simp only [Certification.factAt, Option.some.injEq] at h
            subst h
            exact ⟨v, by simp [Certification.natAt], by simpa using hl⟩
          | succ i =>
            simp only [Certification.factAt] at h
            obtain ⟨w, hw, hb⟩ := scanBridge_none fs vs (pos + 1) hscan i f h
            have e : pos + 1 + i = pos + (i + 1) := by omega
            rw [e] at hb
            exact ⟨w, by simpa [Certification.natAt] using hw, hb⟩

theorem representationVerdict_sound (req : Gnaf.Request) (g : Gnaf.Grammar) (U : List Gnaf.Selector)
    (facts : List Fact) (hf : FactsSound req g U facts) (normal : Nat) (bridge : Option Bridge)
    (x : Certified)
    (h : Certification.representationVerdict normal bridge facts (Certification.sizesOf g U) = .certified x) :
    ∃ (b : Bridge) (normalSize cost : Nat), bridge = some b ∧
      Certification.natAt (Certification.sizesOf g U) normal = some normalSize ∧
      x = Certified.optimum [normal] cost ∧ cost = b.slope * normalSize + b.offset ∧
      LowerSem req g U false cost [normal] := by
  unfold Certification.representationVerdict at h
  cases hn : Certification.natAt (Certification.sizesOf g U) normal with
  | none => rw [hn] at h; simp at h
  | some normalSize =>
    rw [hn] at h
    dsimp only at h
    cases hs : Certification.firstSmaller (Certification.sizesOf g U) normalSize 0 with
    | some o => rw [hs] at h; simp at h
    | none =>
      rw [hs] at h
      dsimp only at h
      cases hb : bridge with
      | none => rw [hb] at h; simp at h
      | some b =>
        rw [hb] at h
        dsimp only at h
        cases hscan : Certification.scanBridge b facts (Certification.sizesOf g U) 0 with
        | some o => rw [hscan] at h; simp at h
        | none =>
          rw [hscan] at h
          dsimp only at h
          cases hfn : Certification.factAt facts normal with
          | none => rw [hfn] at h; simp at h
          | some fn =>
            rw [hfn] at h
            dsimp only at h
            cases fn with
            | admitted cost size =>
              dsimp only at h
              by_cases hc : cost.beq (Certification.LexLeanRuntime.multiply b.slope normalSize + b.offset) = true
              · simp only [hc, if_true, Verdict.certified.injEq] at h
                subst h
                have hcost : cost = b.slope * normalSize + b.offset := Nat.eq_of_beq_eq_true hc
                refine ⟨b, normalSize, cost, rfl, rfl, rfl, hcost, ?_⟩
                obtain ⟨sN, hselN, _⟩ := (natAt_sizesOf g U normal normalSize).mp hn
                obtain ⟨fN, hfactN, hsoundN⟩ := fact_of_member hf hselN
                rw [hfn] at hfactN; cases hfactN
                refine ⟨by simp, ?_, ?_, fun hh => by cases hh⟩
                · intro F' hle k hk
                  have : k = normal := by simpa using hk
                  subst this
                  exact ⟨sN, size, hselN, hsoundN F' hle⟩
                · intro F' hle k s c z hsel hadm
                  obtain ⟨f, hfact, hsound⟩ := fact_of_member hf hsel
                  obtain ⟨v, hv, hbp⟩ := scanBridge_none facts _ 0 hscan k f hfact
                  obtain ⟨s', hsel', hvs⟩ := (natAt_sizesOf g U k v).mp hv
                  rw [hsel] at hsel'; cases hsel'
                  have hge := firstSmaller_none _ _ 0 hs k v hv
                  have hmul : b.slope * normalSize ≤ b.slope * v := Nat.mul_le_mul_left _ hge
                  cases f with
                  | inadmissible => exact absurd hadm (hsound F' hle c z)
                  | admitted c z =>
                    have h0 := hsound F' hle
                    rw [hadm] at h0
                    simp only [Gnaf.Status.admitted.injEq] at h0
                    obtain ⟨rfl, rfl⟩ := h0
                    simp only [Certification.bridgePosition, Certification.LexLeanRuntime.multiply] at hbp
                    by_cases hle' : Nat.ble (b.slope * v + b.offset) c = true
                    · have := Nat.ble_eq.mp hle'
                      omega
                    · simp [hle'] at hbp
                  | atLeast c0 z0 =>
                    obtain ⟨hcle, _⟩ := hsound F' hle c z hadm
                    simp only [Certification.bridgePosition, Certification.LexLeanRuntime.multiply] at hbp
                    by_cases hle' : Nat.ble (b.slope * v + b.offset) c0 = true
                    · have := Nat.ble_eq.mp hle'
                      omega
                    · simp [hle'] at hbp
                  | reduces p => simp [Certification.bridgePosition] at hbp
                  | unresolved => simp [Certification.bridgePosition] at hbp
                  | unproven => simp [Certification.bridgePosition] at hbp
              · simp [hc] at h
            | inadmissible => simp at h
            | atLeast _ _ => simp at h
            | reduces _ => simp at h
            | unresolved => simp at h
            | unproven => simp at h
end LexLeanCertification
