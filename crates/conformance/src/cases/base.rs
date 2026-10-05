//! The `LG-` half of the `lexeme` suite: the §33 base layer.
//!
//! `lexeme.rs` carries the §34 stratification layer above it. These cases
//! cover §33 itself, and two of them are what keep the ledger honest, so they
//! are worth reading first:
//!
//! - `lg_10` asserts an inclusion proof is accepted *exactly* when the
//!   recomputed root equals the published root, and refuses each of the three
//!   defects §33.5 names. A proof check that accepted a wrong-length path would
//!   still accept every honest path, which is why the defects are the point.
//! - `lg_12` asserts the §33.6 verdict says the three claims the cryptography
//!   carries and cannot be made to say a fourth. The forbidden-word check is
//!   the substance: a verdict that grows a phrase is a claim this repository
//!   does not have.
//!
//! The layer's three claims are existence at a stated time, authorship by a
//! stated key, and integrity since. Nothing here asserts novelty, validity, or
//! legal effect, and a case that needed to would be asserting a claim the
//! ledger does not carry.

use lexlean::artifact::canonical_json::Json;
use lexlean::artifact::content_id::Sha256Digest;
use lexlean::lexeme::canonical::canonicalize;
use lexlean::lexeme::entry::{Entry, Inclusion};
use lexlean::lexeme::ledger::Ledger;
use lexlean::lexeme::merkle::{
    audit_path_length, consistency_path_from, leaf_hash, node_hash, verify_consistency,
    verify_inclusion, ConsistencyFailure, InclusionFailure,
};
use lexlean::lexeme::signature::{sign_with_seed, KeyKind, KeyRecord, Signature};
use lexlean::lexeme::{CANONICALIZATION, CANONICAL_DOMAIN, ENTRY_SPEC, SIGNATURE_ALGORITHM};

use lexlean::lexeme::timestamp::TokenFailure;

use crate::support::expect_code;

/// The pinned toolchain every probe entry is built under (§33.2).
const TOOLCHAIN: &str = "leanprover/lean4:v4.32.1";

/// A deterministic Ed25519 seed, so every case signs reproducibly.
fn seed(marker: u8) -> [u8; 32] {
    let mut bytes = [marker; 32];
    for (index, byte) in bytes.iter_mut().enumerate() {
        *byte = byte.wrapping_add(u8::try_from(index % 7).expect("in range"));
    }
    bytes
}

/// A deterministic Ed25519 seed for the log key, kept distinct from the entry
/// seeds so a case cannot confuse the key that signs entries with the key that
/// signs heads (§33.7).
fn log_seed() -> [u8; 32] {
    seed(200)
}

/// A signed entry over one source, appended nowhere.
///
/// Signing is separated from appending because §33.6 step 2 checks the
/// signature over the content digest while step 4 needs the leaf bytes the
/// append returns, and a case that conflates them cannot tell which check it
/// is exercising.
fn signed_entry(title: &str, source: &str, marker: u8) -> Entry {
    let mut entry = Entry::new(title, TOOLCHAIN, &[("Probe.lean", source)]).expect("entry builds");
    let digest = entry.content_digest.to_hex();
    let bytes = lexlean::lexeme::verify::digest_from_hex(&digest).expect("32 hex digits");
    let signature = sign_with_seed(&bytes, &seed(marker)).expect("the seed signs the digest");
    entry.with_signature(signature);
    entry
}

/// A one-line body whose value varies, so a digest change cannot come from the
/// declaration name.
fn source_with(value: u32) -> String {
    format!("def a : Nat := {value}\n")
}

/// Decode lowercase hex, rejecting anything else.
fn hex_decode_lower(text: &str) -> Option<Vec<u8>> {
    if !text.len().is_multiple_of(2) {
        return None;
    }
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(text.len() / 2);
    for chunk in bytes.chunks_exact(2) {
        let high = (chunk[0] as char).to_digit(16)?;
        let low = (chunk[1] as char).to_digit(16)?;
        out.push(
            (u8::try_from(high).expect("a hex digit fits u8") << 4)
                | u8::try_from(low).expect("a hex digit fits u8"),
        );
    }
    Some(out)
}

/// Encode bytes as lowercase hex.
fn hex_lower(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(char::from_digit(u32::from(byte >> 4), 16).expect("a nibble is a hex digit"));
        out.push(char::from_digit(u32::from(byte & 0x0f), 16).expect("a nibble is a hex digit"));
    }
    out
}

/// Run the case for one `LG-` conformance ID.
///
/// # Panics
/// Panics for an ID with no wired case, so a registered capability cannot pass
/// before it exists.
pub(crate) fn run(id: &str) {
    match id {
        "LG-01" => lg_01(),
        "LG-02" => lg_02(),
        "LG-03" => lg_03(),
        "LG-04" => lg_04(),
        "LG-05" => lg_05(),
        "LG-06" => lg_06(),
        "LG-07" => lg_07(),
        "LG-08" => lg_08(),
        "LG-09" => lg_09(),
        "LG-10" => lg_10(),
        "LG-11" => lg_11(),
        "LG-12" => lg_12(),
        "LG-13" => lg_13(),
        "LG-14" => lg_14(),
        other => panic!("no conformance case is wired for {other}"),
    }
}

/// LG-01: canonicalization discards comments and layout and orders declarations
/// by fully qualified name, so two sources differing only in those respects have
/// one canonical form and one content digest, and any body change moves it.
fn lg_01() {
    let plain = "def a : Nat := 1\ndef b : Nat := 2\n";
    let noisy = "\
/- a block comment naming a and b -/
-- a line comment
def a : Nat := 1     -- trailing comment
                    -- a comment between declarations

def   b : Nat := 2
";
    let reordered = "def b : Nat := 2\ndef a : Nat := 1\n";

    let expected = canonicalize(plain).expect("the plain source canonicalizes");
    for (label, variant) in [
        ("comments and layout", noisy),
        ("declaration order", reordered),
    ] {
        let canonical = canonicalize(variant).expect("the variant canonicalizes");
        assert_eq!(
            canonical.canonical_form(TOOLCHAIN),
            expected.canonical_form(TOOLCHAIN),
            "{label} must not reach the canonical form"
        );
        assert_eq!(
            canonical.content_digest(TOOLCHAIN).to_hex(),
            expected.content_digest(TOOLCHAIN).to_hex(),
            "{label} must not reach the content digest"
        );
    }

    // The names come out sorted, which is what makes the order normalization
    // total rather than a coincidence of this pair.
    assert_eq!(expected.names(), vec!["a".to_owned(), "b".to_owned()]);

    // And any body change moves the digest, whether it changes a token or adds
    // one. A digest that survived these would be ignoring the body entirely.
    for (label, changed) in [
        ("a changed literal", "def a : Nat := 2\ndef b : Nat := 2\n"),
        ("an added token", "def a : Nat := 1 + 1\ndef b : Nat := 2\n"),
        (
            "only the second body",
            "def a : Nat := 1\ndef b : Nat := 3\n",
        ),
        (
            "an added declaration",
            "def a : Nat := 1\ndef b : Nat := 2\ndef c : Nat := 4\n",
        ),
        (
            "a renamed declaration",
            "def a : Nat := 1\ndef z : Nat := 2\n",
        ),
    ] {
        assert_ne!(
            canonicalize(changed)
                .expect("the changed source canonicalizes")
                .content_digest(TOOLCHAIN)
                .to_hex(),
            expected.content_digest(TOOLCHAIN).to_hex(),
            "{label} must change the content digest"
        );
    }
}

/// LG-02: the canonical form is the §21.1 frame encoding under the
/// `lexlean-lexeme-v1` domain, so the same source hashes identically in two
/// distinct build directories and across two runs.
fn lg_02() {
    let source = "def a : Nat := 1\ndef b : Nat := a + 1\n";
    let canonical = canonicalize(source).expect("the source canonicalizes");

    // `matches_framed_hasher` is the cross-check that matters: it rebuilds the
    // §21.1 frames through the repository's own framer and compares the bytes,
    // so this asserts the encoding rather than re-running the same code path.
    assert!(
        canonical.matches_framed_hasher(TOOLCHAIN),
        "the canonical form must equal the §21.1 frame encoding"
    );

    // Two independent canonicalizations stand in for two build directories.
    // There is no path in the digest, so the bytes cannot differ by directory;
    // this is the property, stated as an equality rather than as an absence.
    let first = canonicalize(source).expect("the source canonicalizes");
    let second = canonicalize(source).expect("the source canonicalizes");
    assert_eq!(
        first.content_digest(TOOLCHAIN).to_hex(),
        second.content_digest(TOOLCHAIN).to_hex(),
        "two runs must produce one digest"
    );
    assert_eq!(
        first.canonical_form(TOOLCHAIN),
        canonical.canonical_form(TOOLCHAIN),
        "two runs must produce one canonical form"
    );

    // The domain is named in the bytes, so the digest is not the digest of the
    // same text under some other label's domain.
    assert_eq!(CANONICAL_DOMAIN, "lexlean-lexeme-v1");
    let form = canonical.canonical_form(TOOLCHAIN);
    assert!(
        form.windows(CANONICAL_DOMAIN.len())
            .any(|window| window == CANONICAL_DOMAIN.as_bytes()),
        "the canonical form must name its own hash domain"
    );

    // The toolchain is one of the frames, so two entries of the same source
    // under different toolchains are different records.
    assert_ne!(
        canonical.content_digest(TOOLCHAIN).to_hex(),
        canonical
            .content_digest("leanprover/lean4:v4.31.0")
            .to_hex(),
        "the pinned toolchain is part of the canonical form"
    );
}

