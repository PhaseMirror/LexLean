//! CI evidence (SPEC.md §9.4, §22.12): what the performance telemetry may
//! never be part of, and the summary of what it is.
//!
//! A measurement is evidence about one run on one host. A verified artifact is
//! evidence about a project. The first may be read beside the second, never
//! inside it, so the check here is by name: any JSON key of the closed
//! telemetry vocabulary in a verified directory is a measurement that leaked.

use std::collections::BTreeSet;
use std::path::Path;

use lexlean::verify::telemetry::VOCABULARY;

/// Every JSON key of `value`, at any depth.
fn collect_keys(value: &serde_json::Value, into: &mut BTreeSet<String>) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, inner) in map {
                into.insert(key.clone());
                collect_keys(inner, into);
            }
        }
        serde_json::Value::Array(rows) => {
            for row in rows {
                collect_keys(row, into);
            }
        }
        _ => {}
    }
}

/// The telemetry that a verified directory contains, as `path: key` rows:
/// a JSON key of the vocabulary in any `.json` file under it, or a file whose
/// name says it is telemetry.
///
/// # Errors
///
/// A file under `verified` that cannot be read, or a `.json` file that is not
/// JSON (a verified artifact that does not parse is a defect of its own, and
/// one this check cannot clear).
pub fn telemetry_in(verified: &Path) -> Result<Vec<String>, String> {
    let mut found = Vec::new();
    for entry in walkdir::WalkDir::new(verified) {
        let entry = entry.map_err(|error| format!("{}: {error}", verified.display()))?;
        if !entry.file_type().is_file() {
            continue;
        }
        let relative = entry
            .path()
            .strip_prefix(verified)
            .unwrap_or(entry.path())
            .to_string_lossy()
            .replace('\\', "/");
        if relative.to_ascii_lowercase().contains("telemetry") {
            found.push(format!("{relative}: a file named as telemetry"));
        }
        if !relative.ends_with(".json") {
            continue;
        }
        let bytes = std::fs::read(entry.path()).map_err(|error| format!("{relative}: {error}"))?;
        let value: serde_json::Value = serde_json::from_slice(&bytes)
            .map_err(|error| format!("{relative}: not JSON: {error}"))?;
        let mut keys = BTreeSet::new();
        collect_keys(&value, &mut keys);
        for key in VOCABULARY {
            if keys.contains(key) {
                found.push(format!("{relative}: the telemetry key `{key}`"));
            }
        }
    }
    Ok(found)
}

#[cfg(test)]
mod tests {
    use super::telemetry_in;

    #[test]
    fn a_planted_telemetry_key_is_found_wherever_it_is_nested() {
        let directory = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(directory.path().join("process")).expect("mkdir");
        std::fs::write(
            directory.path().join("attestation.json"),
            br#"{"status":"verified","width":3,"processes":[{"tool":"lean","exit_code":0}]}"#,
        )
        .expect("write");
        assert_eq!(
            telemetry_in(directory.path()).expect("scans"),
            Vec::<String>::new(),
            "`width` and `exit_code` are artifact vocabulary, not telemetry"
        );
        std::fs::write(
            directory.path().join("process/lean.json"),
            br#"{"tool":"lean","argv":[],"extra":[{"nested":{"wall_ms":12}}]}"#,
        )
        .expect("write");
        let found = telemetry_in(directory.path()).expect("scans");
        assert_eq!(found, ["process/lean.json: the telemetry key `wall_ms`"]);
        std::fs::write(directory.path().join("verify-telemetry.jsonl"), b"").expect("write");
        assert_eq!(telemetry_in(directory.path()).expect("scans").len(), 2);
    }

    #[test]
    fn an_artifact_that_does_not_parse_is_an_error_not_a_pass() {
        let directory = tempfile::tempdir().expect("tempdir");
        std::fs::write(directory.path().join("broken.json"), b"{").expect("write");
        assert!(telemetry_in(directory.path()).is_err());
    }
}

/// The line `just vv` prints when every recipe has passed; the end of the last
/// recipe's span.
pub const PASSED_LINE: &str = "echo \"vv: the acceptance gate passed\"";

