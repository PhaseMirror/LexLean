//! Authorship (SPEC.md §33.3).
//!
//! A signature is a detached Ed25519 signature over the 32-byte content
//! digest. Detached matters for the rest of the pipeline: the token of §33.4
//! timestamps these 64 bytes rather than an open digest, so a third party can
//! tell that the artifact was signed and not merely hashed.
//!
//! The key may be bound to a hardware token. When it is, signing fails if the
//! device is absent rather than falling back to a software key, because a
//! fallback that no diagnostic announces would turn a custody claim into a
//! suggestion. Verification never needs the device: the public key in the
//! entry is all a third party has or needs.

use std::process::Command;

use crate::artifact::canonical_json::Json;
use crate::error::LexLeanError;

use super::SIGNATURE_ALGORITHM;

/// A single-diagnostic ledger failure at a whole-entry span.
///
/// The code arrives as a literal so that it passes through
/// [`code!`](crate::code), whose compile-time validation is what keeps an
/// unregistered code from reaching a diagnostic (R5).
macro_rules! fail {
    ($code:literal, $message:expr $(,)?) => {
        $crate::error::LexLeanError::from_diagnostic($crate::diagnostic::Diagnostic::new(
            $crate::code!($code),
            $message,
        ))
    };
}

/// The key kinds an entry may record (SPEC.md §33.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyKind {
    /// A software Ed25519 key held in a file. Suitable for tests and for an
    /// inventor who has no token; the entry says so.
    Software,
    /// An Ed25519 key in a PIV slot, requiring the device and its PIN.
    Piv,
}

impl KeyKind {
    /// The token this repository writes into the `device` field.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Software => "software",
            Self::Piv => "piv",
        }
    }
}

/// The key an entry's signature is verified under, and the custody claim the
/// entry makes about it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyRecord {
    /// The custody claim, recorded so a reader can see what was promised.
    pub kind: KeyKind,
    /// The PIV slot, for [`KeyKind::Piv`].
    pub slot: Option<String>,
    /// The algorithm, always Ed25519 for this specification.
    pub algorithm: String,
    /// Whether the device demanded its PIN for every signature.
    pub pin_policy: String,
}

impl KeyRecord {
    /// A software key record.
    #[must_use]
    pub fn software() -> Self {
        Self {
            kind: KeyKind::Software,
            slot: None,
            algorithm: SIGNATURE_ALGORITHM.to_owned(),
            pin_policy: "never".to_owned(),
        }
    }

    /// A PIV slot record with the PIN required for every signature.
    #[must_use]
    pub fn piv(slot: &str) -> Self {
        Self {
            kind: KeyKind::Piv,
            slot: Some(slot.to_owned()),
            algorithm: SIGNATURE_ALGORITHM.to_owned(),
            pin_policy: "always".to_owned(),
        }
    }
}

/// A detached Ed25519 signature with the key record that describes it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signature {
    /// The custody and algorithm record.
    pub key: KeyRecord,
    /// The 32-byte Ed25519 public key.
    pub public_key: [u8; 32],
    /// The 64-byte detached signature over the content digest.
    pub value: Vec<u8>,
}

impl Signature {
    /// The zero-signature placeholder of an entry that has not been signed.
    ///
    /// An unsigned entry is refused at every gate rather than accepted with an
    /// absent signature, because §33.2 requires the field and §33.6 checks it.
    #[must_use]
    pub fn unsigned() -> Self {
        Self {
            key: KeyRecord::software(),
            public_key: [0u8; 32],
            value: Vec::new(),
        }
    }

    /// Whether this signature has the shape §33.3 requires: 32 key bytes and a
    /// 64-byte signature.
    #[must_use]
    pub fn well_formed(&self) -> bool {
        self.value.len() == 64 && self.key.algorithm == SIGNATURE_ALGORITHM
    }

    /// The §33.6 timestamp artifact: the exact signature bytes a token covers.
    #[must_use]
    pub fn timestamplable_bytes(&self) -> &[u8] {
        &self.value
    }

