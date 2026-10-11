//! The execution side of the gate (SPEC.md §9.2, §9.4, §22.11, §22.12): the
//! test inventory, the CI job's resource profile, the width measurement, and
//! the evidence summary.
//!
//! Nothing here changes what `just vv` accepts. These tasks check how the gate
//! is run, or measure it; the measurements go to a directory of CI evidence and
//! are never read back by a recipe.

use std::collections::BTreeSet;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use lexlean::verify::telemetry::{GroupWatch, HostMemory};
use repo_conformance::{evidence, inventory, width};
use serde_json::{json, Value};

use crate::Fail;

/// The first command of a recipe of the Justfile.
fn recipe_command(justfile: &str, name: &str) -> Result<String, Fail> {
    let mut in_recipe = false;
    for line in justfile.lines() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with(' ') || line.starts_with('\t') {
            if in_recipe {
                return Ok(line.trim().trim_start_matches('@').to_owned());
            }
        } else {
            in_recipe = line
                .split_once(':')
                .is_some_and(|(recipe, _)| recipe.trim() == name);
        }
    }
    Err(format!("RP-13: the Justfile has no recipe `{name}` with a command").into())
}

/// Run `cargo test <arguments> -- --list` and return what it wrote, stdout
/// and stderr in the order they were written: `cargo` announces each target on
/// stderr and the harness lists its tests on stdout, so the two have to share
/// one file to be read together.
fn list_tests(root: &Path, arguments: &[String]) -> Result<inventory::Parsed, Fail> {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let mut file = tempfile::tempfile()?;
    let status = Command::new(cargo)
        .current_dir(root)
        .env("CARGO_TERM_COLOR", "never")
        .arg("test")
        .args(arguments)
        .args(["--", "--list"])
        .stdin(Stdio::null())
        .stdout(Stdio::from(file.try_clone()?))
        .stderr(Stdio::from(file.try_clone()?))
        .status()?;
    file.seek(SeekFrom::Start(0))?;
    let mut output = String::new();
    file.read_to_string(&mut output)?;
    if !status.success() {
        let tail: Vec<&str> = output.lines().rev().take(20).collect();
        return Err(format!(
            "RP-13: `cargo test {} -- --list` failed:\n{}",
            arguments.join(" "),
            tail.into_iter().rev().collect::<Vec<_>>().join("\n")
        )
        .into());
    }
    inventory::parse(&output).map_err(Into::into)
}

