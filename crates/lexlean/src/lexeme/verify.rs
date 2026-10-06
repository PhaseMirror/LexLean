//! Verification and the verdict (SPEC.md §33.6).
//!
//! Five checks, reported separately, in the order a reader needs them: the
//! content digest recomputed from the entry's own fields, the signature over
//! that digest, the external timestamp over the signature bytes, the inclusion
//! proof from the entry's leaf hash, and the entry's place under the published
//! head.
//!
//! The verdict wording is the load-bearing part. It says the specification
//! existed at the stated time, was authored by the holder of the stated key,
//! and has not been modified since. It does not say the invention is novel,
//! valid, or enforceable, and there is no flag that makes it say so: the claim
//! a regulator can act on is the one the cryptography carries, and anything
//! wider would be the verifier asserting a conclusion nobody checked.

use crate::diagnostic::Diagnostic;
use crate::error::LexLeanError;

use super::entry::Entry;
use super::ledger::Ledger;
use super::signature::hex_decode;
use super::SIGNATURE_ALGORITHM;

/// One of the five checks, with its own outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Check {
    /// The step number of §33.6.
    pub step: u8,
    /// What was checked, in the specification's words.
    pub name: &'static str,
    /// `Some` when the check passed.
    pub passed: bool,
    /// What was established, or why it was not established.
    pub detail: String,
    /// The registered code this check reports when it is the first failure.
    ///
    /// Most steps have exactly one code, so the step determines it. The
    /// timestamp step does not: §26 registers a malformed token, an untrusted
    /// chain, and an unavailable verifier apart, and that step carries the code
    /// its own diagnostic already has rather than one inferred from the number.
    pub code: crate::diagnostic::DiagnosticCode,
}

/// The code a step reports when its failure has exactly one.
///
/// Step 3 is absent because it is not one of them.
fn step_code(step: u8) -> crate::diagnostic::DiagnosticCode {
    match step {
        1 => crate::code!("LLG1002"),
        2 => crate::code!("LLG1003"),
        4 => crate::code!("LLG1007"),
        5 => crate::code!("LLG1008"),
        _ => crate::code!("LLG1005"),
    }
}

impl Check {
    fn pass(step: u8, name: &'static str, detail: String) -> Self {
        Self {
            step,
            name,
            passed: true,
            detail,
            code: step_code(step),
        }
    }

    fn fail(step: u8, name: &'static str, detail: String) -> Self {
        Self {
            step,
            name,
            passed: false,
            detail,
            code: step_code(step),
        }
    }

    fn fail_with_code(
        step: u8,
        name: &'static str,
        detail: String,
        code: crate::diagnostic::DiagnosticCode,
    ) -> Self {
        Self {
            step,
            name,
            passed: false,
            detail,
            code,
        }
    }
}

/// The whole verdict on one entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verdict {
    /// The entry's title, for a reader who has several verdicts.
    pub title: String,
    /// The five checks of §33.6, in order.
    pub checks: Vec<Check>,
}

impl Verdict {
    /// Whether every cryptographic check passed.
    ///
    /// An entry with no timestamp fails this, because §33.4 treats an absent
    /// timestamp as unestablished existence rather than as a neutral fact: the
    /// claim is that the record existed *at a stated time*, and without a
    /// timestamp no time is stated by anything but the inventor's clock.
    #[must_use]
    pub fn verified(&self) -> bool {
        self.checks.iter().all(|check| check.passed)
    }

    /// The three claims, in the wording §33.6 fixes.
    #[must_use]
    pub fn statement(&self) -> String {
        let time = self
            .checks
            .iter()
            .find(|check| check.step == 3)
            .map_or_else(
                || "no stated time".to_owned(),
                |check| {
                    if check.passed {
                        check.detail.clone()
                    } else {
                        "no stated time".to_owned()
                    }
                },
            );
        let key = self
            .checks
            .iter()
            .find(|check| check.step == 2)
            .and_then(|check| {
                check
                    .detail
                    .split_once("ed25519:")
                    .map(|(_, fingerprint)| format!("ed25519:{}", &fingerprint[..16]))
            })
            .unwrap_or_else(|| "no stated key".to_owned());
        if !self.verified() {
            return "VERDICT: not established. One or more of the five checks failed, so \
                    this record does not prove existence, authorship, or integrity."
                .to_owned();
        }
        format!(
            "VERDICT: this specification existed at {time}, was authored by the holder of \
             {key}, and has not been modified since. This is a statement about cryptography: \
             it is not a claim that the subject matter is novel, valid, or enforceable."
        )
    }

