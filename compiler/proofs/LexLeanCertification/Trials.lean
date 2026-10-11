import LexLeanPreservation.Core
import LexLeanTarget.Gnaf
import LexLeanTarget.Certification
set_option linter.unusedSimpArgs false
namespace LexLeanCertification
open LexLeanTarget.TargetSyntax LexLeanTarget.TargetSemantics LexLeanPreservation
open LexLeanTarget

theorem run_le {n m : Nat} {p : Program} {e : Nat} {args : List Value}
    (h : run n p e args ≠ .exhausted) (hle : n ≤ m) : run m p e args = run n p e args := by
  unfold run at h ⊢
  generalize (LexLeanTarget.TargetSemantics.LexLeanRuntime.index p.functions e : Option Function) = o at h ⊢
  cases o with
  | none => rfl
  | some f =>
    simp only at h ⊢
    generalize bindAll f.parameters args [] = b at h ⊢
    cases b with
    | none => rfl
    | some env =>
      simp only at h ⊢
      exact eval_le h hle

inductive Trial where
  | agrees (steps : Nat)
  | disagrees
  | unknown

def trialOf (sys ref : Program) (F : Nat) (a : Value) : Trial :=
  match run F sys 0 [a] with
  | .value p st =>
    match run F ref 0 [a] with
    | .value e _ => if Gnaf.valueEq p e then .agrees st else .disagrees
    | .overflow _ => .unknown
    | .stuck => .unknown
    | .exhausted => .unknown
  | .overflow _ => .disagrees
  | .stuck => .disagrees
  | .exhausted => .unknown

