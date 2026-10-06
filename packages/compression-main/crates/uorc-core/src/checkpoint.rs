use std::fmt;
use std::error::Error;

/// The magical bytes required to start a replay checkpoint.
pub const CHECKPOINT_MAGIC: &[u8; 24] = b"uorc/replay-checkpoint/2";

/// Errors that can occur when parsing or validating a checkpoint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckpointError {
    /// The checkpoint magic bytes are invalid.
    InvalidMagic,
    /// The identity in the checkpoint does not match the expected identity.
    IdentityMismatch {
        /// The expected identity.
        expected: [u8; 32],
        /// The actual identity found in the checkpoint.
        actual: [u8; 32],
    },
    /// The checkpoint contains additional bytes, which might be an attempt to import trusted state.
    TrustedStateRejected,
}

impl fmt::Display for CheckpointError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CheckpointError::InvalidMagic => write!(f, "Invalid checkpoint magic. Expected `uorc/replay-checkpoint/2`"),
            CheckpointError::IdentityMismatch { expected, actual } => {
                write!(f, "Identity mismatch. Expected target {:?}, got {:?}", expected, actual)
            },
            CheckpointError::TrustedStateRejected => write!(f, "Trusted state import rejected. Checkpoints must not contain opaque state like visited sets."),
        }
    }
}

impl Error for CheckpointError {}

/// A deterministically replayable checkpoint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayCheckpoint {
    /// The identity of the target problem (e.g. query/universe).
    pub target_identity: [u8; 32],
    /// The identity of the scheduling policy used.
    pub policy_identity: [u8; 32],
    /// The number of logical steps to replay from the beginning.
    pub replay_steps: u64,
}

impl ReplayCheckpoint {
    /// Creates a new `ReplayCheckpoint`.
    pub fn new(target_identity: [u8; 32], policy_identity: [u8; 32], replay_steps: u64) -> Self {
        Self {
            target_identity,
            policy_identity,
            replay_steps,
        }
    }

    /// Serializes the checkpoint to a byte vector.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(96);
        buf.extend_from_slice(CHECKPOINT_MAGIC);
        buf.extend_from_slice(&self.target_identity);
        buf.extend_from_slice(&self.policy_identity);
        buf.extend_from_slice(&self.replay_steps.to_le_bytes());
        buf
    }

    /// Parses and validates a checkpoint from bytes.
    ///
    /// This method enforces that no trusted state (like opaque cursors or visited sets)
    /// is included in the checkpoint, as required by Issue 1001.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CheckpointError> {
        if bytes.len() < 96 {
            return Err(CheckpointError::InvalidMagic);
        }

        if &bytes[0..24] != CHECKPOINT_MAGIC {
            return Err(CheckpointError::InvalidMagic);
        }

        let mut target_identity = [0u8; 32];
        target_identity.copy_from_slice(&bytes[24..56]);

        let mut policy_identity = [0u8; 32];
        policy_identity.copy_from_slice(&bytes[56..88]);

        let mut replay_steps_bytes = [0u8; 8];
        replay_steps_bytes.copy_from_slice(&bytes[88..96]);
        let replay_steps = u64::from_le_bytes(replay_steps_bytes);

        // Reject any extra bytes. This explicitly prevents importing trusted states like visited sets.
        if bytes.len() > 96 {
             return Err(CheckpointError::TrustedStateRejected);
        }

        Ok(Self {
            target_identity,
            policy_identity,
            replay_steps,
        })
    }

    /// Verifies the checkpoint against expected identities.
    /// Returns `Ok(())` if they match, or `IdentityMismatch` if they do not.
    pub fn verify_identities(&self, expected_target: &[u8; 32], expected_policy: &[u8; 32]) -> Result<(), CheckpointError> {
        if self.target_identity != *expected_target {
            return Err(CheckpointError::IdentityMismatch {
                expected: *expected_target,
                actual: self.target_identity,
            });
        }
        if self.policy_identity != *expected_policy {
            return Err(CheckpointError::IdentityMismatch {
                expected: *expected_policy,
                actual: self.policy_identity,
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checkpoint_roundtrip() {
        let target = [1u8; 32];
        let policy = [2u8; 32];
        let ckpt = ReplayCheckpoint::new(target, policy, 123456);

        let bytes = ckpt.to_bytes();
        assert_eq!(bytes.len(), 96);

        let parsed = ReplayCheckpoint::from_bytes(&bytes).expect("Should parse");
        assert_eq!(parsed, ckpt);
    }

    #[test]
    fn test_invalid_magic() {
        let mut bytes = vec![0u8; 96];
        bytes[0..24].copy_from_slice(b"uorc/replay-checkpoint/1"); // Wrong version
        assert_eq!(ReplayCheckpoint::from_bytes(&bytes), Err(CheckpointError::InvalidMagic));
    }

    #[test]
    fn test_trusted_state_rejected() {
        let target = [1u8; 32];
        let policy = [2u8; 32];
        let ckpt = ReplayCheckpoint::new(target, policy, 123456);
        let mut bytes = ckpt.to_bytes();

        // Simulate attaching a visited set or opaque cursor to the end
        bytes.push(0xFF);

        assert_eq!(ReplayCheckpoint::from_bytes(&bytes), Err(CheckpointError::TrustedStateRejected));
    }

    #[test]
    fn test_identity_mismatch() {
        let target = [1u8; 32];
        let policy = [2u8; 32];
        let ckpt = ReplayCheckpoint::new(target, policy, 100);

        let expected_target = [1u8; 32];
        let wrong_policy = [3u8; 32];

        assert_eq!(
            ckpt.verify_identities(&expected_target, &wrong_policy),
            Err(CheckpointError::IdentityMismatch {
                expected: wrong_policy,
                actual: policy,
            })
        );
    }
}