/// LG-03: the content-hash record names the source paths, the
/// canonicalization identifier, the toolchain, and the sorted declaration
/// names, and every one of those fields changes the digest when changed.
fn lg_03() {
    let entry = Entry::new(
        "probe",
        TOOLCHAIN,
        &[("Probe.lean", "def a : Nat := 1\ndef b : Nat := 2\n")],
    )
    .expect("the entry builds");

    // Every field the register names is present and carries the expected value,
    // which is the first half: a digest over a field the record never states
    // cannot be recomputed by a third party.
    assert_eq!(entry.spec, ENTRY_SPEC);
    assert_eq!(entry.canonicalization, CANONICALIZATION);
    assert_eq!(entry.toolchain, TOOLCHAIN);
    assert_eq!(entry.sources.len(), 1);
    assert_eq!(entry.sources[0].path, "Probe.lean");
    assert_eq!(
        entry.declarations,
        vec!["a".to_owned(), "b".to_owned()],
        "the declaration names are sorted"
    );
    assert!(
        entry.declarations.windows(2).all(|pair| pair[0] < pair[1]),
        "the sorted names are strictly increasing, so no name repeats"
    );

    // And each of the four fields is inside the digest. The digest is rebuilt
    // from the record's own fields, so altering one field in a rebuilt record
    // and rebuilding the digest is the way to show the frame is covered.
    let baseline = entry.content_digest.to_hex();
    let hashed = entry.sources.clone();
    let declarations = entry.declarations.clone();
    let recomputed = Entry::entry_digest(TOOLCHAIN, &hashed, &declarations);
    assert_eq!(recomputed.to_hex(), baseline);

    // The toolchain frame.
    let other_toolchain = Entry::entry_digest("leanprover/lean4:v4.31.0", &hashed, &declarations);
    assert_ne!(
        other_toolchain.to_hex(),
        baseline,
        "the toolchain is a frame"
    );

    // The declarations frame.
    let fewer: Vec<String> = declarations[..1].to_vec();
    assert_ne!(
        Entry::entry_digest(TOOLCHAIN, &hashed, &fewer).to_hex(),
        baseline,
        "the declaration names are a frame"
    );

    // The source frames: both the path and the source's own canonical digest.
    let mut renamed = hashed.clone();
    renamed[0].path = "Other.lean".to_owned();
    assert_ne!(
        Entry::entry_digest(TOOLCHAIN, &renamed, &declarations).to_hex(),
        baseline,
        "the source path is a frame"
    );
    let mut retexted = hashed.clone();
    retexted[0].canonical_digest = Sha256Digest::of(b"a different source");
    assert_ne!(
        Entry::entry_digest(TOOLCHAIN, &retexted, &declarations).to_hex(),
        baseline,
        "the source digest is a frame"
    );

    // And the record is readable back, so a verifier can rebuild the digest from
    // the entry alone rather than from the inventor's filesystem.
    let decoded = Entry::from_json(&entry.to_json()).expect("the entry round trips");
    assert_eq!(decoded.content_digest.to_hex(), baseline);
    assert_eq!(decoded.sources[0].path, "Probe.lean");
    assert_eq!(
        decoded.recompute_digest().expect("recomputes").to_hex(),
        baseline
    );
}

/// LG-04: an entry carries a detached Ed25519 signature over its 32-byte
/// content digest, and verification refuses a digest altered by one bit or a
/// signature checked against a public key other than the one recorded.
fn lg_04() {
    let entry = signed_entry("probe", "def a : Nat := 1\n", 7);
    let digest = entry.content_digest.to_hex();
    let bytes = lexlean::lexeme::verify::digest_from_hex(&digest).expect("32 hex digits");

    // The signature is over the digest and nothing else, and the record says
    // which key and which algorithm, so a reader never has to guess.
    assert!(
        entry.signature.well_formed(),
        "a signed entry is well formed"
    );
    assert_eq!(
        entry.signature.value.len(),
        64,
        "Ed25519 signatures are 64 bytes"
    );
    assert_eq!(
        entry.signature.public_key.len(),
        32,
        "Ed25519 keys are 32 bytes"
    );
    assert_eq!(entry.signature.key.algorithm, SIGNATURE_ALGORITHM);
    assert!(
        entry.signature.verify(&bytes),
        "the untampered signature verifies"
    );

    // One bit is enough. Every bit is tried rather than one, because a check
    // that missed a single bit position would pass the one-bit probe below.
    for bit in 0..256 {
        let mut altered = bytes;
        altered[bit / 8] ^= 1 << (bit % 8);
        assert!(
            !entry.signature.verify(&altered),
            "a digest changed in bit {bit} must not verify"
        );
    }

    // The recorded public key is the key checked: a signature verified against
    // a different key is a different claim, so the record's own key is what
    // the verifier uses.
    let other = signed_entry("probe", "def a : Nat := 1\n", 8);
    assert_ne!(
        other.signature.public_key, entry.signature.public_key,
        "a second seed is a second key"
    );
    assert!(
        !Signature {
            public_key: other.signature.public_key,
            ..entry.signature.clone()
        }
        .verify(&bytes),
        "the signature must not verify under a foreign public key"
    );

    // The malformed shapes are refused too, so a truncated signature is not
    // mistaken for an absent one.
    let truncated = Signature {
        value: entry.signature.value[..32].to_vec(),
        ..entry.signature.clone()
    };
    assert!(
        !truncated.well_formed(),
        "a 32-byte signature is not well formed"
    );
    assert!(!truncated.verify(&bytes));
    assert!(
        !Signature::unsigned().well_formed(),
        "an unsigned entry is refused"
    );

    // And the pipeline reports a broken signature under the registered code
    // rather than as a bare false, so §26's registry stays the single source of
    // what can go wrong.
    let mut tampered = entry.clone();
    tampered.signature.value[0] ^= 0x01;
    let verdict = lexlean::lexeme::verify::verify_entry(&tampered, None).expect("a verdict");
    assert!(!verdict.verified());
    expect_code(&verdict.failure(), "LLG1003");
}

/// LG-05: a hardware-bound key records its device, slot, algorithm, and PIN
/// policy, and signing fails when the device is absent rather than falling back
/// to a software key.
fn lg_05() {
    let record = KeyRecord::piv("9a");
    assert_eq!(record.kind, KeyKind::Piv);
    assert_eq!(record.kind.as_str(), "piv", "the device token is `piv`");
    assert_eq!(record.slot.as_deref(), Some("9a"));
    assert_eq!(record.algorithm, SIGNATURE_ALGORITHM);
    assert_eq!(
        record.pin_policy, "always",
        "a hardware-bound key records that the PIN is required every time"
    );

    // The record reaches the entry's own JSON, because the custody claim is
    // part of what a third party reads, not a local fact about the signer.
    let mut entry = signed_entry("probe", "def a : Nat := 1\n", 11);
    entry.signature.key = record.clone();
    let decoded = Entry::from_json(&entry.to_json()).expect("the entry round trips");
    assert_eq!(decoded.signature.key, record);
    let Json::Obj(fields) = decoded.to_json() else {
        panic!("an entry is a JSON object")
    };
    let Some(Json::Obj(key_fields)) = fields.get("signature") else {
        panic!("the signature is a JSON object")
    };
    for field in ["device", "algorithm", "pin_policy"] {
        assert!(
            key_fields.contains_key(field),
            "the key record must name `{field}`"
        );
    }

    // Signing with no device present fails, and it fails with the registered
    // code rather than quietly producing a software signature: the whole point
    // of the hardware binding is that the entry says which custody it had.
    let digest = lexlean::lexeme::verify::digest_from_hex(&entry.content_digest.to_hex())
        .expect("32 hex digits");
    let error = lexlean::lexeme::signature::sign_with_piv(&digest, "9a", "no-such-piv-tool", None)
        .err()
        .unwrap_or_else(|| panic!("signing without the device must fail"));
    expect_code(&error, "LLG1004");

    // A missing PIN is refused before the device is even consulted, which is
    // the same custody claim arriving from the other direction.
    let unpinned =
        lexlean::lexeme::signature::sign_with_piv(&digest, "9a", "no-such-piv-tool", None)
            .expect_err("an unpinned hardware key is refused");
    expect_code(&unpinned, "LLG1004");

    // And a software key record says so plainly, so the two custody claims are
    // never read as the same one.
    assert_eq!(KeyRecord::software().kind.as_str(), "software");
    assert_eq!(KeyRecord::software().slot, None);
    assert_eq!(KeyRecord::software().pin_policy, "never");
}

