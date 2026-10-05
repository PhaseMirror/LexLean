//! Prime-indexed stratum identity (SPEC.md §34.1).
//!
//! A lexeme's declarations are the atoms of a *stratum*: an ordered set indexed
//! `0..n-1` in the §33.1 canonical order, which is ascending UTF-8 byte order of
//! fully qualified name and is therefore already total and deterministic. The
//! stratum is addressed by a **prime index** --- the least prime not below the
//! atom count --- and its content digest is the **snapaddr**.
//!
//! # What this is grounded in, and what it does not claim
//!
//! The stratum-and-atom framing comes from the `sovereign-pirtm` compiler
//! core, where an ensemble declares a prime index and its atoms are tagged into
//! a `!pirtm.stratum`, an atom compiling to
//! `pirtm.operator_atom n {receipt = "atom"}`. That much was observed directly.
//!
//! Three larger claims were not, and §34.0 records why. The compiler does not
//! decompose a term into prime-irreducible components, so **no decomposition is
//! claimed here**: the prime index is a stratum selector derived from the atom
//! count, nothing more. The compiler does not enforce that its `prime=` header
//! is actually prime --- `prime=4` is accepted as `VALID` --- so the primality
//! check below is *LexLean's own and strictly stronger* than the compiler's.
//! And `PrimeShift`, `Successor`, and `StratumBoundary` all lower to undeclared
//! `func.call` passthroughs, so there is no evolution semantics to couple to and
//! this module specifies a static identity only.
//!
//! The honest summary: a snapaddr is a second, independently domain-separated
//! identity for a lexeme that additionally binds its prime index and atom
//! count. It is not a semantic identity, and it inherits §33.1's stated limit
//! --- two sources that elaborate alike but tokenize differently receive
//! different addresses.

use crate::artifact::content_id::Sha256Digest;

use super::canonical::{CanonicalSource, FrameWriter};
use super::CANONICALIZATION;

/// The hash domain of the snapaddr (SPEC.md §34.1).
pub const SNAPADDR_DOMAIN: &str = "lexlean-pirtm-v1";

/// The prefix a snapaddr is written with in human-readable output (SPEC.md
/// §34.1). The digest itself is bare lowercase hex in every field that carries
/// it, exactly as §33.2 requires of `content_digest`.
pub const SNAPADDR_PREFIX: &str = "snapaddr:";

/// The largest prime this module will search for.
///
/// The search is bounded rather than open-ended because §25.5 requires every
/// loop in a parser or a derivation to be bounded by an explicit limit, and an
/// unbounded "next prime" search is exactly the shape that turns a hostile
/// input into a hang. The bound is far above any stratum a lexeme can have: the
/// atom count is bounded by the §10 source limits, so a prime beyond this is
/// unreachable and its absence is reported rather than looped on.
const MAX_PRIME: u64 = 1 << 32;

/// Whether `candidate` is prime, by trial division over the primes up to its
/// square root.
///
/// Trial division is the right algorithm at this size: a lexeme has thousands
/// of atoms, not millions, so the search for the next prime is a few hundred
/// divisions, and a sieve over the whole bound would cost far more than it
/// saves. Divisors are tried as `2` and then the odd numbers, so the work is
/// half what the naive loop would do.
#[must_use]
pub fn is_prime(candidate: u64) -> bool {
    if candidate < 2 {
        return false;
    }
    if candidate.is_multiple_of(2) {
        return candidate == 2;
    }
    let mut divisor = 3u64;
    while divisor.saturating_mul(divisor) <= candidate {
        if candidate.is_multiple_of(divisor) {
            return false;
        }
        divisor += 2;
    }
    true
}

/// The least prime not below `count`, which is §34.1's prime index.
///
/// # Errors
/// Returns the reason when `count` is zero, because a lexeme with no
/// declarations is not a stratum (§34.1), and when no prime is found below
/// [`MAX_PRIME`], which §34.1's atom bounds make unreachable but which is
/// reported rather than assumed.
pub fn prime_index(count: usize) -> Result<u64, String> {
    if count == 0 {
        return Err("a lexeme with no declarations is not a stratum".to_owned());
    }
    let count = u64::try_from(count).map_err(|_| "the atom count exceeds u64".to_owned())?;
    let mut candidate = count.max(2);
    if candidate == 2 {
        return Ok(2);
    }
    // Past 2 every prime is odd, so the search may step by two --- but only
    // after the candidate is raised to an odd number. Skipping to the next odd
    // unconditionally would step over `3` itself, and `prime_index(3)` would
    // report no prime at all.
    if candidate.is_multiple_of(2) {
        candidate += 1;
    }
    while candidate <= MAX_PRIME {
        if is_prime(candidate) {
            return Ok(candidate);
        }
        candidate += 2;
    }
    Err(format!(
        "no prime lies in [{count}, {MAX_PRIME}], which §34.1's atom bounds make unreachable"
    ))
}

