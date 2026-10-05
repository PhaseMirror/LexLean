//! The Zeno-Finton signal (SPEC.md §34.3).
//!
//! The Zeno-Finton gain decays toward zero without reaching it. This module
//! holds it as an exact rational,
//!
//! ```text
//! κ(index) = 2^(-index)
//! ```
//!
//! with `κ(0) = 1`, and it is **supplementary and carries no legal weight**.
//! That last sentence is normative and this module is built to make it hard to
//! forget: there is no constructor here that can produce a timestamp, and the
//! only thing a [`ZenoFinton`] can be compared against is another
//! [`ZenoFinton`].
//!
//! # Why this is not a timestamp
//!
//! §33.4 makes an external RFC 3161 timestamp the only time anchor, for a
//! stated reason: a timestamp is meaningful because it comes from an authority
//! the inventor does not control. `κ` is a function of the log's own length ---
//! the inventor appends a leaf, `κ` halves --- so it is exactly the internal
//! signal §33.4 declines to trust. Anyone able to extend the log controls the
//! value, and a value whose adversary controls it carries no ordering claim.
//!
//! The practical consequence for the rest of LexLean is that `κ` never
//! satisfies a §33.4 check and never appears in a §33.6 verdict. This is why
//! [`ZenoFinton`] deliberately exposes no conversion into any timestamp type.

use crate::artifact::canonical_json::Json;
use crate::diagnostic::Diagnostic;
use crate::error::LexLeanError;

use super::rational::Rational;

/// The JSON namespace of the signal, which is a namespace and deliberately not
/// an *authority*. §33.4 reserves authority for the timestamping body a lexeme
/// does not control; naming an internal field `authority` would suggest this
/// signal had one.
pub const DOMAIN: &str = "lexlean-zeno-finton-v1";

/// The largest leaf index whose gain is representable.
///
/// `κ(index)` has denominator `2^index`, and the exact rational type holds an
/// `i128`. `2^126` is the largest power of two an `i128` denominator holds
/// exactly, so index `126` is the last one this module can evaluate and `127`
/// would round away precision rather than compute a different value. The bound
/// is stated rather than left to overflow because §25.5 requires an explicit
/// limit on every unbounded step and an index-driven exponent is the shape that
/// silently becomes a float.
pub const MAX_LEAF_INDEX: u32 = 126;

/// The Zeno-Finton gain at one leaf index: an exact rational `2^(-index)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ZenoFinton {
    leaf_index: u32,
    kappa: Rational,
}

impl ZenoFinton {
    /// The gain at leaf `index`.
    ///
    /// # Errors
    /// Returns the reason when `index` exceeds [`MAX_LEAF_INDEX`], where
    /// `2^index` is no longer exact in the rational type. This is reported
    /// rather than approximated: a κ that had been rounded would no longer
    /// satisfy §34.3's claim that the signal is exactly recomputable, and
    /// rounding is exactly the failure this section exists to exclude.
    pub fn at(index: u32) -> Result<Self, String> {
        if index > MAX_LEAF_INDEX {
            return Err(format!(
                "leaf index {index} exceeds the exact range {MAX_LEAF_INDEX}; \
                 2^{index} is not representable as an exact rational"
            ));
        }
        Ok(Self {
            leaf_index: index,
            // (1/2)^index, i.e. 2^(-index). `pow_u128` takes a non-negative
            // exponent, so the halving lives in the base: raising 1/2 to the
            // index is 2^(-index). Raising the reciprocal of one instead would
            // silently yield 1 at every index.
            kappa: Rational::new(1, 2)?.pow_u128(index)?,
        })
    }

    /// The leaf index this gain was evaluated at.
    #[must_use]
    pub fn leaf_index(self) -> u32 {
        self.leaf_index
    }

    /// §34.3's prohibition, as a function with a caller.
    ///
    /// The gain MAY NOT stand in for the external RFC 3161 token that §33.4
    /// requires, because §33.4 admits a time anchor only when it comes from an
    /// authority the inventor does not control and this signal is a function of
    /// the log's own length --- the inventor appends a leaf and the value halves.
    /// This returns [`LLG1015`](crate::code) unconditionally.
    ///
    /// It exists as code rather than as a paragraph because a prohibition that
    /// has no failure to raise is a prohibition nothing can catch. Anything that
    /// reaches for the signal where a timestamp is required gets a refusal
    /// instead of a silent substitution, which is the failure §34.3 forbids.
    #[must_use]
    pub fn refusal_as_time_anchor(&self) -> LexLeanError {
        LexLeanError::from_diagnostic(Diagnostic::new(
            crate::code!("LLG1015"),
            format!(
                "the Zeno-Finton gain at leaf index {} is supplementary and cannot be a \
                 §33.4 time anchor; an external RFC 3161 token is the only time anchor",
                self.leaf_index
            ),
        ))
    }