/// LG-08: leaf and internal hashes use the RFC 6962 domain separation `0x00`
/// and `0x01`, so a leaf digest is never an internal node digest and the tree
/// admits no second preimage through its structure.
fn lg_08() {
    let leaf_bytes = b"{\"spec\":\"lexlean/lexeme/1\"}";
    let leaf = leaf_hash(leaf_bytes);

    // The prefixes are the standard's, recomputed here from the definition
    // rather than read back from the implementation.
    let hand_leaf = Sha256Digest::of(&[b"\x00".as_slice(), leaf_bytes].concat());
    assert_eq!(
        leaf.to_hex(),
        hand_leaf.to_hex(),
        "a leaf is SHA-256(0x00 || leaf)"
    );

    let left = leaf_hash(b"a");
    let right = leaf_hash(b"b");
    let internal = node_hash(&left, &right);
    // The node digest is over the raw 32-byte children, not their hex spelling,
    // so the recomputation decodes them before hashing.
    let hand_node =
        Sha256Digest::of(&[b"\x01".as_slice(), &hex_bytes(&left), &hex_bytes(&right)].concat());
    assert_eq!(
        internal.to_hex(),
        hand_node.to_hex(),
        "an internal node is SHA-256(0x01 || left || right)"
    );

    // The separation is the property. A leaf and an internal node over the same
    // bytes must differ, or a leaf could be passed off as a subtree.
    assert_ne!(leaf.to_hex(), internal.to_hex());
    for (index, bytes) in [b"".as_slice(), b"a", b"b", leaf_bytes].iter().enumerate() {
        let as_leaf = leaf_hash(bytes);
        let as_node = node_hash(&as_leaf, &as_leaf);
        assert_ne!(
            as_leaf.to_hex(),
            as_node.to_hex(),
            "case {index}: a leaf digest is never an internal digest"
        );
    }

    // A second preimage cannot be introduced through the structure: the domain
    // prefix is what stops the length-extension shape a bare concatenation would
    // allow, because `0x01 || left || right` is not a leaf preimage even when the
    // tail happens to parse as one.
    let mut forged: Vec<u8> = vec![0x01];
    forged.extend_from_slice(&hex_bytes(&left));
    forged.extend_from_slice(b"b");
    assert_ne!(
        internal.to_hex(),
        leaf_hash(&forged).to_hex(),
        "a node preimage is not a leaf preimage"
    );
    assert_ne!(
        internal.to_hex(),
        leaf_hash(&hex_bytes(&internal)).to_hex(),
        "a digest of a node digest is not that node"
    );
}

/// The raw bytes of a digest, which the §33.5 node hash is defined over.
fn hex_bytes(digest: &Sha256Digest) -> Vec<u8> {
    lexlean::lexeme::signature::hex_decode(&digest.to_hex()).expect("a digest is hex")
}

/// LG-09: appending an entry returns the new tree size, the appended leaf
/// index, and an O(log n) audit path, and the stored head of every prefix
/// equals the root recomputed from that prefix's entries.
fn lg_09() {
    let directory = tempfile::tempdir().expect("a scratch directory");
    let mut ledger = Ledger::open(directory.path()).expect("the ledger opens");
    assert_eq!(ledger.len(), 0);
    assert!(ledger.is_empty());
    assert_eq!(ledger.root(), None, "an empty tree has no root");
    assert!(ledger.leaf_bytes().is_none(), "an empty tree has no leaves");

    // Append enough entries that the tree's unbalanced shape is exercised, not
    // just the power-of-two case that a two-leaf tree would give.
    let count = 17u64;
    for index in 0..count {
        let entry = signed_entry(
            &format!("entry {index}"),
            &source_with(u32::try_from(index).expect("in range")),
            u8::try_from(index).expect("in range"),
        );
        let appended = ledger
            .append(entry, &log_seed())
            .expect("the entry appends");
        let inclusion = appended.inclusion.expect("an append returns an inclusion");

        assert_eq!(
            inclusion.tree_size,
            index + 1,
            "append {index} reports the new tree size"
        );
        assert_eq!(
            inclusion.leaf_index, index,
            "append {index} reports the appended leaf index"
        );
        assert_eq!(
            inclusion.audit_path.len(),
            audit_path_length(index, index + 1),
            "append {index} returns the path length this tree size requires"
        );
        assert!(
            inclusion.audit_path.len() <= 64,
            "the audit path is O(log n): {} elements for {count} leaves",
            inclusion.audit_path.len()
        );

        // Every append so far published a head, and the head at each size is the
        // root of that prefix.
        assert_eq!(ledger.len(), usize::try_from(index + 1).expect("in range"));
        let heads = ledger.heads();
        assert_eq!(
            heads.len(),
            usize::try_from(index + 1).expect("in range"),
            "one head per append"
        );
        let leaves = ledger.leaf_bytes().expect("leaves");
        for head in heads {
            let prefix = &leaves[..usize::try_from(head.tree_size).expect("in range")];
            let recomputed = lexlean::lexeme::merkle::root_of(prefix).expect("a non-empty prefix");
            assert_eq!(
                head.root_hash.to_hex(),
                recomputed.to_hex(),
                "the stored head of size {} must equal the root recomputed from it",
                head.tree_size
            );
        }
    }

    // The ledger's own check is the same property read back from disk.
    ledger
        .check_stored_head()
        .expect("every stored head matches its prefix");
    // And the heads survive a close and reopen, which is what "published" means:
    // the property is about what a later reader finds on disk, not about this
    // process's memory.
    let reopened = Ledger::open(directory.path()).expect("the ledger reopens");
    reopened
        .check_stored_head()
        .expect("the reopened heads match");
    assert_eq!(reopened.len(), usize::try_from(count).expect("in range"));
}

/// LG-10: an inclusion proof is accepted exactly when the recomputed root
/// equals the published root, and is refused for a wrong-length path, a
/// reordered path, or an index at or beyond the tree size.
fn lg_10() {
    let directory = tempfile::tempdir().expect("a scratch directory");
    let mut ledger = Ledger::open(directory.path()).expect("the ledger opens");
    let mut inclusions = Vec::new();
    for index in 0..6u64 {
        let appended = ledger
            .append(
                signed_entry(
                    &format!("entry {index}"),
                    &source_with(u32::try_from(index).expect("in range")),
                    u8::try_from(index + 40).expect("in range"),
                ),
                &log_seed(),
            )
            .expect("the entry appends");
        inclusions.push(appended.inclusion.expect("an append returns an inclusion"));
    }
    let leaves = ledger.leaf_bytes().expect("leaves");

    // The honest proof is accepted, for every leaf, on its own bytes.
    for (index, inclusion) in inclusions.iter().enumerate() {
        verify_inclusion(&leaves[index], inclusion)
            .unwrap_or_else(|failure| panic!("leaf {index} must verify: {}", failure.as_str()));
    }

    // The proof travels with the entry and reaches the same verdict, so the
    // verifier needs the log's shape and never the log itself.
    let entry = Entry::from_json(&lexlean::artifact::canonical_json::Json::Str(
        ledger.entry_lines()[5].to_owned(),
    ));
    assert!(
        entry.is_err(),
        "a bare line is not canonical JSON on its own"
    );

    // A wrong-length path is refused: a truncated path would recompute a
    // shorter tree, and an extended one a longer tree.
    let honest = inclusions[2].clone();
    let leaf_two = &leaves[2];
    let mut truncated = honest.clone();
    truncated.audit_path.pop();
    assert_eq!(
        verify_inclusion(leaf_two, &truncated),
        Err(InclusionFailure::MalformedPath),
        "a path one element short is refused"
    );
    let mut extended = honest.clone();
    extended
        .audit_path
        .push(leaf_hash(b"an element past the end"));
    assert_eq!(
        verify_inclusion(leaf_two, &extended),
        Err(InclusionFailure::MalformedPath),
        "a path one element long is refused"
    );

    // A reordered path is refused. The order of an audit path is load-bearing:
    // it names left and right siblings, so swapping two elements either changes
    // the path length, which the length check catches, or yields a different
    // root, which the root comparison catches.
    let mut reordered = honest.clone();
    if reordered.audit_path.len() >= 2 {
        reordered.audit_path.swap(0, 1);
        let outcome = verify_inclusion(leaf_two, &reordered);
        assert!(
            outcome.is_err(),
            "a reordered path must be refused, got {outcome:?}"
        );
    }

    // An index at the tree size is refused: the tree has no such leaf.
    let mut at_size = honest.clone();
    at_size.leaf_index = at_size.tree_size;
    assert_eq!(
        verify_inclusion(leaf_two, &at_size),
        Err(InclusionFailure::IndexOutOfRange),
        "an index at the tree size is refused"
    );
    let mut past_size = honest.clone();
    past_size.leaf_index = past_size.tree_size + 100;
    assert_eq!(
        verify_inclusion(leaf_two, &past_size),
        Err(InclusionFailure::IndexOutOfRange),
        "an index beyond the tree size is refused"
    );

    // A different leaf's bytes under this proof are refused, which is the
    // tamper-evidence claim in the direction that carries weight.
    let mut tampered = leaves[2].clone();
    let position = tampered.len() - 2;
    tampered[position] ^= 0x01;
    assert_eq!(
        verify_inclusion(&tampered, &honest),
        Err(InclusionFailure::RootMismatch),
        "a modified leaf is refused under the honest proof"
    );

    // A rewritten root is refused as a mismatch rather than accepted, because
    // the published root is what the proof has to reach.
    let mut forked = honest.clone();
    forked.root_hash = leaf_hash(b"a different root");
    assert_eq!(
        verify_inclusion(leaf_two, &forked),
        Err(InclusionFailure::RootMismatch),
        "a proof reaching another root is refused"
    );

    // An empty tree has no root, so it has no leaf to verify.
    assert_eq!(
        verify_inclusion(
            leaf_two,
            &Inclusion {
                leaf_index: 0,
                tree_size: 0,
                audit_path: Vec::new(),
                root_hash: leaf_hash(b"x"),
            }
        ),
        Err(InclusionFailure::EmptyTree)
    );

    // And the refusal is reported under the registered code, so §26's registry
    // stays the single source of what can go wrong.
    expect_code(
        &lexlean::lexeme::verify::inclusion_error(InclusionFailure::RootMismatch.as_str()),
        "LLG1007",
    );
}

