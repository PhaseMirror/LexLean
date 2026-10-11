//! The decisions behind the operational width of the verifier (SPEC.md §9.4,
//! §22.11): whether a CI job may use a width, and which measured width a
//! measurement run supports.
//!
//! Both are pure functions of numbers a run has already produced, so the rules
//! can be tested without a large runner and without Lean. What cannot be tested
//! without them — how much memory a width needs and how much time it saves — is
//! measured by `cargo xtask measure-width`, whose output these functions judge.

use lexlean::verify::operational::Operational;

/// The share of a host's wall-time reduction that width above 1 must reach on
/// the example it is measured on, in percent (issue #48).
pub const REQUIRED_REDUCTION_PERCENT: u64 = 25;

/// The default share of the host's memory the verifier's processes may peak at.
pub const DEFAULT_MAX_RSS_PERCENT: u64 = 85;

/// The default swap the verifier's processes may use at their peak: none of
/// the swap a runner has is a budget. The normative job adds a small swap file
/// as overflow memory, and a width that needs it is a width the host cannot
/// hold, so a megabyte of slack is the whole allowance.
pub const DEFAULT_MAX_SWAP_KIB: u64 = 1024;

/// The decision of `cargo xtask check-vv-profile`.
///
/// `width_text` is the text of `LEXLEAN_VERIFY_WIDTH`; `min_memory_mb` the text
/// of the repository variable `LEXLEAN_VV_MIN_MEMORY_MB` (the memory, in MiB, of
/// the runner profile the width was measured on); `mem_total_kib` the host's
/// `MemTotal`.
///
/// # Errors
///
/// A malformed width, or a width above 1 without a declared envelope, with an
/// unreadable host memory, or on a host smaller than the envelope.
pub fn check_profile(
    width_text: Option<&str>,
    min_memory_mb: Option<&str>,
    mem_total_kib: Option<u64>,
) -> Result<String, String> {
    let width = Operational::parse_width(width_text).map_err(|error| error.message)?;
    if width == 1 {
        return Ok("width 1, the conservative profile; no memory envelope is required".to_owned());
    }
    let declared = min_memory_mb
        .filter(|text| !text.is_empty())
        .ok_or_else(|| {
            format!(
                "RP-14: width {width} is above 1 but LEXLEAN_VV_MIN_MEMORY_MB is not set; a width above 1 is accepted only on the memory envelope it was measured on"
            )
        })?;
    let envelope_mb: u64 = declared
        .parse()
        .ok()
        .filter(|mb| *mb > 0)
        .ok_or_else(|| {
            format!("RP-14: LEXLEAN_VV_MIN_MEMORY_MB is `{declared}`, not a positive integer of MiB")
        })?;
    let total_kib = mem_total_kib.ok_or_else(|| {
        format!(
            "RP-14: width {width} needs a host whose memory can be read; this host's cannot"
        )
    })?;
    let total_mb = total_kib / 1024;
    if total_mb < envelope_mb {
        return Err(format!(
            "RP-14: width {width} was measured on a runner of at least {envelope_mb} MiB and this one has {total_mb} MiB"
        ));
    }
    Ok(format!(
        "width {width} on {total_mb} MiB, at or above the declared envelope of {envelope_mb} MiB"
    ))
}

/// What one run of the example at one width measured.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Measurement {
    /// The width.
    pub width: usize,
    /// Wall time of the verification, in milliseconds.
    pub wall_ms: u64,
    /// Peak resident set of the verifier's process tree, in KiB.
    pub peak_rss_kib: Option<u64>,
    /// Peak swap of the verifier's process tree, in KiB.
    pub peak_swap_kib: Option<u64>,
    /// The lowest `MemAvailable` of the host during the run, in KiB.
    pub min_available_kib: Option<u64>,
    /// The attestation ID the run produced; `None` when it did not verify.
    pub attestation_id: Option<String>,
    /// The digest of the published verified directory; `None` when none.
    pub verified_digest: Option<String>,
    /// Why the run was stopped before it finished, when it was.
    pub aborted: Option<String>,
}