    /// The gain as an exact rational, `2^(-index)`.
    #[must_use]
    pub fn kappa(self) -> Rational {
        self.kappa
    }

    /// Whether this gain is strictly smaller than the one at `other`.
    ///
    /// This is §34.3's convergence claim, expressed as the one thing that can
    /// be checked about it: κ is strictly decreasing.
    #[must_use]
    pub fn is_below(self, other: Self) -> bool {
        self.kappa < other.kappa
    }

    /// Read a signal back from an entry field.
    ///
    /// # Errors
    /// Returns the reason when a field is absent, of the wrong type, when the
    /// recorded gain is not the exact `2^(-index)` for the recorded index, or
    /// when the object claims to be anything other than supplementary. The
    /// check against the index is what makes the field verifiable: a caller
    /// cannot record a gain the index does not imply.
    pub fn from_json(value: &Json) -> Result<Self, String> {
        let Json::Obj(fields) = value else {
            return Err("the `zeno-finton` signal is not a JSON object".to_owned());
        };
        let Some(Json::Str(domain)) = fields.get("domain") else {
            return Err("the `zeno-finton.domain` field is missing".to_owned());
        };
        if domain != DOMAIN {
            return Err(format!(
                "the `zeno-finton.domain` field is `{domain}`, not `{DOMAIN}`"
            ));
        }
        let Some(Json::Str(kind)) = fields.get("kind") else {
            return Err("the `zeno-finton.kind` field is missing".to_owned());
        };
        if kind != "supplementary" {
            return Err(format!(
                "the `zeno-finton.kind` field is `{kind}`; §34.3 admits only `supplementary`"
            ));
        }
        let leaf_index = match fields.get("leaf_index") {
            Some(Json::Int(number)) => u32::try_from(*number)
                .map_err(|_| "the `zeno-finton.leaf_index` field is negative".to_owned())?,
            _ => {
                return Err(
                    "the `zeno-finton.leaf_index` field is missing or not an integer".to_owned(),
                )
            }
        };
        let Some(kappa_field) = fields.get("kappa") else {
            return Err("the `zeno-finton.kappa` field is missing".to_owned());
        };
        let signal = Self::at(leaf_index)?;
        if Rational::from_pair(kappa_field)? != signal.kappa() {
            return Err(format!(
                "the recorded gain is not the exact 2^(-{leaf_index}) the index implies"
            ));
        }
        Ok(signal)
    }

