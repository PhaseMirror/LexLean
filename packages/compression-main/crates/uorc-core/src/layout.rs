use std::io::{self, Write};
use crate::graph::GraphBody;

/// A writer that simply counts the bytes written without storing them.
#[derive(Default)]
pub struct SizeCounter {
    /// Total bytes counted.
    pub count: usize,
}

impl SizeCounter {
    /// Create a new SizeCounter.
    pub fn new() -> Self {
        Self { count: 0 }
    }
}

impl Write for SizeCounter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.count += buf.len();
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Computes the exact serialized byte cost of a GraphBody.
pub fn cost_of_graph(graph: &GraphBody) -> usize {
    let mut counter = SizeCounter::new();
    let _ = graph.serialize(&mut counter);
    counter.count
}

/// Context-sensitive topological layout optimizer.
pub struct LayoutOptimizer;

impl LayoutOptimizer {
    /// Evaluates whether sharing an expression via a Reference is cheaper than inlining it,
    /// given a specific index assignment in the current topological layout.
    pub fn should_share(inline_cost: usize, index: usize) -> bool {
        // A Reference costs 1 byte (opcode) + ULEB128(index) bytes.
        let mut counter = SizeCounter::new();
        let _ = counter.write(&[0x01]); // Opcode::Reference
        let _ = crate::uleb128::encode_uleb128(&mut counter, index as u64);
        let reference_cost = counter.count;

        // Sharing is preferred if the cost of the reference is strictly less than inlining it.
        // If it's equal, we might prefer inlining to save evaluation peak memory,
        // but strictly based on serialization size:
        reference_cost < inline_cost
    }

    /// Evaluates contextual dominance between two candidate layouts for the same logical graph.
    /// Returns true if `candidate_a` strictly dominates `candidate_b` in terms of serialized cost.
    pub fn dominates(candidate_a: &GraphBody, candidate_b: &GraphBody) -> bool {
        cost_of_graph(candidate_a) < cost_of_graph(candidate_b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::Instruction;

    #[test]
    fn test_contextual_dominance_sound() {
        // Candidate A: Inline a 1-byte literal twice.
        // Cost: Header(1) + Literal(1) = 1 + 1 + 1 (len) + 1 (data) = 4. 
        // Two literals = 8 bytes.
        // Plus Concat = 3 bytes. Total = 11 bytes.
        let inline_graph = GraphBody {
            instructions: vec![
                Instruction::Literal(b"A".to_vec()),
                Instruction::Literal(b"A".to_vec()),
                Instruction::Concat(0, 1),
            ],
        };

        // Candidate B: Share a 1-byte literal.
        // Literal(1) = 3 bytes.
        // Reference(0) = 2 bytes.
        // Concat(0, 1) = 3 bytes. Total = 8 bytes. Wait, plus graph header (1 byte) = 9 bytes.
        let shared_graph = GraphBody {
            instructions: vec![
                Instruction::Literal(b"A".to_vec()),
                Instruction::Reference(0),
                Instruction::Concat(0, 1),
            ],
        };

        let inline_cost = cost_of_graph(&inline_graph);
        let shared_cost = cost_of_graph(&shared_graph);

        // Sharing a 1-byte literal might or might not be cheaper depending on ULEB limits.
        // Let's print sizes for safety in test logic.
        assert!(LayoutOptimizer::dominates(&shared_graph, &inline_graph));
        assert_eq!(inline_cost, 10); // Wait, Header: 1. Lit1: 3. Lit2: 3. Concat: 3 = 10 bytes.
        assert_eq!(shared_cost, 9); // Header: 1. Lit1: 3. Ref: 2. Concat: 3 = 9 bytes.

        // What if the payload is 0 bytes?
        let empty_inline = GraphBody {
            instructions: vec![
                Instruction::Literal(b"".to_vec()),
                Instruction::Literal(b"".to_vec()),
                Instruction::Concat(0, 1),
            ],
        };
        // Lit0: 2 bytes. Lit1: 2 bytes. Concat: 3. Header: 1 = 8 bytes.

        let empty_shared = GraphBody {
            instructions: vec![
                Instruction::Literal(b"".to_vec()),
                Instruction::Reference(0),
                Instruction::Concat(0, 1),
            ],
        };
        // Lit0: 2 bytes. Ref: 2 bytes. Concat: 3. Header: 1 = 8 bytes.
        
        // In this case, empty_shared cost (8) is NOT < empty_inline (8).
        assert!(!LayoutOptimizer::dominates(&empty_shared, &empty_inline));
        assert!(!LayoutOptimizer::dominates(&empty_inline, &empty_shared));
    }

    #[test]
    fn test_should_share() {
        // Inlining a 1-byte literal costs 3 bytes.
        // Reference to index 0 costs 2 bytes.
        assert!(LayoutOptimizer::should_share(3, 0));
        
        // Inlining a 0-byte literal costs 2 bytes.
        // Reference to index 0 costs 2 bytes.
        // Not cheaper to share!
        assert!(!LayoutOptimizer::should_share(2, 0));

        // Inlining a 1-byte literal (cost 3).
        // Reference to index 128 (requires 2-byte ULEB128) + 1 opcode = 3 bytes.
        // Not cheaper to share!
        assert!(!LayoutOptimizer::should_share(3, 128));
    }
}
