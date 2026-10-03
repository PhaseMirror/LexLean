//! The verification resource profile (SPEC.md §22.11).
//!
//! Width is how many independent proof processes the verifier may have in
//! flight at once. It is a property of the host that runs the verification,
//! never a property of the project being verified, and it is deliberately
//! absent from every identity LexLean computes: no semantic ID, build ID,
//! process record, normalized verification record, or attestation ID reads it.
//! That absence is the whole reason raising it cannot weaken evidence —
//! width can change the wall clock and nothing else.
//!
//! The default is width 1. One Atlas environment already approaches the
//! memory available on a GitHub-hosted runner, and two overlapping Atlas
//! processes made a hosted runner lose its control-plane heartbeat while
//! swapping. An operator who has measured a larger safe envelope on a
//! specific runner selects it there. The repository never selects it on a
//! project's behalf, because a project cannot know the memory of the machine
//! that will verify it.
//!
//! What width may never do is change *what runs*. The topological import
//! barriers of §22.3 are unaffected: a module is elaborated only after every
//! module it imports has completed successfully, whatever the width.

use std::env;

/// The environment variable naming the width.
///
/// The name says "operational" because the value is not part of the
/// language: the same project verifies to the same bytes under every width.
pub const WIDTH_VARIABLE: &str = "LEXLEAN_VERIFY_OPERATIONAL_WIDTH";

/// The widest profile the verifier honours.
///
/// A ceiling clamps a mistyped width downward, and downward is the safe
/// direction: fewer concurrent processes can only cost wall time, and §22.11
/// gives width no other power. No width has ever changed what is proved, so
/// declining to grow past the ceiling costs an operator nothing they would
/// notice.
pub const MAX_WIDTH: usize = 64;

/// An operational verification resource profile.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResourceProfile {
    width: usize,
}

impl ResourceProfile {
    /// The conservative profile: one proof process at a time.
    #[must_use]
    pub const fn conservative() -> Self {
        Self { width: 1 }
    }

    /// An explicit width, clamped into `1..=MAX_WIDTH`.
    ///
    /// Spelled out rather than `Ord::clamp`, which is not yet a const trait.
    #[must_use]
    pub const fn with_width(width: usize) -> Self {
        let width = if width < 1 {
            1
        } else if width > MAX_WIDTH {
            MAX_WIDTH
        } else {
            width
        };
        Self { width }
    }

    /// The profile this host asks for, or the conservative profile when it
    /// asks for nothing usable.
    ///
    /// An absent, unparsable, or out-of-range value is deliberately not a
    /// registered failure. It is a mistyped performance knob in a
    /// non-normative operational input, and it cannot have made the run
    /// stronger or weaker than the default: every rejection path here lands
    /// on a width that is at most the conservative one. Failing a complete
    /// verification over a typo in a tuning variable would trade real evidence
    /// for nothing, and §26 has no code to spend on it.
    #[must_use]
    pub fn from_environment() -> Self {
        Self::from_source(|key| env::var(key).ok())
    }

    /// [`ResourceProfile::from_environment`] over an injected lookup, so the
    /// selection is testable without mutating the process environment, which
    /// is shared state a parallel test binary must not touch.
    #[must_use]
    pub fn from_source(lookup: impl Fn(&str) -> Option<String>) -> Self {
        lookup(WIDTH_VARIABLE).map_or_else(Self::conservative, |value| {
            value
                .trim()
                .parse::<usize>()
                .map_or_else(|_| Self::conservative(), Self::with_width)
        })
    }

    /// How many independent proof processes may run at once.
    #[must_use]
    pub const fn width(self) -> usize {
        self.width
    }
}

impl Default for ResourceProfile {
    /// The conservative profile, so that a caller which forgets to select one
    /// gets the measured-safe width rather than an accidental one.
    fn default() -> Self {
        Self::conservative()
    }
}