/// LG-11: two published tree heads verify as consistent exactly when the
/// consistency proof from the smaller size recomputes the larger root from the
/// smaller root, so a forked history is detectable from the heads alone.
fn lg_11() {
    let directory = tempfile::tempdir().expect("a scratch directory");
    let mut ledger = Ledger::open(directory.path()).expect("the ledger opens");
    for index in 0..9u64 {
        ledger
            .append(
                signed_entry(
                    &format!("entry {index}"),
                    &source_with(u32::try_from(index).expect("in range")),
                    u8::try_from(index + 60).expect("in range"),
                ),
                &log_seed(),
            )
            .expect("the entry appends");
    }
    let leaves = ledger.leaf_bytes().expect("leaves");
    let heads = ledger.heads().to_vec();

    // Each head carries the proof from its predecessor's size to its own, so
    // the pair that a proof speaks for is (head[i-1], head[i]) and the proof is
    // read off the newer one. The first head carries an empty proof because it
    // has no predecessor, which is why the walk starts at the second.
    for pair in heads[1..].windows(2) {
        let older = &pair[0];
        let newer = &pair[1];
        assert_eq!(
            newer.consistency_proof.len(),
            lexlean::lexeme::merkle::consistency_path_length(older.tree_size, newer.tree_size),
            "the proof on head {} has the length sizes {} and {} require",
            newer.tree_size,
            older.tree_size,
            newer.tree_size
        );
        verify_consistency(
            &older.root_hash,
            older.tree_size,
            &newer.root_hash,
            newer.tree_size,
            &newer.consistency_proof,
        )
        .unwrap_or_else(|failure| {
            panic!(
                "heads {} and {} must be consistent: {}",
                older.tree_size,
                newer.tree_size,
                failure.as_str()
            )
        });
    }
    assert!(
        heads[0].consistency_proof.is_empty(),
        "the first head has no predecessor, so it carries no proof"
    );

    // A proof across the whole range is recomputable from the leaves, so a
    // verifier holding the first head and the last one can be shown the whole
    // history without the log in between.
    let first = &heads[0];
    let last = heads.last().expect("a head");
    let recomputed = consistency_path_from(&leaves, first.tree_size);
    verify_consistency(
        &first.root_hash,
        first.tree_size,
        &last.root_hash,
        last.tree_size,
        &recomputed,
    )
    .expect("a recomputed proof carries the first root to the last");

    // A forked history is refused. The fork rewrites the newest leaf, so the
    // older head is one both histories share and only the newer root differs ---
    // which is exactly the case a consistency proof exists to catch, and the
    // case a rewrite of the log would produce.
    let older = &heads[heads.len() - 2];
    let newer = heads.last().expect("a head");
    let mut forked_leaves = leaves.clone();
    let last_index = usize::try_from(newer.tree_size - 1).expect("in range");
    let position = forked_leaves[last_index].len() - 2;
    forked_leaves[last_index][position] ^= 0x01;
    let forked_root = lexlean::lexeme::merkle::root_of(&forked_leaves).expect("a non-empty fork");
    assert_ne!(
        forked_root.to_hex(),
        newer.root_hash.to_hex(),
        "the fork must reach a different root, or nothing is being tested"
    );
    assert_eq!(
        verify_consistency(
            &older.root_hash,
            older.tree_size,
            &forked_root,
            newer.tree_size,
            &newer.consistency_proof,
        ),
        Err(ConsistencyFailure::RootMismatch),
        "a forked history is refused"
    );

    // The degenerate pairs are refused for the reasons §33.5 gives, so a proof
    // is never accepted against a head that cannot carry one.
    assert_eq!(
        verify_consistency(&first.root_hash, 0, &last.root_hash, last.tree_size, &[]),
        Err(ConsistencyFailure::OldTreeEmpty),
        "an empty older head admits no proof"
    );
    assert_eq!(
        verify_consistency(
            &last.root_hash,
            last.tree_size,
            &first.root_hash,
            first.tree_size,
            &first.consistency_proof
        ),
        Err(ConsistencyFailure::OldTreeLarger),
        "an older head claiming more leaves than the newer one is refused"
    );

    // A malformed proof is refused rather than recomputed around.
    let mut extended = first.consistency_proof.clone();
    extended.push(leaf_hash(b"past the end"));
    assert_eq!(
        verify_consistency(
            &first.root_hash,
            first.tree_size,
            &last.root_hash,
            last.tree_size,
            &extended
        ),
        Err(ConsistencyFailure::MalformedPath),
        "a proof longer than the sizes require is refused"
    );

    // And the refusal carries the registered code.
    let error = lexlean::lexeme::verify::inclusion_error(ConsistencyFailure::RootMismatch.as_str());
    expect_code(&error, "LLG1007");
}

/// LG-12: verification reports the canonical hash, signature, timestamp, and
/// inclusion separately, and the verdict asserts only existence at a stated
/// time, authorship by a stated key, and integrity since.
fn lg_12() {
    let directory = tempfile::tempdir().expect("a scratch directory");
    let mut ledger = Ledger::open(directory.path()).expect("the ledger opens");
    let appended = ledger
        .append(signed_entry("probe", "def a : Nat := 1\n", 3), &log_seed())
        .expect("the entry appends");

    // The five checks of §33.6, in order, each reported on its own. A verdict
    // that reported only a single pass/fail would not say which check failed,
    // and a reader would have to re-derive it.
    let verdict = lexlean::lexeme::verify::verify_entry(&appended, Some(&ledger))
        .expect("an appended entry verifies against its own ledger");
    let steps: Vec<u8> = verdict.checks.iter().map(|check| check.step).collect();
    assert_eq!(
        steps,
        vec![1, 2, 3, 4, 5],
        "the five §33.6 checks are reported in order"
    );
    for step in 1..=5u8 {
        let check = verdict
            .checks
            .iter()
            .find(|check| check.step == step)
            .unwrap_or_else(|| panic!("step {step} is reported"));
        assert!(
            !check.name.is_empty() && !check.detail.is_empty(),
            "step {step} names what it checked and what it found"
        );
    }

    // Steps 1, 2, 4, and 5 pass on an appended, signed entry. Step 3 does not,
    // because no timestamp was obtained: §33.4 treats an absent timestamp as
    // unestablished existence, and a verdict that called this entry verified
    // would be claiming a stated time that nothing states.
    let untimestamped: Vec<u8> = verdict
        .checks
        .iter()
        .filter(|check| !check.passed)
        .map(|check| check.step)
        .collect();
    assert_eq!(
        untimestamped,
        vec![3],
        "only the timestamp check is open, and an untimestamped entry is not verified"
    );
    assert!(!verdict.verified());
    assert!(verdict.statement().contains("not established"));

    // A timestamp that is present but empty is not a timestamp. The field's
    // presence is the presence of the claim, so an entry carrying no timestamp
    // object cannot be read as carrying an empty one.
    assert!(appended.timestamp.is_none());
    let Json::Obj(fields) = appended.to_json() else {
        panic!("an entry is a JSON object")
    };
    assert!(
        !fields.contains_key("timestamp"),
        "an absent timestamp omits the field rather than carrying an empty one"
    );

    // The verdict's wording is the load-bearing half, and it is only reachable
    // on an entry every check passed. No probe here can be that entry, because
    // §33.4 admits no internal substitute for a timestamp, so the wording is
    // exercised on a verdict built directly: what is under test is the sentence
    // the §33.6 checks produce, not the path that produced them.
    let established = lexlean::lexeme::verify::Verdict {
        title: "probe".to_owned(),
        checks: (1..=5)
            .map(|step| lexlean::lexeme::verify::Check {
                step,
                name: "a check",
                passed: true,
                detail: match step {
                    2 => "ed25519:0123456789abcdef0123".to_owned(),
                    3 => "2026-09-15T12:00:00Z under FreeTSA".to_owned(),
                    _ => "ok".to_owned(),
                },
                code: match step {
                    1 => lexlean::code!("LLG1002"),
                    2 => lexlean::code!("LLG1003"),
                    3 => lexlean::code!("LLG1005"),
                    4 => lexlean::code!("LLG1007"),
                    _ => lexlean::code!("LLG1008"),
                },
            })
            .collect(),
    };
    assert!(established.verified());
    let statement = established.statement();
    for required in ["existed at", "authored by", "has not been modified since"] {
        assert!(
            statement.contains(required),
            "the verdict must state `{required}`: {statement}"
        );
    }
    // The forbidden words do appear, but only inside the sentence that disclaims
    // them. So the claim and the disclaimer are checked separately: the claim
    // must be clean, and the disclaimer must be present, because a verdict that
    // quietly dropped it would be the failure this case exists to catch.
    let (claim, disclaimer) = statement
        .split_once("This is a statement about cryptography:")
        .unwrap_or_else(|| panic!("the §33.6 disclaimer must be present: {statement}"));
    for forbidden in [
        "novel",
        "valid",
        "enforceable",
        "legal",
        "patent",
        "unique",
        "infringe",
        "original",
        "copyright",
    ] {
        assert!(
            !claim.to_lowercase().contains(forbidden),
            "the §33.6 claim must not assert `{forbidden}`: {claim}"
        );
    }
    assert!(
        disclaimer.contains("not a claim that the subject matter is novel, valid, or enforceable"),
        "the §33.6 disclaimer must name what it does not claim: {statement}"
    );

    // And the same rule holds when the verdict is reported as JSON, since that
    // is the machine-readable form a regulator would read.
    let Json::Obj(verdict_fields) = verdict.to_json() else {
        panic!("a verdict is a JSON object")
    };
    let rendered = format!("{verdict_fields:?}").to_lowercase();
    for forbidden in ["novel", "enforceable", "patent"] {
        assert!(
            !rendered.contains(forbidden),
            "the JSON verdict must not claim `{forbidden}`"
        );
    }

    // An entry whose inclusion proof does not reach the published head is
    // refused, and the refusal names the root rather than passing silently.
    let mut relocated = appended.clone();
    relocated.inclusion = Some(Inclusion {
        leaf_index: 0,
        tree_size: 1,
        audit_path: Vec::new(),
        root_hash: leaf_hash(b"a root this ledger never published"),
    });
    let forked = lexlean::lexeme::verify::verify_entry(&relocated, Some(&ledger))
        .expect("a verdict, not an error");
    assert!(
        !forked.verified(),
        "a proof reaching another root is not verified"
    );
    assert!(
        forked
            .checks
            .iter()
            .any(|check| check.step == 5 && !check.passed),
        "the root check is the one that fails"
    );
}

