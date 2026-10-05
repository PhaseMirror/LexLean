-- Specification for Typed Recursive System (from ADR-PM-AGENT-021)
-- The typed recursive system is z_{t+1} = F_t(z_t; ξ_t) with scalar ξ_t

namespace LexLean.TypedRecursive

open Real

-- The typed recursive system
structure State (n : Type*) where
  val : n → ℝ

-- Evolution function F_t
def Evolution (n : Type*) := State n × ℝ → State n

-- System dynamics z_{t+1} = F_t(z_t; ξ_t)
structure System (n : Type*) where
  F : ℕ → Evolution n  -- time-dependent evolution

end LexLean.TypedRecursive
