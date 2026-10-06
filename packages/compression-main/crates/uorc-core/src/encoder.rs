
use crate::graph::{GraphBody, Instruction};
use crate::evaluator::{StepMachine, Value};

/// Tracks the current best encoded graph for a given target byte sequence.
pub struct IncumbentManager {
    /// The target byte sequence to match.
    pub target_bytes: Vec<u8>,
    /// The current best graph body.
    pub best_graph: GraphBody,
    /// The serialized size of the current best graph body.
    pub best_length: usize,
}

impl IncumbentManager {
    /// Bootstraps an incumbent manager using the canonical raw incumbent (a single Literal instruction).
    pub fn new(target_bytes: Vec<u8>) -> Self {
        let raw_graph = GraphBody {
            instructions: vec![Instruction::Literal(target_bytes.clone())],
        };
        
        let mut buf = Vec::new();
        raw_graph.serialize(&mut buf).expect("serialization of raw graph should not fail");
        let best_length = buf.len();

        Self {
            target_bytes,
            best_graph: raw_graph,
            best_length,
        }
    }

    /// Evaluates a candidate graph. If it correctly reconstructs the target bytes
    /// and its serialized length is strictly less than the current best length,
    /// updates the incumbent and returns true. Otherwise returns false.
    pub fn evaluate_candidate(&mut self, candidate: GraphBody, memory_limit: usize) -> bool {
        // 1. Mandatory verification: Candidate must evaluate without error.
        let mut machine = StepMachine::new(memory_limit);
        let result = match machine.evaluate(&candidate) {
            Ok(v) => v,
            Err(_) => return false,
        };
        
        // 2. Candidate must reconstruct the exact target bytes.
        let result_bytes = match result {
            Value::Bytes(b) => b,
            _ => return false, // Must resolve to bytes
        };
        if result_bytes != self.target_bytes {
            return false;
        }

        // 3. Serialize to check length
        let mut buf = Vec::new();
        if candidate.serialize(&mut buf).is_err() {
            return false;
        }
        
        // 4. Enforce monotonic incumbent updates on length
        if buf.len() < self.best_length {
            self.best_graph = candidate;
            self.best_length = buf.len();
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_incumbent_bootstrap() {
        let target = b"hello".to_vec();
        let manager = IncumbentManager::new(target.clone());
        assert_eq!(manager.target_bytes, target);
        assert_eq!(manager.best_graph.instructions.len(), 1);
        if let Instruction::Literal(ref payload) = manager.best_graph.instructions[0] {
            assert_eq!(payload, b"hello");
        } else {
            panic!("expected Literal instruction");
        }
    }

    #[test]
    fn test_encoder_success_lossless_and_monotone() {
        // Target: "hellohello"
        let target = b"hellohellohellohello".to_vec();
        let mut manager = IncumbentManager::new(target.clone());
        let initial_length = manager.best_length;

        // Better candidate using reference
        let better_graph = GraphBody {
            instructions: vec![
                Instruction::Literal(b"hello".to_vec()), // 0
                Instruction::Reference(0),               // 1
                Instruction::Concat(0, 1),               // 2 -> "hellohello"
                Instruction::Reference(2),               // 3 -> "hellohello"
                Instruction::Concat(2, 3),               // 4 -> "hellohellohellohello"
            ],
        };

        let updated = manager.evaluate_candidate(better_graph.clone(), 1024);
        assert!(updated, "better graph should be accepted");
        assert!(manager.best_length < initial_length, "monotonic update failed");
        assert_eq!(manager.best_graph, better_graph);

        // Worse candidate (raw literal again)
        let worse_graph = GraphBody {
            instructions: vec![Instruction::Literal(b"hellohellohellohello".to_vec())],
        };
        let updated_worse = manager.evaluate_candidate(worse_graph, 1024);
        assert!(!updated_worse, "worse or equal graph should not be accepted");

        // Invalid candidate (wrong bytes)
        let wrong_graph = GraphBody {
            instructions: vec![Instruction::Literal(b"helloworld".to_vec())],
        };
        let updated_wrong = manager.evaluate_candidate(wrong_graph, 1024);
        assert!(!updated_wrong, "graph producing wrong bytes should not be accepted");
    }
}
