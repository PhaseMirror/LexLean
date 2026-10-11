import LexLeanCertification.Lower
set_option linter.unusedSimpArgs false
namespace LexLeanCertification
open LexLeanTarget.TargetSyntax LexLeanTarget.TargetSemantics LexLeanPreservation LexLeanTarget
open Certification (Fact Premise Verdict Certified Obligation)

/-- Strict dominance of (steps, size) pairs: no worse in both, better in one. -/
def Sdom (a b : Nat × Nat) : Prop := a.1 ≤ b.1 ∧ a.2 ≤ b.2 ∧ ¬(b.1 ≤ a.1 ∧ b.2 ≤ a.2)

theorem Sdom.trans {a b c : Nat × Nat} (h1 : Sdom a b) (h2 : Sdom b c) : Sdom a c := by
  obtain ⟨h1a, h1b, h1c⟩ := h1
  obtain ⟨h2a, h2b, h2c⟩ := h2
  refine ⟨by omega, by omega, ?_⟩
  intro ⟨x, y⟩
  apply h1c
  constructor <;> omega

theorem dominates_pair (c z c' z' : Nat) :
    Gnaf.dominates [c, z] [c', z'] = true ↔ Sdom (c, z) (c', z') := by
  simp [Gnaf.dominates, Gnaf.weaklyBelow, Sdom, Bool.eq_false_iff, Nat.ble_eq]
  omega

theorem dominatedIn_iff (cost : List Nat) :
    ∀ (rows : List (Nat × List Nat)), Gnaf.dominatedIn Nat cost rows = true ↔
      ∃ r ∈ rows, Gnaf.dominates r.2 cost = true
  | [] => by simp [Gnaf.dominatedIn]
  | r :: rows => by
      simp only [Gnaf.dominatedIn, Bool.or_eq_true, List.mem_cons, dominatedIn_iff cost rows]
      constructor
      · rintro (h | ⟨r', hr', h⟩)
        · exact ⟨r, Or.inl rfl, h⟩
        · exact ⟨r', Or.inr hr', h⟩
      · rintro ⟨r', hr' | hr', h⟩
        · subst hr'; exact Or.inl h
        · exact Or.inr ⟨r', hr', h⟩

theorem mem_factRows : ∀ (facts : List Fact) (p : Nat) (r : Nat × List Nat),
    r ∈ Certification.factRows facts p ↔
      ∃ i c z, Certification.factAt facts i = some (Fact.admitted c z) ∧ r = (p + i, [c, z])
  | [], p, r => by simp [Certification.factRows, Certification.factAt]
  | f :: fs, p, r => by
      constructor
      · intro h
        cases f with
        | admitted c z =>
          simp only [Certification.factRows, List.mem_cons] at h
          rcases h with rfl | h
          · exact ⟨0, c, z, by simp [Certification.factAt], by simp⟩
          · obtain ⟨i, c', z', hi, rfl⟩ := (mem_factRows fs (p+1) r).mp h
            exact ⟨i+1, c', z', by simpa [Certification.factAt] using hi, by simp; omega⟩
        | inadmissible | atLeast _ _ | reduces _ | unresolved | unproven =>
          all_goals
            simp only [Certification.factRows] at h
            obtain ⟨i, c', z', hi, rfl⟩ := (mem_factRows fs (p+1) r).mp h
            exact ⟨i+1, c', z', by simpa [Certification.factAt] using hi, by simp; omega⟩
      · rintro ⟨i, c, z, hi, rfl⟩
        cases i with
        | zero =>
          simp only [Certification.factAt, Option.some.injEq] at hi
          subst hi
          simp [Certification.factRows]
        | succ i =>
          simp only [Certification.factAt] at hi
          have := (mem_factRows fs (p+1) (p + 1 + i, [c, z])).mpr ⟨i, c, z, hi, rfl⟩
          have e : p + (i + 1) = p + 1 + i := by omega
          rw [e]
          cases f <;> simp only [Certification.factRows, List.mem_cons] <;> simp [this]

