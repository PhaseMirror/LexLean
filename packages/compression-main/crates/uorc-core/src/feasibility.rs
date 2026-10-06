use serde::{Deserialize, Serialize};

/// Exact semantic outcomes for early feasibility gate runs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FeasibilityOutcome {
    /// A strictly smaller candidate graph was synthesized.
    Compressed,
    /// Synthesis bounded out; returned the raw payload wrapped in a literal graph.
    RawBestFound,
    /// Execution bounded out due to memory or time limits without yielding a valid candidate.
    ResourceExhausted,
}

/// A deterministic feasibility report tracking synthesis candidate counts and logical steps.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FeasibilityReport {
    /// Total number of unique graph candidates evaluated.
    pub candidates_evaluated: usize,
    /// Total logical evaluation steps (quanta) consumed.
    pub steps_consumed: usize,
    /// Peak memory footprint consumed during evaluation (in bytes).
    pub peak_memory_bytes: usize,
    /// The exact outcome of the synthesis run.
    pub outcome: FeasibilityOutcome,
}

impl FeasibilityReport {
    /// Create a new report.
    pub fn new(
        candidates_evaluated: usize,
        steps_consumed: usize,
        peak_memory_bytes: usize,
        outcome: FeasibilityOutcome,
    ) -> Self {
        Self {
            candidates_evaluated,
            steps_consumed,
            peak_memory_bytes,
            outcome,
        }
    }
}
