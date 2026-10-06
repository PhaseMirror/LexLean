import Lake
open Lake DSL

package «lexlean» where
  leanOptions := #[
    ⟨`pp.unicode.fun, true⟩,
    ⟨`pp.proofs.withType, false⟩
  ]

require mathlib from git
  "https://github.com/leanprover-community/mathlib4.git" @ "v4.22.0"

@[default_target]
lean_lib Spec where
  srcDir := "lean/specs"

@[default_target]
lean_lib Proofs where
  srcDir := "lean/proofs"