/// The names of the binaries that carry the tests of `repo-conformance`:
/// its library and each of its test targets, from `cargo metadata`.
fn owned_crates(root: &Path) -> Result<BTreeSet<String>, Fail> {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let output = Command::new(cargo)
        .current_dir(root)
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .stdin(Stdio::null())
        .stderr(Stdio::inherit())
        .output()?;
    if !output.status.success() {
        return Err("RP-13: `cargo metadata` failed".into());
    }
    let metadata: Value = serde_json::from_slice(&output.stdout)?;
    let mut crates = BTreeSet::new();
    for package in metadata["packages"].as_array().into_iter().flatten() {
        if package["name"] != inventory::OWNED_PACKAGE {
            continue;
        }
        for target in package["targets"].as_array().into_iter().flatten() {
            let kinds: Vec<&str> = target["kind"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .collect();
            let tested = kinds
                .iter()
                .any(|kind| matches!(*kind, "lib" | "rlib" | "bin" | "test"));
            if let (true, Some(name)) = (tested, target["name"].as_str()) {
                crates.insert(name.replace('-', "_"));
            }
        }
    }
    Ok(crates)
}

/// RP-13: `test` and `bdd` partition `cargo test --workspace --all-features`.
pub fn check_test_inventory(root: &Path) -> Result<(), Fail> {
    let justfile = std::fs::read_to_string(root.join("Justfile"))?;
    let ordinary_line = recipe_command(&justfile, "test")?;
    let owned_line = recipe_command(&justfile, "bdd")?;
    let reference: Vec<String> = inventory::REFERENCE.iter().map(|s| (*s).to_owned()).collect();
    let workspace = list_tests(root, &reference)?;
    let ordinary = list_tests(root, &inventory::recipe_arguments(&ordinary_line)?)?;
    let owned = list_tests(root, &inventory::recipe_arguments(&owned_line)?)?;
    let summary = inventory::compare(&workspace, &ordinary, &owned, &owned_crates(root)?)?;
    println!(
        "check-test-inventory: the {} tests of `cargo test {}` are the {} of `test` and the {} of `bdd`, disjoint, none lost (RP-13)",
        summary.workspace,
        inventory::REFERENCE.join(" "),
        summary.ordinary,
        summary.owned
    );
    Ok(())
}

/// RP-14: the CI job's width and runner are consistent before any build.
pub fn check_vv_profile() -> Result<(), Fail> {
    let width_text = std::env::var("LEXLEAN_VERIFY_WIDTH").ok();
    let envelope = std::env::var("LEXLEAN_VV_MIN_MEMORY_MB").ok();
    let memory = HostMemory::now();
    let verdict = width::check_profile(
        width_text.as_deref(),
        envelope.as_deref(),
        memory.map(|memory| memory.total),
    );
    let runner = std::env::var("RUNNER_NAME").unwrap_or_else(|_| "local".to_owned());
    let line = match &verdict {
        Ok(message) => format!("check-vv-profile: {message} (runner {runner})"),
        Err(reason) => format!("check-vv-profile: {reason} (runner {runner})"),
    };
    println!("{line}");
    if let Some(summary) = std::env::var_os("GITHUB_STEP_SUMMARY") {
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(summary)
        {
            writeln!(file, "**Verifier profile.** {line}").ok();
        }
    }
    verdict.map(|_| ()).map_err(Into::into)
}

/// The digest of a directory: the SHA-256 of every relative path and the
/// bytes of every file under it, in path order. Two directories with the same
/// digest are byte for byte the same tree.
pub fn tree_digest(directory: &Path) -> Result<String, Fail> {
    let mut files: Vec<(String, PathBuf)> = Vec::new();
    for entry in walkdir::WalkDir::new(directory) {
        let entry = entry?;
        if entry.file_type().is_file() {
            let relative = entry
                .path()
                .strip_prefix(directory)
                .unwrap_or(entry.path())
                .to_string_lossy()
                .replace('\\', "/");
            files.push((relative, entry.path().to_path_buf()));
        }
    }
    files.sort();
    let mut rows = String::new();
    for (relative, path) in &files {
        let bytes = std::fs::read(path)?;
        rows.push_str(&format!(
            "{relative}\0{}\0{}\n",
            bytes.len(),
            lexlean::Sha256Digest::of(&bytes).to_hex()
        ));
    }
    Ok(lexlean::Sha256Digest::of(rows.as_bytes()).to_hex())
}

fn option(arguments: &[String], name: &str) -> Option<String> {
    arguments
        .iter()
        .position(|argument| argument == name)
        .and_then(|at| arguments.get(at + 1))
        .cloned()
}

fn number(arguments: &[String], name: &str, default: u64) -> Result<u64, Fail> {
    option(arguments, name).map_or(Ok(default), |text| {
        text.parse()
            .map_err(|_| format!("{name} wants a number, got `{text}`").into())
    })
}

/// What a run of `verify-examples --only` printed.
fn after(log: &str, marker: &str) -> Option<String> {
    log.lines()
        .find_map(|line| line.split_once(marker))
        .map(|(_, rest)| {
            rest.split(|c: char| c == ')' || c.is_whitespace())
                .next()
                .unwrap_or("")
                .to_owned()
        })
        .filter(|text| !text.is_empty())
}

/// The wall time of a verification, from the verifier's own telemetry.
fn verify_wall_ms(directory: &Path) -> Option<u64> {
    std::fs::read_to_string(directory.join("verify-telemetry.jsonl"))
        .ok()?
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .filter(|event| event["event"] == "verify_end")
        .filter_map(|event| event["wall_ms"].as_u64())
        .max()
}

/// One row of the measurement document, and back.
fn row(m: &width::Measurement) -> Value {
    json!({
        "process_width": m.width,
        "wall_ms": m.wall_ms,
        "peak_rss_kib": m.peak_rss_kib,
        "peak_swap_kib": m.peak_swap_kib,
        "mem_available_kib": m.min_available_kib,
        "attestation_id": m.attestation_id,
        "verified_digest": m.verified_digest,
        "aborted": m.aborted,
    })
}

fn from_row(value: &Value) -> Option<width::Measurement> {
    Some(width::Measurement {
        width: usize::try_from(value["process_width"].as_u64()?).ok()?,
        wall_ms: value["wall_ms"].as_u64()?,
        peak_rss_kib: value["peak_rss_kib"].as_u64(),
        peak_swap_kib: value["peak_swap_kib"].as_u64(),
        min_available_kib: value["mem_available_kib"].as_u64(),
        attestation_id: value["attestation_id"].as_str().map(str::to_owned),
        verified_digest: value["verified_digest"].as_str().map(str::to_owned),
        aborted: value["aborted"].as_str().map(str::to_owned),
    })
}

/// Run `verify-examples --only <example>` in a child at one width, watching its
/// process tree and the host's memory, and stopping it if free memory falls
/// below the floor.
#[cfg(unix)]
fn measure_one(
    root: &Path,
    example: &str,
    candidate: usize,
    out: &Path,
    floor_kib: u64,
) -> Result<width::Measurement, Fail> {
    use std::os::unix::process::CommandExt;

    let host = HostMemory::now().ok_or("measure-width reads /proc/meminfo; this host has none")?;
    let directory = out.join(format!("width-{candidate}"));
    if directory.exists() {
        std::fs::remove_dir_all(&directory)?;
    }
    std::fs::create_dir_all(&directory)?;
    let log_path = directory.join("verify-examples.log");
    let log = std::fs::File::create(&log_path)?;
    let mut command = Command::new(std::env::current_exe()?);
    command
        .current_dir(root)
        .arg("verify-examples")
        .args(["--only", example])
        .env("LEXLEAN_VERIFY_WIDTH", candidate.to_string())
        .env("LEXLEAN_CI_TELEMETRY", &directory)
        .stdin(Stdio::null())
        .stdout(Stdio::from(log.try_clone()?))
        .stderr(Stdio::from(log))
        .process_group(0);
    println!("measure-width: {example} at width {candidate} ...");
    let started = Instant::now();
    let mut child = command.spawn()?;
    let mut watch = GroupWatch::new(child.id());
    let mut min_available = host.available;
    let mut aborted: Option<String> = None;
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        watch.sample();
        if let Some(memory) = HostMemory::now() {
            min_available = min_available.min(memory.available);
            if aborted.is_none() && memory.available < floor_kib {
                aborted = Some(format!(
                    "free memory fell to {} MiB, below the floor of {} MiB",
                    memory.available / 1024,
                    floor_kib / 1024
                ));
                // The whole group: the child starts `lake`, which starts
                // `lean`, and a killed leader leaves them running.
                Command::new("kill")
                    .args(["-KILL", &format!("-{}", child.id())])
                    .status()
                    .ok();
            }
        }
        std::thread::sleep(Duration::from_millis(500));
    };
    let child_wall = started.elapsed();
    let log_text = std::fs::read_to_string(&log_path).unwrap_or_default();
    if !status.success() && aborted.is_none() {
        aborted = Some(format!("`verify-examples` exited with {status}"));
    }
    let wall_ms = verify_wall_ms(&directory)
        .unwrap_or_else(|| u64::try_from(child_wall.as_millis()).unwrap_or(u64::MAX));
    let measurement = width::Measurement {
        width: candidate,
        wall_ms,
        peak_rss_kib: watch.peak_rss_kib(),
        peak_swap_kib: watch.peak_swap_kib(),
        min_available_kib: Some(min_available),
        attestation_id: after(&log_text, "verified (attestation "),
        verified_digest: after(&log_text, "verified-digest "),
        aborted,
    };
    println!(
        "measure-width: width {candidate}: verification {:.1} s, peak {} MiB, swap {} MiB, lowest free {} MiB, attestation {}",
        wall_ms as f64 / 1000.0,
        measurement.peak_rss_kib.unwrap_or(0) / 1024,
        measurement.peak_swap_kib.unwrap_or(0) / 1024,
        min_available / 1024,
        measurement.attestation_id.as_deref().unwrap_or("none")
    );
    std::fs::write(
        out.join(format!("measurement-{candidate}.json")),
        serde_json::to_vec_pretty(&row(&measurement))?,
    )?;
    Ok(measurement)
}