/// The wall time of one recipe of `just vv`, read from the gate log.
#[derive(Debug, Clone, PartialEq)]
pub struct RecipeSpan {
    /// The recipe.
    pub recipe: String,
    /// Seconds since the epoch at which its command started.
    pub start: f64,
    /// Seconds since the epoch at which the next recipe (or the end) started.
    pub end: f64,
}

impl RecipeSpan {
    /// The seconds the recipe took.
    #[must_use]
    pub fn seconds(&self) -> f64 {
        (self.end - self.start).max(0.0)
    }
}

/// The first command line of each recipe the `vv` recipe depends on, in order.
fn vv_commands(justfile: &str) -> Vec<(String, String)> {
    let mut dependencies: Vec<String> = Vec::new();
    let mut commands: std::collections::BTreeMap<String, String> =
        std::collections::BTreeMap::new();
    let mut current: Option<String> = None;
    for line in justfile.lines() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with(' ') || line.starts_with('\t') {
            if let Some(name) = &current {
                commands
                    .entry(name.clone())
                    .or_insert_with(|| line.trim().trim_start_matches('@').to_owned());
            }
        } else if let Some((name, rest)) = line.split_once(':') {
            let name = name.trim().to_owned();
            if name == "vv" {
                dependencies = rest.split_whitespace().map(str::to_owned).collect();
            }
            current = Some(name);
        }
    }
    dependencies
        .into_iter()
        .filter_map(|name| commands.get(&name).map(|command| (name, command.clone())))
        .collect()
}

/// The wall time of each recipe of `vv`, from a log written by
/// `just --timestamp --timestamp-format '%s%.3f' vv`: `just` prints
/// `[<seconds since the epoch>] <command>` before each command it runs, so a
/// recipe lasts from its command's line to the next recipe's.
///
/// A recipe whose command never appears (the run failed before it) has no
/// span. The last recipe ends at [`PASSED_LINE`], or at the last stamped line
/// of a log that never reached it.
#[must_use]
pub fn recipe_spans(justfile: &str, log: &str) -> Vec<RecipeSpan> {
    let commands = vv_commands(justfile);
    let mut stamped: Vec<(f64, &str)> = Vec::new();
    for line in log.lines() {
        let Some(rest) = line.strip_prefix('[') else {
            continue;
        };
        let Some((stamp, command)) = rest.split_once("] ") else {
            continue;
        };
        if let Ok(seconds) = stamp.parse::<f64>() {
            stamped.push((seconds, command));
        }
    }
    let last = stamped.last().map_or(0.0, |(seconds, _)| *seconds);
    let end_of_gate = stamped
        .iter()
        .find(|(_, command)| *command == PASSED_LINE)
        .map_or(last, |(seconds, _)| *seconds);
    let mut starts: Vec<(String, f64)> = Vec::new();
    for (recipe, command) in &commands {
        if let Some((seconds, _)) = stamped.iter().find(|(_, line)| line == command) {
            starts.push((recipe.clone(), *seconds));
        }
    }
    starts.sort_by(|a, b| a.1.total_cmp(&b.1));
    let mut spans = Vec::new();
    for (index, (recipe, start)) in starts.iter().enumerate() {
        let end = starts.get(index + 1).map_or(end_of_gate, |next| next.1);
        spans.push(RecipeSpan {
            recipe: recipe.clone(),
            start: *start,
            end,
        });
    }
    spans
}

/// One host sample: `<epoch seconds> <MemAvailable KiB> <swap used KiB>
/// <largest resident set KiB>` separated by tabs.
#[derive(Debug, Clone, PartialEq)]
pub struct Sample {
    /// Seconds since the epoch.
    pub epoch: f64,
    /// `MemAvailable`, in KiB.
    pub available_kib: u64,
    /// Swap in use, in KiB.
    pub swap_used_kib: u64,
    /// The resident set of the largest process, in KiB.
    pub top_rss_kib: u64,
}