/// A lexeme's prime-indexed stratum: its atoms in canonical order, its prime
/// index, and the snapaddr that addresses it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stratum {
    /// The prime index: the least prime not below [`Self::atom_count`].
    pub prime_index: u64,
    /// The atoms, in the §33.1 canonical order.
    pub atoms: Vec<super::canonical::Declaration>,
}

impl Stratum {
    /// Build the stratum of a canonicalized source.
    ///
    /// # Errors
    /// Returns the reason when the source has no declarations, which §34.1
    /// refuses rather than addressing as an empty stratum.
    pub fn of(source: &CanonicalSource) -> Result<Self, String> {
        Ok(Self {
            prime_index: prime_index(source.declarations.len())?,
            atoms: source.declarations.clone(),
        })
    }

    /// The number of atoms in the stratum.
    #[must_use]
    pub fn atom_count(&self) -> usize {
        self.atoms.len()
    }

    /// The canonical bytes of §34.1: the §21.1 frame encoding under
    /// [`SNAPADDR_DOMAIN`] over the canonicalization identifier, the prime
    /// index, the atom count, and each atom's name and body.
    ///
    /// This is a byte string rather than a streamed digest because §34.1 calls
    /// for a form a third party can reproduce: the browser verifier of §33.8
    /// recomputes exactly these bytes, and a layer nobody can recompute is not
    /// a layer a regulator can check.
    #[must_use]
    pub fn canonical_form(&self) -> Vec<u8> {
        let mut frames = FrameWriter::new(SNAPADDR_DOMAIN);
        frames.frame("canonicalization", CANONICALIZATION.as_bytes());
        frames.frame("prime-index", self.prime_index.to_string().as_bytes());
        frames.frame("atom-count", self.atom_count().to_string().as_bytes());
        for atom in &self.atoms {
            frames.frame("atom-name", atom.name.as_bytes());
            frames.frame(
                "atom-body",
                super::canonical::render_tokens(&atom.body).as_bytes(),
            );
        }
        frames.finish()
    }

    /// The snapaddr digest of [`Self::canonical_form`].
    ///
    /// The returned [`SnapAddr`] borrows nothing and carries the digest alone;
    /// the `snapaddr:` prefix is presentation and belongs in output, not in a
    /// field that §33.2 hashes.
    #[must_use]
    pub fn snapaddr(&self) -> SnapAddr {
        SnapAddr(Sha256Digest::of(&self.canonical_form()))
    }
}

/// A lexeme's prime-indexed content address.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SnapAddr(Sha256Digest);

impl SnapAddr {
    /// The bare lowercase hex, as an entry field carries it.
    #[must_use]
    pub fn to_hex(self) -> String {
        self.0.to_hex()
    }

    /// The `snapaddr:<64hex>` form of SPEC.md §34.1, for human-readable
    /// output.
    #[must_use]
    pub fn to_display(self) -> String {
        format!("{SNAPADDR_PREFIX}{}", self.0.to_hex())
    }

    /// Read a bare hex snapaddr back from an entry field.
    ///
    /// # Errors
    /// Returns the reason when `text` is not 64 lowercase hex digits. A
    /// snapaddr is compared for equality, so an uppercase spelling that parsed
    /// would make two byte-different entries compare equal; refusing it keeps
    /// the field a canonical encoding, as §34.1 requires of the receipt's
    /// rational pair for the same reason.
    pub fn from_hex(text: &str) -> Result<Self, String> {
        let digest = Sha256Digest::from_hex(text)?;
        Ok(Self(digest))
    }
}

impl std::fmt::Display for SnapAddr {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.to_display())
    }
}

#[cfg(test)]
mod tests {
    use super::{is_prime, prime_index, Stratum};
    use crate::lexeme::canonical::canonicalize;

    fn stratum_of(source: &str) -> Stratum {
        let canonical = canonicalize(source).expect("the source canonicalizes");
        Stratum::of(&canonical).expect("the source is a stratum")
    }

