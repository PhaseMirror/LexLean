-- Corrected Theorems 3.2 and 3.4 from ADR-PM-AGENT-021

import Mathlib.Analysis.NormedSpace.OperatorNorm.Basic
import Mathlib.Analysis.NormedSpace.SpectralNorm
import Mathlib.LinearAlgebra.Matrix.Basis
import Mathlib.Analysis.Normed.Group.Basic

open Topology Filter ContinuousLinearMap

namespace LexLean.Contraction

-- Theorem 3.2 (Residual drift, corrected)
-- H = H_c ⊕ H_v, T H_v ⊆ H_v
-- ||T^t v|| >= e^(μ t) ||v|| for all v in H_v, t >= 0, μ > 0

-- This is a general spectral gap formulation
structure ExpandingComponent (𝕜 : Type*) {E : Type*} [NormedAddCommGroup E] [NormedField 𝕜] [NormedSpace 𝕜 E]
  (T : E →ₗ[𝕜] E) where
  μ : ℝ
  hμ_pos : μ > 0
  invariant : ∃ (P_v : E →ₗ[𝕜] E), (P_v.comp P_v = P_v) ∧ 
    (∀ v, P_v (T v) = T (P_v v)) ∧  -- invariance T H_v ⊆ H_v
    (∀ v, T (P_v v) = P_v (T v)) ∧
    (∀ v t : ℕ, ‖T^[t] (P_v v)‖ ≥ Real.exp (μ * t) * ‖P_v v‖)

-- Theorem statement (formal structure)
def is_expanding_on_component {𝕜 : Type*} {E : Type*} [NormedAddCommGroup E] [NormedField 𝕜] [NormedSpace 𝕜 E]
  (T : E →ₗ[𝕜] E) : Prop :=
  ∃ comp : ExpandingComponent 𝕜 T, True

-- Theorem 3.4 (Damped proximal contraction, corrected)
-- q = 1 - α_min + α_max * M < 1, where M = sup_t ||M_t||_op, with 0 < α_min ≤ α_π ≤ α_max < 1

structure DampedProximalParams where
  α_min : ℝ
  α_max : ℝ
  hα : 0 < α_min ∧ α_min ≤ α_max ∧ α_max < 1
  M : ℝ
  hM_nonneg : M ≥ 0
  M_op : ℝ  -- sup_t ||M_t||_op = M_op
  q : ℝ := 1 - α_min + α_max * M_op
  hq_lt_one : q < 1

def is_damped_proximal_contracting (p : DampedProximalParams) : Prop := p.q < 1

end LexLean.Contraction
