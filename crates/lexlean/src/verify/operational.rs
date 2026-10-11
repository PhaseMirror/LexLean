//! The operational settings of one verification (SPEC.md §22.11, §22.12):
//! how many proof processes run at once and where performance telemetry goes.
//!
//! Both are read from the environment of the process that calls `verify` and
//! from nowhere else. Neither is a field of the project configuration, of the
//! lock, or of a request: a setting that could be written into a project would
//! be an input of the verification it only means to schedule, and the
//! artifacts it produces would then have to say which width made them. They do
//! not. Nothing here is read by a stage, hashed, or recorded.

use std::cell::RefCell;
use std::path::PathBuf;

use crate::code;
use crate::diagnostic::Diagnostic;

/// The environment variable that selects the verification process width.
pub const WIDTH_VARIABLE: &str = "LEXLEAN_VERIFY_WIDTH";

/// The environment variable that names the telemetry directory.
pub const TELEMETRY_VARIABLE: &str = "LEXLEAN_CI_TELEMETRY";

/// The largest width accepted. A bound exists so that a typo such as `800`
/// is refused instead of starting that many Lean processes; no host this
/// repository targets could hold more than a few at once, and 64 is already
/// far past the largest width anyone could validate on a hosted runner.
pub const MAX_WIDTH: usize = 64;

/// The conservative profile: one proof process at a time. A single Atlas
/// environment approaches the memory of the normative runner, and overlapping
/// two on it once made the hosted runner lose its control-plane heartbeat
/// while swapping, so any larger width is an opt-in for a host whose memory
/// has been measured.
pub const DEFAULT_WIDTH: usize = 1;

/// The operational settings of a verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Operational {
    width: usize,
    telemetry: Option<PathBuf>,
}

impl Default for Operational {
    fn default() -> Self {
        Self {
            width: DEFAULT_WIDTH,
            telemetry: None,
        }
    }
}

thread_local! {
    /// A scoped replacement for the environment, so that a test of another
    /// width does not have to change the environment of the whole process.
    static SCOPED: RefCell<Option<Operational>> = const { RefCell::new(None) };
}

impl Operational {
    /// The settings with an explicit width and telemetry directory.
    ///
    /// # Errors
    ///
    /// `LLV7017` when `width` is outside `1..=64`.
    pub fn new(width: usize, telemetry: Option<PathBuf>) -> Result<Self, Diagnostic> {
        if !(1..=MAX_WIDTH).contains(&width) {
            return Err(invalid(&width.to_string()));
        }
        Ok(Self { width, telemetry })
    }

    /// The number of proof processes that may run at once.
    #[must_use]
    pub fn width(&self) -> usize {
        self.width
    }

    /// The directory that receives telemetry, when there is one.
    #[must_use]
    pub fn telemetry(&self) -> Option<&std::path::Path> {
        self.telemetry.as_deref()
    }

    /// Parse the text of the width variable: unset or empty selects the
    /// default, and anything but a plain decimal integer in `1..=64` is
    /// refused (a sign, a space, a fraction, or a word is a misconfigured
    /// job, not a request for the default).
    ///
    /// # Errors
    ///
    /// `LLV7017` for a malformed or out-of-range value.
    pub fn parse_width(text: Option<&str>) -> Result<usize, Diagnostic> {
        let Some(text) = text.filter(|text| !text.is_empty()) else {
            return Ok(DEFAULT_WIDTH);
        };
        if !text.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(invalid(text));
        }
        match text.parse::<usize>() {
            Ok(width) if (1..=MAX_WIDTH).contains(&width) => Ok(width),
            Ok(_) | Err(_) => Err(invalid(text)),
        }
    }

    /// [`Self::parse_width`] over what the environment holds. A value that is
    /// not Unicode is a malformed value, not an absent one: reading it as
    /// unset would silently verify at the default width in a job that asked
    /// for another.
    ///
    /// # Errors
    ///
    /// `LLV7017` for a malformed, out-of-range, or non-Unicode value.
    pub fn parse_width_os(value: Option<&std::ffi::OsStr>) -> Result<usize, Diagnostic> {
        match value {
            None => Ok(DEFAULT_WIDTH),
            Some(raw) => match raw.to_str() {
                Some(text) => Self::parse_width(Some(text)),
                None => Err(invalid(&raw.to_string_lossy())),
            },
        }
    }

    /// The settings the environment selects, unless a scoped replacement is
    /// in force on this thread.
    ///
    /// # Errors
    ///
    /// `LLV7017` for a malformed width variable.
    pub fn from_environment() -> Result<Self, Diagnostic> {
        if let Some(scoped) = SCOPED.with(|cell| cell.borrow().clone()) {
            return Ok(scoped);
        }
        let width = Self::parse_width_os(std::env::var_os(WIDTH_VARIABLE).as_deref())?;
        let telemetry = std::env::var_os(TELEMETRY_VARIABLE)
            .filter(|value| !value.is_empty())
            .map(PathBuf::from);
        Ok(Self { width, telemetry })
    }

    /// Run `body` with these settings in force on the current thread instead
    /// of what the environment says. For tests of another width or of
    /// telemetry, which would otherwise have to change the environment of a
    /// process that other tests are reading.
    pub fn scoped<T>(self, body: impl FnOnce() -> T) -> T {
        /// Restores the previous replacement even when `body` panics.
        struct Restore(Option<Operational>);
        impl Drop for Restore {
            fn drop(&mut self) {
                let previous = self.0.take();
                SCOPED.with(|cell| *cell.borrow_mut() = previous);
            }
        }
        let previous = SCOPED.with(|cell| cell.borrow_mut().replace(self));
        let _restore = Restore(previous);
        body()
    }
}

