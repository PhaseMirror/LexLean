import LexLeanCertification.Factor
import LexLeanCertification.Trials
set_option linter.unusedSimpArgs false
namespace LexLeanCertification
open LexLeanTarget.TargetSyntax LexLeanTarget.TargetSemantics LexLeanPreservation LexLeanTarget

/-- A system's status at a given fuel: `Gnaf.status` with the request's fuel replaced. -/
def statusAt (req : Gnaf.Request) (g : Gnaf.Grammar) (s : Gnaf.Selector) (F : Nat) : Gnaf.Status :=
  match Gnaf.statusOn (Gnaf.realize g s) req.reference F req.domain with
  | .admitted steps _ =>
    .admitted (steps + Certification.staticCharge req g s) (Certification.staticSize g s)
  | .inadmissible => .inadmissible
  | .unresolved => .unresolved

theorem status_eq (req : Gnaf.Request) (g : Gnaf.Grammar) (s : Gnaf.Selector) :
    Gnaf.status req g s = statusAt req g s req.machine.fuel := rfl

theorem trial_agrees_inv {sys ref : Program} {F : Nat} {a : Value} {st : Nat}
    (h : trialOf sys ref F a = .agrees st) :
    run F sys 0 [a] = .value (match run F sys 0 [a] with | .value p _ => p | _ => a) st ∧
    ∃ p e r, run F sys 0 [a] = .value p st ∧ run F ref 0 [a] = .value e r ∧ Gnaf.valueEq p e = true := by
  unfold trialOf at h
  cases hs : run F sys 0 [a] with
  | value p stp =>
    rw [hs] at h; simp only [] at h
    cases hr : run F ref 0 [a] with
    | value e r =>
      rw [hr] at h; simp only [] at h
      by_cases hv : Gnaf.valueEq p e = true
      · simp only [hv, if_true, Trial.agrees.injEq] at h
        subst h
        exact ⟨rfl, p, e, r, rfl, rfl, hv⟩
      · simp only [hv, if_false] at h; cases h
    | overflow _ => rw [hr] at h; simp at h
    | stuck => rw [hr] at h; simp at h
    | exhausted => rw [hr] at h; simp at h
  | overflow _ => rw [hs] at h; simp at h
  | stuck => rw [hs] at h; simp at h
  | exhausted => rw [hs] at h; simp at h

theorem trial_agrees_of {sys ref : Program} {F : Nat} {a p e : Value} {st r : Nat}
    (hs : run F sys 0 [a] = .value p st) (hr : run F ref 0 [a] = .value e r)
    (hv : Gnaf.valueEq p e = true) : trialOf sys ref F a = .agrees st := by
  unfold trialOf
  rw [hs]; simp only []; rw [hr]; simp only [hv, if_true]

theorem domain_lists : ∀ (D : List Value) (l : Nat), Certification.allLists D l = true →
    ∀ a ∈ D, ∃ items, a = Value.list items
  | [], _, _, a, ha => by simp at ha
  | b :: D, l, h, a, ha => by
      simp only [Certification.allLists, Bool.and_eq_true] at h
      rcases List.mem_cons.mp ha with rfl | ha
      · cases a <;> simp [Certification.isList] at h <;> exact ⟨_, rfl⟩
      · exact domain_lists D l h.2 a ha

theorem domain_shorter : ∀ (D : List Value) (l : Nat), Certification.allShorter D l = true →
    ∀ a ∈ D, ∃ items, a = Value.list items ∧ items.length < l
  | [], _, _, a, ha => by simp at ha
  | b :: D, l, h, a, ha => by
      simp only [Certification.allShorter, Bool.and_eq_true] at h
      rcases List.mem_cons.mp ha with rfl | ha
      · cases a <;> simp [Certification.isShorter] at h <;> exact ⟨_, rfl, h.1⟩
      · exact domain_shorter D l h.2 a ha

