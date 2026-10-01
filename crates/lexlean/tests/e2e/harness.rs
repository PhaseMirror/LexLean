//! E2E Test Harness: CLI process runner, public Engine API runner, and sandboxed test projects.

use std::path::{Path, PathBuf};
use std::process::Command;

use camino::Utf8PathBuf;
use lexlean::api::Engine;
use lexlean::error::LexLeanError;

/// Locates the repository root.
pub fn repo_root() -> Utf8PathBuf {
    let manifest = Utf8PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .ancestors()
        .nth(2)
        .expect("crates/lexlean is two levels below root")
        .to_path_buf()
}

/// Output captured from a CLI invocation.
#[derive(Debug, Clone)]
pub struct CliOutput {
    /// Process exit code.
    pub exit_code: i32,
    /// Standard output.
    pub stdout: String,
    /// Standard error.
    pub stderr: String,
}

impl CliOutput {
    /// Asserts that the command succeeded (exit code 0).
    pub fn assert_success(&self) {
        assert_eq!(
            self.exit_code, 0,
            "Expected exit code 0, got {}. stderr:\n{}",
            self.exit_code, self.stderr
        );
    }

    /// Asserts that the command failed with non-zero exit code.
    pub fn assert_failure(&self) {
        assert_ne!(
            self.exit_code, 0,
            "Expected failure, got exit code 0. stdout:\n{}",
            self.stdout
        );
    }

    /// Asserts that the command stderr contains the given diagnostic code.
    #[allow(dead_code)]
    pub fn assert_stderr_contains_code(&self, code: &str) {
        self.assert_failure();
        assert!(
            self.stderr.contains(code),
            "Expected stderr to contain diagnostic code `{code}`, got:\n{}",
            self.stderr
        );
    }

    /// Parses stdout as canonical command result JSON.
    pub fn parse_json(&self) -> serde_json::Value {
        serde_json::from_str(&self.stdout).unwrap_or_else(|err| {
            panic!(
                "Failed to parse stdout as JSON: {err}\nstdout was:\n{}\nstderr was:\n{}",
                self.stdout, self.stderr
            )
        })
    }

    /// Asserts that the JSON output reports the given diagnostic error code.
    pub fn assert_json_diagnostic_code(&self, code: &str) {
        let json = self.parse_json();
        let diagnostics = json
            .get("diagnostics")
            .and_then(|d| d.as_array())
            .expect("JSON output must have `diagnostics` array");
        let found = diagnostics
            .iter()
            .any(|diag| diag.get("code").and_then(|c| c.as_str()) == Some(code));
        assert!(
            found,
            "Expected JSON diagnostics to contain code `{code}`, found:\n{diagnostics:#?}"
        );
    }
}

/// Black-box CLI runner executing the compiled `lexlean` binary as a child process.
pub struct CliRunner {
    bin_path: PathBuf,
}

impl CliRunner {
    /// Creates a new CLI runner locating the compiled `lexlean` binary.
    pub fn new() -> Self {
        let bin = env!("CARGO_BIN_EXE_lexlean");
        let bin_path = PathBuf::from(bin);
        assert!(
            bin_path.exists(),
            "lexlean binary does not exist at {}",
            bin_path.display()
        );
        Self { bin_path }
    }

    /// Executes `lexlean` with the specified arguments in a given working directory.
    pub fn run_in(&self, working_dir: &Path, args: &[&str]) -> CliOutput {
        let output = Command::new(&self.bin_path)
            .args(args)
            .current_dir(working_dir)
            .output()
            .unwrap_or_else(|err| {
                panic!(
                    "Failed to execute {} {:?}: {err}",
                    self.bin_path.display(),
                    args
                )
            });

        CliOutput {
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        }
    }

    /// Executes `lexlean` with the specified arguments in the repository root.
    pub fn run(&self, args: &[&str]) -> CliOutput {
        self.run_in(repo_root().as_std_path(), args)
    }
}

impl Default for CliRunner {
    fn default() -> Self {
        Self::new()
    }
}

/// A hermetic, sandboxed test project in a temporary directory.
pub struct TestProject {
    #[allow(dead_code)]
    temp_dir: tempfile::TempDir,
    /// Absolute UTF-8 path to the project root.
    pub root: Utf8PathBuf,
    cli_runner: CliRunner,
}

impl TestProject {
    /// Creates a project by cloning an existing example from the repository.
    pub fn from_example(name: &str) -> Self {
        let temp_dir = tempfile::Builder::new()
            .prefix(&format!("e2e-{name}-"))
            .tempdir()
            .expect("create tempdir for test project");
        let example_src = repo_root().join("examples").join(name);
        assert!(
            example_src.exists(),
            "Example {} not found at {}",
            name,
            example_src
        );

        // Copy files recursively, skipping existing build artifacts
        for entry in walkdir::WalkDir::new(example_src.as_std_path())
            .into_iter()
            .flatten()
        {
            let rel = entry
                .path()
                .strip_prefix(example_src.as_std_path())
                .expect("relative path");
            let rel_str = rel.to_string_lossy();
            if rel_str.starts_with(".lexlean") || rel_str.starts_with("expected") {
                continue;
            }
            let dest = temp_dir.path().join(rel);
            if entry.file_type().is_dir() {
                std::fs::create_dir_all(&dest).expect("create dir");
            } else if entry.file_type().is_file() {
                if let Some(parent) = dest.parent() {
                    std::fs::create_dir_all(parent).expect("create parent dir");
                }
                std::fs::copy(entry.path(), &dest).expect("copy file");
            }
        }

        let root =
            Utf8PathBuf::from_path_buf(temp_dir.path().to_path_buf()).expect("utf8 temp path");
        Self {
            temp_dir,
            root,
            cli_runner: CliRunner::new(),
        }
    }