/// The envelope a candidate width must stay inside.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Envelope {
    /// The host's `MemTotal`, in KiB.
    pub mem_total_kib: u64,
    /// The share of it the process tree may peak at, in percent.
    pub max_rss_percent: u64,
    /// The swap the process tree may peak at, in KiB.
    pub max_swap_kib: u64,
}

/// Why a candidate was or was not accepted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    /// The baseline, width 1.
    Baseline,
    /// Identical evidence, inside the envelope.
    Valid,
    /// Not accepted, with the reason.
    Rejected(String),
}

/// The verdict of a measurement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verdict {
    /// The status of each measured width, in order.
    pub statuses: Vec<(usize, Status)>,
    /// Whether every width that verified produced the evidence of width 1.
    pub identical: bool,
    /// The width the measurement supports: the fastest accepted width above 1.
    pub selected: Option<usize>,
    /// The wall-time reduction of the selected width against width 1, in
    /// tenths of a percent; `None` when the selected width is not faster.
    pub reduction_tenths: Option<u64>,
    /// Whether the reduction reaches [`REQUIRED_REDUCTION_PERCENT`].
    pub meets_criterion: bool,
    /// What the reader needs to know that the fields above do not say.
    pub notes: Vec<String>,
}

/// Judge the measurements of one example at ascending widths.
///
/// A width is accepted only when it verified, produced the attestation ID and
/// verified-directory digest of width 1, and kept its processes inside the
/// envelope. The selected width is the accepted one with the least wall time
/// (the lower width on a tie): a wider width that is slower is not an
/// optimization. The criterion is a fact about this run on this runner, and
/// the verdict says `meets_criterion` only when the numbers do.
#[must_use]
pub fn decide(measurements: &[Measurement], envelope: &Envelope) -> Verdict {
    let mut notes = Vec::new();
    let mut statuses = Vec::new();
    let baseline = measurements.iter().find(|m| m.width == 1);
    let Some(baseline) = baseline else {
        notes.push("width 1 was not measured; there is no baseline".to_owned());
        return Verdict {
            statuses,
            identical: false,
            selected: None,
            reduction_tenths: None,
            meets_criterion: false,
            notes,
        };
    };
    let baseline_ok = baseline.aborted.is_none()
        && baseline.attestation_id.is_some()
        && baseline.verified_digest.is_some();
    if !baseline_ok {
        notes.push("width 1 did not verify; there is no baseline".to_owned());
    }
    statuses.push((1, Status::Baseline));
    let mut identical = baseline_ok;
    let mut best: Option<&Measurement> = None;
    for measurement in measurements.iter().filter(|m| m.width != 1) {
        let status = if !baseline_ok {
            Status::Rejected("there is no baseline to compare with".to_owned())
        } else if let Some(reason) = &measurement.aborted {
            Status::Rejected(format!("stopped: {reason}"))
        } else if measurement.attestation_id.is_none() || measurement.verified_digest.is_none() {
            Status::Rejected("it did not verify".to_owned())
        } else if measurement.attestation_id != baseline.attestation_id
            || measurement.verified_digest != baseline.verified_digest
        {
            identical = false;
            Status::Rejected("its verified evidence differs from width 1's".to_owned())
        } else if let Some(reason) = outside(measurement, envelope) {
            Status::Rejected(reason)
        } else {
            Status::Valid
        };
        if status == Status::Valid
            && best.is_none_or(|current| measurement.wall_ms < current.wall_ms)
        {
            best = Some(measurement);
        }
        statuses.push((measurement.width, status));
    }
    let Some(selected) = best else {
        notes.push(
            "no width above 1 is accepted; the issue's wall-time criterion is open".to_owned(),
        );
        return Verdict {
            statuses,
            identical,
            selected: None,
            reduction_tenths: None,
            meets_criterion: false,
            notes,
        };
    };
    let reduction_tenths = baseline
        .wall_ms
        .checked_sub(selected.wall_ms)
        .filter(|_| baseline.wall_ms > 0)
        .map(|saved| saved.saturating_mul(1000) / baseline.wall_ms);
    let meets_criterion = selected
        .wall_ms
        .saturating_mul(100)
        <= baseline
            .wall_ms
            .saturating_mul(100 - REQUIRED_REDUCTION_PERCENT);
    if !meets_criterion {
        notes.push(format!(
            "the selected width {} reduces the wall time by less than {REQUIRED_REDUCTION_PERCENT} percent; the criterion is not met",
            selected.width
        ));
    }
    Verdict {
        statuses,
        identical,
        selected: Some(selected.width),
        reduction_tenths,
        meets_criterion,
        notes,
    }
}