theorem mem_rowsOfMembers : ∀ (rows : List (Nat × List Nat)) (m : List Nat) (r : Nat × List Nat),
    r ∈ Certification.rowsOfMembers rows m ↔ r ∈ rows ∧ r.1 ∈ m
  | [], m, r => by simp [Certification.rowsOfMembers]
  | x :: rows, m, r => by
      simp only [Certification.rowsOfMembers]
      by_cases h : Gnaf.natIn x.1 m = true
      · simp only [h, if_true, List.mem_cons, mem_rowsOfMembers rows m r]
        have hx := (natIn_iff _ _).mp h
        constructor
        · rintro (rfl | ⟨a, b⟩)
          · exact ⟨Or.inl rfl, hx⟩
          · exact ⟨Or.inr a, b⟩
        · rintro ⟨rfl | a, b⟩
          · exact Or.inl rfl
          · exact Or.inr ⟨a, b⟩
      · simp only [h, if_false, Bool.not_eq_true, Bool.false_eq_true, mem_rowsOfMembers rows m r, List.mem_cons]
        have hx : x.1 ∉ m := fun hh => by simp [(natIn_iff _ _).mpr hh] at h
        constructor
        · rintro ⟨a, b⟩; exact ⟨Or.inr a, b⟩
        · rintro ⟨rfl | a, b⟩
          · exact absurd b hx
          · exact ⟨a, b⟩

theorem scanFrontier_none {members rows} : ∀ (facts : List Fact) (pos : Nat),
    Certification.scanFrontier members rows facts pos = none →
    ∀ i f, Certification.factAt facts i = some f →
      Certification.frontierPosition (Gnaf.natIn (pos + i) members) rows f (pos + i) = none
  | [], _, _, i, f, h => by simp [Certification.factAt] at h
  | x :: fs, pos, hscan, i, f, h => by
      simp only [Certification.scanFrontier] at hscan
      cases hl : Certification.frontierPosition (Gnaf.natIn pos members) rows x pos with
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
          have := scanFrontier_none fs (pos + 1) hscan i f h
          have e : pos + 1 + i = pos + (i + 1) := by omega
          rw [e] at this; exact this

theorem frontierVerdict_inv {members facts x}
    (h : Certification.frontierVerdict members facts = .certified x) :
    Certification.ascending members 0 = true ∧ Certification.allBelow members facts.length = true ∧
    Certification.scanFrontier members (Certification.rowsOfMembers (Certification.factRows facts 0) members)
      facts 0 = none ∧ members ≠ [] ∧ x = Certified.frontier members := by
  unfold Certification.frontierVerdict at h
  dsimp only at h
  have hlen : Certification.LexLeanRuntime.length facts = facts.length := rfl
  rw [hlen] at h
  by_cases h1 : Certification.ascending members 0 = true
  · by_cases h2 : Certification.allBelow members facts.length = true
    · simp only [h1, h2, Bool.and_self, Bool.not_true, Bool.false_eq_true, if_false] at h
      cases hs : Certification.scanFrontier members
          (Certification.rowsOfMembers (Certification.factRows facts 0) members) facts 0 with
      | some o => rw [hs] at h; simp at h
      | none =>
        rw [hs] at h
        simp only [] at h
        cases members with
        | nil => simp at h
        | cons a rest =>
          simp only [Verdict.certified.injEq] at h
          exact ⟨h1, h2, rfl, List.cons_ne_nil _ _, h.symm⟩
    · rw [Bool.not_eq_true] at h2
      simp only [h1, Bool.true_and, h2, Bool.not_false, if_true] at h
      cases h
  · rw [Bool.not_eq_true] at h1
    simp only [h1, Bool.false_and, Bool.not_false, if_true] at h
    cases h

