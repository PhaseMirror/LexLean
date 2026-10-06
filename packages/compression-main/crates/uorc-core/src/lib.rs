//! Universal Object Reference Compression (UORC) Core Library.
//!
//! Provides the parser, serializer, and reference machine execution models
//! for `uorc/archive/1` and Profile 02.

pub mod uleb128;
pub mod archive;
pub mod graph;
pub mod evaluator;
/// Encoder synthesis and incumbent management.
pub mod encoder;
/// Synthesis engine for graph generation.
pub mod synthesis;
/// Deterministic task scheduler.
pub mod scheduler;
/// Typed anti-unification for expression trees.
pub mod anti_unify;
/// Contextual layout and sharing optimization.
pub mod layout;
/// Universe descriptor and independent exhaustive enumeration.
pub mod universe;
/// Certificate encoding and checker (UORP v02).
pub mod certificate;
/// Prefix rejection and lower bound coverage.
pub mod prefix;
/// Multiobjective frontier analysis.
pub mod frontier;
/// Domain-separated identity and canonical serialization.
pub mod identity;
/// Reconstruction receipt schemas and validation.
pub mod receipt;
/// Deterministic replay checkpoint schemas and validation.
pub mod checkpoint;
/// Modeled computational API roots.
pub mod api;
/// Authority manifest acquisition and freezing.
pub mod manifest;
/// NIST SHA-256 vector oracle checks.
pub mod crypto_oracle;
/// Z3 and cvc5 oracle suites and strict parsers.
pub mod solver_oracle;
/// Benchmark plan schemas and strict comparator enforcement.
pub mod benchmark;
/// Corpus validation and sealing workflow.
pub mod corpus;
/// External comparator integration.
pub mod comparator;
/// Early feasibility gate and reporting.
pub mod feasibility;
/// Diagnostic inventory and bounded public failures.
pub mod diagnostics;

pub mod theorems;
pub mod fixtures;
pub mod mutation;
pub mod reproducibility;