theorem trial_stable {sys ref F F' a} (hle : F ≤ F') :
    (∀ st, trialOf sys ref F a = .agrees st → trialOf sys ref F' a = .agrees st) ∧
    (trialOf sys ref F a = .disagrees → trialOf sys ref F' a = .disagrees) := by
  constructor
  · intro st h
    unfold trialOf at h ⊢
    cases hs : run F sys 0 [a] with
    | value p stp =>
      rw [hs] at h; simp only at h
      cases hr : run F ref 0 [a] with
      | value e str =>
        rw [hr] at h; simp only at h
        rw [run_le (by rw [hs]; exact nofun) hle, hs, run_le (by rw [hr]; exact nofun) hle, hr]
        exact h
      | overflow _ => rw [hr] at h; simp at h
      | stuck => rw [hr] at h; simp at h
      | exhausted => rw [hr] at h; simp at h
    | overflow _ => rw [hs] at h; simp at h
    | stuck => rw [hs] at h; simp at h
    | exhausted => rw [hs] at h; simp at h
  · intro h
    unfold trialOf at h ⊢
    cases hs : run F sys 0 [a] with
    | value p stp =>
      rw [hs] at h; simp only at h
      cases hr : run F ref 0 [a] with
      | value e str =>
        rw [hr] at h; simp only at h
        rw [run_le (by rw [hs]; exact nofun) hle, hs, run_le (by rw [hr]; exact nofun) hle, hr]
        exact h
      | overflow _ => rw [hr] at h; simp at h
      | stuck => rw [hr] at h; simp at h
      | exhausted => rw [hr] at h; simp at h
    | overflow o =>
      rw [run_le (by rw [hs]; exact nofun) hle, hs]
    | stuck =>
      rw [run_le (by rw [hs]; exact nofun) hle, hs]
    | exhausted => rw [hs] at h; simp at h

theorem statusOn_nil (sys ref : Program) (F : Nat) :
    Gnaf.statusOn sys ref F [] = .admitted 0 0 := rfl

theorem statusOn_cons (sys ref : Program) (F : Nat) (a : Value) (D : List Value) :
    Gnaf.statusOn sys ref F (a :: D) =
      (match trialOf sys ref F a with
       | .agrees st =>
         (match Gnaf.statusOn sys ref F D with
          | .admitted r z => .admitted (st + r) z
          | .inadmissible => .inadmissible
          | .unresolved => .unresolved)
       | .disagrees => .inadmissible
       | .unknown => .unresolved) := by
  unfold trialOf
  simp only [Gnaf.statusOn]
  cases run F sys 0 [a] with
  | value p st =>
    simp only []
    cases run F ref 0 [a] with
    | value e _ =>
      simp only []
      cases hv : Gnaf.valueEq p e <;> rfl
    | overflow _ => rfl
    | stuck => rfl
    | exhausted => rfl
  | overflow _ => rfl
  | stuck => rfl
  | exhausted => rfl

/-- Every domain argument agrees, with these steps in total. -/
inductive AllAgree (sys ref : Program) (F : Nat) : List Value → Nat → Prop where
  | nil : AllAgree sys ref F [] 0
  | cons {a : Value} {D : List Value} {st r : Nat} :
      trialOf sys ref F a = .agrees st → AllAgree sys ref F D r → AllAgree sys ref F (a :: D) (st + r)

theorem allAgree_of_admitted {sys ref F} : ∀ (D : List Value) {c z},
    Gnaf.statusOn sys ref F D = .admitted c z → z = 0 ∧ AllAgree sys ref F D c
  | [], c, z, h => by
      rw [statusOn_nil] at h; cases h; exact ⟨rfl, AllAgree.nil⟩
  | a :: D, c, z, h => by
      rw [statusOn_cons] at h
      cases ht : trialOf sys ref F a with
      | agrees st =>
        rw [ht] at h
        simp only [] at h
        cases hr : Gnaf.statusOn sys ref F D with
        | admitted r z' =>
          rw [hr] at h
          simp only [Gnaf.Status.admitted.injEq] at h
          obtain ⟨rfl, rfl⟩ := h
          have := allAgree_of_admitted D hr
          exact ⟨this.1, AllAgree.cons ht this.2⟩
        | inadmissible => rw [hr] at h; simp at h
        | unresolved => rw [hr] at h; simp at h
      | disagrees => rw [ht] at h; simp at h
      | unknown => rw [ht] at h; simp at h

theorem admitted_of_allAgree {sys ref F} {D : List Value} {c : Nat} (h : AllAgree sys ref F D c) :
    Gnaf.statusOn sys ref F D = .admitted c 0 := by
  induction h with
  | nil => rfl
  | cons ht _ ih => rw [statusOn_cons, ht]; simp only []; rw [ih]

theorem allAgree_stable {sys ref F F'} (hle : F ≤ F') {D : List Value} {c : Nat}
    (h : AllAgree sys ref F D c) : AllAgree sys ref F' D c := by
  induction h with
  | nil => exact AllAgree.nil
  | cons ht _ ih => exact AllAgree.cons ((trial_stable hle).1 _ ht) ih

theorem allAgree_mem {sys ref F} {D : List Value} {c : Nat} (h : AllAgree sys ref F D c)
    {a : Value} (ha : a ∈ D) : ∃ st, trialOf sys ref F a = .agrees st := by
  induction h with
  | nil => simp at ha
  | cons ht _ ih =>
    rcases List.mem_cons.mp ha with rfl | ha
    · exact ⟨_, ht⟩
    · exact ih ha

theorem allAgree_sublist {sys ref F} {D : List Value} {c : Nat} (h : AllAgree sys ref F D c) :
    ∀ {S : List Value}, S.Sublist D → ∃ x, x ≤ c ∧ AllAgree sys ref F S x := by
  induction h with
  | nil => intro S hS; cases hS; exact ⟨0, Nat.le_refl _, AllAgree.nil⟩
  | @cons a D st r ht _ ih =>
    intro S hS
    cases hS with
    | cons _ hS' =>
      obtain ⟨x, hx, hax⟩ := ih hS'
      exact ⟨x, by omega, hax⟩
    | cons_cons _ hS' =>
      obtain ⟨x, hx, hax⟩ := ih hS'
      exact ⟨st + x, by omega, AllAgree.cons ht hax⟩

/-- A system that disagrees on a domain argument at some fuel is never admitted at a larger one. -/
theorem disagree_not_admitted {sys ref F F'} (hle : F ≤ F') {D : List Value} {a : Value}
    (ha : a ∈ D) (ht : trialOf sys ref F a = .disagrees) :
    ∀ c z, Gnaf.statusOn sys ref F' D ≠ .admitted c z := by
  intro c z h
  obtain ⟨_, hall⟩ := allAgree_of_admitted D h
  obtain ⟨st, hst⟩ := allAgree_mem hall ha
  rw [(trial_stable hle).2 ht] at hst
  cases hst

/-- A sublist of the domain on which the system is found inadmissible. -/
theorem inadmissible_witness {sys ref F} : ∀ (S : List Value),
    Gnaf.statusOn sys ref F S = .inadmissible → ∃ a ∈ S, trialOf sys ref F a = .disagrees
  | [], h => by rw [statusOn_nil] at h; cases h
  | a :: S, h => by
      rw [statusOn_cons] at h
      cases ht : trialOf sys ref F a with
      | agrees st =>
        rw [ht] at h
        simp only [] at h
        cases hr : Gnaf.statusOn sys ref F S with
        | admitted r z' => rw [hr] at h; simp at h
        | inadmissible =>
          obtain ⟨b, hb, hbt⟩ := inadmissible_witness S hr
          exact ⟨b, List.mem_cons_of_mem _ hb, hbt⟩
        | unresolved => rw [hr] at h; simp at h
      | disagrees => exact ⟨a, List.mem_cons_self, ht⟩
      | unknown => rw [ht] at h; simp at h

theorem pickFrom_sublist : ∀ (D : List Value) (i : Nat) (ps : List Nat),
    (Certification.pickFrom D i ps).Sublist D
  | [], i, ps => by simp [Certification.pickFrom]
  | a :: D, i, [] => by simp [Certification.pickFrom]
  | a :: D, i, p :: ps => by
      simp only [Certification.pickFrom]
      by_cases h : Nat.beq p i = true
      · simp only [h, if_true]
        exact List.Sublist.cons_cons _ (pickFrom_sublist D (i+1) ps)
      · simp only [h, if_false, Bool.not_eq_true] at *
        exact List.Sublist.cons _ (pickFrom_sublist D (i+1) (p :: ps))

end LexLeanCertification