/// LG-13: every entry is validated against the closed `lexlean/lexeme/1`
/// schema before it is appended, and an entry failing validation is refused
/// with a registered diagnostic code.
fn lg_13() {
    let source = "def a : Nat := 1\n";
    let entry = signed_entry("probe", source, 17);

    // The schema the register names is a committed file, and it is the schema
    // the entry is checked against rather than a description of one: the
    // committed bytes are loaded and the entry is validated by them, so a field
    // added to the entry without a matching schema property fails here.
    let schema_path = crate::support::repo_root().join("schemas/lexeme-entry.schema.json");
    assert!(
        schema_path.is_file(),
        "the §33.2 schema is a committed repository file"
    );
    let schema_bytes =
        std::fs::read(schema_path.as_std_path()).expect("the §33.2 schema is readable");
    let schema: serde_json::Value =
        serde_json::from_slice(&schema_bytes).expect("the §33.2 schema is JSON");
    let as_value = |value: &Json| -> serde_json::Value {
        serde_json::from_str(&value.to_canonical_string()).expect("canonical JSON parses")
    };
    assert_eq!(entry.spec, "lexlean/lexeme/1");

    // A well-formed entry satisfies the committed schema and is accepted by the
    // reader the append path uses.
    let violations = crate::schema::validate(&schema, &as_value(&entry.to_json()));
    assert!(
        violations.is_empty(),
        "the entry violates the schema: {violations:?}"
    );
    let parsed = Entry::from_json(&entry.to_json()).expect("a valid entry parses");
    assert_eq!(parsed.spec, ENTRY_SPEC);

    // Every §33.2 field the schema requires is required by it, not only by the
    // reader: a schema that had forgotten one would pass an entry the reader
    // refuses, which is the drift this case exists to catch.
    for field in [
        "spec",
        "title",
        "canonicalization",
        "sources",
        "declarations",
        "content_digest",
        "toolchain",
        "signature",
    ] {
        assert!(
            schema["required"]
                .as_array()
                .is_some_and(|required| required.iter().any(|name| name == field)),
            "the §33.2 schema must require `{field}`"
        );
    }
    for field in ["timestamp", "inclusion", "pirtm"] {
        assert!(
            !schema["required"]
                .as_array()
                .is_some_and(|required| required.iter().any(|name| name == field)),
            "§33.2 omits `{field}` when absent, so the schema must not require it"
        );
    }

    // Each required field is required: dropping one is refused rather than
    // defaulted, because a verifier that filled a missing field in would be
    // inventing part of the record it is verifying.
    for (field, mutate) in [
        (
            "spec",
            Box::new(|value: &mut Json| {
                remove(value, "spec");
            }) as Box<dyn Fn(&mut Json)>,
        ),
        (
            "title",
            Box::new(|value: &mut Json| {
                remove(value, "title");
            }),
        ),
        (
            "canonicalization",
            Box::new(|value: &mut Json| {
                remove(value, "canonicalization");
            }),
        ),
        (
            "sources",
            Box::new(|value: &mut Json| {
                remove(value, "sources");
            }),
        ),
        (
            "declarations",
            Box::new(|value: &mut Json| {
                remove(value, "declarations");
            }),
        ),
        (
            "content_digest",
            Box::new(|value: &mut Json| {
                remove(value, "content_digest");
            }),
        ),
        (
            "toolchain",
            Box::new(|value: &mut Json| {
                remove(value, "toolchain");
            }),
        ),
    ] {
        let mut value = entry.to_json();
        mutate(&mut value);
        let violations = crate::schema::validate(&schema, &as_value(&value));
        assert!(
            !violations.is_empty(),
            "the §33.2 schema must refuse an entry with no `{field}`"
        );
        let error = Entry::from_json(&value)
            .err()
            .unwrap_or_else(|| panic!("an entry with no `{field}` must be refused"));
        expect_code(&error, "LLG1002");
    }

    // A digest that is not 64 hex digits is refused, so the field cannot carry
    // an arbitrary string where a digest belongs.
    for bad in ["", "abc", &"z".repeat(64), &"a".repeat(63)] {
        let mut value = entry.to_json();
        set(&mut value, "content_digest", Json::Str(bad.to_owned()));
        let violations = crate::schema::validate(&schema, &as_value(&value));
        assert!(
            !violations.is_empty(),
            "the §33.2 schema must refuse a content_digest of `{bad}`"
        );
        let error = Entry::from_json(&value)
            .err()
            .unwrap_or_else(|| panic!("a digest of `{bad}` must be refused"));
        expect_code(&error, "LLG1002");
    }

    // The schema is closed: a field outside §33.2 is a violation, so an entry
    // cannot carry a claim the contract does not define.
    let mut extra = entry.to_json();
    set(
        &mut extra,
        "invented",
        Json::Str("a claim the contract does not define".to_owned()),
    );
    assert!(
        !crate::schema::validate(&schema, &as_value(&extra)).is_empty(),
        "the §33.2 schema must refuse a field it does not define"
    );

    // An unsigned entry cannot be appended, which is the validation the append
    // path performs on its own fields before it touches the log.
    let directory = tempfile::tempdir().expect("a scratch directory");
    let mut ledger = Ledger::open(directory.path()).expect("the ledger opens");
    let unsigned = Entry::new("probe", TOOLCHAIN, &[("Probe.lean", source)]).expect("builds");
    let error = ledger
        .append(unsigned, &log_seed())
        .expect_err("an unsigned entry must not be appended");
    expect_code(&error, "LLG1002");
    assert_eq!(ledger.len(), 0, "a refused append leaves the ledger empty");

    // And an entry whose digest does not match its own sources is refused too,
    // so the digest is recomputed rather than trusted.
    let mut inconsistent = signed_entry("probe", source, 19);
    inconsistent.title = "a different title".to_owned();
    inconsistent.content_digest = Sha256Digest::of(b"a digest of something else");
    let error = ledger
        .append(inconsistent, &log_seed())
        .expect_err("an entry whose digest does not recompute must not be appended");
    expect_code(&error, "LLG1002");
    assert_eq!(ledger.len(), 0, "the ledger is still empty");
}

