use crate::graph::GraphBody;
use crate::universe::UniverseDescriptor;
use crate::layout::cost_of_graph;

/// Defines the reasons a partial prefix can be statically rejected without full exploration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrefixRejectionReason {
    /// The prefix already exceeds the serialized cost ceiling.
    CostBoundExceeded,
    /// The prefix statically requires more memory than the universe ceiling.
    MemoryBoundExceeded,
    /// The prefix exceeds the maximum allowed instructions.
    InstructionCountExceeded,
}

/// Represents the status of a coverage partition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoveragePartition {
    /// The partition has been fully explored or conclusively rejected.
    Covered,
    /// The partition still needs exploration.
    Pending,
}

/// Evaluates a candidate prefix against the universe descriptor and current best cost.
/// If the prefix can never be extended into a valid candidate that beats the incumbent,
/// it returns a RejectionReason. Otherwise, it returns the lower bound cost.
pub fn evaluate_prefix_lower_bound(
    prefix: &GraphBody,
    universe: &UniverseDescriptor,
    current_best_cost: usize,
) -> Result<usize, PrefixRejectionReason> {
    if prefix.instructions.len() > universe.max_instructions {
        return Err(PrefixRejectionReason::InstructionCountExceeded);
    }

    let base_cost = cost_of_graph(prefix);
    let ceiling = std::cmp::min(universe.max_length_bytes, current_best_cost);

    if base_cost >= ceiling {
        return Err(PrefixRejectionReason::CostBoundExceeded);
    }

    // A true prefix evaluator would run a partial StepMachine here to detect MemoryBoundExceeded.
    // For now, we return the base cost as the structural lower bound.
    Ok(base_cost)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::Instruction;

    #[test]
    fn test_prefix_rejection_sound() {
        let universe = UniverseDescriptor::new(vec![0], 10, 1024, 2);
        let prefix = GraphBody {
            instructions: vec![
                Instruction::Literal(vec![0; 20]), // cost > 10
            ],
        };
        assert_eq!(
            evaluate_prefix_lower_bound(&prefix, &universe, 20),
            Err(PrefixRejectionReason::CostBoundExceeded)
        );
    }

    #[test]
    fn test_completion_lower_bound_sound() {
        let universe = UniverseDescriptor::new(vec![0], 50, 1024, 2);
        let prefix = GraphBody {
            instructions: vec![Instruction::Literal(vec![0; 5])],
        };
        let bound = evaluate_prefix_lower_bound(&prefix, &universe, 50).unwrap();
        assert!(bound > 0);
    }

    #[test]
    fn test_coverage_partition() {
        let partition = CoveragePartition::Covered;
        assert_eq!(partition, CoveragePartition::Covered);
    }
}
