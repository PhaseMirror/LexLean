//! Public-key verification for the RFC 3161 chain, delegated to OpenSSL
//! (SPEC.md §33.4).
//!
//! # Why this is not a Rust RSA implementation
//!
//! The obvious implementation is a dependency on a Rust RSA crate, and it was
//! one. It is not any more, for a reason that is a gate rather than a
//! preference: `rsa` 0.9.10 carries RUSTSEC-2023-0071, the Marvin timing
//! attack, with no patch available in any release, and `deny.toml` admits no
//! advisory ignores --- an ignore outlives the dependency that needed it, so
//! the next reader cannot tell whether it still applies.
//!
//! The usual argument for waving that through is that this code path verifies
//! a *public* signature over a token that is already in hand, so there is no
//! private key for a timing channel to leak. That argument is sound, and it is
//! exactly why it is a bad thing to encode in `deny.toml`: it is a judgement
//! about one call site that the file cannot scope, it stops being true the
//! moment someone adds a signing path, and it trains the next reader to accept
//! an advisory on a similar argument. Removing the crate removes the question.
//!
//! What replaces it is an external, independently audited implementation.
//! OpenSSL is already the reference implementation of PKCS#1 v1.5 and of the
//! X.509 chain this section verifies, its constant-time RSA verify is the
//! upstream fix the Rust crate is waiting on, and a regulator auditing a §33.4
//! verdict is far more likely to trust it than to audit a vendored one.
//!
//! The cost is real and is paid in the open rather than hidden: `openssl`
//! becomes a runtime requirement of `lexeme verify`, and it is an external
//! dependency that §22.3's process record must cover. That is why every
//! invocation here goes through [`crate::verify::child::run`] --- the §25.2
//! no-shell rule, the §25.4 allow-list environment, the §25.5 timeout and
//! output cap, and the executable's SHA-256 in the record all come from the one
//! implementation the repository already trusts, rather than from a second
//! hand-rolled spawn that would have to be audited separately.
//!
//! Nothing here is a fallback. When OpenSSL is absent, verification fails as
//! [`Pkcs1Failure::Provider`] and the verdict reports the timestamp
//! unestablished; it never falls back to a weaker check, and it never reports
//! a timestamp as verified on the strength of a parse alone.

use camino::Utf8Path;

use crate::artifact::content_id::Sha256Digest;
use crate::config::Limits;
use crate::verify::child::{self, ChildHome, ChildSpec, Normalizer};

/// The tool label every OpenSSL invocation carries into its process record.
const TOOL: &str = "openssl";

/// The resource policy for one verification, §25.5.
///
/// These are the `init` defaults narrowed to what a signature check can reach.
/// The timeout is far below the project default because the work is a single
/// modular exponentiation over a file this module just wrote: anything slower
/// is a hung provider, and waiting out the project default would turn a
/// five-second fault into a five-minute one. The output cap is small for the
/// same reason --- OpenSSL answers a verification with one line, and the
/// failure text it emits is bounded, so a large stdout means the wrong program
/// answered.
const VERIFY_LIMITS: Limits = Limits {
    max_file_bytes: 4_194_304,
    max_total_source_bytes: 67_108_864,
    max_primitive_atoms: 2_000_000,
    max_token_lattice_edges: 4_000_000,
    max_parse_states: 4_000_000,
    max_ir_nodes: 2_000_000,
    max_scope_depth: 1024,
    max_import_depth: 128,
    max_diagnostics: 256,
    max_child_output_bytes: 65_536,
    child_timeout_ms: 20_000,
};

/// Why one PKCS#1 v1.5 verification produced no verdict.
///
/// The two cases are kept apart because they are different claims. A
/// [`Self::Rejected`] says the signature is wrong, which is a statement about
/// the token. A [`Self::Provider`] says the question was never asked, which is
/// a statement about this machine; conflating them would let a missing
/// OpenSSL be reported as a forged timestamp, which is the one confusion a
/// provenance tool cannot afford.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pkcs1Failure {
    /// The verification provider is absent, would not start, or exceeded a
    /// §25.5 limit. The timestamp is unestablished, not disproved.
    Provider(String),
    /// The provider ran and the signature does not verify.
    Rejected(String),
}

impl std::fmt::Display for Pkcs1Failure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Provider(reason) => write!(formatter, "{reason}"),
            Self::Rejected(reason) => write!(formatter, "{reason}"),
        }
    }
}

