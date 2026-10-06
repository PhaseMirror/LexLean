use sha2::{Sha256, Digest};
use crate::universe::UniverseDescriptor;

/// Domain strings for different entity types in UORC.
/// Domain for archive entities.
pub const DOMAIN_ARCHIVE: &[u8] = b"uorc/archive/1\x00";
/// Domain for query entities.
pub const DOMAIN_QUERY: &[u8] = b"uorc/query/1\x00";
/// Domain for certificate entities.
pub const DOMAIN_CERTIFICATE: &[u8] = b"uorc/certificate/2\x00";

/// Computes a canonical domain-separated SHA-256 identity.
pub fn compute_identity(domain: &[u8], payload: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(domain);
    hasher.update(payload);
    let result = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&result);
    out
}

/// A restricted JSON Canonicalization Scheme (JCS) helper for descriptors.
/// In a full implementation, this would enforce RFC 8785 strictness.
/// Here we ensure exact field ordering and lack of whitespace.
pub fn jcs_serialize_universe(universe: &UniverseDescriptor) -> String {
    // Exact lexicographic key ordering: max_instructions, max_length_bytes, max_memory_bytes, target_bytes
    // target_bytes is serialized as a hex string for determinism.
    let target_hex = hex::encode(&universe.target_bytes);
    format!(
        r#"{{"max_instructions":{},"max_length_bytes":{},"max_memory_bytes":{},"target_bytes":"{}"}}"#,
        universe.max_instructions,
        universe.max_length_bytes,
        universe.max_memory_bytes,
        target_hex
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_separation_collision_safe() {
        let payload = b"common payload";
        let id_archive = compute_identity(DOMAIN_ARCHIVE, payload);
        let id_query = compute_identity(DOMAIN_QUERY, payload);

        // Even with the same payload, distinct domains produce completely distinct identities.
        assert_ne!(id_archive, id_query);
    }

    #[test]
    fn test_jcs_determinism() {
        let u1 = UniverseDescriptor::new(vec![0xAA, 0xBB], 50, 1024, 2);
        let u2 = UniverseDescriptor::new(vec![0xAA, 0xBB], 50, 1024, 2);

        let json1 = jcs_serialize_universe(&u1);
        let json2 = jcs_serialize_universe(&u2);

        assert_eq!(json1, json2);
        assert_eq!(
            json1,
            r#"{"max_instructions":2,"max_length_bytes":50,"max_memory_bytes":1024,"target_bytes":"aabb"}"#
        );
    }
}
