use crate::certificate::CertificateStatus;
use crate::evaluator::ResourceLedger;

/// The integrity and evaluation result of an archive reconstruction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodingIntegrity {
    /// Graph successfully reconstructed and validated against its internal integrity check.
    Verified,
    /// Evaluated successfully but payload bytes did not match expected integrity hash.
    Mismatch,
    /// Structural failure or bounds exceeded prior to successful evaluation.
    FatalError(String),
}

/// A formal reconstruction receipt conforming to `uorc/reconstruction-receipt/2`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReconstructionReceipt {
    /// Domain-separated SHA-256 identity of the evaluated archive.
    pub archive_identity: [u8; 32],
    /// The structural decoding and internal integrity status.
    pub decoding_integrity: DecodingIntegrity,
    /// External verification against original uncompressed corpus (if available).
    pub full_byte_equal: Option<bool>,
    /// Result of certificate (Route 00) bounded verification.
    pub certificate_status: Option<CertificateStatus>,
    /// Deterministic abstract usage metrics.
    pub usage_ledger: ResourceLedger,
    /// Identifier for deployment/versioning evidence surfaces.
    pub deployment_evidence_ref: Option<String>,
}

impl ReconstructionReceipt {
    /// Validates the receipt to ensure no unearned status fields are incorrectly claimed.
    pub fn is_valid(&self) -> bool {
        // Missing original input (or failed decode) cannot produce a positive `full_byte_equal` claim.
        if self.decoding_integrity != DecodingIntegrity::Verified && self.full_byte_equal == Some(true) {
            return false;
        }
        
        // Cannot claim a verified certificate if decoding integrity failed.
        if let Some(CertificateStatus::Verified) = self.certificate_status {
            if self.decoding_integrity != DecodingIntegrity::Verified {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_receipt_schema() {
        let receipt = ReconstructionReceipt {
            archive_identity: [0; 32],
            decoding_integrity: DecodingIntegrity::Verified,
            full_byte_equal: Some(true),
            certificate_status: Some(CertificateStatus::Verified),
            usage_ledger: ResourceLedger::default(),
            deployment_evidence_ref: None,
        };
        assert!(receipt.is_valid());
    }

    #[test]
    fn test_unearned_full_byte_equal_claim_rejected() {
        let receipt = ReconstructionReceipt {
            archive_identity: [0; 32],
            decoding_integrity: DecodingIntegrity::Mismatch,
            // Invalid: claiming byte equality when internal integrity mismatched.
            full_byte_equal: Some(true),
            certificate_status: None,
            usage_ledger: ResourceLedger::default(),
            deployment_evidence_ref: None,
        };
        assert!(!receipt.is_valid());
    }

    #[test]
    fn test_unearned_certificate_claim_rejected() {
        let receipt = ReconstructionReceipt {
            archive_identity: [0; 32],
            decoding_integrity: DecodingIntegrity::FatalError("OOM".into()),
            full_byte_equal: None,
            // Invalid: claiming verified canonical minimum on an archive that crashed.
            certificate_status: Some(CertificateStatus::Verified),
            usage_ledger: ResourceLedger::default(),
            deployment_evidence_ref: None,
        };
        assert!(!receipt.is_valid());
    }
}