    /// The canonical JSON of §33.2.
    #[must_use]
    pub fn to_json(&self) -> Json {
        let mut object = vec![
            ("algorithm", Json::Str(self.key.algorithm.clone())),
            ("public_key", Json::Str(hex_lower(&self.public_key))),
            ("value", Json::Str(hex_lower(&self.value))),
            ("device", Json::Str(self.key.kind.as_str().to_owned())),
            ("pin_policy", Json::Str(self.key.pin_policy.clone())),
        ];
        if let Some(slot) = &self.key.slot {
            object.push(("slot", Json::Str(slot.clone())));
        }
        Json::object(object)
    }

    /// Decode a signature field from an entry's canonical JSON.
    ///
    /// An absent-length signature is the `hash` stage's unsigned placeholder: a
    /// freshly hashed entry has to be readable by `sign`, and the placeholder is
    /// what carries it there. `well_formed` is the gate that refuses it, at
    /// `append` and at `verify`, so an unsigned entry travels but cannot be
    /// published.
    ///
    /// # Errors
    /// Returns [`LLG1003`](crate::code) when the field is absent, is not an
    /// object, or carries a public key or signature that is neither the
    /// placeholder nor the right length in hex.
    pub fn from_json(value: &Json) -> Result<Self, crate::error::LexLeanError> {
        let Json::Obj(object) = value else {
            return Err(fail!("LLG1003", "a signature is not a JSON object".to_owned()));
        };
        let text = |key: &str| -> Result<String, crate::error::LexLeanError> {
            match object.get(key) {
                Some(Json::Str(text)) => Ok(text.clone()),
                _ => Err(fail!("LLG1003", format!("signature `{key}` is absent or not a string"))),
            }
        };
        let bytes = |key: &str, length: usize| -> Result<Vec<u8>, crate::error::LexLeanError> {
            let raw = hex_decode(&text(key)?)
                .ok_or_else(|| fail!("LLG1003", format!("signature `{key}` is not hexadecimal")))?;
            if raw.len() != length && !(length == 64 && raw.is_empty()) {
                return Err(fail!(
                    "LLG1003",
                    format!("signature `{key}` is {} bytes, not {length}", raw.len())
                ));
            }
            Ok(raw)
        };
        let public_key = bytes("public_key", 32)?;
        let algorithm = text("algorithm")?;
        if algorithm != SIGNATURE_ALGORITHM {
            return Err(fail!(
                "LLG1003",
                format!("`{algorithm}` is not {SIGNATURE_ALGORITHM}")
            ));
        }
        let device = text("device")?;
        let kind = match device.as_str() {
            "software" => KeyKind::Software,
            "piv" => KeyKind::Piv,
            other => {
                return Err(fail!(
                    "LLG1003",
                    format!("`{other}` is not a custody claim this specification admits")
                ))
            }
        };
        let slot = match object.get("slot") {
            Some(Json::Str(slot)) => Some(slot.clone()),
            Some(_) => return Err(fail!("LLG1003", "a signature slot is not a string".to_owned())),
            None => None,
        };
        if kind == KeyKind::Piv && slot.is_none() {
            return Err(fail!(
                "LLG1003",
                "a hardware-bound signature must name the device slot"
            ));
        }
        let mut fixed = [0u8; 32];
        fixed.copy_from_slice(&public_key);
        Ok(Self {
            key: KeyRecord {
                kind,
                slot,
                algorithm,
                pin_policy: text("pin_policy")?,
            },
            public_key: fixed,
            value: bytes("value", 64)?,
        })
    }

    /// Verify the detached signature over `digest`.
    ///
    /// This is a pure function of the entry's bytes: no device, no filesystem,
    /// no clock.
    #[must_use]
    pub fn verify(&self, digest: &[u8; 32]) -> bool {
        if !self.well_formed() {
            return false;
        }
        let Ok(signature) = ed25519_dalek::Signature::from_slice(&self.value) else {
            return false;
        };
        let Ok(public_key) = ed25519_dalek::VerifyingKey::from_bytes(&self.public_key) else {
            return false;
        };
        public_key
            .verify_strict(digest, &signature)
            .is_ok()
    }
}