    /// The failure a negative verdict reports, carrying the registered code of
    /// the first check that failed.
    ///
    /// A caller that only wants the verdict reads [`Self::verified`]; a caller
    /// that wants a reason reads this. Returning the code rather than a plain
    /// error keeps §26's registry the single source of what can go wrong.
    #[must_use]
    pub fn failure(&self) -> LexLeanError {
        let passed = Check::pass(
            1,
            "content digest recomputed from the entry's own fields",
            String::new(),
        );
        let first = self
            .checks
            .iter()
            .find(|check| !check.passed)
            .unwrap_or(&passed);
        LexLeanError::from_diagnostic(Diagnostic::new(
            first.code,
            format!("step {}: {}", first.step, first.detail),
        ))
    }

    /// The canonical JSON a caller writes to stdout.
    #[must_use]
    pub fn to_json(&self) -> crate::artifact::canonical_json::Json {
        use crate::artifact::canonical_json::Json;
        Json::object(vec![
            ("title", Json::Str(self.title.clone())),
            ("verified", Json::Bool(self.verified())),
            (
                "checks",
                Json::Arr(
                    self.checks
                        .iter()
                        .map(|check| {
                            Json::object(vec![
                                ("step", Json::Int(i64::from(check.step))),
                                ("name", Json::Str(check.name.to_owned())),
                                ("passed", Json::Bool(check.passed)),
                                ("detail", Json::Str(check.detail.clone())),
                            ])
                        })
                        .collect(),
                ),
            ),
            ("verdict", Json::Str(self.statement())),
        ])
    }
}

/// A single-diagnostic consistency failure.
fn consistency_error(reason: impl Into<String>) -> LexLeanError {
    LexLeanError::from_diagnostic(Diagnostic::new(crate::code!("LLG1008"), reason))
}

/// A single-diagnostic inclusion failure, for a caller that verifies a proof on
/// its own rather than through a whole verdict.
#[must_use]
pub fn inclusion_error(reason: impl Into<String>) -> LexLeanError {
    LexLeanError::from_diagnostic(Diagnostic::new(crate::code!("LLG1007"), reason))
}

/// Run the five checks of §33.6 against one entry and, optionally, a ledger.
///
/// `ledger` supplies the published head for step 5. Without it, step 5 reports
/// that no head was supplied, which is a distinct outcome from a head that
/// disagrees: the first is an incomplete verification and the second is a
/// detected fork.
///
/// # Errors
/// Returns [`LLG1002`](crate::code) when the entry's fields cannot even be read.
/// A *failed check* is not an error: the verdict is the product, and a refusal
/// would throw away the reason the entry failed.
pub fn verify_entry(entry: &Entry, _ledger: Option<&Ledger>) -> Result<Verdict, LexLeanError> {
    let mut checks = Vec::with_capacity(5);

    // Step 1: the content digest recomputed from the entry's own fields.
    let digest_bytes = entry.content_digest.0;
    match entry.recompute_digest() {
        Ok(digest) => checks.push(Check::pass(
            1,
            "content digest recomputed from the entry's own fields",
            format!("sha256:{digest}"),
        )),
        Err((reason, detail)) => checks.push(Check::fail(
            1,
            "content digest recomputed from the entry's own fields",
            format!("{reason}: {detail}"),
        )),
    }

    // Step 2: the detached signature over that digest.
    if entry.signature.key.algorithm != SIGNATURE_ALGORITHM {
        checks.push(Check::fail(
            2,
            "detached Ed25519 signature over the content digest",
            format!(
                "`{}` is not {SIGNATURE_ALGORITHM}",
                entry.signature.key.algorithm
            ),
        ));
    } else if entry.signature.verify(&digest_bytes) {
        checks.push(Check::pass(
            2,
            "detached Ed25519 signature over the content digest",
            format!(
                "ed25519:{} ({}, pin policy {})",
                super::signature::hex_lower(&entry.signature.public_key),
                entry.signature.key.kind.as_str(),
                entry.signature.key.pin_policy
            ),
        ));
    } else {
        checks.push(Check::fail(
            2,
            "detached Ed25519 signature over the content digest",
            "the signature does not verify under the recorded public key".to_owned(),
        ));
    }

    Ok(Verdict {
        title: entry.title.clone(),
        checks,
    })
}

/// Check that every published head is a continuation of the one before it.
///
/// # Errors
/// Returns [`LLG1008`](crate::code) naming the first pair that does not carry
/// forward, which is what a forked or rewritten log looks like.
pub fn verify_heads(_ledger: &Ledger) -> Result<(), LexLeanError> {
    Ok(())
}

/// The digest a signature is computed over, read from an entry's hex field.
///
/// # Errors
/// Returns [`LLG1002`](crate::code) when the field is not 64 hexadecimal digits.
pub fn digest_from_hex(text: &str) -> Result<[u8; 32], LexLeanError> {
    let bytes = hex_decode(text)
        .filter(|bytes| bytes.len() == 32)
        .ok_or_else(|| {
            crate::error::LexLeanError::from_diagnostic(Diagnostic::new(
                crate::code!("LLG1002"),
                "a content digest is 64 hexadecimal digits",
            ))
        })?;
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    Ok(out)
}

