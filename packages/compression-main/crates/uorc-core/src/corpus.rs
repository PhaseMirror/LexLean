use serde::{Deserialize, Serialize};

/// Identifies a strictly verified external corpus component.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CorpusAsset {
    /// The canonical identity/name (e.g., `enwik8`).
    pub id: String,
    /// The immutable exact byte size required.
    pub exact_size: usize,
    /// The expected exact SHA-256 digest of the raw byte stream.
    pub exact_digest: String,
    /// Provenance tags and rights constraints (e.g. `Public Domain`).
    pub provenance: String,
}

/// A sealed, immutable inventory of benchmark corpus assets.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CorpusInventory {
    /// Ordered list of strictly checked benchmark corpus suites.
    pub assets: Vec<CorpusAsset>,
}

/// Errors that can occur when verifying a corpus against a sealed inventory.
#[derive(Debug, PartialEq, Eq)]
pub enum CorpusError {
    /// A generic JSON parsing error.
    ParseError(String),
    /// The specified asset ID was not found in the inventory.
    MissingAsset(String),
    /// The actual file size did not match the strictly required byte count.
    SizeMismatch {
        /// Asset identifier.
        id: String,
        /// Required size.
        expected: usize,
        /// Actual size.
        actual: usize
    },
    /// The actual file SHA-256 digest did not match the strictly required digest.
    DigestMismatch {
        /// Asset identifier.
        id: String,
        /// Required digest.
        expected: String,
        /// Actual digest.
        actual: String
    },
}

impl CorpusInventory {
    /// Parses a sealed corpus inventory from JSON.
    pub fn from_json(json: &str) -> Result<Self, CorpusError> {
        serde_json::from_str(json).map_err(|e| CorpusError::ParseError(e.to_string()))
    }

    /// Validates an actual file stream against the sealed corpus rules.
    pub fn verify_asset(&self, id: &str, size: usize, digest_hex: &str) -> Result<(), CorpusError> {
        let asset = self.assets.iter().find(|a| a.id == id).ok_or_else(|| {
            CorpusError::MissingAsset(id.to_string())
        })?;

        if size != asset.exact_size {
            return Err(CorpusError::SizeMismatch {
                id: id.to_string(),
                expected: asset.exact_size,
                actual: size,
            });
        }

        if digest_hex != asset.exact_digest.to_lowercase() {
            return Err(CorpusError::DigestMismatch {
                id: id.to_string(),
                expected: asset.exact_digest.clone(),
                actual: digest_hex.to_string(),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_corpus_verification() {
        let json = r#"{
            "assets": [
                {
                    "id": "enwik8",
                    "exact_size": 100000000,
                    "exact_digest": "abcdef1234567890",
                    "provenance": "Public Domain"
                }
            ]
        }"#;

        let inventory = CorpusInventory::from_json(json).unwrap();

        // Exact match
        assert_eq!(inventory.verify_asset("enwik8", 100000000, "abcdef1234567890"), Ok(()));

        // Missing
        assert_eq!(
            inventory.verify_asset("silesia", 100, "111"), 
            Err(CorpusError::MissingAsset("silesia".to_string()))
        );

        // Size mismatch
        assert_eq!(
            inventory.verify_asset("enwik8", 9999, "abcdef1234567890"),
            Err(CorpusError::SizeMismatch {
                id: "enwik8".to_string(),
                expected: 100000000,
                actual: 9999
            })
        );

        // Digest mismatch
        assert_eq!(
            inventory.verify_asset("enwik8", 100000000, "wrongdigest"),
            Err(CorpusError::DigestMismatch {
                id: "enwik8".to_string(),
                expected: "abcdef1234567890".to_string(),
                actual: "wrongdigest".to_string()
            })
        );
    }
}