    /// The canonical JSON of the signal, as §34.3 has an entry record it.
    ///
    /// The field is named `kappa` and carries the exact pair. It is never named
    /// `timestamp`, and there is no code path that could put this value into a
    /// timestamp field, because §34.3 forbids it rather than merely
    /// discouraging it.
    #[must_use]
    pub fn to_json(self) -> Json {
        Json::object(vec![
            ("domain", Json::Str(DOMAIN.to_owned())),
            ("kappa", self.kappa.to_json()),
            ("kind", Json::Str("supplementary".to_owned())),
            ("leaf_index", Json::Int(i64::from(self.leaf_index))),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::{ZenoFinton, DOMAIN, MAX_LEAF_INDEX};
    use crate::lexeme::rational::Rational;

    #[test]
    fn the_gain_halves_at_each_step_from_one() {
        assert_eq!(
            ZenoFinton::at(0).expect("valid").kappa().to_pair(),
            (1, 1),
            "κ(0) = 1"
        );
        assert_eq!(ZenoFinton::at(1).expect("valid").kappa().to_pair(), (1, 2));
        assert_eq!(ZenoFinton::at(2).expect("valid").kappa().to_pair(), (1, 4));
        assert_eq!(
            ZenoFinton::at(10).expect("valid").kappa().to_pair(),
            (1, 1024)
        );
        // A power of two is always reduced, so the numerator stays one.
        for index in 0..=MAX_LEAF_INDEX {
            let (numerator, denominator) = ZenoFinton::at(index).expect("valid").kappa().to_pair();
            assert_eq!(numerator, 1, "κ({index}) must be in lowest terms");
            assert!(denominator > 0);
        }
    }

    #[test]
    fn each_step_is_exactly_half_of_the_step_before() {
        let two = Rational::new(2, 1).expect("valid");
        for index in 1..=MAX_LEAF_INDEX {
            let previous = ZenoFinton::at(index - 1).expect("valid").kappa();
            let current = ZenoFinton::at(index).expect("valid").kappa();
            // 2·κ(index) = 2·2^(-index) = 2^(-(index-1)) = κ(index-1).
            assert_eq!(
                current.mul(two).expect("multiply").to_pair(),
                previous.to_pair(),
                "doubling κ({index}) must recover κ({}) exactly",
                index - 1
            );
        }
    }

    #[test]
    fn the_signal_strictly_decreases_and_never_reaches_zero() {
        // §34.3's convergence claim, in the rational form it actually makes:
        // strictly decreasing, and positive at every finite index. It is never
        // zero, so "never reaching zero" is not something this type can even
        // represent, and there is no sentinel for it.
        let mut previous = ZenoFinton::at(0).expect("valid");
        for index in 1..=MAX_LEAF_INDEX {
            let current = ZenoFinton::at(index).expect("valid");
            assert!(
                current.is_below(previous),
                "κ({index}) must be below κ({})",
                index - 1
            );
            assert!(
                current.kappa() > Rational::new(0, 1).expect("valid"),
                "κ({index}) must stay positive"
            );
            previous = current;
        }
    }

    #[test]
    fn an_index_past_the_exact_range_is_refused_rather_than_rounded() {
        // The boundary is named on both sides: 126 is evaluated exactly, and
        // 127 --- `MAX_LEAF_INDEX + 1` --- is the first index refused.
        assert_eq!(MAX_LEAF_INDEX, 126);
        assert_eq!(MAX_LEAF_INDEX + 1, 127);
        assert!(ZenoFinton::at(MAX_LEAF_INDEX).is_ok());
        assert_eq!(
            ZenoFinton::at(126).expect("126 is exact").kappa,
            Rational::new(1, 1i128 << 126).expect("2^126 is representable"),
            "126 is exactly 2^-126, a reduced rational with no rounding"
        );
        // Why 126 and not 127: 2^126 fits in i128, 2^127 does not, so 127 is
        // the first index with no exact rational to return.
        assert_eq!(
            1i128 << 126,
            85_070_591_730_234_615_865_843_651_857_942_052_864
        );
        assert!(
            (1i128 << 126).checked_mul(2).is_none(),
            "2^127 exceeds i128, which is what makes 127 the first refused index"
        );
        let error =
            ZenoFinton::at(MAX_LEAF_INDEX + 1).expect_err("past the exact range must be refused");
        assert!(error.contains("exact range"), "{error}");
    }

    #[test]
    fn offering_the_signal_as_a_time_anchor_is_refused_with_its_registered_code() {
        // §34.3's prohibition has a failure to raise, so a caller that reaches
        // for the signal where §33.4 requires a token is refused instead of
        // silently handed a substitution.
        let error = ZenoFinton::at(4).expect("valid").refusal_as_time_anchor();
        let codes: Vec<&str> = error
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code.as_str())
            .collect();
        assert_eq!(codes, vec!["LLG1015"]);
        assert!(error.diagnostics[0].message.contains("RFC 3161"));
    }

    #[test]
    fn the_signal_is_recorded_as_supplementary_and_never_as_a_timestamp() {
        // §34.3 is normative that κ MUST NOT be reported as a timestamp and
        // MUST NOT satisfy a §33.4 check. The JSON is the only place this value
        // can leave the module, so its field names carry the prohibition.
        let json = ZenoFinton::at(7).expect("valid").to_json();
        let crate::artifact::canonical_json::Json::Obj(fields) = &json else {
            unreachable!("to_json returns an object")
        };
        assert_eq!(
            fields.get("kind"),
            Some(&crate::artifact::canonical_json::Json::Str(
                "supplementary".to_owned()
            ))
        );
        assert!(
            !fields.contains_key("timestamp"),
            "the signal must have no timestamp field to be mistaken for one"
        );
        assert_eq!(
            fields.get("domain"),
            Some(&crate::artifact::canonical_json::Json::Str(
                DOMAIN.to_owned()
            )),
            "the field names a namespace, not an authority"
        );
        assert!(
            !fields.contains_key("authority"),
            "§33.4 reserves authority for an external timestamping body, so an \
             internal signal must not claim one"
        );
    }
}
