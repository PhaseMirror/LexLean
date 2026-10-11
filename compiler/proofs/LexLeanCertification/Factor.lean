import LexLeanCertification.Tail
set_option linter.unusedSimpArgs false
namespace LexLeanCertification
open LexLeanTarget.TargetSyntax LexLeanTarget.TargetSemantics LexLeanPreservation LexLeanTarget

/-- The entry's condition for threshold `t` on the list `items`. -/
def condE (t : Nat) : Expr :=
  Expr.prim Prim.natLt [Expr.prim Prim.length [Expr.var 0], Expr.value Ty.nat (Value.nat t)]

theorem cond_eval8 (P : Program) (t : Nat) (items : List Value) :
    (items.length < 2^64 ∧ ∃ s, eval 8 P [(0, Value.list items)] (condE t) = .value (.bool (items.length.blt t)) s) ∨
    (¬ items.length < 2^64 ∧ ∃ s, eval 8 P [(0, Value.list items)] (condE t) = .overflow s) := by
  have hlen : LexLeanRuntime.Lengthable.length items = items.length := rfl
  by_cases hl : items.length < 2^64
  · left
    refine ⟨hl, ?_⟩
    simp [condE, eval, evalList, lookup, depth, primitive, natResult, hl, hlen, chargeResult, LexLeanRuntime.length]
  · right
    refine ⟨hl, ?_⟩
    simp [condE, eval, evalList, lookup, depth, primitive, natResult, hl, hlen, chargeResult, LexLeanRuntime.length]
end LexLeanCertification
namespace LexLeanCertification
open LexLeanTarget.TargetSyntax LexLeanTarget.TargetSemantics LexLeanPreservation LexLeanTarget

theorem eval_fuel_indep {P : Program} {env e} {n m : Nat} (hn : eval n P env e ≠ .exhausted)
    (hm : eval m P env e ≠ .exhausted) : eval n P env e = eval m P env e := by
  rcases Nat.le_total n m with h | h
  · exact (eval_le hn h).symm
  · exact eval_le hm h

theorem cond_value (P : Program) (t : Nat) (items : List Value) (n : Nat) (v : Value) (s : Nat)
    (h : eval n P [(0, Value.list items)] (condE t) = .value v s) :
    v = .bool (items.length.blt t) ∧ items.length < 2^64 := by
  rcases cond_eval8 P t items with ⟨hl, s8, h8⟩ | ⟨hl, s8, h8⟩
  · have := eval_fuel_indep (n := n) (m := 8) (by rw [h]; exact nofun) (by rw [h8]; exact nofun)
    rw [h, h8] at this; cases this; exact ⟨rfl, hl⟩
  · exfalso
    have := eval_fuel_indep (n := n) (m := 8) (by rw [h]; exact nofun) (by rw [h8]; exact nofun)
    rw [h, h8] at this; cases this

theorem call_eval (R : Program) (ent : Function) (plans : List Function)
    (hR : R.functions = ent :: plans) (a : Value) (j n : Nat) :
    eval (n+3) R [(0, a)] (Expr.call (j+1) [Expr.var 0]) =
      (match (LexLeanRuntime.index plans j : Option Function) with
       | none => Outcome.stuck
       | some fn => match bindAll fn.parameters [a] [] with
         | none => Outcome.stuck
         | some b => addSteps (eval (n+2) R b fn.body) 2) := by
  simp [eval, evalList, lookup, depth, hR, index_succ]
  rfl

theorem call_small (R : Program) (a : Value) (j k : Nat) (hk : k < 3) :
    eval k R [(0, a)] (Expr.call j [Expr.var 0]) = .exhausted := by
  match k, hk with
  | 0, _ => rfl
  | 1, _ => simp [eval, evalList]
  | 2, _ => simp [eval, evalList]
end LexLeanCertification
namespace LexLeanCertification
open LexLeanTarget.TargetSyntax LexLeanTarget.TargetSemantics LexLeanPreservation LexLeanTarget

theorem planFunctions_isolated : ∀ (plans : List Gnaf.Plan),
    Certification.plansIsolated plans = true →
    ∀ f ∈ Gnaf.planFunctions plans, Certification.exprIsolated f.body = true
  | [], _, f, hf => by simp [Gnaf.planFunctions] at hf
  | p :: rest, h, f, hf => by
      simp only [Certification.plansIsolated, Bool.and_eq_true] at h
      simp only [Gnaf.planFunctions, List.mem_cons] at hf
      rcases hf with rfl | hf
      · exact h.1
      · exact planFunctions_isolated rest h.2 f hf

theorem realize_functions (g : Gnaf.Grammar) (s : Gnaf.Selector) :
    (Gnaf.realize g s).functions =
      ({ parameters := [0], types := [g.argument], result := g.result, body := Gnaf.entryBody s } : Function)
        :: Gnaf.planFunctions g.plans := rfl

theorem tail_realize (g : Gnaf.Grammar) (s q : Gnaf.Selector)
    (hiso : Certification.plansIsolated g.plans = true) : Tail (Gnaf.realize g s) (Gnaf.realize g q) :=
  ⟨⟨_, _, _, realize_functions g s, realize_functions g q, planFunctions_isolated _ hiso⟩⟩