/// Why a measurement leaves the envelope, when it does.
fn outside(measurement: &Measurement, envelope: &Envelope) -> Option<String> {
    let limit_kib = envelope
        .mem_total_kib
        .saturating_mul(envelope.max_rss_percent)
        / 100;
    match measurement.peak_rss_kib {
        None => return Some("its peak memory was not measured".to_owned()),
        Some(peak) if peak > limit_kib => {
            return Some(format!(
                "its peak resident set {peak} KiB exceeds {} percent of the host's memory ({limit_kib} KiB)",
                envelope.max_rss_percent
            ));
        }
        Some(_) => {}
    }
    match measurement.peak_swap_kib {
        None => Some("its peak swap was not measured".to_owned()),
        Some(swap) if swap > envelope.max_swap_kib => Some(format!(
            "its peak swap {swap} KiB exceeds {} KiB",
            envelope.max_swap_kib
        )),
        Some(_) => None,
    }
}

/// May the next width be tried after these measurements? A candidate is started
/// only if every earlier one was accepted, so a width that failed, differed, or
/// left the envelope ends the ascent.
#[must_use]
pub fn may_try_next(measurements: &[Measurement], envelope: &Envelope) -> bool {
    let verdict = decide(measurements, envelope);
    !verdict.statuses.is_empty()
        && verdict
            .statuses
            .iter()
            .all(|(_, status)| !matches!(status, Status::Rejected(_)))
}

#[cfg(test)]
mod tests {
    use super::{
        check_profile, decide, may_try_next, Envelope, Measurement, Status, DEFAULT_MAX_RSS_PERCENT,
        DEFAULT_MAX_SWAP_KIB,
    };

    fn run(width: usize, wall_ms: u64, rss: u64, id: &str) -> Measurement {
        Measurement {
            width,
            wall_ms,
            peak_rss_kib: Some(rss),
            peak_swap_kib: Some(0),
            min_available_kib: Some(1_000_000),
            attestation_id: Some(id.to_owned()),
            verified_digest: Some(format!("digest-{id}")),
            aborted: None,
        }
    }

    fn envelope() -> Envelope {
        Envelope {
            mem_total_kib: 32 * 1024 * 1024,
            max_rss_percent: DEFAULT_MAX_RSS_PERCENT,
            max_swap_kib: DEFAULT_MAX_SWAP_KIB,
        }
    }

    #[test]
    fn the_conservative_profile_needs_nothing_and_a_wider_one_needs_an_envelope() {
        assert!(check_profile(None, None, None).is_ok());
        assert!(check_profile(Some(""), None, None).is_ok());
        assert!(check_profile(Some("1"), None, None).is_ok());
        let missing = check_profile(Some("2"), None, Some(64 * 1024 * 1024)).expect_err("no envelope");
        assert!(missing.contains("LEXLEAN_VV_MIN_MEMORY_MB"), "{missing}");
        let unreadable = check_profile(Some("2"), Some("32000"), None).expect_err("no host memory");
        assert!(unreadable.contains("memory can be read"), "{unreadable}");
        let small = check_profile(Some("2"), Some("32000"), Some(16 * 1024 * 1024))
            .expect_err("a smaller runner");
        assert!(small.contains("16384 MiB"), "{small}");
        assert!(check_profile(Some("2"), Some("32000"), Some(32 * 1024 * 1024)).is_ok());
        for bad in ["0", "65", "two", "-1"] {
            assert!(check_profile(Some(bad), Some("1"), Some(1 << 30)).is_err(), "{bad}");
        }
        assert!(check_profile(Some("2"), Some("zero"), Some(1 << 30)).is_err());
        assert!(check_profile(Some("2"), Some("0"), Some(1 << 30)).is_err());
    }