/// Measure one example at several widths and judge them.
///
/// Three forms: `--widths 1,2,3` measures ascending widths one after another
/// on this runner and starts a wider one only if every narrower one was
/// accepted; `--width N` measures that one width and leaves its row in `--out`
/// (for a matrix of runners of one profile); `--combine DIR...` judges the rows
/// those runs left. See SPEC.md §9.4 for what is measured and what is claimed.
pub fn measure_width(root: &Path, arguments: &[String]) -> Result<(), Fail> {
    let example = option(arguments, "--example").unwrap_or_else(|| "uor-atlas".to_owned());
    let out = PathBuf::from(
        option(arguments, "--out").unwrap_or_else(|| "ci-evidence/width".to_owned()),
    );
    let host_total = HostMemory::now().map(|memory| memory.total);
    let envelope = |total: u64| -> Result<width::Envelope, Fail> {
        Ok(width::Envelope {
            mem_total_kib: total,
            max_rss_percent: number(arguments, "--max-rss-percent", width::DEFAULT_MAX_RSS_PERCENT)?,
            max_swap_kib: number(arguments, "--max-swap-kib", width::DEFAULT_MAX_SWAP_KIB)?,
        })
    };
    let floor_kib = number(arguments, "--floor-available-mb", 1024)? * 1024;
    std::fs::create_dir_all(&out)?;

    if let Some(at) = arguments.iter().position(|argument| argument == "--combine") {
        let mut measurements: Vec<width::Measurement> = Vec::new();
        for directory in &arguments[at + 1..] {
            if directory.starts_with("--") {
                break;
            }
            for entry in walkdir::WalkDir::new(directory).into_iter().flatten() {
                let name = entry.file_name().to_string_lossy();
                if name.starts_with("measurement-") && name.ends_with(".json") {
                    let value: Value = serde_json::from_slice(&std::fs::read(entry.path())?)?;
                    measurements.extend(from_row(&value));
                }
            }
        }
        measurements.sort_by_key(|measurement| measurement.width);
        let total = number(arguments, "--mem-total-kib", host_total.unwrap_or(0))?;
        if total == 0 {
            return Err("--combine needs --mem-total-kib of the measured runner when this host's memory cannot be read".into());
        }
        return finish(&out, &example, &envelope(total)?, floor_kib, &measurements);
    }

    let total = host_total.ok_or("measure-width reads /proc/meminfo; this host has none")?;
    let envelope = envelope(total)?;
    #[cfg(not(unix))]
    {
        let _ = (root, floor_kib, &envelope);
        Err("measure-width needs a unix host (process groups and /proc)".into())
    }
    #[cfg(unix)]
    {
        if let Some(text) = option(arguments, "--width") {
            let candidate: usize = text.parse().map_err(|_| "--width is an integer")?;
            // The row is left for `--combine`; one width alone has nothing to
            // be judged against.
            measure_one(root, &example, candidate, &out, floor_kib)?;
            return Ok(());
        }
        let widths: Vec<usize> = option(arguments, "--widths")
            .unwrap_or_else(|| "1,2,3,4".to_owned())
            .split(',')
            .map(|text| text.trim().parse::<usize>())
            .collect::<Result<_, _>>()
            .map_err(|_| "--widths is a comma-separated list of integers")?;
        if widths.first() != Some(&1) || widths.windows(2).any(|pair| pair[0] >= pair[1]) {
            return Err("--widths starts at 1 and ascends (the baseline comes first, and a wider width is tried only after a narrower one)".into());
        }
        let mut measurements: Vec<width::Measurement> = Vec::new();
        for &candidate in &widths {
            if !measurements.is_empty() && !width::may_try_next(&measurements, &envelope) {
                println!("measure-width: width {candidate} and wider are not tried; an earlier width was not accepted");
                break;
            }
            measurements.push(measure_one(root, &example, candidate, &out, floor_kib)?);
            write_measurement(&out, &example, &envelope, floor_kib, &measurements)?;
        }
        finish(&out, &example, &envelope, floor_kib, &measurements)
    }
}

