//! Tier 3: Cross-Feature Combinations (Pairwise Coverage).
//!
//! Covers:
//! - Matrix 1: Language Version x Diagnostic Format ({1.0, 1.1, invalid} x {human, json}).
//! - Matrix 2: Language Version x CLI Commands ({1.0, 1.1} x {check, build, fmt, lock, explain}).
//! - Matrix 3: Lock State x Project Version ({valid, tampered, missing} x {1.0, 1.1}).
//! - Matrix 4: Color Mode x Diagnostic Format ({auto, always, never} x {human, json}).

use super::harness::TestProject;

/// Tier 3, Matrix 1: Pairwise combination of Language Version and Diagnostic Output Format.
#[test]
fn test_tier3_pairwise_version_x_diagnostic_format() {
    let versions = ["1.0", "1.1", "1.99"];
    let formats = ["human", "json"];

    for version in versions {
        for format in formats {
            let project = TestProject::from_example("semantic-1.1");
            project.edit(
                "lexlean.toml",
                "language = \"1.1\"",
                &format!("language = \"{version}\""),
            );

            let output = project.cli(&["--diagnostic-format", format, "check"]);

            if version == "1.99" {
                output.assert_failure();
                if format == "json" {
                    output.assert_json_diagnostic_code("LLC0103");
                } else {
                    assert!(
                        output.stderr.contains("LLC0103"),
                        "Stderr in human mode must mention LLC0103: {}",
                        output.stderr
                    );
                }
            } else {
                // If version is supported, check output structure
                if format == "json" {
                    let json = output.parse_json();
                    assert!(json.get("command").is_some());
                    assert!(json.get("exit_code").is_some());
                }
            }
        }
    }
}

/// Tier 3, Matrix 2: Pairwise combination of Language Version and CLI Subcommands.
#[test]
fn test_tier3_pairwise_version_x_subcommands() {
    let project_10 = TestProject::from_example("nat-add-zero");
    let project_11 = TestProject::from_example("semantic-1.1");

    let subcommands = [
        vec!["lock", "--check"],
        vec!["fmt", "--check"],
        vec!["check"],
        vec!["explain", "LLC0103"],
    ];

    for args in &subcommands {
        let out_10 = project_10.cli(args);
        out_10.assert_success();

        let out_11 = project_11.cli(args);
        out_11.assert_success();
    }
}

/// Tier 3, Matrix 3: Pairwise combination of Lock State and Project Language Version.
#[test]
fn test_tier3_pairwise_lock_state_x_project_version() {
    let test_cases = [("nat-add-zero", "1.0"), ("semantic-1.1", "1.1")];

    for (example, _version) in test_cases {
        // State 1: Synchronized clean lockfile
        let p_clean = TestProject::from_example(example);
        p_clean.cli(&["lock", "--check"]).assert_success();

        // State 2: Tampered lockfile
        let p_tampered = TestProject::from_example(example);
        p_tampered.edit(
            "lexlean.toml",
            "max_primitive_atoms = 2000000",
            "max_primitive_atoms = 3000000",
        );
        let out_tampered = p_tampered.cli(&["lock", "--check"]);
        out_tampered.assert_failure();

        // State 3: Missing lockfile
        let p_missing = TestProject::from_example(example);
        std::fs::remove_file(p_missing.root.join("lexlean.lock").as_std_path()).unwrap();
        let out_missing = p_missing.cli(&["lock", "--check"]);
        out_missing.assert_failure();
    }
}

/// Tier 3, Matrix 4: Pairwise combination of Color Mode and Diagnostic Format.
#[test]
fn test_tier3_pairwise_color_mode_x_diagnostic_format() {
    let colors = ["auto", "always", "never"];
    let formats = ["human", "json"];

    for color in colors {
        for format in formats {
            let project = TestProject::from_example("semantic-1.1");
            project.edit("lexlean.toml", "language = \"1.1\"", "language = \"9.9\"");

            let output = project.cli(&["--color", color, "--diagnostic-format", format, "check"]);
            output.assert_failure();

            if format == "json" {
                // In JSON mode, output must be valid parseable JSON regardless of color mode
                let json = output.parse_json();
                assert_eq!(json.get("command").and_then(|c| c.as_str()), Some("check"));
                output.assert_json_diagnostic_code("LLC0103");
            } else {
                // Human mode must contain error message
                assert!(output.stderr.contains("LLC0103"));
            }
        }
    }
}
