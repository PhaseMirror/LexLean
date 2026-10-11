import LexLeanPreservation.Core
import LexLeanTarget.Certification
set_option linter.unusedSimpArgs false
namespace LexLeanCertification
open LexLeanTarget.TargetSyntax LexLeanTarget.TargetSemantics LexLeanPreservation LexLeanTarget

/-- Two programs that differ at most in their entry function (index 0). -/
structure Tail (P Q : Program) : Prop where
  entries : ∃ e1 e2 plans, P.functions = e1 :: plans ∧ Q.functions = e2 :: plans ∧
    ∀ f ∈ plans, Certification.exprIsolated f.body = true

theorem index_zero (x : Function) (xs : List Function) :
    (LexLeanTarget.TargetSemantics.LexLeanRuntime.index (x :: xs) 0 : Option Function) = some x := rfl
theorem index_succ (x : Function) (xs : List Function) (k : Nat) :
    (LexLeanTarget.TargetSemantics.LexLeanRuntime.index (x :: xs) (k+1) : Option Function)
      = LexLeanTarget.TargetSemantics.LexLeanRuntime.index xs k := rfl

theorem index_mem : ∀ (xs : List Function) (k : Nat) (f : Function),
    (LexLeanTarget.TargetSemantics.LexLeanRuntime.index xs k : Option Function) = some f → f ∈ xs
  | [], k, f, h => by cases k <;> simp [LexLeanRuntime.index, LexLeanRuntime.listIndex, LexLeanRuntime.Indexable.index] at h
  | x :: xs, 0, f, h => by
      rw [index_zero] at h; cases h; exact List.mem_cons_self
  | x :: xs, k+1, f, h => by
      rw [index_succ] at h; exact List.mem_cons_of_mem _ (index_mem xs k f h)

theorem eval_tail {P Q : Program} (T : Tail P Q) : ∀ n,
    (∀ env e, Certification.exprIsolated e = true → eval n P env e = eval n Q env e) ∧
    (∀ env es, Certification.exprsIsolated es = true → evalList n P env es = evalList n Q env es) ∧
    (∀ env v arms, Certification.armsIsolated arms = true → evalArms n P env v arms = evalArms n Q env v arms) := by
  obtain ⟨e1, e2, plans, hP, hQ, hplans⟩ := T.entries
  intro n
  induction n with
  | zero => exact ⟨fun _ _ _ => rfl, fun _ _ _ => rfl, fun _ _ _ _ => rfl⟩
  | succ n ih =>
    obtain ⟨ihE, ihL, ihA⟩ := ih
    refine ⟨?_, ?_, ?_⟩
    · intro env e he
      cases e with
      | value t v => rfl
      | var x => rfl
      | «let» x t b body =>
        simp only [Certification.exprIsolated, Bool.true_and, Bool.and_eq_true] at he
        simp only [eval.eq_4, ihE _ _ he.1, ihE _ _ he.2]
      | cond c t f =>
        simp only [Certification.exprIsolated, Bool.true_and, Bool.and_eq_true] at he
        simp only [eval.eq_5, ihE _ _ he.1, ihE _ _ he.2.1, ihE _ _ he.2.2]
      | «match» t s arms =>
        simp only [Certification.exprIsolated, Bool.true_and, Bool.and_eq_true] at he
        simp only [eval.eq_6, ihE _ _ he.1, ihA _ _ _ he.2]
      | build shape t ops =>
        simp only [Certification.exprIsolated, Bool.true_and] at he
        simp only [eval.eq_7, ihL _ _ he]
      | call callee ops =>
        simp only [Certification.exprIsolated, Bool.and_eq_true, Bool.not_eq_true'] at he
        have hne : callee ≠ 0 := by
          intro h; rw [h] at he; simp at he
        simp only [eval.eq_8, ihL _ _ he.2]
        have hidx : (LexLeanRuntime.index P.functions callee : Option Function)
            = LexLeanRuntime.index Q.functions callee := by
          cases callee with
          | zero => exact absurd rfl hne
          | succ k => rw [hP, hQ, index_succ, index_succ]
        cases hl : evalList n Q env ops with
        | values vs ss =>
          simp only [hidx]
          generalize hf : (LexLeanRuntime.index Q.functions callee : Option Function) = o
          cases o with
          | none => rfl
          | some fn =>
            have hmem : fn ∈ plans := by
              have := index_mem _ _ _ hf
              rw [hQ] at this
              cases callee with
              | zero => exact absurd rfl hne
              | succ k =>
                rw [hQ, index_succ] at hf
                exact index_mem plans k fn hf
            simp only []
            generalize hb : bindAll fn.parameters vs [] = b
            cases b with
            | none => rfl
            | some bound => simp only []; rw [ihE _ _ (hplans fn hmem)]
        | overflow _ => simp only [hidx]
        | stuck => simp only [hidx]
        | exhausted => simp only [hidx]
      | closure callee caps =>
        simp only [Certification.exprIsolated, Bool.true_and] at he
        simp only [eval.eq_9, ihL _ _ he]
      | apply t ops =>
        simp [Certification.exprIsolated] at he
      | prim op ops =>
        simp only [Certification.exprIsolated, Bool.true_and] at he
        simp only [eval.eq_11, ihL _ _ he]
      | first i =>
        simp only [Certification.exprIsolated, Bool.true_and] at he
        simp only [eval.eq_12, ihE _ _ he]
      | second i =>
        simp only [Certification.exprIsolated, Bool.true_and] at he
        simp only [eval.eq_13, ihE _ _ he]
      | field i k =>
        simp only [Certification.exprIsolated, Bool.true_and] at he
        simp only [eval.eq_14, ihE _ _ he]
    · intro env es he
      cases es with
      | nil => rfl
      | cons e rest =>
        simp only [Certification.exprsIsolated, Bool.and_eq_true] at he
        simp only [evalList.eq_3, ihE _ _ he.1, ihL _ _ he.2]
    · intro env v arms he
      cases arms with
      | nil => rfl
      | cons a rest =>
        cases a with
        | arm shape binders body =>
          simp only [Certification.armsIsolated, Certification.armIsolated, Bool.true_and, Bool.and_eq_true] at he
          simp only [evalArms.eq_3, ihE _ _ he.1, ihA _ _ _ he.2]
end LexLeanCertification
