//! Tier 2: Boundary & Corner Cases (>=5 tests per feature).
//!
//! Covers:
//! - Feature 1: Version boundaries (future version 1.3, historical 0.9, empty, malformed, non-numeric, extreme size).
//! - Feature 2: Lockfile boundaries (corrupted spec, missing lockfile, invalid sha256, tampered hashes, missing fields).

use super::harness::TestProject;

// ============================================================================
// Feature 1: Language Version Boundaries (>=5 test cases)
// ============================================================================

/// Tier 2, Feature 1, Test 1: Future unsupported version `language = "1.3"` fails closed with LLC0103.
#[test]
fn test_tier2_unsupported_future_version_fails_closed() {
    let project = TestProject::from_example("semantic-1.1");
    project.edit("lexlean.toml", "language = \"1.1\"", "language = \"1.3\"");

    project.load_engine_fails_with("LLC0103");

    let output = project.cli(&["check"]);
    output.assert_failure();
    assert!(output.stderr.contains("LLC0103"));
}

/// Tier 2, Feature 1, Test 2: Historical unsupported version `language = "0.9"` fails closed with LLC0103.
#[test]
fn test_tier2_unsupported_historical_version_fails_closed() {
    let project = TestProject::from_example("semantic-1.1");
    project.edit("lexlean.toml", "language = \"1.1\"", "language = \"0.9\"");

    project.load_engine_fails_with("LLC0103");
}

/// Tier 2, Feature 1, Test 3: Empty language version string `language = ""` fails closed with LLC0103.
#[test]
fn test_tier2_empty_version_string_fails_closed() {
    let project = TestProject::from_example("semantic-1.1");
    project.edit("lexlean.toml", "language = \"1.1\"", "language = \"\"");

    project.load_engine_fails_with("LLC0103");
}

/// Tier 2, Feature 1, Test 4: Three-component semver `language = "1.2.0"` fails closed with LLC0103.
#[test]
fn test_tier2_malformed_semver_fails_closed() {
    let project = TestProject::from_example("semantic-1.1");
    project.edit("lexlean.toml", "language = \"1.1\"", "language = \"1.2.0\"");

    project.load_engine_fails_with("LLC0103");
}

/// Tier 2, Feature 1, Test 5: Non-numeric language version string fails closed with LLC0103.
#[test]
fn test_tier2_non_numeric_version_fails_closed() {
    let project = TestProject::from_example("semantic-1.1");
    project.edit(
        "lexlean.toml",
        "language = \"1.1\"",
        "language = \"beta-v0.4\"",
    );

    project.load_engine_fails_with("LLC0103");
}

/// Tier 2, Feature 1, Test 6: Extreme 8KB language version string fails closed without crash or buffer overflow.
#[test]
fn test_tier2_extreme_version_string_fails_closed() {
    let project = TestProject::from_example("semantic-1.1");
    let extreme_version = "1.".to_owned() + &"9".repeat(8192);
    project.edit(
        "lexlean.toml",
        "language = \"1.1\"",
        &format!("language = \"{extreme_version}\""),
    );

    project.load_engine_fails_with("LLC0103");
}

// ============================================================================
// Feature 2: Lockfile & Migration Boundaries (>=5 test cases)
// ============================================================================

/// Tier 2, Feature 2, Test 7: Corrupted lockfile spec `spec = "lexlean/lock/999"` fails closed with LLC0103.
#[test]
fn test_tier2_unsupported_lock_spec_fails_closed() {
    let project = TestProject::from_example("semantic-1.1");
    project.edit(
        "lexlean.lock",
        "spec = \"lexlean/lock/1\"",
        "spec = \"lexlean/lock/999\"",
    );

    let output = project.cli(&["check"]);
    output.assert_failure();
    assert!(
        output.stderr.contains("LLC0103"),
        "Corrupted lock spec must fail with LLC0103, got:\n{}",
        output.stderr
    );
}

/// Tier 2, Feature 2, Test 8: Missing lockfile when running `lock --check` fails closed.
#[test]
fn test_tier2_missing_lockfile_on_check_fails_closed() {
    let project = TestProject::from_example("semantic-1.1");
    let lock_path = project.root.join("lexlean.lock");
    std::fs::remove_file(lock_path.as_std_path()).expect("remove lockfile");

    let output = project.cli(&["lock", "--check"]);
    output.assert_failure();
}

/// Tier 2, Feature 2, Test 9: Malformed SHA-256 digest in lockfile fails closed.
#[test]
fn test_tier2_corrupt_sha256_in_lock_fails_closed() {
    let project = TestProject::from_example("semantic-1.1");
    // Replace a 64-char valid hex with non-hex characters
    project.edit(
        "lexlean.lock",
        "project_config_sha256 = \"",
        "project_config_sha256 = \"NOT_HEX_ZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZ",
    );

    let output = project.cli(&["lock", "--check"]);
    output.assert_failure();
}

/// Tier 2, Feature 2, Test 10: Tampered package tree hash fails closed with LLC0102 or LLR3001.
#[test]
fn test_tier2_tampered_package_tree_hash_fails_closed() {
    let project = TestProject::from_example("semantic-1.1");
    let lock_content = project.read("lexlean.lock");
    // Flip first char of tree_sha256
    let tampered = lock_content.replacen("tree_sha256 = \"c", "tree_sha256 = \"d", 1);
    project.write("lexlean.lock", &tampered);

    let output = project.cli(&["lock", "--check"]);
    output.assert_failure();
    assert!(
        output.stderr.contains("LLC0102")
            || output.stderr.contains("LLR3001")
            || output.stderr.contains("digest mismatch"),
        "Tampered package tree hash must be rejected: {}",
        output.stderr
    );
}

/// Tier 2, Feature 2, Test 11: Missing required field `project_config_sha256` in lockfile fails closed.
#[test]
fn test_tier2_missing_required_lock_field_fails_closed() {
    let project = TestProject::from_example("semantic-1.1");
    let lock_content = project.read("lexlean.lock");
    // Strip project_config_sha256 line
    let lines: Vec<&str> = lock_content
        .lines()
        .filter(|l| !l.starts_with("project_config_sha256"))
        .collect();
    project.write("lexlean.lock", &lines.join("\n"));

    let output = project.cli(&["check"]);
    output.assert_failure();
}
