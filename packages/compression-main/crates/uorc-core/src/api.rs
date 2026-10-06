use crate::archive::{ArchiveFrame, Block};
use crate::graph::GraphBody;
use crate::evaluator::{StepMachine, Value};
use crate::scheduler::SynthesisTask;
use std::io::Cursor;
use std::fmt;
use std::error::Error;

/// Types of semantic errors that occur during API usage.
#[derive(Debug, PartialEq, Eq)]
pub enum ApiError {
    /// Failed to decompress because the archive magic bytes or structure is invalid.
    InvalidArchive,
    /// Failed to evaluate the decompression graph.
    EvaluationFailure,
    /// Failed to find a valid compression graph within the allowed bounds.
    SynthesisFailure,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::InvalidArchive => write!(f, "Invalid archive formatting or magic bytes."),
            ApiError::EvaluationFailure => write!(f, "Evaluation of the decompression graph failed."),
            ApiError::SynthesisFailure => write!(f, "Synthesis failed to find a valid compression graph."),
        }
    }
}

impl Error for ApiError {}

/// Compress arbitrary bytes into a UORC archive payload.
/// This acts as the programmatic API root for compression.
pub fn compress(target_bytes: &[u8], memory_limit: usize) -> Result<Vec<u8>, ApiError> {
    // Before launching search, verify the baseline literal representation even fits in memory.
    if target_bytes.len() > memory_limit {
        return Err(ApiError::SynthesisFailure);
    }

    // We launch a synthesis task to find a compression graph.
    let mut task = SynthesisTask::new(target_bytes.to_vec(), memory_limit);
    
    // For Epic 11, we bound the effort natively to demonstrate the operational API.
    // Real deployments use the checkpointing API to stretch this over time.
    let _ = task.step(1000); // We ignore the return as we just take the best found so far.
    
    // The IncumbentManager starts with a base literal encoding, so this is safe.
    let mut graph_bytes = Vec::new();
    task.incumbent.best_graph.serialize(&mut graph_bytes).map_err(|_| ApiError::SynthesisFailure)?;

    let frame = ArchiveFrame {
        blocks: vec![Block { payload: graph_bytes }],
    };

    let mut archive_payload = Vec::new();
    frame.serialize(&mut archive_payload).map_err(|_| ApiError::SynthesisFailure)?;
    
    Ok(archive_payload)
}

/// Decompress a UORC archive payload back to the original bytes.
/// This acts as the programmatic API root for decompression.
pub fn decompress(archive_payload: &[u8], memory_limit: usize) -> Result<Vec<u8>, ApiError> {
    let frame = ArchiveFrame::parse(Cursor::new(archive_payload))
        .map_err(|_| ApiError::InvalidArchive)?;
    
    if frame.blocks.is_empty() {
        return Err(ApiError::InvalidArchive);
    }
    
    let graph = GraphBody::parse(Cursor::new(&frame.blocks[0].payload))
        .map_err(|_| ApiError::InvalidArchive)?;
    
    let mut evaluator = StepMachine::new(memory_limit);
    let result = evaluator.evaluate(&graph).map_err(|_| ApiError::EvaluationFailure)?;
    
    match result {
        Value::Bytes(b) => Ok(b),
        _ => Err(ApiError::EvaluationFailure)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_compress_decompress_roundtrip() {
        let payload = b"Hello, UORC Operational API!".to_vec();
        
        // Compress
        let compressed = compress(&payload, 1024).expect("Compression should succeed");
        assert!(!compressed.is_empty());
        
        // Decompress
        let decompressed = decompress(&compressed, 1024).expect("Decompression should succeed");
        
        // Exact reconstruction
        assert_eq!(payload, decompressed);
    }

    #[test]
    fn test_api_invalid_archive() {
        let garbage = vec![0xFF, 0x00, 0x11, 0x22];
        let err = decompress(&garbage, 1024).unwrap_err();
        assert_eq!(err, ApiError::InvalidArchive);
    }

    #[test]
    fn test_api_evaluation_failure_due_to_memory_limit() {
        let payload = b"Large payload that exceeds memory limits!".to_vec();
        let compressed = compress(&payload, 1024).expect("Compression should succeed");
        
        // Attempt decompression with a tiny memory budget (e.g., 5 bytes)
        let err = decompress(&compressed, 5).unwrap_err();
        assert_eq!(err, ApiError::EvaluationFailure);
    }
}
