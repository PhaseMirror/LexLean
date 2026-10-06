use sha2::{Sha256, Digest};

/// Represents a single NIST CAVP test vector for SHA-256.
pub struct CavpVector {
    /// The message payload bytes.
    pub msg: Vec<u8>,
    /// The expected message digest in hex format.
    pub md: String,
}

/// Executes a suite of NIST CAVP vectors against the runtime SHA-256 implementation.
/// This acts as the vector oracle completeness check.
///
/// Returns `Ok(())` if all vectors match, or an error string identifying the failure.
pub fn verify_sha256_vectors(vectors: &[CavpVector]) -> Result<(), String> {
    for (i, vector) in vectors.iter().enumerate() {
        let mut hasher = Sha256::new();
        hasher.update(&vector.msg);
        let result = hasher.finalize();
        let digest_hex = hex::encode(result);

        if digest_hex != vector.md.to_lowercase() {
            return Err(format!(
                "Vector {} failed: expected {}, got {}",
                i, vector.md, digest_hex
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nist_cavp_sha256_short_messages() {
        // A few actual test vectors from NIST CAVP (ShortMsg)
        let vectors = vec![
            // Len = 0
            CavpVector {
                msg: b"".to_vec(),
                md: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".into(),
            },
            // Len = 24 (3 bytes) "abc"
            CavpVector {
                msg: b"abc".to_vec(),
                md: "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad".into(),
            },
            // Len = 448 (56 bytes)
            CavpVector {
                msg: b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq".to_vec(),
                md: "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1".into(),
            },
        ];

        assert_eq!(verify_sha256_vectors(&vectors), Ok(()));
    }

    #[test]
    fn test_reject_malformed_vector() {
        let vectors = vec![
            CavpVector {
                msg: b"abc".to_vec(),
                md: "0000000000000000000000000000000000000000000000000000000000000000".into(),
            },
        ];

        let result = verify_sha256_vectors(&vectors);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Vector 0 failed: expected"));
    }
}
