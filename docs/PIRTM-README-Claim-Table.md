# PIRTM Ground-Truth Claim Table

This matrix maps system capabilities to physical artifacts on the current tree.

| Subsystem / Feature | Status | Artifact / Proof | ADR / Ledger |
|---|---|---|---|
| UORC Core Parser & Compressor | ✅ Complete | `packages/compression-main/crates/uorc-core/src/api.rs` (Tests passing) | ADR-015 |
| Checkpoint Determinism | ✅ Complete | `uorc-core/src/checkpoint.rs` & `thm_checkpoint_determinism` | AX-UORC-004 |
| Multiobjective Frontier | ✅ Complete | `uorc-core/src/frontier.rs` | ADR-014 |
| Evaluator Bounded Halting | ✅ Complete | `uorc-core/src/evaluator.rs` | AX-UORC-003 |
| SMT Solver Oracles | ✅ Complete | `uorc-core/src/solver_oracle.rs` | Issue 1304 |
| Formal Theorem Register | ✅ Complete | `uorc-core/src/theorems.rs` | Issue 1601 |
| Bounded Iteration Proofs | ✅ Complete | `lean/ADR/BoundedIteration.lean` | ADR-013 |
| Zeno Damping Controller | ✅ Complete | `lean/ADR/ZenoController.lean` | AX-ZENO-001 |
| Zero-Drift CI Gate | ✅ Complete | `.github/workflows/sedona_spine_ci.yml` | Issue 1701 |
| Mutation Evidence Suite | ✅ Complete | `uorc-core/src/mutation.rs` | Issue 1702 |
| Clean-path Reproducibility | ✅ Complete | `uorc-core/src/reproducibility.rs` | Issue 1703 |
