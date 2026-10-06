use crate::graph::GraphBody;
use crate::universe::{UniverseDescriptor, ExhaustiveEnumerator};
use crate::layout::cost_of_graph;
use crate::evaluator::{StepMachine, Value};

/// A point on the multiobjective frontier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrontierPoint {
    /// The graph body that achieved this point.
    pub graph: GraphBody,
    /// The total serialized byte cost.
    pub serialized_cost: usize,
    /// The peak live memory used during evaluation.
    pub peak_memory: usize,
}

impl FrontierPoint {
    /// Returns true if `self` strictly dominates `other`.
    pub fn dominates(&self, other: &FrontierPoint) -> bool {
        (self.serialized_cost <= other.serialized_cost && self.peak_memory <= other.peak_memory) &&
        (self.serialized_cost < other.serialized_cost || self.peak_memory < other.peak_memory)
    }
}

/// A collection of mutually incomparable optimal points.
#[derive(Debug, Clone, Default)]
pub struct Frontier {
    /// The incomparable points defining the frontier.
    pub points: Vec<FrontierPoint>,
}

impl Frontier {
    /// Attempts to insert a point. Returns true if it was added, false if it was rejected (dominated).
    pub fn insert(&mut self, point: FrontierPoint) -> bool {
        // Check if dominated by any existing point
        for existing in &self.points {
            if existing.dominates(&point) {
                return false;
            }
            if existing.serialized_cost == point.serialized_cost && existing.peak_memory == point.peak_memory {
                // Identity tie policy: Keep the first one discovered.
                return false;
            }
        }

        // Remove any points that the new point dominates
        self.points.retain(|existing| !point.dominates(existing));
        self.points.push(point);
        true
    }
}

/// The result of a frontier analysis operation.
#[derive(Debug, Clone)]
pub enum FrontierReceipt {
    /// The space was completely explored.
    Complete(Frontier),
    /// The search was cut off due to budget; estimates outstanding permutations.
    Incomplete {
        /// The current best frontier found so far.
        frontier: Frontier,
        /// Estimated search permutations remaining.
        outstanding_permutations: u64,
    },
}

/// Bounded exhaustive replay to calculate the multiobjective frontier.
pub fn analyze_frontier(universe: &UniverseDescriptor, quantum: usize) -> FrontierReceipt {
    let mut enumerator = ExhaustiveEnumerator::new(universe.clone());
    let mut frontier = Frontier::default();

    let mut completed = false;

    for _ in 0..quantum {
        if let Some(candidate) = enumerator.next_candidate() {
            if universe.is_admitted(&candidate) {
                let mut machine = StepMachine::new(universe.max_memory_bytes);
                if let Ok(Value::Bytes(b)) = machine.evaluate(&candidate) {
                    if b == universe.target_bytes {
                        let pt = FrontierPoint {
                            serialized_cost: cost_of_graph(&candidate),
                            peak_memory: machine.ledger.peak_live_memory,
                            graph: candidate,
                        };
                        frontier.insert(pt);
                    }
                }
            }
        } else {
            completed = true;
            break;
        }
    }

    if completed {
        FrontierReceipt::Complete(frontier)
    } else {
        let instructions_left = universe.max_instructions.saturating_sub(enumerator.current_instruction_count);
        let current_layer_left = 21_u64.saturating_sub(enumerator.permutations_checked);
        let outstanding = (instructions_left as u64 * 21) + current_layer_left;

        FrontierReceipt::Incomplete {
            frontier,
            outstanding_permutations: outstanding,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_frontier_sound_complete() {
        let target = b"frontier".to_vec(); // won't naturally hit exactly in our stub generator unless we pass quantum long enough?
        // Wait, ExhaustiveEnumerator yields exact match at (current_instruction_count == 1 && permutations == 10)
        let universe = UniverseDescriptor::new(target, 50, 1024, 2);
        let receipt = analyze_frontier(&universe, 100);
        
        match receipt {
            FrontierReceipt::Complete(f) => {
                // Must have found at least one point (the simulated match at tick 10)
                assert!(!f.points.is_empty());
            },
            _ => panic!("Expected complete receipt"),
        }
    }

    #[test]
    fn test_frontier_incomplete() {
        let target = b"frontier".to_vec();
        let universe = UniverseDescriptor::new(target, 50, 1024, 2);
        
        // Quantum 2 is too small to complete
        let receipt = analyze_frontier(&universe, 2);
        
        match receipt {
            FrontierReceipt::Incomplete { outstanding_permutations, .. } => {
                assert!(outstanding_permutations > 0);
            },
            _ => panic!("Expected incomplete receipt"),
        }
    }
}