/-- The semantic content of a certified frontier, over the universe `U`. -/
structure FrontierSem (req : Gnaf.Request) (g : Gnaf.Grammar) (U : List Gnaf.Selector)
    (members : List Nat) : Prop where
  nonempty : members ≠ []
  admitted : ∀ F', req.machine.fuel ≤ F' → ∀ k, k ∈ members →
    ∃ s c z, Certification.selectorAt U k = some s ∧ statusAt req g s F' = .admitted c z
  nondominated : ∀ F', req.machine.fuel ≤ F' → ∀ k s c z, k ∈ members →
    Certification.selectorAt U k = some s → statusAt req g s F' = .admitted c z →
    ∀ k' s' c' z', Certification.selectorAt U k' = some s' → statusAt req g s' F' = .admitted c' z' →
      ¬Sdom (c', z') (c, z)
  complete : ∀ F', req.machine.fuel ≤ F' → ∀ k s c z, Certification.selectorAt U k = some s →
    statusAt req g s F' = .admitted c z →
    k ∈ members ∨ ∃ m sm cm zm, m ∈ members ∧ Certification.selectorAt U m = some sm ∧
      statusAt req g sm F' = .admitted cm zm ∧ Sdom (cm, zm) (c, z)


theorem frontierVerdict_sound (req : Gnaf.Request) (g : Gnaf.Grammar) (U : List Gnaf.Selector)
    (facts : List Fact) (hf : FactsSound req g U facts) (members : List Nat) (x : Certified)
    (h : Certification.frontierVerdict members facts = .certified x) :
    x = Certified.frontier members ∧ FrontierSem req g U members := by
  obtain ⟨hasc, hbelow, hscan, hne, hx⟩ := frontierVerdict_inv h
  refine ⟨hx, ?_⟩
  generalize hrows : Certification.rowsOfMembers (Certification.factRows facts 0) members = rows at hscan
  have hnone := scanFrontier_none (members := members) (rows := rows) facts 0 hscan
  have pos_ok : ∀ k f, Certification.factAt facts k = some f →
      Certification.frontierPosition (Gnaf.natIn k members) rows f k = none := by
    intro k f hk; simpa using hnone k f hk
  have hlen := facts_length hf
  -- the member rows
  have row_iff : ∀ r, r ∈ rows ↔ ∃ m cm zm, m ∈ members ∧
      Certification.factAt facts m = some (Fact.admitted cm zm) ∧ r = (m, [cm, zm]) := by
    intro r
    rw [← hrows, mem_rowsOfMembers, mem_factRows]
    constructor
    · rintro ⟨⟨i, c, z, hi, rfl⟩, hm⟩
      simp only [Nat.zero_add] at hi hm ⊢
      exact ⟨i, c, z, hm, hi, rfl⟩
    · rintro ⟨m, cm, zm, hm, hfact, rfl⟩
      exact ⟨⟨m, cm, zm, hfact, by simp⟩, hm⟩
  have dom_iff : ∀ c z, Gnaf.dominatedIn Nat [c, z] rows = true ↔ ∃ m cm zm, m ∈ members ∧
      Certification.factAt facts m = some (Fact.admitted cm zm) ∧ Sdom (cm, zm) (c, z) := by
    intro c z
    rw [dominatedIn_iff]
    constructor
    · rintro ⟨r, hr, hd⟩
      obtain ⟨m, cm, zm, hm, hfact, rfl⟩ := (row_iff r).mp hr
      exact ⟨m, cm, zm, hm, hfact, (dominates_pair _ _ _ _).mp hd⟩
    · rintro ⟨m, cm, zm, hm, hfact, hd⟩
      exact ⟨(m, [cm, zm]), (row_iff _).mpr ⟨m, cm, zm, hm, hfact, rfl⟩, (dominates_pair _ _ _ _).mpr hd⟩
  -- members are admitted and undominated
  have member_fact : ∀ k, k ∈ members → ∃ s c z, Certification.selectorAt U k = some s ∧
      Certification.factAt facts k = some (Fact.admitted c z) ∧
      Gnaf.dominatedIn Nat [c, z] rows = false := by
    intro k hk
    have hk' : k < U.length := by
      have := allBelow_lt members facts.length k hbelow hk
      rw [hlen] at this; exact this
    obtain ⟨s, hsel⟩ := selectorAt_of_lt U k hk'
    obtain ⟨f, hfact, hsound⟩ := fact_of_member hf hsel
    have hok := pos_ok k f hfact
    have hin : Gnaf.natIn k members = true := (natIn_iff _ _).mpr hk
    cases f with
    | admitted c z =>
      simp only [Certification.frontierPosition, hin, if_true] at hok
      cases hd : Gnaf.dominatedIn Nat [c, z] rows
      · exact ⟨s, c, z, hsel, hfact, hd⟩
      · simp [hd] at hok
    | inadmissible => simp [Certification.frontierPosition, hin] at hok
    | atLeast c z => simp [Certification.frontierPosition, hin] at hok
    | reduces p => simp [Certification.frontierPosition, hin] at hok
    | unresolved => simp [Certification.frontierPosition, hin] at hok
    | unproven => simp [Certification.frontierPosition, hin] at hok
  have fact_sel : ∀ m f, Certification.factAt facts m = some f →
      ∃ s, Certification.selectorAt U m = some s ∧ FactSound req g U s f := fun m f h =>
    selector_of_fact hf h
  have complete : ∀ F', req.machine.fuel ≤ F' → ∀ n, ∀ k s c z, c + z = n →
      Certification.selectorAt U k = some s → statusAt req g s F' = .admitted c z →
      k ∈ members ∨ ∃ m sm cm zm, m ∈ members ∧ Certification.selectorAt U m = some sm ∧
        statusAt req g sm F' = .admitted cm zm ∧ Sdom (cm, zm) (c, z) := by
    intro F' hle n
    induction n using Nat.strongRecOn with
    | _ n ih =>
      intro k s c z hn hsel hadm
      obtain ⟨f, hfact, hsound⟩ := fact_of_member hf hsel
      have hok := pos_ok k f hfact
      -- a dominated system has a dominating member
      have via : ∀ c0 z0, Gnaf.dominatedIn Nat [c0, z0] rows = true → ∀ c z, c0 ≤ c → z0 = z →
          k ∈ members ∨ ∃ m sm cm zm, m ∈ members ∧ Certification.selectorAt U m = some sm ∧
            statusAt req g sm F' = .admitted cm zm ∧ Sdom (cm, zm) (c, z) := by
        intro c0 z0 hd c z hc hz
        obtain ⟨m, cm, zm, hm, hfm, hsd⟩ := (dom_iff _ _).mp hd
        obtain ⟨sm, hsm, hsmsound⟩ := fact_sel m _ hfm
        refine Or.inr ⟨m, sm, cm, zm, hm, hsm, hsmsound F' hle, ?_⟩
        obtain ⟨a1, a2, a3⟩ := hsd
        refine ⟨by omega, by omega, ?_⟩
        intro ⟨b1, b2⟩
        apply a3; constructor <;> omega
      cases f with
      | inadmissible => exact absurd hadm (hsound F' hle c z)
      | admitted c z =>
        have h0 := hsound F' hle
        rw [hadm] at h0
        simp only [Gnaf.Status.admitted.injEq] at h0
        obtain ⟨rfl, rfl⟩ := h0
        by_cases hin : Gnaf.natIn k members = true
        · exact Or.inl ((natIn_iff _ _).mp hin)
        · simp only [Bool.not_eq_true] at hin
          simp only [Certification.frontierPosition, hin, Bool.false_eq_true, if_false] at hok
          cases hd : Gnaf.dominatedIn Nat [c, z] rows
          · simp [hd] at hok
          · exact via c z hd c z (Nat.le_refl _) rfl
      | atLeast c0 z0 =>
        obtain ⟨hcle, hz⟩ := hsound F' hle c z hadm
        have hnin : Gnaf.natIn k members = false := by
          cases hin : Gnaf.natIn k members
          · rfl
          · simp [Certification.frontierPosition, hin] at hok
        simp only [Certification.frontierPosition, hnin, Bool.false_eq_true, if_false] at hok
        cases hd : Gnaf.dominatedIn Nat [c0, z0] rows
        · simp [hd] at hok
        · exact via c0 z0 hd c z hcle hz.symm
      | reduces p =>
        obtain ⟨q, c', z', hq, hqadm, hlt, hz⟩ := hsound F' hle c z hadm
        have hnin : Gnaf.natIn k members = false := by
          cases hin : Gnaf.natIn k members
          · rfl
          · simp [Certification.frontierPosition, hin] at hok
        rcases ih (c' + z') (by omega) p q c' z' rfl hq hqadm with hmem | ⟨m, sm, cm, zm, hm, hsm, hsmadm, hsd⟩
        · refine Or.inr ⟨p, q, c', z', hmem, hq, hqadm, ?_⟩
          refine ⟨by omega, hz, ?_⟩
          intro ⟨b1, b2⟩; omega
        · refine Or.inr ⟨m, sm, cm, zm, hm, hsm, hsmadm, ?_⟩
          obtain ⟨a1, a2, a3⟩ := hsd
          refine ⟨by omega, by omega, ?_⟩
          intro ⟨b1, b2⟩; apply a3; constructor <;> omega
      | unresolved => simp [Certification.frontierPosition] at hok
      | unproven => simp [Certification.frontierPosition] at hok
  refine ⟨hne, ?_, ?_, fun F' hle k s c z hsel hadm => complete F' hle _ k s c z rfl hsel hadm⟩
  · intro F' hle k hk
    obtain ⟨s, c, z, hsel, hfact, _⟩ := member_fact k hk
    obtain ⟨s', hsel', hsound⟩ := fact_sel k _ hfact
    rw [hsel] at hsel'; cases hsel'
    exact ⟨s, c, z, hsel, hsound F' hle⟩
  · intro F' hle k s c z hk hsel hadm k' s' c' z' hsel' hadm' hdom
    obtain ⟨s0, c0, z0, hsel0, hfact0, hnd⟩ := member_fact k hk
    obtain ⟨f, hfact, hsound⟩ := fact_of_member hf hsel
    rw [hfact0] at hfact; cases hfact
    have h0 := hsound F' hle
    rw [hadm] at h0; simp only [Gnaf.Status.admitted.injEq] at h0; obtain ⟨rfl, rfl⟩ := h0
    have hdominated : Gnaf.dominatedIn Nat [c, z] rows = true := by
      rw [dom_iff]
      rcases complete F' hle _ k' s' c' z' rfl hsel' hadm' with hm | ⟨m, sm, cm, zm, hm, hsm, hsmadm, hsd⟩
      · obtain ⟨s2, c2, z2, hsel2, hfact2, _⟩ := member_fact k' hm
        obtain ⟨f2, hfact2', hsound2⟩ := fact_of_member hf hsel'
        rw [hfact2] at hfact2'; cases hfact2'
        have h2 := hsound2 F' hle
        rw [hadm'] at h2; simp only [Gnaf.Status.admitted.injEq] at h2; obtain ⟨rfl, rfl⟩ := h2
        exact ⟨k', c', z', hm, hfact2, hdom⟩
      · obtain ⟨s2, c2, z2, hsel2, hfact2, _⟩ := member_fact m hm
        obtain ⟨f2, hfact2', hsound2⟩ := fact_of_member hf hsm
        rw [hfact2] at hfact2'; cases hfact2'
        have h2 := hsound2 F' hle
        rw [hsmadm] at h2; simp only [Gnaf.Status.admitted.injEq] at h2; obtain ⟨rfl, rfl⟩ := h2
        exact ⟨m, cm, zm, hm, hfact2, Sdom.trans hsd hdom⟩
    rw [hnd] at hdominated; cases hdominated
end LexLeanCertification