/// The OpenSSL `-sha*` flag for a PKCS#1 v1.5 signature algorithm OID.
///
/// Only the four SHA-1 and SHA-2 PKCS#1 v1.5 algorithms §33.4 admits are
/// listed. Anything else is refused by name rather than passed through: a
/// provider that is handed an algorithm OID it does not recognise may fall back
/// to a default, and a verifier that silently verifies under a different digest
/// than the token names has stopped verifying the token.
fn digest_flag(algorithm: &str) -> Option<&'static str> {
    match algorithm {
        "1.3.14.3.2.29" => Some("-sha1"),
        "1.2.840.113549.1.1.11" => Some("-sha256"),
        "1.2.840.113549.1.1.12" => Some("-sha384"),
        "1.2.840.113549.1.1.13" => Some("-sha512"),
        _ => None,
    }
}

/// The OpenSSL `-sha*` flag for an ECDSA signature algorithm OID.
///
/// §33.4 verifies a token's signature under whatever `SubjectPublicKeyInfo`
/// the certificate carries, and it does not name an algorithm. A verifier that
/// admits only RSA therefore refuses every authority whose certificate holds an
/// elliptic-curve key, and the shipped default authority
/// (`DEFAULT_TSA_URL`) is such an authority, so an RSA-only reader cannot
/// verify the configuration it ships with. The OIDs are listed rather than
/// derived for the same reason [`digest_flag`] lists them: a provider handed an
/// unrecognised OID may verify under a digest the token never named.
fn ecdsa_digest_flag(algorithm: &str) -> Option<&'static str> {
    match algorithm {
        "1.2.840.10045.4.1" => Some("-sha1"),
        "1.2.840.10045.4.3.1" => Some("-sha224"),
        "1.2.840.10045.4.3.2" => Some("-sha256"),
        "1.2.840.10045.4.3.3" => Some("-sha384"),
        "1.2.840.10045.4.3.4" => Some("-sha512"),
        _ => None,
    }
}

/// Resolve the provider, make an isolated scratch directory, and run `work` in
/// it, removing the directory on every path out.
///
/// The directory is created with create-new semantics inside a fresh temporary
/// directory and removed when this function returns, so two verifications
/// running at once cannot see each other's key material.
fn with_provider_scratch(
    work: impl FnOnce(&Utf8Path, Sha256Digest, &Utf8Path) -> Result<(), Pkcs1Failure>,
) -> Result<(), Pkcs1Failure> {
    let (program, digest) = provider()?;
    let scratch_dir = tempfile::Builder::new()
        .prefix("lexeme-verify-")
        .tempdir()
        .map_err(|io_error| {
            Pkcs1Failure::Provider(format!(
                "a scratch directory could not be created: {io_error}"
            ))
        })?;
    let scratch =
        camino::Utf8PathBuf::from_path_buf(scratch_dir.path().to_path_buf()).map_err(|path| {
            Pkcs1Failure::Provider(format!(
                "the scratch directory is not valid UTF-8: {}",
                path.to_string_lossy()
            ))
        })?;
    work(&program, digest, &scratch)
}

/// Resolve `openssl`, hash the executable, and return the pair a
/// [`ChildSpec`] needs.
///
/// The digest is computed here rather than left to the child because §22.3
/// records the executable that ran, and a record that cannot name which build
/// of the verifier produced a verdict is not evidence of anything.
fn provider() -> Result<(camino::Utf8PathBuf, Sha256Digest), Pkcs1Failure> {
    let program = child::resolve_on_path("openssl").map_err(|diagnostic| {
        Pkcs1Failure::Provider(format!(
            "no `openssl` executable is available to verify this timestamp: {}",
            diagnostic.message
        ))
    })?;
    let bytes = std::fs::read(&program).map_err(|io_error| {
        Pkcs1Failure::Provider(format!("`openssl` could not be read: {io_error}"))
    })?;
    Ok((program, Sha256Digest::of(&bytes)))
}

/// Run one OpenSSL invocation inside `scratch` under the §25.4 environment.
///
/// Returns the exit code and the normalized stderr. The scratch directory is
/// both the working directory and the isolated home, so a provider that tries
/// to read a configuration file finds none rather than the invoking user's.
fn invoke(
    program: &Utf8Path,
    digest: Sha256Digest,
    scratch: &Utf8Path,
    argv: Vec<String>,
) -> Result<(i32, String), Pkcs1Failure> {
    // The normalizer replaces the scratch path with itself as a prefix token.
    // An OpenSSL diagnostic quotes absolute paths, and §22.7 requires a record
    // to carry none, so the one directory this module creates is the one prefix
    // that gets elided.
    let normalizer = Normalizer::new(scratch, scratch, scratch, scratch);
    let record = child::run(
        &ChildSpec {
            tool: TOOL,
            module: Some("lexeme".to_owned()),
            program,
            executable_sha256: digest,
            argv,
            cwd: scratch,
            extra_env: Vec::new(),
            home: ChildHome::Isolated { home: scratch },
        },
        &VERIFY_LIMITS,
        &normalizer,
    )
    .map_err(|diagnostic| {
        Pkcs1Failure::Provider(format!(
            "`openssl` did not complete: {}",
            diagnostic.message
        ))
    })?;
    Ok((record.exit_code, record.stderr))
}

