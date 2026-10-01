//! Tier 4: Real-World Application Scenarios.
//!
//! Covers:
//! - Scenario 1: Complete end-to-end project lifecycle (init -> configure -> lock -> check -> build -> fmt -> clean).
//! - Scenario 2: Project migration workflow (1.0 -> 1.1 migration preserving byte-stability, relock, verify).
//! - Scenario 3: Tamper and drift detection in multi-module workspace (fail-closed CI gate simulation).

use super::harness::{CliRunner, TestProject};

/// Tier 4, Scenario 1: Complete end-to-end lifecycle of a new LexLean project.
#[test]
fn test_tier4_full_project_lifecycle() {
    let runner = CliRunner::new();
    let temp = tempfile::tempdir().expect("tempdir");
    let project_dir = temp.path().join("lifecycle-project");

    // Step 1: Initialize new project via CLI
    let init_out = runner.run(&[
        "init",
        project_dir.to_str().unwrap(),
        "--name",
        "lifecycle-project",
        "--module-prefix",
        "LifecycleProject",
        "--language",
        "1.1",
    ]);
    init_out.assert_success();

    // Verify initial structure
    assert!(project_dir.join("lexlean.toml").exists());
    assert!(project_dir.join("src").join("Main.lex.tex").exists());

    // Step 2: Lock project dependencies
    let lock_out = runner.run_in(&project_dir, &["lock"]);
    lock_out.assert_success();
    assert!(project_dir.join("lexlean.lock").exists());

    // Step 3: Check project semantics
    let check_out = runner.run_in(&project_dir, &["check", "--all"]);
    check_out.assert_success();

    // Step 4: Build project artifacts
    let build_out = runner.run_in(&project_dir, &["build", "--all"]);
    build_out.assert_success();

    // Verify build artifacts layout: .lexlean/build/<build_id>/manifest.json
    let build_dir = project_dir.join(".lexlean").join("build");
    assert!(build_dir.exists(), "Build directory must exist after build");
    let entries: Vec<_> = std::fs::read_dir(&build_dir)
        .expect("read build dir")
        .flatten()
        .collect();
    assert!(
        !entries.is_empty(),
        "Build directory must contain at least one build ID directory"
    );

    // Step 5: Format check
    let fmt_out = runner.run_in(&project_dir, &["fmt", "--check", "--all"]);
    fmt_out.assert_success();

    // Step 6: Clean workspace
    let clean_out = runner.run_in(&project_dir, &["clean"]);
    clean_out.assert_success();
    assert!(
        !build_dir.exists(),
        "Build directory must be removed after clean"
    );
}

/// Tier 4, Scenario 2: Project migration workflow from Language 1.0 to Language 1.1.
#[test]
fn test_tier4_project_migration_workflow() {
    let project = TestProject::from_example("nat-add-zero");

    // Step 1: Verify baseline Language 1.0 builds successfully
    let base_check = project.cli(&["check"]);
    base_check.assert_success();

    let base_build = project.cli(&["build"]);
    base_build.assert_success();

    // Step 2: Upgrade configuration to Language 1.1
    project.edit("lexlean.toml", "language = \"1.0\"", "language = \"1.1\"");
    // Upgrade glossary reference to 1.1.0
    project.edit(
        "src/Main.lex.tex",
        "\\useglossary{lexlean.std.nat@1.0.0}",
        "\\useglossary{lexlean.std.nat@1.1.0}",
    );

    // Step 3: Relock with Language 1.1
    let relock_out = project.cli(&["lock"]);
    relock_out.assert_success();

    let updated_lock = project.read("lexlean.lock");
    assert!(updated_lock.contains("language = \"1.1\""));

    // Step 4: Verify upgraded project builds cleanly
    let post_check = project.cli(&["check"]);
    post_check.assert_success();

    let post_build = project.cli(&["build"]);
    post_build.assert_success();
}

/// Tier 4, Scenario 3: Continuous integration tamper and drift detection in multi-module workspace.
#[test]
fn test_tier4_tamper_detection_in_workspace() {
    let project = TestProject::from_example("semantic-1.1");

    // Step 1: Initial state verifies cleanly
    project.cli(&["check"]).assert_success();
    project.cli(&["lock", "--check"]).assert_success();

    // Step 2: Simulate untracked / silent drift by injecting unrecognized macro
    project.edit(
        "src/Main.lex.tex",
        "\\begin{lexlean}{Main}",
        "\\begin{lexlean}{Main}\n\\invalidMacro{tampered}",
    );

    // Step 3: Verify compiler detects syntax drift immediately on check
    let check_fail = project.cli(&["check"]);
    check_fail.assert_failure();

    // Step 4: Verify JSON diagnostic output contains structured error
    let json_check = project.cli(&["--diagnostic-format", "json", "check"]);
    json_check.assert_failure();
    let json = json_check.parse_json();
    let exit_code = json.get("exit_code").and_then(|e| e.as_i64()).unwrap();
    assert_ne!(exit_code, 0);
    let diags = json.get("diagnostics").and_then(|d| d.as_array()).unwrap();
    assert!(
        !diags.is_empty(),
        "Must report structured diagnostics on tamper"
    );
}
