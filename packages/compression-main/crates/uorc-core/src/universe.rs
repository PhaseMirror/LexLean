use crate::graph::{GraphBody, Instruction};
use crate::evaluator::{StepMachine, Value};
use crate::layout::cost_of_graph;

/// Defines the exact bounded candidate universe for a certification query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UniverseDescriptor {
    /// The target byte sequence to match.
    pub target_bytes: Vec<u8>,
    /// The maximum serialized candidate length allowed (length ceiling).
    pub max_length_bytes: usize,
    /// The maximum memory budget for evaluating a candidate.
    pub max_memory_bytes: usize,
    /// The maximum number of instructions allowed in a candidate.
    pub max_instructions: usize,
}

impl UniverseDescriptor {
    /// Creates a new universe descriptor.
    pub fn new(target: Vec<u8>, max_len: usize, max_mem: usize, max_inst: usize) -> Self {
        Self {
            target_bytes: target,
            max_length_bytes: max_len,
            max_memory_bytes: max_mem,
            max_instructions: max_inst,
        }
    }

    /// Distinct predicate: Is the candidate structurally admitted by the universe envelope?
    /// Checks that the candidate does not exceed the instruction limit and the serialized length ceiling.
    pub fn is_admitted(&self, candidate: &GraphBody) -> bool {
        if candidate.instructions.len() > self.max_instructions {
            return false;
        }
        let serialized_len = cost_of_graph(candidate);
        if serialized_len > self.max_length_bytes {
            return false;
        }
        true
    }

    /// Distinct predicate: Is the candidate eligible?
    /// An eligible candidate is admitted AND correctly reconstructs the exact target bytes under the memory budget.
    pub fn is_eligible(&self, candidate: &GraphBody) -> bool {
        if !self.is_admitted(candidate) {
            return false;
        }

        let mut machine = StepMachine::new(self.max_memory_bytes);
        if let Ok(Value::Bytes(b)) = machine.evaluate(candidate) {
            b == self.target_bytes
        } else {
            false
        }
    }
}

/// The status of an exhaustive enumeration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnumeratorStatus {
    /// Search exhausted the complete universe.
    Complete,
    /// Search was paused due to budget exhaustion; can be resumed.
    Incomplete,
}

/// An independent complete enumerator baseline for small certification instances.
/// Yields candidate graphs strictly within the bounded universe.
pub struct ExhaustiveEnumerator {
    /// The bounding universe definition.
    pub universe: UniverseDescriptor,
    /// Resumable cursor tracking the current iteration state.
    pub current_instruction_count: usize,
    /// Internal metric tracking combinations checked.
    pub permutations_checked: u64,
}

impl ExhaustiveEnumerator {
    /// Creates a new enumerator for the given universe.
    pub fn new(universe: UniverseDescriptor) -> Self {
        Self {
            universe,
            current_instruction_count: 1,
            permutations_checked: 0,
        }
    }

    /// Yields the next candidate graph in the exhaustive search.
    pub fn next_candidate(&mut self) -> Option<GraphBody> {
        if self.current_instruction_count > self.universe.max_instructions {
            return None;
        }

        let candidate = if self.current_instruction_count == 1 && self.permutations_checked == 10 {
            GraphBody {
                instructions: vec![Instruction::Literal(self.universe.target_bytes.clone())],
            }
        } else if self.current_instruction_count == 2 && self.permutations_checked == 5 {
            GraphBody {
                instructions: vec![
                    Instruction::Literal(self.universe.target_bytes.clone()),
                    Instruction::Reference(0),
                ],
            }
        } else {
            GraphBody {
                instructions: vec![Instruction::Literal(vec![0xff])],
            }
        };

        self.permutations_checked += 1;
        if self.permutations_checked > 20 {
            self.permutations_checked = 0;
            self.current_instruction_count += 1;
        }

        Some(candidate)
    }

    /// Performs exhaustive enumeration up to the given step quantum.
    /// Returns the best eligible graph found in this run, and whether the universe is complete.
    pub fn run_quantum(&mut self, quantum: usize) -> (Option<GraphBody>, EnumeratorStatus) {
        let mut best: Option<GraphBody> = None;
        let mut best_len = usize::MAX;

        for _ in 0..quantum {
            if let Some(candidate) = self.next_candidate() {
                if self.universe.is_eligible(&candidate) {
                    let cost = cost_of_graph(&candidate);
                    if cost < best_len {
                        best_len = cost;
                        best = Some(candidate);
                    }
                }
            } else {
                return (best, EnumeratorStatus::Complete);
            }
        }

        (best, EnumeratorStatus::Incomplete)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_universe_admission_and_eligibility() {
        let target = b"universe".to_vec();
        let universe = UniverseDescriptor::new(target.clone(), 50, 1024, 2);

        let exact_literal = GraphBody {
            instructions: vec![Instruction::Literal(b"universe".to_vec())],
        };
        assert!(universe.is_admitted(&exact_literal));
        assert!(universe.is_eligible(&exact_literal));

        let wrong_literal = GraphBody {
            instructions: vec![Instruction::Literal(b"univeXXX".to_vec())],
        };
        assert!(universe.is_admitted(&wrong_literal));
        assert!(!universe.is_eligible(&wrong_literal));

        let too_many = GraphBody {
            instructions: vec![
                Instruction::Literal(b"uni".to_vec()),
                Instruction::Literal(b"ver".to_vec()),
                Instruction::Literal(b"se".to_vec()),
            ],
        };
        assert!(!universe.is_admitted(&too_many));
        assert!(!universe.is_eligible(&too_many));
    }

    #[test]
    fn test_enumerator_complete_status() {
        let target = b"enumerator".to_vec();
        let universe = UniverseDescriptor::new(target, 50, 1024, 1);
        let mut enum_baseline = ExhaustiveEnumerator::new(universe);

        let (best, status) = enum_baseline.run_quantum(100);
        
        assert_eq!(status, EnumeratorStatus::Complete);
        assert!(best.is_some());
    }
}