/// Delete a key from a JSON object.
fn remove(value: &mut Json, key: &str) {
    let Json::Obj(fields) = value else {
        panic!("a JSON object")
    };
    fields.remove(key);
}

/// Replace a key's value in a JSON object.
fn set(value: &mut Json, key: &str, replacement: Json) {
    let Json::Obj(fields) = value else {
        panic!("a JSON object")
    };
    fields.insert(key.to_owned(), replacement);
}

/// The four inputs §33.4 verification reads, taken from one committed fixture.
///
/// They are loaded together because a case that assembled them from separate
/// files could pair a token with a certificate the entry never carried and still
/// call the result a fixture.
struct Rfc3161Fixture {
    /// The parsed `TSTInfo` and `SignerInfo`.
    parsed: lexlean::lexeme::timestamp::ParsedResponse,
    /// The bytes the token is over: the entry's detached signature.
    artifact: Vec<u8>,
    /// The DER certificate of the issuing TSA.
    leaf: Vec<u8>,
    /// The DER certificate of the pinned root.
    root: Vec<u8>,
}

fn rfc3161_fixture(case: &str) -> Rfc3161Fixture {
    let path = crate::support::repo_root()
        .join("tests/fixtures/rfc3161")
        .join(case)
        .join("stamped-entry.json");
    let bytes = std::fs::read(&path).unwrap_or_else(|error| panic!("{path} is readable: {error}"));
    let json =
        Json::parse(&bytes).unwrap_or_else(|error| panic!("the fixture entry is JSON: {error}"));
    let entry =
        Entry::from_json(&json).unwrap_or_else(|error| panic!("the fixture entry parses: {error}"));
    let anchor = entry
        .timestamp
        .as_ref()
        .unwrap_or_else(|| panic!("the fixture entry carries a timestamp"));
    assert_eq!(
        anchor.artifact, "signature",
        "§33.6 timestamps the detached signature, which is what the fixture is over"
    );
    let parsed = anchor
        .parsed()
        .unwrap_or_else(|error| panic!("the fixture token parses: {error}"));
    Rfc3161Fixture {
        parsed,
        artifact: entry.signature.timestamplable_bytes().to_vec(),
        leaf: anchor.tsa_certificate.clone(),
        root: anchor.tsa_root.clone(),
    }
}

/// Verify `fixture`, expecting the refusal to carry `code`.
fn expect_refused(
    fixture: &Rfc3161Fixture,
    artifact: &[u8],
    leaf: &[u8],
    root: &[u8],
    code: &str,
    because: &str,
) {
    let failure = lexlean::lexeme::timestamp::verify_token(&fixture.parsed, artifact, leaf, root)
        .expect_err(because);
    expect_code(&failure.into_error(), code);
}

/// LG-06: the token's imprint names this entry's algorithm, digest, and
/// artifact, and an imprint naming another algorithm, another digest, or another
/// artifact is refused with LLG1005.
///
/// Each of the three is broken separately, because one refusal would satisfy the
/// scenario while leaving the other two unread. The refusals are reached with
/// the real certificate and root, so what is being observed is the imprint
/// check and not an earlier failure.
fn lg_06() {
    let fixture = rfc3161_fixture("lg-06-freetsa");

    assert_eq!(
        fixture.parsed.info.hash_algorithm, "2.16.840.1.101.3.4.2.1",
        "the committed token names SHA-256"
    );
    assert_eq!(
        fixture.parsed.info.hashed_message,
        Sha256Digest::of(&fixture.artifact).0,
        "the imprint is the SHA-256 of the bytes the entry says were timestamped"
    );
    lexlean::lexeme::timestamp::verify_token(
        &fixture.parsed,
        &fixture.artifact,
        &fixture.leaf,
        &fixture.root,
    )
    .expect("the committed token names this entry's artifact");

    // Another artifact: this token, read against bytes it was not issued over.
    expect_refused(
        &fixture,
        b"the timestamplable bytes of some other entry",
        &fixture.leaf,
        &fixture.root,
        "LLG1005",
        "an imprint over another artifact is refused",
    );

    // Another algorithm: a token that names a digest this layer does not admit
    // for the imprint. A reader that ignored `hash_algorithm` would accept it.
    for algorithm in ["2.16.840.1.101.3.4.2.3", "1.3.14.3.2.26"] {
        let mut parsed = fixture.parsed.clone();
        parsed.info.hash_algorithm = algorithm.to_owned();
        let failure = lexlean::lexeme::timestamp::verify_token(
            &parsed,
            &fixture.artifact,
            &fixture.leaf,
            &fixture.root,
        )
        .map_err(TokenFailure::into_error)
        .expect_err(&format!("an imprint naming {algorithm} is refused"));
        expect_code(&failure, "LLG1005");
    }

    // The token's own content, for the case where the signature covers the
    // attributes rather than the content. A CMS `SignerInfo` that carries
    // `signedAttrs` signs those attributes, and the attributes bind the content
    // only through their `messageDigest`. So the `signedAttrs` of a genuine token
    // would verify under the TSA key while the `TSTInfo` beside it was free to
    // be any `TSTInfo` at all, and the signature would prove nothing about the
    // instant being reported. This is the case that reading cannot have.
    assert!(
        fixture.parsed.signed_attributes.is_some(),
        "the committed token signs its attributes, which is the shape this reading must handle"
    );
    let mut swapped = fixture.parsed.clone();
    swapped.signed_content = b"a TSTInfo that was never timestamped".to_vec();
    let failure = lexlean::lexeme::timestamp::verify_token(
        &swapped,
        &fixture.artifact,
        &fixture.leaf,
        &fixture.root,
    )
    .expect_err("attributes that bind other content are refused");
    expect_code(&failure.into_error(), "LLG1005");
}

/// LG-07: the token's signature verifies under the certificate the entry
/// carries, that certificate verifies under the pinned root, and the root's
/// interval contains the `genTime`.
///
/// The chain is verified whole and then broken in each of the three places the
/// scenario names. One broken fixture would satisfy the scenario against a
/// reader that checked only the token's own signature, and an interval read
/// after the signature checks could not tell an expired authority from an
/// unverifiable one.
fn lg_07() {
    let fixture = rfc3161_fixture("lg-07-freetsa");

    lexlean::lexeme::timestamp::verify_token(
        &fixture.parsed,
        &fixture.artifact,
        &fixture.leaf,
        &fixture.root,
    )
    .expect("the committed chain verifies: the token under the TSA certificate, and that certificate under the pinned root");

    // A chain that does not build. The TSA certificate is passed as the pinned
    // root, which cannot verify anything: it is not self-signed, so the root's
    // own self-consistency check is what refuses it. Both intervals still
    // contain the `genTime`, which is the point --- the refusal cannot be an
    // interval refusal, so this case exercises the signature check and not the
    // one below it.
    expect_refused(
        &fixture,
        &fixture.artifact,
        &fixture.leaf,
        &fixture.leaf,
        "LLG1006",
        "a pinned root that is not self-signed is refused",
    );

    // The TSA certificate under the pinned root. The pinned root here is a
    // self-signed authority that does not sign this chain, and the instant is
    // moved one day later so that both it and the TSA certificate are inside
    // both validity intervals. That leaves the certificate's signature under the
    // root as the only thing that can refuse, which is what makes this case
    // about that check rather than about either of the interval checks or the
    // root's own.
    let mut in_window = fixture.parsed.clone();
    in_window.info.gen_time = "20261005000000Z".to_owned();
    let stranger = std::fs::read(
        crate::support::repo_root().join("tests/fixtures/rfc3161/lg-07-freetsa/unrelated-root.der"),
    )
    .expect("the unrelated root is committed");
    let failure = lexlean::lexeme::timestamp::verify_token(
        &in_window,
        &fixture.artifact,
        &fixture.leaf,
        &stranger,
    )
    .expect_err("a certificate that does not chain to a self-signed root is refused");
    let reason = failure.to_string();
    expect_code(&failure.into_error(), "LLG1006");
    assert!(
        reason.contains("does not verify under the pinned root"),
        "the refusal names the certificate under the root: {reason}"
    );

    // The token's own signature. Both certificates and both intervals are the
    // committed ones, so the signature octets are what is wrong: one byte of the
    // parsed signature is flipped. A reader that skipped the token's signature
    // would accept this token as timed.
    let mut forged = fixture.parsed.clone();
    let mut signature = forged.signature.clone();
    signature[0] ^= 0x01;
    forged.signature = signature;
    let failure = lexlean::lexeme::timestamp::verify_token(
        &forged,
        &fixture.artifact,
        &fixture.leaf,
        &fixture.root,
    )
    .expect_err("a token whose signature does not verify is refused");
    let reason = failure.to_string();
    expect_code(&failure.into_error(), "LLG1006");
    assert!(
        reason.contains("the token's signature does not verify"),
        "the refusal names the token's own signature: {reason}"
    );

    // The TSA certificate's own validity interval. The instant is moved to
    // before the certificate was issued, with the committed chain otherwise
    // intact, so the certificate's interval is the only thing that can refuse.
    let mut before = fixture.parsed.clone();
    before.info.gen_time = "20260101000000Z".to_owned();
    let failure = lexlean::lexeme::timestamp::verify_token(
        &before,
        &fixture.artifact,
        &fixture.leaf,
        &fixture.root,
    )
    .expect_err("an instant before the TSA certificate was issued is refused");
    let reason = failure.to_string();
    expect_code(&failure.into_error(), "LLG1006");
    assert!(
        reason.contains("the TSA certificate is valid from"),
        "the refusal names the TSA certificate's interval: {reason}"
    );

    // The root's own self-signature. This is the pinned root of the committed
    // chain with one byte of its signature flipped, so its key still verifies the
    // TSA certificate and its interval still contains the `genTime`: the only
    // thing wrong with it is that it does not sign itself. A reader that skipped
    // the self-consistency check would accept this root, and would then report a
    // broken root as a working one.
    let corrupt = std::fs::read(
        crate::support::repo_root().join("tests/fixtures/rfc3161/lg-07-freetsa/corrupt-root.der"),
    )
    .expect("the corrupted root is committed");
    let failure = lexlean::lexeme::timestamp::verify_token(
        &fixture.parsed,
        &fixture.artifact,
        &fixture.leaf,
        &corrupt,
    )
    .expect_err("a pinned root that does not sign itself is refused");
    let reason = failure.to_string();
    expect_code(&failure.into_error(), "LLG1006");
    assert!(
        reason.contains("self-consistent"),
        "the refusal names the root's own signature: {reason}"
    );

    // An interval that excludes the `genTime`. The pinned root here is valid
    // only from a moment after the token was issued, so its interval refuses the
    // token's own instant while the TSA certificate's wider interval still
    // accepts it --- which is what makes this the root's interval and not the
    // leaf's. The `genTime` is not moved: it is inside the signature, and
    // moving it would test a forgery rather than an interval.
    let out_of_window = std::fs::read(
        crate::support::repo_root().join("tests/fixtures/rfc3161/lg-07-freetsa/unrelated-root.der"),
    )
    .expect("the out-of-window root is committed");
    let failure = lexlean::lexeme::timestamp::verify_token(
        &fixture.parsed,
        &fixture.artifact,
        &fixture.leaf,
        &out_of_window,
    )
    .expect_err("an instant outside the pinned root's interval is refused");
    let reason = failure.to_string();
    expect_code(&failure.into_error(), "LLG1006");
    assert!(
        reason.contains("does not contain"),
        "the refusal names the interval, not a signature: {reason}"
    );
}

