//! LexLean v0.4 Dual-Track E2E Test Suite.
//!
//! Requirement-driven, opaque-box testing of the LexLean compiler across:
//! - Track 1: Process-boundary CLI execution.
//! - Track 2: Public programmatic Engine API.
//!
//! Organized in 4 Tiers:
//! - Tier 1: Feature Coverage (>=5 tests per feature)
//! - Tier 2: Boundary & Corner Cases (>=5 tests per feature)
//! - Tier 3: Cross-Feature Combinations (Pairwise matrix)
//! - Tier 4: Real-World Application Scenarios
//!
//! Specification reference: `TEST_INFRA.md`.

#[path = "e2e/harness.rs"]
mod harness;

#[path = "e2e/tier1_feature.rs"]
mod tier1_feature;

#[path = "e2e/tier2_boundary.rs"]
mod tier2_boundary;

#[path = "e2e/tier3_combinatorial.rs"]
mod tier3_combinatorial;

#[path = "e2e/tier4_scenarios.rs"]
mod tier4_scenarios;