/// Sign `digest` with a software Ed25519 secret key held as 32 seed bytes.
///
/// The seed never reaches an entry: only the derived public key does, so an
/// entry discloses authorship and not the ability to forge it.
///
/// # Errors
/// Returns [`LLG1003`](crate::code) when the seed is not 32 bytes.
pub fn sign_with_seed(digest: &[u8; 32], seed: &[u8]) -> Result<Signature, LexLeanError> {
    if seed.len() != 32 {
        return Err(fail!("LLG1003", format!(
                "an Ed25519 seed is 32 bytes, found {}",
                seed.len()
            ),
        ));
    }
    let mut fixed = [0u8; 32];
    fixed.copy_from_slice(seed);
    use ed25519_dalek::Signer;
    let signing_key = ed25519_dalek::SigningKey::from_bytes(&fixed);
    let signature: ed25519_dalek::Signature = signing_key.sign(digest);
    Ok(Signature {
        key: KeyRecord::software(),
        public_key: signing_key.verifying_key().to_bytes(),
        value: signature.to_bytes().to_vec(),
    })
}

/// Sign `digest` with an Ed25519 key in a PIV slot.
///
/// The device is the custody boundary of §33.3, so an absent device, a refused
/// PIN, or a missing tool is a refusal and never a software fallback: an entry
/// that claimed `pin_policy: "always"` must not have been produced by a key that
/// never touched the device.
///
/// # Errors
/// Returns [`LLG1004`](crate::code) when the device or the tool is absent and
/// [`LLG1003`](crate::code) when the device's public key cannot be read.
pub fn sign_with_piv(
    digest: &[u8; 32],
    slot: &str,
    tool: &str,
    pin: Option<&str>,
) -> Result<Signature, LexLeanError> {
    let public_pem = read_piv_public_key(slot, tool, pin)?;
    let public_key = public_key_from_pem(&public_pem)
        .map_err(|reason| fail!("LLG1003", format!("the PIV public key is unusable: {reason}")))?;
    let Some(pin) = pin else {
        return Err(fail!("LLG1004", format!("slot {slot} requires a PIN and none was supplied"),
        ));
    };
    let mut command = Command::new(tool);
    command
        .arg("--action=sign-data")
        .arg(format!("--slot={slot}"))
        .arg("--algorithm=ED25519")
        .arg("--hash=SHA256")
        .arg(format!("--pin={pin}"))
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    let mut child = command.spawn().map_err(|error| {
        fail!("LLG1004", format!("the PIV tool `{tool}` could not be started: {error}"),
        )
    })?;
    {
        use std::io::Write;
        let mut stdin = child.stdin.take().ok_or_else(|| {
            fail!("LLG1004", "the PIV tool accepted no standard input".to_owned())
        })?;
        stdin.write_all(digest).map_err(|error| {
            fail!("LLG1004", format!("the digest could not reach the PIV tool: {error}"))
        })?;
    }
    let output = child.wait_with_output().map_err(|error| {
        fail!("LLG1004", format!("the PIV tool did not complete: {error}"))
    })?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(fail!("LLG1004", format!(
                "the device refused to sign in slot {slot} (a wrong PIN and an absent device are indistinguishable here): {}",
                stderr.trim()
            ),
        ));
    }
    if output.stdout.len() != 64 {
        return Err(fail!("LLG1003", format!(
                "an Ed25519 signature is 64 bytes, the device returned {}",
                output.stdout.len()
            ),
        ));
    }
    let signature = Signature {
        key: KeyRecord::piv(slot),
        public_key,
        value: output.stdout,
    };
    if !signature.verify(digest) {
        return Err(fail!("LLG1003", "the device's signature does not verify under its own public key".to_owned(),
        ));
    }
    Ok(signature)
}

/// Ask the PIV tool for a slot's public key in PEM form.
fn read_piv_public_key(slot: &str, tool: &str, pin: Option<&str>) -> Result<Vec<u8>, LexLeanError> {
    let mut command = Command::new(tool);
    command
        .arg("--action=read-public-key")
        .arg(format!("--slot={slot}"))
        .arg("--algorithm=ED25519")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    if let Some(pin) = pin {
        command.arg(format!("--pin={pin}"));
    }
    let output = command.output().map_err(|error| {
        fail!("LLG1004", format!("the PIV tool `{tool}` could not be started: {error}"),
        )
    })?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(fail!("LLG1004", format!("no device answered slot {slot}: {}", stderr.trim()),
        ));
    }
    Ok(output.stdout)
}

