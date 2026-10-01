//! Tier 1: Feature Coverage (>=5 tests per feature).
//!
//! Covers:
//! - Feature 1: Language Version Boundary (1.0, 1.1, 1.2 boundary contract, CLI init, version reporting).
//! - Feature 2: Lockfile & Migration Contract (Lockfile v1 read, lock --check, relock, config digest validation).

use super::harness::{CliRunner, TestProject};
use lexlean::api::{CheckRequest, LockRequest, Selection};

// ============================================================================
// Feature 1: Language Version Boundary (>=5 test cases)
// ============================================================================

/// Tier 1, Feature 1, Test 1: A project declared with `language = "1.0"` loads via Engine.
#[test]
fn test_tier1_lang10_declaration_and_engine_load() {
    let project = TestProject::from_example("nat-add-zero");
    let _engine = project
        .load_engine()
        .expect("Language 1.0 project must load cleanly");
    let config = project.read("lexlean.toml");
    assert!(config.contains("language = \"1.0\""));
    assert!(config.contains("name = \"nat-add-zero\""));
}

/// Tier 1, Feature 1, Test 2: A project declared with `language = "1.1"` loads via Engine.
#[test]
fn test_tier1_lang11_declaration_and_engine_load() {
    let project = TestProject::from_example("semantic-1.1");
    let _engine = project
        .load_engine()
        .expect("Language 1.1 project must load cleanly");
    let config = project.read("lexlean.toml");
    assert!(config.contains("language = \"1.1\""));
    assert!(config.contains("name = \"semantic-fixture\""));
}

/// Tier 1, Feature 1, Test 3: CLI `--version` reports compiler identity, latest language version, and toolchain.
#[test]
fn test_tier1_cli_version_flag() {
    let runner = CliRunner::new();
    let output = runner.run(&["--version"]);
    output.assert_success();

    let stdout = &output.stdout;
    assert!(
        stdout.contains("lexlean"),
        "Version output must name lexlean: {stdout}"
    );
    assert!(
        stdout.contains("leanprover/lean4:v4.32.1"),
        "Version output must cite pinned toolchain: {stdout}"
    );
    assert!(
        stdout.contains("language"),
        "Version output must cite supported language versions: {stdout}"
    );
}

/// Tier 1, Feature 1, Test 4: CLI `init` creates a valid project skeleton.
#[test]
fn test_tier1_cli_init_project() {
    let runner = CliRunner::new();
    let temp = tempfile::tempdir().expect("tempdir");
    let dest = temp.path().join("init-test");

    let output = runner.run(&[
        "init",
        dest.to_str().unwrap(),
        "--name",
        "init-test",
        "--module-prefix",
        "InitTest",
        "--language",
        "1.1",
    ]);
    output.assert_success();

    let config_path = dest.join("lexlean.toml");
    assert!(config_path.exists(), "lexlean.toml must be generated");
    let config = std::fs::read_to_string(&config_path).expect("read config");
    assert!(config.contains("name = \"init-test\""));
    assert!(config.contains("language = \"1.1\""));
    assert!(
        dest.join("src").join("Main.lex.tex").exists(),
        "Main.lex.tex must be created"
    );
}

/// Tier 1, Feature 1, Test 5: Language 1.2 Boundary & Migration Contract.
/// If Language 1.2 is implemented, it loads cleanly; if pre-M1, it fails closed with LLC0103.
#[test]
fn test_tier1_lang12_boundary_contract() {
    let project = TestProject::from_example("semantic-1.1");
    project.edit("lexlean.toml", "language = \"1.1\"", "language = \"1.2\"");

    if lexlean::supports_language("1.2") {
        let _engine = project
            .load_engine()
            .expect("Language 1.2 project must load when supported");
    } else {
        project.load_engine_fails_with("LLC0103");
    }
}

