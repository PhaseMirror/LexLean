use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Represents a frozen external authority (e.g., Lean kernel, Z3, CAVP vectors)
/// used as an oracle or dependency in the UORC pipeline.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthorityManifest {
    /// The unique identifier for this authority (e.g., "z3-4.12.2").
    pub id: String,
    /// The exact source URL or local path.
    pub source: String,
    /// The frozen SHA-256 digest of the acquired artifact.
    pub digest: String,
    /// The license under which this authority is acquired.
    pub license: String,
    /// The designated role of this authority (e.g., "theorem_checker", "crypto_vector").
    pub role: String,
}

/// An inventory of all frozen authorities, enforcing immutable provenance.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuthorityInventory {
    /// A map of authority identifiers to their manifests.
    pub authorities: HashMap<String, AuthorityManifest>,
}

impl AuthorityInventory {
    /// Parses an inventory from a JSON string.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Serializes the inventory to a JSON string.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Validates that an authority is present and its digest exactly matches the expected digest.
    pub fn verify_digest(&self, id: &str, expected_digest: &str) -> bool {
        if let Some(auth) = self.authorities.get(id) {
            auth.digest == expected_digest
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_authority_manifest_roundtrip() {
        let auth = AuthorityManifest {
            id: "z3-4.12.2".into(),
            source: "https://github.com/Z3Prover/z3/releases/tag/z3-4.12.2".into(),
            digest: "deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef".into(),
            license: "MIT".into(),
            role: "smt_solver".into(),
        };

        let mut inventory = AuthorityInventory::default();
        inventory.authorities.insert(auth.id.clone(), auth.clone());

        let json = inventory.to_json().unwrap();
        let parsed = AuthorityInventory::from_json(&json).unwrap();

        assert_eq!(parsed.authorities.get("z3-4.12.2").unwrap(), &auth);
    }

    #[test]
    fn test_digest_verification_rejects_missing_or_mismatch() {
        let auth = AuthorityManifest {
            id: "cavp-sha256".into(),
            source: "nist.gov".into(),
            digest: "1234".into(),
            license: "Public Domain".into(),
            role: "crypto_vector".into(),
        };

        let mut inventory = AuthorityInventory::default();
        inventory.authorities.insert(auth.id.clone(), auth.clone());

        // Exact match
        assert!(inventory.verify_digest("cavp-sha256", "1234"));

        // Mismatch
        assert!(!inventory.verify_digest("cavp-sha256", "5678"));

        // Missing
        assert!(!inventory.verify_digest("unknown-oracle", "0000"));
    }
}