    /// Creates a minimal valid project with the given name and language version.
    #[allow(dead_code)]
    pub fn new_minimal(name: &str, language: &str) -> Self {
        let temp_dir = tempfile::Builder::new()
            .prefix(&format!("e2e-minimal-{name}-"))
            .tempdir()
            .expect("create tempdir");
        let root =
            Utf8PathBuf::from_path_buf(temp_dir.path().to_path_buf()).expect("utf8 temp path");

        let config = format!(
            r#"spec = "lexlean/project/1"
name = "{name}"
language = "{language}"
module_prefix = "E2ETest"
source_roots = ["src"]
entrypoints = ["src/Main.lex.tex"]
build_root = ".lexlean"
lockfile = "lexlean.lock"
lean_workspace = "."
lean_toolchain = "leanprover/lean4:v4.32.1"

[[lexicon_source]]
package = "lexlean.std.nat"
kind = "builtin"

[limits]
max_file_bytes = 4194304
max_total_source_bytes = 67108864
max_primitive_atoms = 2000000
max_token_lattice_edges = 4000000
max_parse_states = 4000000
max_ir_nodes = 2000000
max_scope_depth = 1024
max_import_depth = 128
max_diagnostics = 256
max_child_output_bytes = 16777216
child_timeout_ms = 300000
"#
        );

        let src_dir = temp_dir.path().join("src");
        std::fs::create_dir_all(&src_dir).expect("create src dir");
        std::fs::write(temp_dir.path().join("lexlean.toml"), config).expect("write lexlean.toml");

        // Copy lean-toolchain and minimal lake files from examples/nat-add-zero
        let example_root = repo_root().join("examples/nat-add-zero");
        for file in &["lean-toolchain", "lakefile.toml", "lake-manifest.json"] {
            let src = example_root.join(file);
            if src.exists() {
                std::fs::copy(src.as_std_path(), temp_dir.path().join(file)).expect("copy file");
            }
        }

        // Minimal Main.lex.tex
        let source_content = r#"\documentclass{article}
\usepackage{lexlean}
\begin{document}
\begin{lexmodule}{Main}
\begin{lexsection}{Section}
\end{lexsection}
\end{lexmodule}
\end{document}
"#;
        std::fs::write(src_dir.join("Main.lex.tex"), source_content).expect("write Main.lex.tex");

        Self {
            temp_dir,
            root,
            cli_runner: CliRunner::new(),
        }
    }

    /// Reads a file relative to project root.
    pub fn read(&self, relative: &str) -> String {
        std::fs::read_to_string(self.root.join(relative).as_std_path())
            .unwrap_or_else(|err| panic!("Failed to read {relative}: {err}"))
    }

    /// Writes a file relative to project root.
    pub fn write(&self, relative: &str, content: &str) {
        let dest = self.root.join(relative);
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent.as_std_path()).expect("create parent");
        }
        std::fs::write(dest.as_std_path(), content)
            .unwrap_or_else(|err| panic!("Failed to write {relative}: {err}"));
    }

    /// Edits a file by replacing `from` with `to`.
    pub fn edit(&self, relative: &str, from: &str, to: &str) {
        let text = self.read(relative);
        assert!(
            text.contains(from),
            "File `{relative}` does not contain target string: `{from}`"
        );
        self.write(relative, &text.replacen(from, to, 1));
    }

    /// Executes `lexlean` CLI inside this project's directory.
    pub fn cli(&self, args: &[&str]) -> CliOutput {
        self.cli_runner.run_in(self.root.as_std_path(), args)
    }

    /// Loads the project using the public `Engine` API.
    pub fn load_engine(&self) -> Result<Engine, LexLeanError> {
        Engine::load(&self.root.join("lexlean.toml"))
    }

    /// Loads the project expecting an error.
    pub fn load_engine_err(&self) -> LexLeanError {
        match self.load_engine() {
            Ok(_) => panic!("Expected project loading to fail, but it succeeded"),
            Err(err) => err,
        }
    }

    /// Loads the project expecting failure with the given diagnostic code.
    pub fn load_engine_fails_with(&self, code: &str) -> LexLeanError {
        let err = self.load_engine_err();
        Self::assert_error_code(&err, code);
        err
    }

    /// Asserts error contains the given diagnostic code.
    pub fn assert_error_code(error: &LexLeanError, code: &str) {
        let found = error.diagnostics.iter().any(|d| d.code.as_str() == code);
        assert!(
            found,
            "Expected diagnostic code `{code}`, found: {:?}",
            error
                .diagnostics
                .iter()
                .map(|d| d.code.as_str())
                .collect::<Vec<_>>()
        );
    }
}