theorem domain_long : ∀ (D : List Value) (l : Nat), Certification.allLong D l = true →
    ∀ a ∈ D, ∃ items, a = Value.list items ∧ l ≤ items.length
  | [], _, _, a, ha => by simp at ha
  | b :: D, l, h, a, ha => by
      simp only [Certification.allLong, Bool.and_eq_true] at h
      rcases List.mem_cons.mp ha with rfl | ha
      · cases a <;> simp [Certification.isLong] at h <;> exact ⟨_, rfl, h.1⟩
      · exact domain_long D l h.2 a ha

/-- What an eligible schema application says about the system's code and domain: the rewritten
system `fixed c` runs, on every domain argument, exactly the plan the dispatch runs. -/
theorem eligible_branch (req : Gnaf.Request) (g : Gnaf.Grammar) (s q : Gnaf.Selector)
    (schema : Certification.Schema)
    (hrw : Certification.rewrite schema s = some q) (hel : Certification.eligible req g s schema = true) :
    Certification.plansIsolated g.plans = true ∧
    ∃ t sm lg c, s = .dispatch t sm lg ∧ q = .fixed c ∧
      ∀ a ∈ req.domain, ∃ items, a = Value.list items ∧ (if items.length.blt t then sm else lg) = c := by
  cases s with
  | fixed p =>
    cases schema <;> simp [Certification.rewrite] at hrw
  | dispatch t sm lg =>
    simp only [Certification.eligible, Bool.and_eq_true] at hel
    refine ⟨hel.1, ?_⟩
    cases schema with
    | collapse =>
      simp only [Certification.rewrite] at hrw
      by_cases hs : Nat.beq sm lg = true
      · simp only [hs, if_true, Option.some.injEq] at hrw
        subst hrw
        have hbeq : sm = lg := Nat.eq_of_beq_eq_true hs
        simp only [Bool.and_eq_true] at hel
        refine ⟨t, sm, lg, sm, rfl, rfl, ?_⟩
        intro a ha
        obtain ⟨items, hit⟩ := domain_lists _ _ hel.2.2 a ha
        refine ⟨items, hit, ?_⟩
        subst hbeq
        simp
      · simp only [hs, if_false] at hrw; cases hrw
    | specializeSmall =>
      simp only [Certification.rewrite, Option.some.injEq] at hrw
      subst hrw
      refine ⟨t, sm, lg, sm, rfl, rfl, ?_⟩
      intro a ha
      obtain ⟨items, hit, hlt⟩ := domain_shorter _ _ hel.2 a ha
      refine ⟨items, hit, ?_⟩
      have : items.length.blt t = true := by simp [Nat.blt_eq]; exact hlt
      simp [this]
    | specializeLarge =>
      simp only [Certification.rewrite, Option.some.injEq] at hrw
      subst hrw
      refine ⟨t, sm, lg, lg, rfl, rfl, ?_⟩
      intro a ha
      obtain ⟨items, hit, hle⟩ := domain_long _ _ hel.2 a ha
      refine ⟨items, hit, ?_⟩
      have : items.length.blt t = false := by
        cases h : items.length.blt t
        · rfl
        · have := Nat.blt_eq.mp h; omega
      simp [this]