    #[test]
    fn a_faster_identical_width_inside_the_envelope_is_selected() {
        let measurements = [
            run(1, 1000, 14_000_000, "a"),
            run(2, 600, 20_000_000, "a"),
            run(3, 500, 26_000_000, "a"),
        ];
        let verdict = decide(&measurements, &envelope());
        assert_eq!(verdict.selected, Some(3));
        assert!(verdict.identical && verdict.meets_criterion);
        assert_eq!(verdict.reduction_tenths, Some(500));
        assert_eq!(verdict.statuses[0], (1, Status::Baseline));
    }

    #[test]
    fn a_width_whose_evidence_differs_is_never_selected() {
        let measurements = [run(1, 1000, 14_000_000, "a"), run(2, 100, 20_000_000, "b")];
        let verdict = decide(&measurements, &envelope());
        assert_eq!(verdict.selected, None);
        assert!(!verdict.identical && !verdict.meets_criterion);
        assert!(matches!(&verdict.statuses[1].1, Status::Rejected(why) if why.contains("differs")));
        assert!(!may_try_next(&measurements, &envelope()));
    }

    #[test]
    fn a_width_outside_the_memory_envelope_is_never_selected() {
        let over = [run(1, 1000, 14_000_000, "a"), run(2, 100, 30_000_000, "a")];
        assert_eq!(decide(&over, &envelope()).selected, None);
        let mut swapping = run(2, 100, 20_000_000, "a");
        swapping.peak_swap_kib = Some(DEFAULT_MAX_SWAP_KIB + 1);
        assert_eq!(
            decide(&[run(1, 1000, 14_000_000, "a"), swapping], &envelope()).selected,
            None
        );
        let mut unmeasured = run(2, 100, 20_000_000, "a");
        unmeasured.peak_rss_kib = None;
        assert_eq!(
            decide(&[run(1, 1000, 14_000_000, "a"), unmeasured], &envelope()).selected,
            None
        );
    }

    #[test]
    fn a_reduction_under_the_criterion_is_reported_and_not_met() {
        let measurements = [run(1, 1000, 14_000_000, "a"), run(2, 800, 20_000_000, "a")];
        let verdict = decide(&measurements, &envelope());
        assert_eq!(verdict.selected, Some(2));
        assert_eq!(verdict.reduction_tenths, Some(200));
        assert!(!verdict.meets_criterion, "20 percent is not 25");
        let exact = decide(
            &[run(1, 1000, 14_000_000, "a"), run(2, 750, 20_000_000, "a")],
            &envelope(),
        );
        assert!(exact.meets_criterion, "exactly 25 percent meets it");
    }

    #[test]
    fn a_slower_wider_width_is_not_an_optimization() {
        let measurements = [run(1, 1000, 14_000_000, "a"), run(2, 1200, 20_000_000, "a")];
        let verdict = decide(&measurements, &envelope());
        assert_eq!(verdict.selected, Some(2));
        assert!(!verdict.meets_criterion);
        assert_eq!(verdict.reduction_tenths, None);
    }

    #[test]
    fn without_a_baseline_or_after_an_aborted_run_nothing_is_decided() {
        assert_eq!(decide(&[run(2, 1, 1, "a")], &envelope()).selected, None);
        let mut stopped = run(2, 100, 1, "a");
        stopped.aborted = Some("free memory fell below the floor".to_owned());
        let measurements = [run(1, 1000, 14_000_000, "a"), stopped];
        assert_eq!(decide(&measurements, &envelope()).selected, None);
        assert!(!may_try_next(&measurements, &envelope()));
        assert!(may_try_next(&measurements[..1], &envelope()));
    }
}