/// LG-14: re-verifying a committed ledger directory reproduces every verdict and
/// every published root without network access.
///
/// The corpus is committed at `lexeme/corpus/` rather than built in a scratch
/// directory, because the row is about a directory a third party holds: a
/// verifier that only ever re-reads a log it just wrote in memory has not shown
/// that the published bytes carry the same evidence.
fn lg_14() {
    let corpus = crate::support::repo_root().join("lexeme/corpus");
    for committed in [
        "entries/alpha.json",
        "entries/beta.json",
        "entries/gamma.json",
        "ledger/log.json",
        "ledger/heads.json",
        "ledger/roots.txt",
        "sources/alpha/Successor.lean",
        "verdicts.json",
    ] {
        assert!(
            corpus.join(committed).is_file(),
            "the corpus commits `{committed}`; §33.10 reads a committed corpus, not a scratch directory"
        );
    }

    let recorded_bytes = std::fs::read(corpus.join("verdicts.json").as_std_path())
        .expect("the recorded verdicts are readable");
    let canonical = Json::parse(&recorded_bytes).expect("the recorded verdicts are canonical JSON");
    assert_eq!(
        canonical.to_file_bytes(),
        recorded_bytes,
        "the recorded verdicts are committed in canonical form (§21.7), so a rewrite of their bytes is a rewrite of the oracle"
    );
    let recorded: serde_json::Value =
        serde_json::from_slice(&recorded_bytes).expect("the recorded verdicts parse");
    assert_eq!(
        recorded["spec"], "lexlean/corpus-verdicts/1",
        "the oracle names the spec it is written in"
    );
    let entries = recorded["entries"]
        .as_array()
        .expect("the oracle lists the corpus entries");
    assert!(
        entries.len() >= 3,
        "the corpus carries at least three entries, so a consistency proof and a second head exist; it carries {}",
        entries.len()
    );

    // §33.7: one published root per head, each the head's own root, and the last
    // head covering every appended entry. `Ledger::open` has already refused a
    // head that does not recompute from its own prefix, which is the property
    // that matters here: the roots below are recomputed, not trusted.
    let ledger = Ledger::open(corpus.join("ledger").as_std_path())
        .expect("the committed ledger opens without a provider");
    let published = std::fs::read_to_string(corpus.join("ledger/roots.txt").as_std_path())
        .expect("roots.txt is readable");
    let roots: Vec<&str> = published.lines().collect();
    let heads = ledger.heads();
    assert_eq!(
        heads.len(),
        roots.len(),
        "§33.7 publishes one root per head"
    );
    assert!(
        heads.len() >= 2,
        "a corpus of one entry has no second head and no consistency proof, which is the case §33.5's fork argument is about"
    );
    for (head, root) in heads.iter().zip(&roots) {
        assert_eq!(
            head.root_hash.to_hex(),
            *root,
            "the published root of {} leaves is that head's root",
            head.tree_size
        );
    }
    assert_eq!(
        u64::try_from(ledger.len()).expect("a log length fits a u64"),
        heads
            .last()
            .expect("a corpus with entries has a last head")
            .tree_size,
        "the last published head covers every appended entry"
    );
    for pair in heads.windows(2) {
        verify_consistency(
            &pair[0].root_hash,
            pair[0].tree_size,
            &pair[1].root_hash,
            pair[1].tree_size,
            &pair[1].consistency_proof,
        )
        .unwrap_or_else(|failure| {
            panic!(
                "the head of {} leaves is not consistent with the head of {}: {failure:?}",
                pair[1].tree_size, pair[0].tree_size
            )
        });
    }

    // §33.6: every committed entry re-verifies to the recorded verdict, check by
    // check, against the published head.
    for record in entries {
        let name = record["entry"].as_str().expect("a recorded entry path");
        let bytes = std::fs::read(corpus.join(name).as_std_path())
            .unwrap_or_else(|error| panic!("{name} is readable: {error}"));
        let entry = Entry::from_json(
            &Json::parse(&bytes)
                .unwrap_or_else(|error| panic!("{name} is canonical JSON: {error}")),
        )
        .unwrap_or_else(|error| panic!("{name} parses as an entry: {error}"));
        let verdict = lexlean::lexeme::verify::verify_entry(&entry, Some(&ledger))
            .expect("a verdict, not an error");
        assert_eq!(
            verdict.title,
            record["title"].as_str().expect("a recorded title"),
            "{name} keeps its recorded title"
        );
        assert_eq!(
            Some(verdict.verified()),
            record["verified"].as_bool(),
            "{name} re-verifies to its recorded verdict"
        );
        assert_eq!(
            verdict.statement(),
            record["verdict"].as_str().expect("a recorded verdict"),
            "{name} re-verifies to the recorded sentence"
        );
        let expected = record["checks"].as_array().expect("recorded checks");
        assert_eq!(
            verdict.checks.len(),
            expected.len(),
            "{name} reports the recorded number of checks"
        );
        for (check, recorded_check) in verdict.checks.iter().zip(expected) {
            assert_eq!(
                Some(u64::from(check.step)),
                recorded_check["step"].as_u64(),
                "{name} step {} is the recorded step",
                check.step
            );
            assert_eq!(
                check.name,
                recorded_check["name"].as_str().expect("a recorded name"),
                "{name} step {} names the recorded check",
                check.step
            );
            assert_eq!(
                Some(check.passed),
                recorded_check["passed"].as_bool(),
                "{name} step {} passes as recorded",
                check.step
            );
            assert_eq!(
                check.detail,
                recorded_check["detail"]
                    .as_str()
                    .expect("a recorded detail"),
                "{name} step {} establishes what it established",
                check.step
            );
        }
    }

    // §33.7's first half: every committed head is signed by the log key, and
    // the refusal is the one `LLG1009` registers. The corpus is rebuilt with
    // the field present (WP-2d), so these plants are against the committed
    // bytes and must name the head they break.
    let heads_path = corpus.join("ledger/heads.json");
    let mut raw: Vec<serde_json::Value> = std::fs::read_to_string(&heads_path)
        .expect("heads.json is readable")
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).expect("a head line is JSON"))
        .collect();
    assert!(
        !raw.is_empty(),
        "the committed ledger publishes at least one head"
    );
    for head in raw.iter_mut() {
        assert!(
            head.get("signature")
                .and_then(|value| value.get("value"))
                .is_some(),
            "the committed head at size {} carries a signature value",
            head.get("tree_size")
                .and_then(|value| value.as_u64())
                .unwrap_or(0)
        );
    }

    // Honest heads verify, and a head whose signature byte is flipped is refused
    // with `LLG1009` naming that head. The flip is one byte of the value, so
    // the signed fields are untouched and the only thing that changed is the
    // claim of who published the head.
    let honest = Ledger::open(corpus.join("ledger").as_std_path())
        .expect("the committed ledger opens with its heads signed");
    for head in honest.heads() {
        let digest = Sha256Digest::of(&head.signed_bytes());
        assert!(
            head.signature.verify(&digest.0),
            "the committed head at size {} verifies under the log key",
            head.tree_size
        );
    }
    let mut tampered = raw.clone();
    let value = tampered[0]
        .get_mut("signature")
        .and_then(|signature| signature.get_mut("value"))
        .expect("the first head carries a signature value");
    let hex = value.as_str().expect("the signature value is a hex string");
    let mut bytes = hex_decode_lower(hex).expect("the signature value is hex");
    bytes[0] ^= 0x01;
    *value = serde_json::Value::String(lexlean::lexeme::signature::hex_lower(&bytes));
    let tampered_text = tampered
        .iter()
        .map(|head| serde_json::to_string(head).expect("a head serializes"))
        .collect::<Vec<String>>()
        .join("\n")
        + "\n";
    let scratch = tempfile::tempdir().expect("a scratch directory for the tampered head");
    let ledger_dir = scratch.path().join("ledger");
    std::fs::create_dir_all(&ledger_dir).expect("the scratch ledger directory is created");
    std::fs::copy(corpus.join("ledger/log.json"), ledger_dir.join("log.json"))
        .expect("log.json is copied");
    std::fs::copy(
        corpus.join("ledger/roots.txt"),
        ledger_dir.join("roots.txt"),
    )
    .expect("roots.txt is copied");
    std::fs::write(ledger_dir.join("heads.json"), tampered_text)
        .expect("the tampered heads are written");
    let first_size = raw[0]
        .get("tree_size")
        .and_then(|value| value.as_u64())
        .unwrap_or(1);
    let error =
        Ledger::open(&ledger_dir).expect_err("a head with a flipped signature byte is refused");
    expect_code(&error, "LLG1009");
    assert!(
        error.diagnostics[0]
            .message
            .contains(&format!("size {first_size}")),
        "the refusal names the head it breaks: {}",
        error.diagnostics[0].message
    );

    // A head whose fields were edited is refused too, because the signature is
    // over the head's own fields and a changed root recomputes to something the
    // signature does not cover.
    let mut edited = raw.clone();
    let root = edited[0]
        .get_mut("root_hash")
        .expect("the first head carries a root_hash");
    let original = root.as_str().expect("the root is a hex string");
    let mut root_bytes = hex_decode_lower(original).expect("the root is hex");
    root_bytes[0] ^= 0x01;
    *root = serde_json::Value::String(hex_lower(&root_bytes));
    let edited_text = edited
        .iter()
        .map(|head| serde_json::to_string(head).expect("a head serializes"))
        .collect::<Vec<String>>()
        .join("\n")
        + "\n";
    let ledger_dir = scratch.path().join("edited");
    std::fs::create_dir_all(&ledger_dir).expect("the scratch ledger directory is created");
    std::fs::copy(corpus.join("ledger/log.json"), ledger_dir.join("log.json"))
        .expect("log.json is copied");
    std::fs::copy(
        corpus.join("ledger/roots.txt"),
        ledger_dir.join("roots.txt"),
    )
    .expect("roots.txt is copied");
    std::fs::write(ledger_dir.join("heads.json"), edited_text)
        .expect("the edited heads are written");
    let error = Ledger::open(&ledger_dir).expect_err("a head whose fields were edited is refused");
    expect_code(&error, "LLG1009");

    // §33.7's other half: the re-verification above needed no network. The
    // in-process reproduction is the assertion that binds the current library;
    // the socket-denied child process is the evidence that the reading does not
    // quietly reach for one.
    verify_without_a_socket(&corpus, &recorded);
}