/// Write the judged evidence and fail if the widths measured disagree.
fn finish(
    out: &Path,
    example: &str,
    envelope: &width::Envelope,
    floor_kib: u64,
    measurements: &[width::Measurement],
) -> Result<(), Fail> {
    write_measurement(out, example, envelope, floor_kib, measurements)?;
    let verdict = width::decide(measurements, envelope);
    println!("{}", render_verdict(example, &verdict));
    if !verdict.identical {
        return Err("measure-width: the verified evidence of two widths differs, or width 1 did not verify (§22.11)".into());
    }
    Ok(())
}

fn render_verdict(example: &str, verdict: &width::Verdict) -> String {
    let mut text = format!("measure-width: {example}:");
    for (width, status) in &verdict.statuses {
        text.push_str(&format!("\n  width {width}: {status:?}"));
    }
    text.push_str(&format!("\n  identical evidence: {}", verdict.identical));
    match (verdict.selected, verdict.reduction_tenths) {
        (Some(selected), Some(tenths)) => text.push_str(&format!(
            "\n  selected width {selected}: {}.{} percent faster than width 1; criterion of {} percent {}",
            tenths / 10,
            tenths % 10,
            width::REQUIRED_REDUCTION_PERCENT,
            if verdict.meets_criterion { "MET" } else { "NOT MET" }
        )),
        (Some(selected), None) => text.push_str(&format!(
            "\n  selected width {selected} is not faster than width 1; criterion NOT MET"
        )),
        (None, _) => text.push_str("\n  no width above 1 accepted; criterion NOT MET"),
    }
    for note in &verdict.notes {
        text.push_str(&format!("\n  note: {note}"));
    }
    text
}

