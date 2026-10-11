//! The execution side of the gate (SPEC.md §9.2, §9.4, §22.12): the test
//! inventory, the digest of a verified directory, and the evidence summary.
//!
//! Nothing here changes what `just vv` accepts. These tasks check how the gate
//! is run, or report it; the reports go to a directory of CI evidence and are
//! never read back by a recipe.

use std::collections::BTreeSet;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use repo_conformance::{evidence, inventory};
use serde_json::Value;

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
    let reference: Vec<String> = inventory::REFERENCE
        .iter()
        .map(|s| (*s).to_owned())
        .collect();
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
    let directory =
        PathBuf::from(option(arguments, "--evidence").unwrap_or_else(|| "ci-evidence".to_owned()));
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
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(step)
        {
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