theorem run_entry (R : Program) (ent : Function) (plans : List Function) (hR : R.functions = ent :: plans)
    (hp : ent.parameters = [0]) (F : Nat) (a : Value) :
    run F R 0 [a] = eval F R [(0, a)] ent.body := by
  unfold run
  rw [hR]
  simp [LexLeanRuntime.index, LexLeanRuntime.Indexable.index, LexLeanRuntime.listIndex, hp, bindAll]

theorem addSteps_value {r : Outcome} {k : Nat} {v : Value} {s : Nat} (h : addSteps r k = .value v s) :
    ∃ r', r = .value v r' ∧ s = r' + k := by
  cases r with
  | value w r' => simp only [addSteps, Outcome.value.injEq] at h; obtain ⟨rfl, rfl⟩ := h; exact ⟨r', rfl, rfl⟩
  | overflow _ => simp [addSteps] at h
  | stuck => simp [addSteps] at h
  | exhausted => simp [addSteps] at h

/-- Entering a plan from the entry one level of fuel deeper changes nothing. -/
theorem call_succ {P Q : Program} (T : Tail P Q) (ent1 ent2 : Function) (plans : List Function)
    (hP : P.functions = ent1 :: plans) (hQ : Q.functions = ent2 :: plans)
    (a : Value) (j k : Nat) (v : Value) (s0 : Nat)
    (h : eval k P [(0, a)] (Expr.call (j+1) [Expr.var 0]) = .value v s0) :
    eval (k+1) Q [(0, a)] (Expr.call (j+1) [Expr.var 0]) = .value v s0 := by
  obtain ⟨e1, e2, pl, hP', hQ', hiso⟩ := T.entries
  have hpl : pl = plans := by
    rw [hP] at hP'; exact (List.cons.inj hP').2.symm
  subst hpl
  by_cases hk : k < 3
  · rw [call_small _ _ _ _ hk] at h; cases h
  · obtain ⟨n, rfl⟩ : ∃ n, k = n + 3 := ⟨k - 3, by omega⟩
    rw [call_eval P ent1 pl hP] at h
    show eval ((n+1)+3) Q [(0, a)] (Expr.call (j+1) [Expr.var 0]) = _
    rw [call_eval Q ent2 pl hQ]
    generalize hf : (LexLeanRuntime.index pl j : Option Function) = o at h ⊢
    cases o with
    | none => simp at h
    | some fn =>
      simp only [] at h ⊢
      generalize hb : bindAll fn.parameters [a] [] = b at h ⊢
      cases b with
      | none => simp at h
      | some bound =>
        simp only [] at h ⊢
        obtain ⟨r', hr', hs0⟩ := addSteps_value h
        have hfn : fn ∈ pl := index_mem pl j fn hf
        have h1 := (eval_tail T (n+3)).1 bound fn.body (hiso fn hfn)
        have h2 : eval (n+3) P bound fn.body = eval (n+2) P bound fn.body :=
          eval_le (n := n+2) (m := n+3) (by rw [hr']; exact nofun) (Nat.le_succ _)
        have : eval (n+1+2) Q bound fn.body = Outcome.value v r' := by
          show eval (n+3) Q bound fn.body = _
          rw [← h1, h2, hr']
        rw [this]
        simp only [addSteps]
        rw [hs0]

theorem dispatch_cheaper (g : Gnaf.Grammar) (t sm lg : Nat) (items : List Value) (F : Nat) (v : Value)
    (st : Nat) (hiso : Certification.plansIsolated g.plans = true)
    (h : run F (Gnaf.realize g (.dispatch t sm lg)) 0 [.list items] = .value v st) :
    ∃ stq, stq < st ∧
      run F (Gnaf.realize g (.fixed (if items.length.blt t then sm else lg))) 0 [.list items] = .value v stq := by
  rw [run_entry _ _ _ (realize_functions g _) rfl] at h
  cases F with
  | zero => simp [eval] at h
  | succ k =>
    simp only [Gnaf.entryBody] at h
    rw [eval.eq_5] at h
    cases hc : eval k (Gnaf.realize g (.dispatch t sm lg)) [(0, Value.list items)] (condE t) with
    | value cv cs =>
      obtain ⟨hcv, hlen⟩ := cond_value _ t items k cv cs hc
      subst hcv
      simp only [condE] at hc
      rw [hc] at h
      simp only [] at h
      have key : ∀ j, addSteps (eval k (Gnaf.realize g (.dispatch t sm lg)) [(0, Value.list items)]
            (Expr.call (j+1) [Expr.var 0])) (cs+1) = .value v st →
          ∃ stq, stq < st ∧ run (k+1) (Gnaf.realize g (.fixed j)) 0 [.list items] = .value v stq := by
        intro j hj
        obtain ⟨s0, hs0, hst⟩ := addSteps_value hj
        refine ⟨s0, by omega, ?_⟩
        rw [run_entry _ _ _ (realize_functions g _) rfl]
        simp only [Gnaf.entryBody]
        exact call_succ (tail_realize g (.dispatch t sm lg) (.fixed j) hiso) _ _ _
          (realize_functions g _) (realize_functions g _) _ j k v s0 hs0
      cases hb : items.length.blt t with
      | true => simp only [hb, if_true] at h ⊢; exact key sm h
      | false => simp only [hb, if_true, Bool.false_eq_true, if_false] at h ⊢; exact key lg h
    | overflow _ => simp only [condE] at hc; rw [hc] at h; simp at h
    | stuck => simp only [condE] at hc; rw [hc] at h; simp at h
    | exhausted => simp only [condE] at hc; rw [hc] at h; simp at h
end LexLeanCertification
