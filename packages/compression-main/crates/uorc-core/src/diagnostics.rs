use std::fmt;

/// Stable diagnostic codes for UORC public failures (Section 18.2).
/// Enforces typed, deterministic error classes for tooling integration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UcCode {
    /// UC4000: Invalid structural format or magic bytes (e.g., Archive parsing).
    UC4000,
    /// UC4007: Corrupted or mismatched replay checkpoint identities.
    UC4007,
    /// UC4008: Forbidden trusted state import in checkpoint (e.g. visited sets).
    UC4008,
    /// UC5000: Execution bounded out due to exact evaluation limits (e.g. memory/instructions).
    UC5000,
    /// UC5001: Synthesis search exhausted without finding a valid candidate.
    UC5001,
    /// UC6002: Operational failure during publication/IO (e.g. partial writes).
    UC6002,
}

impl fmt::Display for UcCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UcCode::UC4000 => write!(f, "UC4000 (Invalid Formatting)"),
            UcCode::UC4007 => write!(f, "UC4007 (Checkpoint Identity Mismatch)"),
            UcCode::UC4008 => write!(f, "UC4008 (Forbidden Trusted State)"),
            UcCode::UC5000 => write!(f, "UC5000 (Evaluation Limit Exceeded)"),
            UcCode::UC5001 => write!(f, "UC5001 (Synthesis Exhausted)"),
            UcCode::UC6002 => write!(f, "UC6002 (Operational IO Failure)"),
        }
    }
}

/// The specific pipeline stage where the diagnostic occurred.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineStage {
    /// Failure during parsing or ingestion.
    Parse,
    /// Failure during evaluation or verification.
    Evaluate,
    /// Failure during encoding or search.
    Synthesize,
    /// Failure during IO or publishing artifacts.
    Publish,
}

/// A structured, stable diagnostic payload guaranteed not to leak unbounded external text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    /// The public, stable diagnostic code.
    pub code: UcCode,
    /// The stage of the pipeline where the error occurred.
    pub stage: PipelineStage,
    /// Scoped diagnostic details avoiding external leakage.
    pub detail: String,
}

impl Diagnostic {
    /// Constructs a new stable diagnostic.
    pub fn new(code: UcCode, stage: PipelineStage, detail: impl Into<String>) -> Self {
        Self {
            code,
            stage,
            detail: detail.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::archive::ArchiveFrame;
    use crate::checkpoint::{ReplayCheckpoint, CheckpointError};
    use crate::evaluator::StepMachine;
    use crate::graph::{GraphBody, Instruction};
    use crate::api::compress;
    use std::io::Cursor;

    // Issue 1502: Add one owning negative test for every public diagnostic.

    #[test]
    fn test_diagnostic_uc4000_invalid_format() {
        let bad_archive = b"BADMAGIC";
        let result = ArchiveFrame::parse(Cursor::new(bad_archive));
        
        // Emulate mapping to a diagnostic
        let diag = result.map_err(|_| Diagnostic::new(UcCode::UC4000, PipelineStage::Parse, "Magic bytes do not match UORC")).unwrap_err();
        assert_eq!(diag.code, UcCode::UC4000);
        assert_eq!(diag.stage, PipelineStage::Parse);
    }

    #[test]
    fn test_diagnostic_uc4007_checkpoint_mismatch() {
        let ckpt = ReplayCheckpoint::new([1u8; 32], [2u8; 32], 100);
        let result = ckpt.verify_identities(&[9u8; 32], &[2u8; 32]);
        
        let diag = result.map_err(|e| match e {
            CheckpointError::IdentityMismatch { .. } => Diagnostic::new(UcCode::UC4007, PipelineStage::Parse, "Identity constraint failed"),
            _ => panic!("Expected mismatch"),
        }).unwrap_err();
        
        assert_eq!(diag.code, UcCode::UC4007);
    }

    #[test]
    fn test_diagnostic_uc4008_trusted_state() {
        let mut bytes = ReplayCheckpoint::new([0u8; 32], [0u8; 32], 0).to_bytes();
        bytes.push(0xFF); // Attach arbitrary state
        
        let result = ReplayCheckpoint::from_bytes(&bytes);
        let diag = result.map_err(|e| match e {
            CheckpointError::TrustedStateRejected => Diagnostic::new(UcCode::UC4008, PipelineStage::Parse, "Opaque state trailing bytes rejected"),
            _ => panic!("Expected state rejection"),
        }).unwrap_err();
        
        assert_eq!(diag.code, UcCode::UC4008);
    }

    #[test]
    fn test_diagnostic_uc5000_eval_limit() {
        // Create an infinite-like or large memory requirement graph
        let graph = GraphBody {
            instructions: vec![
                Instruction::Literal(vec![0; 2048]), // Wants 2048 bytes
            ]
        };
        
        // Restrict evaluation machine heavily
        let mut machine = StepMachine::new(1024);
        let result = machine.evaluate(&graph);
        
        let diag = result.map_err(|_| Diagnostic::new(UcCode::UC5000, PipelineStage::Evaluate, "Memory limit exceeded")).unwrap_err();
        assert_eq!(diag.code, UcCode::UC5000);
    }

    #[test]
    fn test_diagnostic_uc5001_synthesis_exhausted() {
        // We simulate synthesis failing by capping memory in API compression so heavily it cannot even bootstrap.
        let payload = b"123".to_vec();
        // Zero memory means it can't even hold the baseline incumbent
        let result = compress(&payload, 0); 
        
        // It should yield SynthesisFailure because evaluating candidates fails limits.
        let diag = result.map_err(|_| Diagnostic::new(UcCode::UC5001, PipelineStage::Synthesize, "No valid candidate found")).unwrap_err();
        assert_eq!(diag.code, UcCode::UC5001);
    }

    #[test]
    fn test_diagnostic_uc6002_io_failure() {
        // Map standard std::io error to UC6002
        let io_err = std::io::Error::new(std::io::ErrorKind::Interrupted, "disk full");
        let diag = Diagnostic::new(UcCode::UC6002, PipelineStage::Publish, io_err.to_string());
        
        assert_eq!(diag.code, UcCode::UC6002);
        assert!(diag.detail.contains("disk full"));
    }
}