/// Tier 1, Feature 1, Test 6: Engine check executes on entrypoints for supported language.
#[test]
fn test_tier1_engine_check_entrypoints() {
    let project = TestProject::from_example("semantic-1.1");
    let engine = project.load_engine().expect("load project");
    let result = engine
        .check(CheckRequest {
            selection: Selection::Entrypoints,
        })
        .expect("check entrypoints must succeed");

    assert!(!result.units.is_empty(), "Checked units must not be empty");
    for (name, unit) in &result.units {
        assert_eq!(&unit.module, name);
        assert!(!unit.summary.lean_module.is_empty());
    }
}

// ============================================================================
// Feature 2: Lockfile & Migration Contract (>=5 test cases)
// ============================================================================

/// Tier 1, Feature 2, Test 7: Lockfile v1 parsing and verification against embedded compiler semantics.
#[test]
fn test_tier1_lockfile_v1_loading() {
    let project = TestProject::from_example("semantic-1.1");
    let lock_text = project.read("lexlean.lock");
    assert!(lock_text.contains("spec = \"lexlean/lock/1\""));
    assert!(lock_text.contains("language = \"1.1\""));
    assert!(lock_text.contains("id = \"lexlean.core\""));

    let engine = project.load_engine().expect("load project");
    let lock_res = engine
        .lock(LockRequest {
            check_only: true,
            allow_network: false,
        })
        .expect("lockfile must validate");
    assert!(!lock_res.bytes.is_empty());
}

/// Tier 1, Feature 2, Test 8: CLI `lock --check` passes cleanly when synchronized.
#[test]
fn test_tier1_lock_check_clean() {
    let project = TestProject::from_example("semantic-1.1");
    let output = project.cli(&["lock", "--check"]);
    output.assert_success();
}

/// Tier 1, Feature 2, Test 9: Relocking preserves package tree digests and manifest hashes.
#[test]
fn test_tier1_lock_relock_preserves_hashes() {
    let project = TestProject::from_example("semantic-1.1");
    let engine = project.load_engine().expect("load project");

    let original_lock = project.read("lexlean.lock");

    // Re-lock
    let lock_result = engine
        .lock(LockRequest {
            check_only: false,
            allow_network: false,
        })
        .expect("relock must succeed");

    assert!(!lock_result.bytes.is_empty());

    let updated_lock = project.read("lexlean.lock");
    assert_eq!(
        original_lock, updated_lock,
        "Re-locking an unmodified project must produce byte-identical lockfile"
    );
}

/// Tier 1, Feature 2, Test 10: Modifying project config causes `lock --check` to fail closed with LLR3001 or LLC0103.
#[test]
fn test_tier1_lock_digest_tamper_detection() {
    let project = TestProject::from_example("semantic-1.1");
    // Modify limits in lexlean.toml to change its SHA-256 without updating lockfile
    project.edit(
        "lexlean.toml",
        "max_scope_depth = 1024",
        "max_scope_depth = 2048",
    );

    let output = project.cli(&["lock", "--check"]);
    output.assert_failure();
    // Must fail closed with either LLR3001 (lock out of date) or LLC0102 / LLC0103
    assert!(
        output.stderr.contains("LLR3001")
            || output.stderr.contains("LLC0102")
            || output.stderr.contains("LLC0103"),
        "Expected stale lock diagnostic on modified config, got:\n{}",
        output.stderr
    );
}

/// Tier 1, Feature 2, Test 11: Lockfile migration contract between language versions.
#[test]
fn test_tier1_lockfile_v2_migration_boundary() {
    let project = TestProject::from_example("nat-add-zero");
    let initial_config = project.read("lexlean.toml");
    assert!(initial_config.contains("language = \"1.0\""));

    // Upgrade project config to 1.1 and re-lock
    project.edit("lexlean.toml", "language = \"1.0\"", "language = \"1.1\"");
    let engine = project.load_engine().expect("load 1.1");
    let relock = engine
        .lock(LockRequest {
            check_only: false,
            allow_network: false,
        })
        .expect("relock after version upgrade");
    assert!(!relock.bytes.is_empty());

    let updated_lock = project.read("lexlean.lock");
    assert!(updated_lock.contains("language = \"1.1\""));
    assert_eq!(project.cli(&["lock", "--check"]).exit_code, 0);
}