/// Convert a DER public key to the PEM form `openssl dgst -verify` reads.
///
/// An RSA key reaches a certificate two ways, and §33.4 accepts both: wrapped
/// in a `SubjectPublicKeyInfo`, or as the bare PKCS#1 `RSAPublicKey` inside a
/// key's `subjectPublicKey` bit string. The first is what a certificate's own
/// key looks like and the second is what a `SignerInfo`'s looks like, so
/// refusing the second would refuse every token that names its own key. The two
/// are told apart by asking OpenSSL to parse each way rather than by sniffing
/// the DER, because the sniffing test would be a second parser to keep correct.
fn public_key_pem(
    program: &Utf8Path,
    digest: Sha256Digest,
    scratch: &Utf8Path,
    key_der: &[u8],
) -> Result<(), Pkcs1Failure> {
    let write = |name: &str, bytes: &[u8]| -> Result<camino::Utf8PathBuf, Pkcs1Failure> {
        let path = scratch.join(name);
        std::fs::write(&path, bytes).map_err(|io_error| {
            Pkcs1Failure::Provider(format!("`{name}` could not be staged: {io_error}"))
        })?;
        Ok(path)
    };
    let key_path = write("key.der", key_der)?;
    let pem_path = scratch.join("key.pem");

    let as_subject_public_key_info = vec![
        "pkey".to_owned(),
        "-pubin".to_owned(),
        "-inform".to_owned(),
        "DER".to_owned(),
        "-in".to_owned(),
        key_path.to_string(),
        "-out".to_owned(),
        pem_path.to_string(),
    ];
    // The `SubjectPublicKeyInfo` parse is attempted first and its diagnostic is
    // deliberately dropped: a bare PKCS#1 key fails it in terms that say
    // nothing useful once the PKCS#1 parse succeeds, and only the failure of
    // both is a real finding.
    let (exit_code, _) = invoke(program, digest, scratch, as_subject_public_key_info)?;
    if exit_code == 0 {
        return Ok(());
    }
    let as_pkcs1 = vec![
        "rsa".to_owned(),
        "-RSAPublicKey_in".to_owned(),
        "-inform".to_owned(),
        "DER".to_owned(),
        "-in".to_owned(),
        key_path.to_string(),
        "-pubout".to_owned(),
        "-out".to_owned(),
        pem_path.to_string(),
    ];
    let (exit_code, stderr) = invoke(program, digest, scratch, as_pkcs1)?;
    if exit_code == 0 {
        return Ok(());
    }
    Err(Pkcs1Failure::Provider(format!(
        "the key is neither an elliptic-curve nor an RSA public key: {}",
        stderr.trim()
    )))
}

/// Verify one RSA PKCS#1 v1.5 signature over `message` under the DER public key
/// `subject_public_key_info`.
///
/// This is the §33.4 workhorse: the root's self-signature, the TSA
/// certificate's signature under the root, and the token's own signature under
/// the TSA certificate are three calls to it.
///
/// The message and signature are staged as files rather than passed on the
/// command line because both are binary and both may contain a NUL, and an
/// argument vector cannot carry either faithfully. The staging directory is
/// created with create-new semantics inside a fresh temporary directory and
/// removed when this function returns, so two verifications running at once
/// cannot see each other's key material.
pub fn verify_pkcs1v15(
    subject_public_key_info: &[u8],
    algorithm: &str,
    message: &[u8],
    signature: &[u8],
) -> Result<(), Pkcs1Failure> {
    let Some(flag) = digest_flag(algorithm) else {
        return Err(Pkcs1Failure::Rejected(format!(
            "{algorithm} is not an admitted PKCS#1 v1.5 digest"
        )));
    };
    with_provider_scratch(|program, digest, scratch| {
        verify_in(
            program,
            digest,
            scratch,
            flag,
            subject_public_key_info,
            message,
            signature,
        )
    })
}