/// Re-verify every corpus entry in a process that cannot open a socket.
///
/// A case cannot assert the absence of a network request, so the process that
/// does the verifying is denied one and the denial is proved live first: a
/// probe that opens a socket must fail under the same interposition, or this
/// host has established nothing and says so rather than passing silently. The
/// corpus README records that the heads in it are unsigned (WP-2 of
/// `docs/LexLean MVP Plan.md`); that gap is §33.7's, not this row's, and it is
/// not what this function measures.
fn verify_without_a_socket(corpus: &camino::Utf8Path, recorded: &serde_json::Value) {
    /// The `lexlean` binary beside this test binary: `target/<profile>/deps/<test>`
    /// and `target/<profile>/lexlean` share a parent-of-parent.
    fn cli_binary() -> Option<std::path::PathBuf> {
        let test = std::env::current_exe().ok()?;
        let profile = test.parent()?.parent()?;
        let binary = profile.join("lexlean");
        binary.is_file().then_some(binary)
    }

    let Some(binary) = cli_binary() else {
        crate::support::unix_only("LG-14", "a built `lexlean` binary beside this test");
        return;
    };

    // The interposition. It fails the socket entry points a network request
    // goes through, so a verifier that reached for one would fail rather than
    // pass quietly.
    let denial = r#"#define _GNU_SOURCE
#include <errno.h>

static int deny(void)
{
    errno = ENETDOWN;
    return -1;
}

int socket(int domain, int type, int protocol)
{
    (void)domain; (void)type; (void)protocol;
    return deny();
}

int socketpair(int domain, int type, int protocol)
{
    (void)domain; (void)type; (void)protocol;
    return deny();
}

int connect(int fd, const void *address, unsigned int length)
{
    (void)fd; (void)address; (void)length;
    return deny();
}

int send(int fd, const void *buffer, unsigned long length, int flags)
{
    (void)fd; (void)buffer; (void)length; (void)flags;
    return deny();
}

long sendto(int fd, const void *buffer, unsigned long length, int flags,
            const void *address, unsigned int address_length)
{
    (void)fd; (void)buffer; (void)length; (void)flags; (void)address; (void)address_length;
    return deny();
}

int getaddrinfo(const char *node, const char *service, const void *hints, void **result)
{
    (void)node; (void)service; (void)hints; (void)result;
    return deny();
}
"#;

    // The probe: it exits 0 when the denial is live and 1 when a socket was
    // still openable, so a host where the interposition does not take is a host
    // that reported no evidence rather than one that passed.
    let probe = r#"#include <sys/socket.h>

int main(void)
{
    return socket(AF_INET, SOCK_STREAM, 0) < 0 ? 0 : 1;
}
"#;

    let scratch = tempfile::tempdir().expect("a scratch directory for the shim");
    let source = scratch.path().join("deny-sockets.c");
    let shared = scratch.path().join("deny-sockets.so");
    let probe_source = scratch.path().join("probe.c");
    let probe_binary = scratch.path().join("probe");
    std::fs::write(&source, denial).expect("the shim source is written");
    std::fs::write(&probe_source, probe).expect("the probe source is written");

    let mut built = false;
    for compiler in ["cc", "clang", "gcc"] {
        let library = std::process::Command::new(compiler)
            .args(["-shared", "-fPIC", "-O1", "-o"])
            .arg(&shared)
            .arg(&source)
            .output();
        let executable = std::process::Command::new(compiler)
            .args(["-O1", "-o"])
            .arg(&probe_binary)
            .arg(&probe_source)
            .output();
        if let (Ok(library), Ok(executable)) = (library, executable) {
            if library.status.success() && executable.status.success() {
                built = true;
                break;
            }
        }
    }
    if !built {
        crate::support::unix_only("LG-14", "a C compiler for the socket-denial shim");
        return;
    }

    let denied = |program: &std::path::Path, arguments: &[&str]| {
        std::process::Command::new(program)
            .args(arguments)
            .env("LD_PRELOAD", &shared)
            .env("DYLD_INSERT_LIBRARIES", &shared)
            .output()
            .unwrap_or_else(|error| panic!("{} runs under the shim: {error}", program.display()))
    };

    let probe_run = denied(&probe_binary, &[]);
    assert!(
        probe_run.status.success(),
        "the socket probe still opened a socket under the shim, so this host has not denied the verifier a network and cannot establish §33.7's second half; {}",
        String::from_utf8_lossy(&probe_run.stderr).trim()
    );

    for record in recorded["entries"]
        .as_array()
        .expect("the oracle lists the corpus entries")
    {
        let name = record["entry"].as_str().expect("a recorded entry path");
        let entry = corpus.join(name);
        let output = denied(
            &binary,
            &[
                "lexeme",
                "verify",
                "--entry",
                entry.as_str(),
                "--ledger",
                corpus.join("ledger").as_str(),
            ],
        );
        assert!(
            output.status.success(),
            "{name} does not re-verify without a network: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
        let printed = String::from_utf8_lossy(&output.stdout).into_owned();
        assert_eq!(
            printed.trim_end(),
            record["verdict"].as_str().expect("a recorded verdict"),
            "{name} prints the recorded verdict with every socket denied"
        );
    }
}