fn write_measurement(
    out: &Path,
    example: &str,
    envelope: &width::Envelope,
    floor_kib: u64,
    measurements: &[width::Measurement],
) -> Result<(), Fail> {
    let verdict = width::decide(measurements, envelope);
    let rows: Vec<Value> = measurements.iter().map(row).collect();
    let statuses: Vec<Value> = verdict
        .statuses
        .iter()
        .map(|(width, status)| json!({ "process_width": width, "status": format!("{status:?}") }))
        .collect();
    let document = json!({
        "example": example,
        "mem_total_kib": envelope.mem_total_kib,
        "max_rss_percent": envelope.max_rss_percent,
        "max_swap_kib": envelope.max_swap_kib,
        "floor_available_kib": floor_kib,
        "required_reduction_percent": width::REQUIRED_REDUCTION_PERCENT,
        "measurements": rows,
        "statuses": statuses,
        "identical": verdict.identical,
        "selected_process_width": verdict.selected,
        "reduction_tenths_of_percent": verdict.reduction_tenths,
        "meets_criterion": verdict.meets_criterion,
        "notes": verdict.notes,
    });
    std::fs::write(
        out.join("width-measurement.json"),
        serde_json::to_vec_pretty(&document)?,
    )?;
    std::fs::write(
        out.join("width-measurement.md"),
        format!(
            "# Verifier width measurement\n\nEvidence of execution speed on one runner; a timing is never a claim about a semantic property.\n\n```text\n{}\n```\n",
            render_verdict(example, &verdict)
        ),
    )?;
    Ok(())
}

/// Read the JSON lines of `file` under `directory`, dropping what is not JSON.
fn json_lines(directory: &Path, file: &str) -> Vec<Value> {
    std::fs::read_to_string(directory.join(file))
        .map(|text| {
            text.lines()
                .filter_map(|line| serde_json::from_str(line).ok())
                .collect()
        })
        .unwrap_or_default()
}

/// Summarize the evidence directory of a gate run into `summary.md`.
pub fn ci_summary(root: &Path, arguments: &[String]) -> Result<(), Fail> {
    let directory = PathBuf::from(option(arguments, "--evidence").unwrap_or_else(|| "ci-evidence".to_owned()));
    let justfile = std::fs::read_to_string(root.join("Justfile"))?;
    let log = std::fs::read_to_string(directory.join("vv.txt")).unwrap_or_default();
    let spans = evidence::recipe_spans(&justfile, &log);
    let samples = evidence::parse_samples(
        &std::fs::read_to_string(directory.join("resources.tsv")).unwrap_or_default(),
    );
    let verifier = json_lines(&directory, lexlean::verify::telemetry::VERIFY_FILE);
    let examples = json_lines(&directory, "examples-telemetry.jsonl");
    let summary = evidence::render_summary(&spans, &samples, &verifier, &examples);
    std::fs::write(directory.join("summary.md"), &summary)?;
    if let Some(step) = std::env::var_os("GITHUB_STEP_SUMMARY") {
        if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(step) {
            writeln!(file, "{summary}").ok();
        }
    }
    println!(
        "ci-summary: {} recipes, {} host samples, {} verifier events, {} examples",
        spans.len(),
        samples.len(),
        verifier.len(),
        examples.len()
    );
    Ok(())
}