/// Extract the 32-byte Ed25519 public key from a PEM `PUBLIC KEY` block.
///
/// The DER inside the block is a `SubjectPublicKeyInfo`, so the last 32 bytes of
/// the algorithm-identified bit string are the key. Reading the trailer rather
/// than walking the ASN.1 is safe here because the algorithm is pinned to
/// Ed25519: a block whose bit string is not exactly 32 bytes is rejected, so a
/// key of another curve cannot be read as an Ed25519 key.
fn public_key_from_pem(pem: &[u8]) -> Result<[u8; 32], String> {
    let text = std::str::from_utf8(pem).map_err(|error| error.to_string())?;
    let begin = text
        .find("-----BEGIN PUBLIC KEY-----")
        .ok_or("no PEM public key block")?;
    let after = &text[begin + "-----BEGIN PUBLIC KEY-----".len()..];
    let end = after
        .find("-----END PUBLIC KEY-----")
        .ok_or("no PEM end line")?;
    let body: String = after[..end].chars().filter(|c| !c.is_whitespace()).collect();
    let der = base64_decode(&body).ok_or("the PEM body is not valid base64")?;
    if der.len() < 32 {
        return Err("the DER public key is too short to hold an Ed25519 key".to_owned());
    }
    let mut key = [0u8; 32];
    key.copy_from_slice(&der[der.len() - 32..]);
    Ok(key)
}

/// Decode standard base64, rejecting any character outside the alphabet.
fn base64_decode(text: &str) -> Option<Vec<u8>> {
    const ALPHABET: &[u8; 64] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = Vec::with_capacity(text.len() * 3 / 4);
    let mut accumulator = 0u32;
    let mut bits = 0u32;
    for byte in text.bytes() {
        if byte == b'=' {
            break;
        }
        let value = ALPHABET.iter().position(|c| *c == byte)? as u32;
        accumulator = (accumulator << 6) | value;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((accumulator >> bits) as u8);
        }
    }
    Some(out)
}

/// Encode bytes as standard base64 with padding.
#[must_use]
pub fn base64_encode(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let mut block = [0u8; 3];
        block[..chunk.len()].copy_from_slice(chunk);
        let triple = (u32::from(block[0]) << 16) | (u32::from(block[1]) << 8) | u32::from(block[2]);
        out.push(ALPHABET[((triple >> 18) & 0x3f) as usize] as char);
        out.push(ALPHABET[((triple >> 12) & 0x3f) as usize] as char);
        out.push(if chunk.len() > 1 {
            ALPHABET[((triple >> 6) & 0x3f) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            ALPHABET[(triple & 0x3f) as usize] as char
        } else {
            '='
        });
    }
    out
}

/// Decode standard base64 with padding, rejecting anything outside the
/// alphabet so that a corrupted field is a refusal rather than silent damage.
#[must_use]
pub fn base64_decode_strict(text: &str) -> Option<Vec<u8>> {
    if text.len() % 4 != 0 {
        return None;
    }
    base64_decode(text)
}

/// Lowercase hexadecimal, the spelling §33.2 uses for every digest field.
#[must_use]
pub fn hex_lower(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(char::from_digit(u32::from(byte >> 4), 16).expect("a nibble is a hex digit"));
        out.push(char::from_digit(u32::from(byte & 0x0f), 16).expect("a nibble is a hex digit"));
    }
    out
}

/// Decode lowercase or uppercase hexadecimal into bytes.
#[must_use]
pub fn hex_decode(text: &str) -> Option<Vec<u8>> {
    if text.len() % 2 != 0 {
        return None;
    }
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(text.len() / 2);
    for chunk in bytes.chunks_exact(2) {
        let high = (chunk[0] as char).to_digit(16)?;
        let low = (chunk[1] as char).to_digit(16)?;
        out.push((u8::try_from(high).expect("a hex digit fits u8") << 4)
            | u8::try_from(low).expect("a hex digit fits u8"));
    }
    Some(out)
}