/// Parse `resources.tsv`; rows that are not four numbers are dropped, because
/// a sampler killed mid-line leaves a torn last row and the rest is still
/// evidence.
#[must_use]
pub fn parse_samples(text: &str) -> Vec<Sample> {
    text.lines()
        .filter_map(|line| {
            let mut fields = line.split('\t');
            Some(Sample {
                epoch: fields.next()?.trim().parse().ok()?,
                available_kib: fields.next()?.trim().parse().ok()?,
                swap_used_kib: fields.next()?.trim().parse().ok()?,
                top_rss_kib: fields.next()?.trim().parse().ok()?,
            })
        })
        .collect()
}

fn mib(kib: u64) -> u64 {
    kib / 1024
}

/// The Markdown summary of one gate run: the wall time of each recipe with the
/// host's memory extremes while it ran, the verifier's stages, and its slowest
/// processes. Everything in it is a measurement of this run; none of it is an
/// input of any recipe (SPEC.md §9.4).
#[must_use]
pub fn render_summary(
    spans: &[RecipeSpan],
    samples: &[Sample],
    verifier: &[serde_json::Value],
    examples: &[serde_json::Value],
) -> String {
    use std::fmt::Write as _;
    let mut out = String::new();
    out.push_str("## Wall time per `just vv` recipe\n\n| Recipe | Seconds | Minutes | Lowest free memory (MiB) | Highest swap in use (MiB) | Largest process (MiB) |\n| --- | ---: | ---: | ---: | ---: | ---: |\n");
    let mut total = 0.0;
    for span in spans {
        let within: Vec<&Sample> = samples
            .iter()
            .filter(|sample| sample.epoch >= span.start && sample.epoch < span.end)
            .collect();
        let low = within.iter().map(|s| s.available_kib).min();
        let swap = within.iter().map(|s| s.swap_used_kib).max();
        let top = within.iter().map(|s| s.top_rss_kib).max();
        let cell =
            |value: Option<u64>| value.map_or_else(|| "n/a".to_owned(), |v| mib(v).to_string());
        total += span.seconds();
        let _ = writeln!(
            out,
            "| `{}` | {:.0} | {:.1} | {} | {} | {} |",
            span.recipe,
            span.seconds(),
            span.seconds() / 60.0,
            cell(low),
            cell(swap),
            cell(top)
        );
    }
    let _ = writeln!(out, "| total | {total:.0} | {:.1} | | | |", total / 60.0);

    let ms = |event: &serde_json::Value| event["wall_ms"].as_u64().unwrap_or(0);
    let mut stages: std::collections::BTreeMap<(String, String), (u64, u64)> =
        std::collections::BTreeMap::new();
    let mut projects: std::collections::BTreeMap<String, (u64, u64)> =
        std::collections::BTreeMap::new();
    for event in verifier {
        let project = event["project"].as_str().unwrap_or("?").to_owned();
        match event["event"].as_str() {
            Some("stage") => {
                let stage = event["stage"].as_str().unwrap_or("?").to_owned();
                let slot = stages.entry((project, stage)).or_insert((0, 0));
                slot.0 += 1;
                slot.1 += ms(event);
            }
            Some("verify_end") => {
                let slot = projects.entry(project).or_insert((0, 0));
                slot.0 += 1;
                slot.1 += ms(event);
            }
            _ => {}
        }
    }
    if !projects.is_empty() {
        out.push_str(
            "\n## Verifications\n\n| Project | Verifications | Seconds |\n| --- | ---: | ---: |\n",
        );
        let mut rows: Vec<_> = projects.iter().collect();
        rows.sort_by_key(|row| std::cmp::Reverse(row.1 .1));
        for (project, (count, wall)) in rows {
            let _ = writeln!(
                out,
                "| {project} | {count} | {:.1} |",
                *wall as f64 / 1000.0
            );
        }
        out.push_str(
            "\n## Verification stages (all verifications)\n\n| Stage | Seconds |\n| --- | ---: |\n",
        );
        let mut by_stage: std::collections::BTreeMap<&str, u64> = std::collections::BTreeMap::new();
        for ((_, stage), (_, wall)) in &stages {
            *by_stage.entry(stage).or_insert(0) += wall;
        }
        let mut rows: Vec<_> = by_stage.into_iter().collect();
        rows.sort_by_key(|row| std::cmp::Reverse(row.1));
        for (stage, wall) in rows {
            let _ = writeln!(out, "| {stage} | {:.1} |", wall as f64 / 1000.0);
        }
        let mut processes: Vec<&serde_json::Value> = verifier
            .iter()
            .filter(|event| event["event"] == "process")
            .collect();
        processes.sort_by_key(|event| std::cmp::Reverse(ms(event)));
        out.push_str("\n## Slowest verifier processes\n\n| Project | Stage | Tool | Module or audit member | Seconds | Peak RSS (MiB) | Peak swap (MiB) |\n| --- | --- | --- | --- | ---: | ---: | ---: |\n");
        for event in processes.into_iter().take(15) {
            let kib = |key: &str| {
                event[key]
                    .as_u64()
                    .map_or_else(|| "n/a".to_owned(), |v| mib(v).to_string())
            };
            let _ = writeln!(
                out,
                "| {} | {} | {} | {} | {:.1} | {} | {} |",
                event["project"].as_str().unwrap_or("?"),
                event["stage"].as_str().unwrap_or("?"),
                event["tool"].as_str().unwrap_or("?"),
                event["module"].as_str().unwrap_or(""),
                ms(event) as f64 / 1000.0,
                kib("peak_rss_kib"),
                kib("peak_swap_kib")
            );
        }
    }
    if !examples.is_empty() {
        out.push_str(
            "\n## Examples\n\n| Example | Seconds | Verifier width |\n| --- | ---: | ---: |\n",
        );
        for event in examples {
            let _ = writeln!(
                out,
                "| {} | {:.1} | {} |",
                event["example"].as_str().unwrap_or("?"),
                ms(event) as f64 / 1000.0,
                event["process_width"].as_u64().unwrap_or(0)
            );
        }
    }
    out
}