theorem allAgree_cheaper {P Q ref : Program} {F : Nat} {D : List Value} {c : Nat}
    (hall : AllAgree P ref F D c) :
    (∀ a ∈ D, ∀ st, trialOf P ref F a = .agrees st → ∃ st', st' < st ∧ trialOf Q ref F a = .agrees st') →
      ∃ c', c' ≤ c ∧ (D ≠ [] → c' < c) ∧ AllAgree Q ref F D c' := by
  induction hall with
  | nil => intro _; exact ⟨0, Nat.le_refl _, fun h => absurd rfl h, AllAgree.nil⟩
  | @cons a D st r ht _ ih =>
    intro hq
    obtain ⟨st', hlt, hq'⟩ := hq a List.mem_cons_self st ht
    obtain ⟨r', hr, _, hrest'⟩ := ih (fun b hb => hq b (List.mem_cons_of_mem _ hb))
    exact ⟨st' + r', by omega, fun _ => by omega, AllAgree.cons hq' hrest'⟩

/-- An eligible schema's rewriting runs, on each domain argument the system agrees on, the same
value in strictly fewer steps. -/
theorem schema_trial (req : Gnaf.Request) (g : Gnaf.Grammar) (s q : Gnaf.Selector)
    (schema : Certification.Schema)
    (hrw : Certification.rewrite schema s = some q) (hel : Certification.eligible req g s schema = true)
    (F : Nat) :
    ∀ a ∈ req.domain, ∀ st, trialOf (Gnaf.realize g s) req.reference F a = .agrees st →
      ∃ st', st' < st ∧ trialOf (Gnaf.realize g q) req.reference F a = .agrees st' := by
  obtain ⟨hiso, t, sm, lg, c, rfl, rfl, hbranch⟩ := eligible_branch req g s q schema hrw hel
  intro a ha st hst
  obtain ⟨items, rfl, hc⟩ := hbranch a ha
  obtain ⟨_, p, e, r, hs, hr, hv⟩ := trial_agrees_inv hst
  obtain ⟨stq, hlt, hq⟩ := dispatch_cheaper g t sm lg items F p st hiso hs
  rw [hc] at hq
  exact ⟨stq, hlt, trial_agrees_of hq hr hv⟩

/-- The schemas' preservation: a rewritten system that is admitted at a fuel is admitted at it,
at a strictly lower cost and no greater size, whenever its static code and preparation are no
greater. -/
theorem schema_cheaper (req : Gnaf.Request) (g : Gnaf.Grammar) (s q : Gnaf.Selector)
    (schema : Certification.Schema)
    (hrw : Certification.rewrite schema s = some q) (hel : Certification.eligible req g s schema = true)
    (hsize : Certification.staticSize g q ≤ Certification.staticSize g s)
    (hcharge : Certification.staticCharge req g q ≤ Certification.staticCharge req g s)
    (hD : req.domain ≠ []) (F : Nat) (c z : Nat) (h : statusAt req g s F = .admitted c z) :
    ∃ c' z', statusAt req g q F = .admitted c' z' ∧ c' < c ∧ z' ≤ z := by
  unfold statusAt at h
  cases hs : Gnaf.statusOn (Gnaf.realize g s) req.reference F req.domain with
  | admitted steps size =>
    rw [hs] at h; simp only [Gnaf.Status.admitted.injEq] at h
    obtain ⟨rfl, rfl⟩ := h
    obtain ⟨_, hall⟩ := allAgree_of_admitted _ hs
    obtain ⟨c', _, hlt, hallq⟩ := allAgree_cheaper hall (schema_trial req g s q schema hrw hel F)
    have hq := admitted_of_allAgree hallq
    refine ⟨c' + Certification.staticCharge req g q, Certification.staticSize g q, ?_, ?_, hsize⟩
    · unfold statusAt; rw [hq]
    · have := hlt hD; omega
  | inadmissible => rw [hs] at h; simp at h
  | unresolved => rw [hs] at h; simp at h

/-- A system's single-argument status that is admitted is the argument's trial agreeing at that cost. -/
theorem argumentStatus_admitted {req : Gnaf.Request} {g : Gnaf.Grammar} {F : Nat} {s : Gnaf.Selector}
    {a : Value} {c z : Nat} (h : Certification.argumentStatus req g F s a = .admitted c z) :
    trialOf (Gnaf.realize g s) req.reference F a = .agrees c := by
  unfold Certification.argumentStatus at h
  rw [statusOn_cons] at h
  cases ht : trialOf (Gnaf.realize g s) req.reference F a with
  | agrees st =>
    rw [ht] at h
    simp only [statusOn_nil, Gnaf.Status.admitted.injEq] at h
    obtain ⟨h1, _⟩ := h
    simp only [Trial.agrees.injEq]; omega
  | disagrees => rw [ht] at h; simp at h
  | unknown => rw [ht] at h; simp at h

theorem argumentStatus_of_agrees {req : Gnaf.Request} {g : Gnaf.Grammar} {F : Nat} {s : Gnaf.Selector}
    {a : Value} {c : Nat} (h : trialOf (Gnaf.realize g s) req.reference F a = .agrees c) :
    Certification.argumentStatus req g F s a = .admitted c 0 := by
  unfold Certification.argumentStatus
  rw [statusOn_cons, h]
  simp [statusOn_nil]

theorem observations_of_trials (req : Gnaf.Request) (g : Gnaf.Grammar) (F : Nat) (s q : Gnaf.Selector) :
    ∀ D : List Value,
      (∀ a ∈ D, ∀ st, trialOf (Gnaf.realize g s) req.reference F a = .agrees st →
        ∃ st', st' < st ∧ trialOf (Gnaf.realize g q) req.reference F a = .agrees st') →
      Certification.sameObservationsAt req g F s q D = true ∧
        Certification.strictlyCheaperAt req g F s q D = true
  | [], _ => by simp [Certification.sameObservationsAt, Certification.strictlyCheaperAt]
  | a :: D, h => by
    have ih := observations_of_trials req g F s q D (fun b hb => h b (List.mem_cons_of_mem _ hb))
    have hhead : Certification.keepsObservation (Certification.argumentStatus req g F s a)
          (Certification.argumentStatus req g F q a) = true ∧
        Certification.cheaperObservation (Certification.argumentStatus req g F s a)
          (Certification.argumentStatus req g F q a) = true := by
      cases hs : Certification.argumentStatus req g F s a with
      | admitted c z =>
        obtain ⟨st', hlt, hq⟩ := h a List.mem_cons_self c (argumentStatus_admitted hs)
        rw [argumentStatus_of_agrees hq]
        simp [Certification.keepsObservation, Certification.cheaperObservation, Nat.blt_eq, hlt]
      | inadmissible => simp [Certification.keepsObservation, Certification.cheaperObservation]
      | unresolved => simp [Certification.keepsObservation, Certification.cheaperObservation]
    constructor
    · simp only [Certification.sameObservationsAt, Bool.and_eq_true]
      exact ⟨hhead.1, ih.1⟩
    · simp only [Certification.strictlyCheaperAt, Bool.and_eq_true]
      exact ⟨hhead.2, ih.2⟩

/-- Preservation for every fuel: an eligible schema application's rewriting keeps the system's
observations over the request's domain and is strictly cheaper on each, as the module defines them. -/
theorem schema_preserves_at (req : Gnaf.Request) (g : Gnaf.Grammar) (s q : Gnaf.Selector)
    (schema : Certification.Schema)
    (hrw : Certification.rewrite schema s = some q) (hel : Certification.eligible req g s schema = true)
    (F : Nat) :
    Certification.sameObservationsAt req g F s q req.domain = true ∧
      Certification.strictlyCheaperAt req g F s q req.domain = true :=
  observations_of_trials req g F s q req.domain (schema_trial req g s q schema hrw hel F)

/-- The module's `improves` implies its `preserves`, for every request, grammar, system, and schema. -/
theorem improves_preserves (req : Gnaf.Request) (g : Gnaf.Grammar) (s : Gnaf.Selector)
    (schema : Certification.Schema)
    (h : Certification.improves req g s schema = true) :
    Certification.preserves req g s schema = true := by
  unfold Certification.improves at h
  cases hrw : Certification.rewrite schema s with
  | none => rw [hrw] at h; simp at h
  | some q =>
    rw [hrw] at h
    simp only [Bool.and_eq_true] at h
    have hel := h.1
    have hp := schema_preserves_at req g s q schema hrw hel req.machine.fuel
    unfold Certification.preserves
    rw [hrw]
    simp only [Bool.and_eq_true]
    refine ⟨?_, ?_, ?_⟩
    · unfold Certification.improves; rw [hrw]; simp only [Bool.and_eq_true]; exact h
    · exact hp.1
    · exact hp.2

end LexLeanCertification
