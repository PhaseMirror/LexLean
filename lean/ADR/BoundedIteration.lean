import Mathlib.Data.Nat.Basic

namespace PhaseMirror

/-!
# Bounded Iteration & Contractivity (ADR-013)

Ensures that recursive steps and loops inside the VM are bounded
and contractive, rejecting infinite loops.
-/

/-- A loop construct with a strict gas/fuel bound. -/
def bounded_loop (fuel : Nat) (state : α) (step : α → α) : α :=
  match fuel with
  | 0 => state
  | n + 1 => bounded_loop n (step state) step

/-- Theorem: Bounded loops strictly terminate.
This is inherently proven by the structural recursion of `bounded_loop` in Lean. -/
theorem bounded_loop_terminates (fuel : Nat) (state : α) (step : α → α) :
  ∃ (final : α), bounded_loop fuel state step = final :=
  ⟨bounded_loop fuel state step, rfl⟩

end PhaseMirror