    #[test]
    fn primality_is_decided_correctly_across_the_small_range() {
        let primes: Vec<u64> = (0..64).filter(|n| is_prime(*n)).collect();
        assert_eq!(
            primes,
            vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61]
        );
        assert!(!is_prime(0));
        assert!(!is_prime(1));
        assert!(!is_prime(4));
        assert!(is_prime(97));
        assert!(!is_prime(561), "561 is Carmichael and must not pass");
    }

    #[test]
    fn the_prime_index_is_the_least_prime_not_below_the_count() {
        // One atom takes 2, not 1: §34.1 requires the index to be prime, and 1
        // is not, so the least prime not below 1 is 2.
        assert_eq!(prime_index(1).expect("valid"), 2);
        assert_eq!(prime_index(2).expect("valid"), 2);
        assert_eq!(prime_index(3).expect("valid"), 3);
        assert_eq!(prime_index(4).expect("valid"), 5);
        assert_eq!(prime_index(5).expect("valid"), 5);
        assert_eq!(prime_index(6).expect("valid"), 7);
        assert_eq!(prime_index(8).expect("valid"), 11);
        assert_eq!(prime_index(100).expect("valid"), 101);
        for count in 1..200usize {
            let index = prime_index(count).expect("valid");
            assert!(is_prime(index), "{index} is not prime");
            assert!(index >= u64::try_from(count).expect("in range"));
        }
        // The bound is reached, not merely asserted positive: a count whose
        // least prime is 2^32 or more is reported rather than searched for.
        assert!(prime_index(1_000_000).is_ok());
    }

    #[test]
    fn an_empty_lexeme_is_not_a_stratum() {
        assert!(prime_index(0).is_err());
    }

    #[test]
    fn comments_layout_and_order_do_not_change_the_snapaddr() {
        let plain = "def a : Nat := 1\ndef b : Nat := 2\n";
        let noisy = "\
/- a block comment -/
def a : Nat := 1   -- trailing
-- a line comment

def   b : Nat := 2
";
        // Declaration order swapped: §33.1 sorts by fully qualified name, so
        // the stratum is the same set of atoms in the same order.
        let swapped = "def b : Nat := 2\ndef a : Nat := 1\n";
        let base = stratum_of(plain).snapaddr();
        assert_eq!(stratum_of(noisy).snapaddr(), base);
        assert_eq!(stratum_of(swapped).snapaddr(), base);
    }

    #[test]
    fn changing_an_atom_body_changes_the_snapaddr() {
        let base = stratum_of("def a : Nat := 1\n").snapaddr();
        assert_ne!(stratum_of("def a : Nat := 2\n").snapaddr(), base);
        // Adding an atom changes the atom count as well as the body list, so
        // both frames move.
        assert_ne!(
            stratum_of("def a : Nat := 1\ndef b : Nat := 2\n").snapaddr(),
            base
        );
    }

    #[test]
    fn the_snapaddr_round_trips_through_hex_and_is_display_prefixed() {
        let stratum = stratum_of("def a : Nat := 1\n");
        let snapaddr = stratum.snapaddr();
        let hex = snapaddr.to_hex();
        assert_eq!(hex.len(), 64);
        assert!(hex
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)));
        assert_eq!(
            super::SnapAddr::from_hex(&hex).expect("valid"),
            snapaddr,
            "a snapaddr must survive the hex round trip"
        );
        assert_eq!(snapaddr.to_display(), format!("snapaddr:{hex}"));
    }

    #[test]
    fn a_malformed_snapaddr_is_refused() {
        assert!(super::SnapAddr::from_hex("").is_err(), "empty");
        assert!(super::SnapAddr::from_hex(&"a".repeat(63)).is_err(), "short");
        assert!(
            super::SnapAddr::from_hex(&"A".repeat(64)).is_err(),
            "uppercase must not parse, so two spellings cannot compare equal"
        );
        assert!(
            super::SnapAddr::from_hex(&"z".repeat(64)).is_err(),
            "not hex"
        );
    }

    #[test]
    fn the_base_layer_digest_is_independent_of_the_snapaddr() {
        // LP-12's premise. The entry carries both identities and each is
        // recomputed against its own frames, so the two domains must differ:
        // if they did not, a snapaddr would be a content digest wearing
        // another name and a verifier could not tell which check it had passed.
        assert_ne!(crate::lexeme::CANONICAL_DOMAIN, super::SNAPADDR_DOMAIN);

        let source = "def a : Nat := 1\ndef b : Nat := 2\n";
        let canonical = canonicalize(source).expect("valid");
        let stratum = Stratum::of(&canonical).expect("a stratum");
        assert_ne!(
            stratum.snapaddr().to_hex(),
            canonical
                .content_digest("leanprover/lean4:v4.32.1")
                .to_hex(),
            "the two identities must not coincide for one source"
        );
    }
}