/// Verify one signature under the DER public key `subject_public_key_info`,
/// admitting either RSA PKCS#1 v1.5 or ECDSA as `algorithm` names.
///
/// The two families are dispatched on the algorithm OID and share one provider
/// path, because §33.4 requires the signature to verify under the key the
/// certificate carries and does not constrain that key's family. Dispatching on
/// the OID rather than on the key material keeps the reader from verifying
/// under a digest the token did not name.
pub fn verify_signature(
    subject_public_key_info: &[u8],
    algorithm: &str,
    message: &[u8],
    signature: &[u8],
) -> Result<(), Pkcs1Failure> {
    let flag = digest_flag(algorithm)
        .or_else(|| ecdsa_digest_flag(algorithm))
        .ok_or_else(|| {
            Pkcs1Failure::Rejected(format!(
                "{algorithm} is not an admitted signature algorithm"
            ))
        })?;
    with_provider_scratch(|program, digest, scratch| {
        verify_in(
            program,
            digest,
            scratch,
            flag,
            subject_public_key_info,
            message,
            signature,
        )
    })
}

/// The body of [`verify_pkcs1v15`], with the staging directory already made.
fn verify_in(
    program: &Utf8Path,
    digest: Sha256Digest,
    scratch: &Utf8Path,
    flag: &str,
    subject_public_key_info: &[u8],
    message: &[u8],
    signature: &[u8],
) -> Result<(), Pkcs1Failure> {
    public_key_pem(program, digest, scratch, subject_public_key_info)?;
    let stage = |name: &str, bytes: &[u8]| -> Result<String, Pkcs1Failure> {
        let path = scratch.join(name);
        std::fs::write(&path, bytes).map_err(|io_error| {
            Pkcs1Failure::Provider(format!("`{name}` could not be staged: {io_error}"))
        })?;
        Ok(path.to_string())
    };
    let message_path = stage("message.bin", message)?;
    let signature_path = stage("signature.bin", signature)?;
    let argv = vec![
        "dgst".to_owned(),
        flag.to_owned(),
        "-verify".to_owned(),
        scratch.join("key.pem").to_string(),
        "-signature".to_owned(),
        signature_path,
        message_path,
    ];
    let (exit_code, stderr) = invoke(program, digest, scratch, argv)?;
    match exit_code {
        0 => Ok(()),
        // A non-zero exit is OpenSSL saying the signature does not verify. Its
        // stderr is a two-line diagnostic that ends in `Verification failure`;
        // the first line names the reason, and the whole thing is already
        // bounded by `max_child_output_bytes`.
        _ => Err(Pkcs1Failure::Rejected(format!(
            "the signature does not verify: {}",
            stderr.lines().next().unwrap_or("no reason given").trim()
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::{digest_flag, verify_pkcs1v15, Pkcs1Failure};

    #[test]
    fn the_four_admitted_algorithms_map_to_a_flag() {
        assert_eq!(digest_flag("1.3.14.3.2.29"), Some("-sha1"));
        assert_eq!(digest_flag("1.2.840.113549.1.1.11"), Some("-sha256"));
        assert_eq!(digest_flag("1.2.840.113549.1.1.12"), Some("-sha384"));
        assert_eq!(digest_flag("1.2.840.113549.1.1.13"), Some("-sha512"));
    }

    #[test]
    fn an_unadmitted_algorithm_is_refused_rather_than_defaulted() {
        // ECDSA and the PSS OIDs are refused by name: a provider handed an
        // algorithm it does not know may fall back to a default digest, and a
        // verifier that quietly checks a different digest has stopped
        // verifying the token.
        for refused in [
            "1.2.840.10045.4.3.2",
            "1.2.840.113549.1.1.10",
            "2.16.840.1.101.3.4.3.2",
            "",
        ] {
            assert_eq!(digest_flag(refused), None, "{refused} should be refused");
        }
    }

    #[test]
    fn a_non_rsa_key_is_a_provider_failure_not_a_rejection() {
        // The distinction is the point of the type: a key that is not an RSA
        // public key is a question this module could not pose, and reporting it
        // as a forged signature would be a false statement about the token.
        let error = verify_pkcs1v15(b"not a key", "1.2.840.113549.1.1.11", b"m", b"sig")
            .expect_err("a non-key must not verify");
        assert!(
            matches!(error, Pkcs1Failure::Provider(_)),
            "expected a provider failure, got {error:?}"
        );
    }

    #[test]
    fn a_refused_algorithm_never_reaches_the_provider() {
        let error = verify_pkcs1v15(b"", "1.2.840.10045.4.3.2", b"m", b"sig")
            .expect_err("an unadmitted algorithm must not verify");
        assert!(matches!(error, Pkcs1Failure::Rejected(_)), "{error:?}");
    }
}
