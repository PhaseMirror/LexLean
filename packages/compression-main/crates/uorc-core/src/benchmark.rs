use serde::{Deserialize, Serialize};

/// The explicit magic string required for version 2 benchmark plans.
pub const BENCHMARK_PLAN_MAGIC: &str = "uorc/benchmark-plan/2";

/// Cache policies defining the execution state of the system prior to measurement.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CachePolicy {
    /// Cold cache: no prior data loaded, system caches flushed if possible.
    Cold,
    /// Warm cache: target data previously executed/loaded.
    Warm,
}

/// The accounting basis specifies exactly what bytes are measured in the result.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AccountingBasis {
    /// Includes all framing, magic bytes, dictionary metadata, and the raw payload.
    TotalArchiveStream,
    /// Includes only the raw logical blocks, excluding generic transport framing.
    LogicalPayloadOnly,
}

/// A sealed, versioned benchmark plan ensuring measurement reproducibility.
/// It strictly defines comparators, data, and cache states to prevent hidden advantages.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BenchmarkPlan {
    /// Must exactly match `BENCHMARK_PLAN_MAGIC`.
    pub magic: String,
    /// The external comparator toolchains to run (e.g., `zstd`, `brotli`, `xz`).
    pub comparators: Vec<String>,
    /// The corpus target identities (e.g., `silesia`, `enwik8`).
    pub corpus_targets: Vec<String>,
    /// Explicit accounting definition to ensure apples-to-apples byte counts.
    pub accounting_basis: AccountingBasis,
    /// Cache warming policy enforcing measurement conditions.
    pub cache_policy: CachePolicy,
    /// Number of mandatory repetitions for statistical significance. Must be > 0.
    pub repetitions: usize,
}

/// Errors that can occur when parsing or validating a benchmark plan.
#[derive(Debug, PartialEq, Eq)]
pub enum BenchmarkPlanError {
    /// The magic string does not match `uorc/benchmark-plan/2`.
    InvalidMagic,
    /// The repetitions count is zero.
    InvalidRepetitions,
    /// There was a structural or typing error in the JSON format.
    ParseError(String),
}

impl BenchmarkPlan {
    /// Parses and strictly validates a benchmark plan from JSON.
    pub fn from_json(json: &str) -> Result<Self, BenchmarkPlanError> {
        let plan: BenchmarkPlan = serde_json::from_str(json)
            .map_err(|e| BenchmarkPlanError::ParseError(e.to_string()))?;

        if plan.magic != BENCHMARK_PLAN_MAGIC {
            return Err(BenchmarkPlanError::InvalidMagic);
        }

        if plan.repetitions == 0 {
            return Err(BenchmarkPlanError::InvalidRepetitions);
        }

        Ok(plan)
    }

    /// Serializes the benchmark plan.
    pub fn to_json(&self) -> Result<String, BenchmarkPlanError> {
        serde_json::to_string_pretty(self)
            .map_err(|e| BenchmarkPlanError::ParseError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_benchmark_plan() {
        let json = r#"{
            "magic": "uorc/benchmark-plan/2",
            "comparators": ["zstd", "brotli"],
            "corpus_targets": ["enwik8", "silesia"],
            "accounting_basis": "TotalArchiveStream",
            "cache_policy": "Cold",
            "repetitions": 5
        }"#;

        let plan = BenchmarkPlan::from_json(json).expect("should parse valid plan");
        assert_eq!(plan.comparators.len(), 2);
        assert_eq!(plan.accounting_basis, AccountingBasis::TotalArchiveStream);
        assert_eq!(plan.cache_policy, CachePolicy::Cold);
        assert_eq!(plan.repetitions, 5);
    }

    #[test]
    fn test_benchmark_plan_invalid_magic() {
        let json = r#"{
            "magic": "uorc/benchmark-plan/1", 
            "comparators": ["zstd"],
            "corpus_targets": ["silesia"],
            "accounting_basis": "LogicalPayloadOnly",
            "cache_policy": "Warm",
            "repetitions": 10
        }"#;

        let err = BenchmarkPlan::from_json(json).unwrap_err();
        assert_eq!(err, BenchmarkPlanError::InvalidMagic);
    }

    #[test]
    fn test_benchmark_plan_invalid_repetitions() {
        let json = r#"{
            "magic": "uorc/benchmark-plan/2",
            "comparators": ["zstd"],
            "corpus_targets": ["silesia"],
            "accounting_basis": "TotalArchiveStream",
            "cache_policy": "Cold",
            "repetitions": 0
        }"#;

        let err = BenchmarkPlan::from_json(json).unwrap_err();
        assert_eq!(err, BenchmarkPlanError::InvalidRepetitions);
    }

    #[test]
    fn test_benchmark_plan_missing_fields_fail_closed() {
        // Missing accounting_basis
        let json = r#"{
            "magic": "uorc/benchmark-plan/2",
            "comparators": ["zstd"],
            "corpus_targets": ["silesia"],
            "cache_policy": "Cold",
            "repetitions": 3
        }"#;

        let err = BenchmarkPlan::from_json(json).unwrap_err();
        if let BenchmarkPlanError::ParseError(msg) = err {
            assert!(msg.contains("missing field `accounting_basis`"));
        } else {
            panic!("Expected ParseError");
        }
    }
}
