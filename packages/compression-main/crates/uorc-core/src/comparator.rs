use std::process::{Command, Stdio};
use std::io::Write;
use std::fmt;
use std::error::Error;

/// Defines a standard interface for external compression comparators.
pub trait Comparator {
    /// Identifies the comparator (e.g., "zstd").
    fn id(&self) -> &str;
    
    /// Compresses the input payload according to a specific effort plan.
    /// Returns the raw compressed bytes, or an error.
    fn compress(&self, input: &[u8], high_ratio: bool) -> Result<Vec<u8>, ComparatorError>;

    /// Decompresses the given payload.
    /// Used for mandatory full-reconstruction verification.
    fn decompress(&self, payload: &[u8]) -> Result<Vec<u8>, ComparatorError>;
}

/// Errors that can occur during comparator execution.
#[derive(Debug, PartialEq, Eq)]
pub enum ComparatorError {
    /// The external tool failed or returned a non-zero exit code.
    ExecutionFailed(String),
    /// Reconstructed bytes do not perfectly match the original payload.
    ReconstructionMismatch,
}

impl fmt::Display for ComparatorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ComparatorError::ExecutionFailed(msg) => write!(f, "Execution failed: {}", msg),
            ComparatorError::ReconstructionMismatch => write!(f, "Reconstruction mismatch: bytes do not perfectly match the original payload"),
        }
    }
}

impl Error for ComparatorError {}

/// Runs a comparator end-to-end, enforcing the mandatory reconstruction check (Issue 1403).
pub fn evaluate_comparator(comp: &dyn Comparator, input: &[u8], high_ratio: bool) -> Result<usize, ComparatorError> {
    // 1. Compress the data.
    let compressed = comp.compress(input, high_ratio)?;

    // 2. Mandatory validation: "Validate each comparator output by full reconstruction compare."
    let reconstructed = comp.decompress(&compressed)?;
    if reconstructed != input {
        return Err(ComparatorError::ReconstructionMismatch);
    }

    // 3. Return the compressed size metric.
    Ok(compressed.len())
}

/// Adapter for the `zstd` CLI.
pub struct ZstdComparator;

impl Comparator for ZstdComparator {
    fn id(&self) -> &str {
        "zstd"
    }

    fn compress(&self, input: &[u8], high_ratio: bool) -> Result<Vec<u8>, ComparatorError> {
        let level = if high_ratio { "-19" } else { "-3" };
        let mut child = Command::new("zstd")
            .arg(level)
            .arg("-c") // Output to stdout
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| ComparatorError::ExecutionFailed(e.to_string()))?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(input).map_err(|e| ComparatorError::ExecutionFailed(e.to_string()))?;
        }

        let output = child.wait_with_output().map_err(|e| ComparatorError::ExecutionFailed(e.to_string()))?;
        if output.status.success() {
            Ok(output.stdout)
        } else {
            Err(ComparatorError::ExecutionFailed(String::from_utf8_lossy(&output.stderr).to_string()))
        }
    }

    fn decompress(&self, payload: &[u8]) -> Result<Vec<u8>, ComparatorError> {
        let mut child = Command::new("zstd")
            .arg("-d")
            .arg("-c")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| ComparatorError::ExecutionFailed(e.to_string()))?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(payload).map_err(|e| ComparatorError::ExecutionFailed(e.to_string()))?;
        }

        let output = child.wait_with_output().map_err(|e| ComparatorError::ExecutionFailed(e.to_string()))?;
        if output.status.success() {
            Ok(output.stdout)
        } else {
            Err(ComparatorError::ExecutionFailed(String::from_utf8_lossy(&output.stderr).to_string()))
        }
    }
}