#[cfg(test)]
mod summary_tests {
    use super::{parse_samples, recipe_spans, render_summary};

    const JUSTFILE: &str = "\
vv: a b c
    @echo \"vv: the acceptance gate passed\"

a:
    cargo a

b:
    cargo b --flag

c:
    cargo c
";

    #[test]
    fn a_recipe_lasts_from_its_command_to_the_next_recipes() {
        let log = "\
[100.000] cargo a
noise
[130.500] cargo b --flag
[190.000] cargo c
[200.250] echo \"vv: the acceptance gate passed\"
vv: the acceptance gate passed
";
        let spans = recipe_spans(JUSTFILE, log);
        let seconds: Vec<(&str, f64)> = spans
            .iter()
            .map(|span| (span.recipe.as_str(), span.seconds()))
            .collect();
        assert_eq!(seconds, [("a", 30.5), ("b", 59.5), ("c", 10.25)]);
    }

    #[test]
    fn a_log_cut_short_ends_at_its_last_stamp_and_misses_the_recipes_not_reached() {
        let spans = recipe_spans(
            JUSTFILE,
            "[10.0] cargo a\n[25.0] cargo b --flag\n[40.0] other\n",
        );
        assert_eq!(spans.len(), 2);
        assert_eq!(spans[1].seconds(), 15.0);
    }

    #[test]
    fn the_summary_carries_the_memory_extremes_of_each_window() {
        let log = "[100.0] cargo a\n[160.0] cargo b --flag\n[220.0] echo \"vv: the acceptance gate passed\"\n";
        let samples = parse_samples(
            "100.5\t8000000\t0\t2000000\n130\t3000000\t1024000\t4000000\nbad row\n170\t7000000\t0\t1000000\n",
        );
        assert_eq!(samples.len(), 3);
        let summary = render_summary(&recipe_spans(JUSTFILE, log), &samples, &[], &[]);
        assert!(
            summary.contains("| `a` | 60 | 1.0 | 2929 | 1000 | 3906 |"),
            "{summary}"
        );
        assert!(
            summary.contains("| `b` | 60 | 1.0 | 6835 | 0 | 976 |"),
            "{summary}"
        );
    }
}
