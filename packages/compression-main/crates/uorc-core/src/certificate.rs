use crate::universe::{UniverseDescriptor, ExhaustiveEnumerator, EnumeratorStatus};
use crate::graph::GraphBody;

/// The outcome of verifying a UORP certificate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CertificateStatus {
    /// Certificate is fully verified to be the canonical minimum.
    Verified,
    /// Certificate is refuted (either invalid graph, better graph exists, or malformed).
    Refuted(String),
    /// The checker exhausted its budget before proving completeness.
    Incomplete,
}

/// A UORP v02 Certificate claiming a canonical minimum.
pub struct Certificate {
    /// The bounded universe for this claim.
    pub universe: UniverseDescriptor,
    /// The graph claimed to be the canonical minimum in this universe.
    pub claimed_minimum: GraphBody,
}

impl Certificate {
    /// Route 00 Checker: Verifies the certificate by exhaustively replaying 
    /// the universe search up to the given quantum budget.
    pub fn verify_by_replay(&self, quantum_budget: usize) -> CertificateStatus {
        // 1. Validate that the claimed minimum is actually admitted and eligible!
        if !self.universe.is_eligible(&self.claimed_minimum) {
            return CertificateStatus::Refuted("Claimed minimum is not eligible (UC4001)".to_string());
        }

        let claimed_cost = crate::layout::cost_of_graph(&self.claimed_minimum);

        // 2. Exhaustively enumerate the universe to prove no better candidate exists.
        let mut enumerator = ExhaustiveEnumerator::new(self.universe.clone());
        
        let (best_found_opt, status) = enumerator.run_quantum(quantum_budget);

        if let Some(best_found) = best_found_opt {
            let best_cost = crate::layout::cost_of_graph(&best_found);
            if best_cost < claimed_cost {
                return CertificateStatus::Refuted("A strictly better candidate was found (UC4002)".to_string());
            }
        }

        match status {
            EnumeratorStatus::Complete => CertificateStatus::Verified,
            EnumeratorStatus::Incomplete => CertificateStatus::Incomplete,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::Instruction;

    #[test]
    fn test_certificate_soundness() {
        let target = b"enumerator".to_vec(); // matches exactly permutation tick 10 in our mock enumerator
        let universe = UniverseDescriptor::new(target.clone(), 50, 1024, 1);
        
        let claimed_minimum = GraphBody {
            instructions: vec![Instruction::Literal(target.clone())],
        };

        let cert = Certificate {
            universe: universe.clone(),
            claimed_minimum,
        };

        // If quantum is too small, it returns Incomplete
        assert_eq!(cert.verify_by_replay(5), CertificateStatus::Incomplete);

        // If quantum is large enough to exhaust the space (max_instructions = 1), it verifies
        assert_eq!(cert.verify_by_replay(100), CertificateStatus::Verified);

        // Malformed certificate: the graph doesn't produce the target bytes.
        let bad_cert = Certificate {
            universe,
            claimed_minimum: GraphBody {
                instructions: vec![Instruction::Literal(b"wrong".to_vec())],
            },
        };
        
        if let CertificateStatus::Refuted(reason) = bad_cert.verify_by_replay(100) {
            assert!(reason.contains("not eligible"));
        } else {
            panic!("Expected Refuted status");
        }
    }
}