fn invalid(text: &str) -> Diagnostic {
    Diagnostic::new(
        code!("LLV7017"),
        format!(
            "{WIDTH_VARIABLE} is `{text}`; the verification process width is a decimal integer from 1 to {MAX_WIDTH} (unset or empty selects {DEFAULT_WIDTH})"
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::{Operational, DEFAULT_WIDTH, MAX_WIDTH};

    #[test]
    fn the_width_is_a_plain_decimal_integer_in_range() {
        assert_eq!(Operational::parse_width(None), Ok(DEFAULT_WIDTH));
        assert_eq!(Operational::parse_width(Some("")), Ok(DEFAULT_WIDTH));
        assert_eq!(Operational::parse_width(Some("1")), Ok(1));
        assert_eq!(Operational::parse_width(Some("07")), Ok(7));
        assert_eq!(
            Operational::parse_width(Some(&MAX_WIDTH.to_string())),
            Ok(MAX_WIDTH)
        );
        for bad in [
            "0",
            "00",
            "65",
            "+2",
            "-1",
            " 2",
            "2 ",
            "2.0",
            "two",
            "0x2",
            "99999999999999999999999999",
        ] {
            let error = Operational::parse_width(Some(bad)).expect_err(bad);
            assert_eq!(error.code.as_str(), "LLV7017", "{bad}");
        }
    }

    #[test]
    fn a_value_that_is_not_unicode_is_malformed_not_absent() {
        #[cfg(unix)]
        let raw = {
            use std::os::unix::ffi::OsStringExt;
            std::ffi::OsString::from_vec(vec![b'2', 0xff])
        };
        #[cfg(windows)]
        let raw = {
            use std::os::windows::ffi::OsStringExt;
            std::ffi::OsString::from_wide(&[0x0032, 0xD800])
        };
        #[cfg(any(unix, windows))]
        {
            assert!(raw.to_str().is_none());
            let error = Operational::parse_width_os(Some(&raw)).expect_err("not Unicode");
            assert_eq!(error.code.as_str(), "LLV7017");
        }
        assert_eq!(Operational::parse_width_os(None), Ok(DEFAULT_WIDTH));
        assert_eq!(
            Operational::parse_width_os(Some(std::ffi::OsStr::new("3"))),
            Ok(3)
        );
    }

    /// The environment itself, not only the parser: a non-Unicode value of the
    /// variable is refused by [`Operational::from_environment`]. The variable
    /// is read by nothing else in this test binary, so setting it here races
    /// with no other test.
    #[cfg(unix)]
    #[test]
    fn the_environment_is_read_as_bytes() {
        use std::os::unix::ffi::OsStringExt;
        std::env::set_var(
            super::WIDTH_VARIABLE,
            std::ffi::OsString::from_vec(vec![b'3', 0xfe]),
        );
        let refused = Operational::from_environment();
        std::env::set_var(super::WIDTH_VARIABLE, "3");
        let accepted = Operational::from_environment();
        std::env::remove_var(super::WIDTH_VARIABLE);
        assert_eq!(refused.expect_err("not Unicode").code.as_str(), "LLV7017");
        assert_eq!(accepted.expect("3").width(), 3);
    }

    #[test]
    fn a_scoped_replacement_wins_and_is_restored() {
        let inner = Operational::new(3, None).expect("width 3");
        let seen = inner.clone().scoped(|| {
            Operational::from_environment().expect("a scoped value never reads the environment")
        });
        assert_eq!(seen, inner);
        let after = std::panic::catch_unwind(|| {
            Operational::new(2, None)
                .expect("width 2")
                .scoped(|| panic!("a failing body"))
        });
        assert!(after.is_err());
        let restored = super::SCOPED.with(|cell| cell.borrow().clone());
        assert_eq!(restored, None, "a panicking body leaves nothing in force");
    }

    #[test]
    fn a_width_outside_the_range_is_refused() {
        assert!(Operational::new(0, None).is_err());
        assert!(Operational::new(MAX_WIDTH + 1, None).is_err());
        assert_eq!(Operational::new(2, None).map(|op| op.width()), Ok(2));
    }
}
