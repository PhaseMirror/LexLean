//! The differential validation oracle (SPEC.md §17.19): `lean4-prod`, an
//! independent Lean 4 to Rust generator, acquired at a pinned commit and run
//! against the Lean LexLean generates.
//!
//! The oracle is validation evidence only. Everything here is test
//! infrastructure of the conformance crate: the shipped crate never names it,
//! no identity LexLean computes takes it as an input, and the gate that
//! checks that (`audit-oracle-isolation`) reads this crate's pin file, not
//! the other way round.
//!
//! The reason this module is careful about *failure* is the whole point of
//! the integration. An oracle that could not be fetched, did not build,
//! crashed, was killed, or produced nothing has said nothing, and a harness
//! that read that silence as agreement would report a validation that never
//! happened. So there are exactly two kinds of answer from the tool,
//! [`Verdict::Accepted`] and [`Verdict::Refused`] (the oracle's own, structured
//! refusal of a program), and everything else is [`Unavailable`], which no
//! caller can turn into a validated result.

use std::collections::BTreeMap;
use std::fmt;
use std::io::Read as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::support::repo_root;

/// The role of the oracle in every statement this repository makes about it.
pub const ROLE: &str = "validation-evidence";

/// The domain of the canonical frame of the oracle's files.
const TREE_DOMAIN: &[u8] = b"lexlean-oracle-tree-v1\0";

/// The marker the oracle's exporter writes in front of the reason it
/// refuses a program. Only this marker makes a failed run a refusal.
const EXPORT_REFUSAL: &str = "prod-export failed: ";

/// The marker the oracle's code generator writes in front of the reason it
/// refuses a program.
const CODEGEN_REFUSAL: &str = "Code generation refused: ";

/// The pin of the oracle (`oracle/lean4-prod.toml`).
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Pin {
    /// The schema tag, `lexlean/oracle-pin/1`.
    pub spec: String,
    /// The oracle's name.
    pub name: String,
    /// The role: always [`ROLE`].
    pub role: String,
    /// Where the oracle is fetched from.
    pub remote: String,
    /// The exact commit.
    pub commit: String,
    /// The commit's git tree, for a reader with git.
    pub git_tree: String,
    /// The path prefixes of the files the integration uses.
    pub used: Vec<String>,
    /// How many files those are.
    pub tree_files: usize,
    /// The SHA-256 of their canonical frame.
    pub tree_sha256: String,
    /// The Lean toolchain the oracle is built with.
    pub lean_toolchain: String,
    /// The Rust toolchain the oracle is built with.
    pub rust_toolchain: String,
    /// The Lake target that is built.
    pub lean_target: String,
    /// The Cargo package that is built.
    pub cargo_package: String,
    /// The bound on one invocation, in seconds.
    pub timeout_seconds: u64,
    /// The licence finding.
    pub licence: Licence,
}

/// What the pin records about the oracle's licence.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Licence {
    /// What the oracle's manifests declare.
    pub declared: String,
    /// Whether the oracle's repository ships a licence text.
    pub text_shipped: bool,
    /// Whether any of the oracle is vendored here.
    pub vendored: bool,
    /// Why.
    pub reason: String,
}

fn is_hex(text: &str, length: usize) -> bool {
    text.len() == length
        && text
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

impl Pin {
    /// Read the committed pin.
    ///
    /// # Panics
    ///
    /// Panics when the committed pin is not valid: that is a defect of the
    /// repository, not a condition of the host.
    #[must_use]
    pub fn committed() -> Self {
        let text = std::fs::read_to_string(repo_root().join("oracle/lean4-prod.toml").as_std_path())
            .expect("oracle/lean4-prod.toml");
        Self::parse(&text).unwrap_or_else(|reason| panic!("oracle/lean4-prod.toml: {reason}"))
    }

    /// Parse and validate a pin.
    ///
    /// # Errors
    ///
    /// Returns the first way the text is not a complete pin: an unknown or
    /// missing field, a role other than [`ROLE`], a commit or digest that is
    /// not lowercase hexadecimal of the right length, or an unusable list of
    /// used paths.
    pub fn parse(text: &str) -> Result<Self, String> {
        let pin: Self = toml::from_str(text).map_err(|error| error.to_string())?;
        if pin.spec != "lexlean/oracle-pin/1" {
            return Err(format!("unknown pin schema `{}`", pin.spec));
        }
        if pin.role != ROLE {
            return Err(format!(
                "the oracle's role is `{}`; it is `{ROLE}` and nothing else",
                pin.role
            ));
        }
        if !is_hex(&pin.commit, 40) || !is_hex(&pin.git_tree, 40) {
            return Err("the commit and its git tree are 40 lowercase hexadecimal digits".into());
        }
        if !is_hex(&pin.tree_sha256, 64) {
            return Err("tree_sha256 is 64 lowercase hexadecimal digits".into());
        }
        if pin.used.is_empty()
            || pin
                .used
                .iter()
                .any(|path| path.is_empty() || path.starts_with('/') || path.contains(".."))
        {
            return Err("`used` names relative path prefixes of the checkout".into());
        }
        if pin.tree_files == 0 || pin.timeout_seconds == 0 {
            return Err("tree_files and timeout_seconds are positive".into());
        }
        if pin.licence.vendored {
            return Err(
                "nothing of the oracle is vendored: the repository ships no licence text".into(),
            );
        }
        Ok(pin)
    }

    fn uses(&self, relative: &str) -> bool {
        self.used.iter().any(|prefix| {
            relative == prefix.trim_end_matches('/') || relative.starts_with(prefix.as_str())
        })
    }
}

/// The SHA-256 of the canonical frame of the files of `root` the pin uses,
/// and how many there are.
///
/// # Errors
///
/// Returns the I/O error that kept a file from being read.
pub fn tree_digest(root: &Path, pin: &Pin) -> Result<(String, usize), String> {
    let mut files: Vec<(String, PathBuf)> = Vec::new();
    for entry in walkdir::WalkDir::new(root)
        .follow_links(false)
        .sort_by_file_name()
        .into_iter()
        .filter_entry(|entry| {
            let name = entry.file_name().to_string_lossy();
            !(entry.file_type().is_dir() && matches!(name.as_ref(), ".git" | ".lake" | "target"))
        })
    {
        let entry = entry.map_err(|error| error.to_string())?;
        if !entry.file_type().is_file() {
            continue;
        }
        let relative = entry
            .path()
            .strip_prefix(root)
            .map_err(|error| error.to_string())?
            .components()
            .map(|part| part.as_os_str().to_string_lossy().into_owned())
            .collect::<Vec<_>>()
            .join("/");
        if pin.uses(&relative) {
            files.push((relative, entry.path().to_path_buf()));
        }
    }
    files.sort_by(|left, right| left.0.as_bytes().cmp(right.0.as_bytes()));
    let mut hasher = Sha256::new();
    hasher.update(TREE_DOMAIN);
    for (relative, path) in &files {
        let bytes = std::fs::read(path).map_err(|error| format!("{relative}: {error}"))?;
        hasher.update((relative.len() as u64).to_be_bytes());
        hasher.update(relative.as_bytes());
        hasher.update((bytes.len() as u64).to_be_bytes());
        hasher.update(Sha256::digest(&bytes));
    }
    let hex = hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    Ok((hex, files.len()))
}

/// Why the oracle could not say anything. None of these is a validation, and
/// none is a disagreement: the gate that meets one fails on the normative
/// host and reports "not validated" elsewhere.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Unavailable {
    /// No checkout, and none could be fetched.
    NotAcquired(String),
    /// A checkout exists but is not the pinned one.
    PinMismatch(String),
    /// The pinned oracle could not be built with the pinned toolchains.
    BuildFailed {
        /// The step.
        step: String,
        /// The tool's output.
        detail: String,
    },
    /// A tool of the oracle failed in a way that is not its structured
    /// refusal of a program: a crash, a signal, a timeout, an unreadable or
    /// missing output.
    ToolFailed {
        /// The step.
        step: String,
        /// What happened.
        detail: String,
    },
    /// The run had nothing to validate: a comparison over zero cases says
    /// nothing, and is never read as agreement.
    Vacuous(String),
}

impl fmt::Display for Unavailable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotAcquired(detail) => write!(f, "the oracle was not acquired: {detail}"),
            Self::PinMismatch(detail) => write!(f, "the checkout is not the pinned oracle: {detail}"),
            Self::BuildFailed { step, detail } => {
                write!(f, "the oracle did not build at `{step}`: {detail}")
            }
            Self::ToolFailed { step, detail } => {
                write!(f, "the oracle failed at `{step}` without refusing the program: {detail}")
            }
            Self::Vacuous(detail) => write!(f, "nothing was validated: {detail}"),
        }
    }
}

impl std::error::Error for Unavailable {}

/// Where an oracle refusal arose.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Stage {
    /// The Lean exporter (`prod-export`) refused the root.
    Export,
    /// The Rust code generator (`prod`) refused the exported module.
    Codegen,
}

/// The oracle's answer about one root: it generated Rust, or it refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    /// The oracle generated this package: its files by path.
    Accepted {
        /// The exported intermediate representation.
        ir: String,
        /// The generated crate's files.
        package: BTreeMap<String, Vec<u8>>,
    },
    /// The oracle refused the program, in its own structured words.
    Refused {
        /// Which stage refused.
        stage: Stage,
        /// The reason, as the oracle printed it (first line).
        message: String,
    },
}

/// An acquired and built oracle.
#[derive(Debug)]
pub struct Oracle {
    pin: Pin,
    exporter: PathBuf,
    prod: PathBuf,
    scratch: PathBuf,
    bin: PathBuf,
}

/// Where the oracle is cached: a directory of the build, or the environment's.
fn cache_root() -> PathBuf {
    if let Some(dir) = std::env::var_os("LEXLEAN_ORACLE_CACHE") {
        return PathBuf::from(dir);
    }
    let target = std::env::var_os("CARGO_TARGET_DIR").map_or_else(
        || repo_root().join("target").into_std_path_buf(),
        PathBuf::from,
    );
    target.join("lexlean-oracle")
}

/// A command of the pinned toolchain's own `bin`, so a tool that spawns
/// another (Lake spawns `lean`) finds the pinned one whatever an elan proxy
/// would choose.
fn toolchain_command(program: &str, cwd: &Path) -> Command {
    let bin = crate::preservation::toolchain_bin();
    let path = std::env::join_paths(
        std::iter::once(bin.clone())
            .chain(std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default())),
    )
    .expect("a search path");
    let mut command = Command::new(bin.join(program));
    command.current_dir(cwd).env("PATH", path);
    command
}

/// What a finished process left.
struct Ran {
    status: Option<i32>,
    stdout: String,
    stderr: String,
}

/// Run `command` to completion within `limit`, or kill it.
fn run(mut command: Command, limit: Duration) -> Result<Ran, String> {
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn().map_err(|error| format!("cannot start: {error}"))?;
    let mut out = child.stdout.take().ok_or("no stdout")?;
    let mut err = child.stderr.take().ok_or("no stderr")?;
    let stdout = std::thread::spawn(move || {
        let mut text = Vec::new();
        let _ = out.read_to_end(&mut text);
        text
    });
    let stderr = std::thread::spawn(move || {
        let mut text = Vec::new();
        let _ = err.read_to_end(&mut text);
        text
    });
    let start = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {
                if start.elapsed() > limit {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(format!("killed after {} s", limit.as_secs()));
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(error) => return Err(format!("cannot wait: {error}")),
        }
    };
    let text = |bytes: Vec<u8>| String::from_utf8_lossy(&bytes).into_owned();
    Ok(Ran {
        status: status.code(),
        stdout: text(stdout.join().unwrap_or_default()),
        stderr: text(stderr.join().unwrap_or_default()),
    })
}

fn last_lines(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    lines[lines.len().saturating_sub(12)..].join("\n")
}

fn copy_tree(from: &Path, to: &Path, pin: &Pin, root: &Path) -> Result<(), String> {
    for entry in walkdir::WalkDir::new(from)
        .follow_links(false)
        .into_iter()
        .filter_entry(|entry| {
            let name = entry.file_name().to_string_lossy();
            !(entry.file_type().is_dir() && matches!(name.as_ref(), ".git" | ".lake" | "target"))
        })
    {
        let entry = entry.map_err(|error| error.to_string())?;
        let relative = entry.path().strip_prefix(root).map_err(|e| e.to_string())?;
        let text = relative
            .components()
            .map(|part| part.as_os_str().to_string_lossy().into_owned())
            .collect::<Vec<_>>()
            .join("/");
        if !entry.file_type().is_file() || !pin.uses(&text) {
            continue;
        }
        let destination = to.join(relative);
        if let Some(parent) = destination.parent() {
            std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        std::fs::copy(entry.path(), &destination).map_err(|error| error.to_string())?;
    }
    Ok(())
}

/// Fetch the pinned commit of the oracle into `dir`.
fn fetch(pin: &Pin, dir: &Path) -> Result<(), Unavailable> {
    let fail = |detail: String| Unavailable::NotAcquired(detail);
    std::fs::create_dir_all(dir).map_err(|error| fail(error.to_string()))?;
    for arguments in [
        vec!["init", "--quiet"],
        vec!["fetch", "--quiet", "--depth", "1", pin.remote.as_str(), pin.commit.as_str()],
        vec!["checkout", "--quiet", "--detach", "FETCH_HEAD"],
    ] {
        let mut command = Command::new("git");
        command.args(&arguments).current_dir(dir);
        let ran = run(command, Duration::from_secs(pin.timeout_seconds))
            .map_err(|detail| fail(format!("git {}: {detail}", arguments[0])))?;
        if ran.status != Some(0) {
            return Err(fail(format!(
                "git {} exited {:?}: {}",
                arguments[0],
                ran.status,
                last_lines(&ran.stderr)
            )));
        }
    }
    Ok(())
}

/// Acquire the pinned oracle and build it, once per process.
///
/// The checkout is `$LEXLEAN_ORACLE_DIR` when set and a fetch of the pinned
/// commit into the cache otherwise. Either is refused unless the files the pin
/// names hash to its `tree_sha256` and, when it is a git checkout, `HEAD` is
/// the pinned commit.
///
/// # Errors
///
/// [`Unavailable`] in every case where the oracle cannot be used: the caller
/// decides what that means for its gate, and none of it is agreement.
pub fn acquire() -> Result<&'static Oracle, Unavailable> {
    static ORACLE: OnceLock<Result<Oracle, Unavailable>> = OnceLock::new();
    ORACLE
        .get_or_init(|| Oracle::acquire_from(&Pin::committed()))
        .as_ref()
        .map_err(Clone::clone)
}

impl Oracle {
    /// Acquire and build the oracle `pin` names.
    ///
    /// # Errors
    ///
    /// As [`acquire`].
    pub fn acquire_from(pin: &Pin) -> Result<Self, Unavailable> {
        let cache = cache_root();
        std::fs::create_dir_all(&cache)
            .map_err(|error| Unavailable::NotAcquired(format!("{}: {error}", cache.display())))?;
        // One process builds; the others wait for the finished build.
        let lock = std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(cache.join("lock"))
            .map_err(|error| Unavailable::NotAcquired(error.to_string()))?;
        lock.lock()
            .map_err(|error| Unavailable::NotAcquired(error.to_string()))?;
        let source = match std::env::var_os("LEXLEAN_ORACLE_DIR") {
            Some(dir) => PathBuf::from(dir),
            None => {
                let dir = cache.join("source");
                if !dir.join(".git").is_dir() {
                    let _ = std::fs::remove_dir_all(&dir);
                    fetch(pin, &dir)?;
                }
                dir
            }
        };
        if !source.is_dir() {
            return Err(Unavailable::NotAcquired(format!(
                "{} is not a directory",
                source.display()
            )));
        }
        let (digest, files) = tree_digest(&source, pin).map_err(Unavailable::NotAcquired)?;
        if digest != pin.tree_sha256 || files != pin.tree_files {
            return Err(Unavailable::PinMismatch(format!(
                "{} files hash to {digest}; the pin names {} files hashing to {}",
                files, pin.tree_files, pin.tree_sha256
            )));
        }
        if source.join(".git").exists() {
            let mut command = Command::new("git");
            command.args(["rev-parse", "HEAD"]).current_dir(&source);
            let head = run(command, Duration::from_secs(60))
                .ok()
                .filter(|ran| ran.status == Some(0))
                .map(|ran| ran.stdout.trim().to_owned());
            if head.as_deref() != Some(pin.commit.as_str()) {
                return Err(Unavailable::PinMismatch(format!(
                    "HEAD is {head:?}; the pin names {}",
                    pin.commit
                )));
            }
        }
        let build = cache.join(format!("build-{}", &pin.tree_sha256[..16]));
        let exporter = build
            .join("lean/.lake/build/bin")
            .join(format!("{}{}", pin.lean_target, std::env::consts::EXE_SUFFIX));
        let prod = build
            .join("target/release")
            .join(format!("prod{}", std::env::consts::EXE_SUFFIX));
        if !(exporter.is_file() && prod.is_file()) {
            let _ = std::fs::remove_dir_all(&build);
            std::fs::create_dir_all(&build)
                .map_err(|error| Unavailable::NotAcquired(error.to_string()))?;
            copy_tree(&source, &build, pin, &source)
                .map_err(Unavailable::NotAcquired)?;
            Self::build(pin, &build)?;
        }
        let scratch = cache.join("scratch");
        std::fs::create_dir_all(&scratch)
            .map_err(|error| Unavailable::NotAcquired(error.to_string()))?;
        Ok(Self {
            pin: pin.clone(),
            exporter,
            prod,
            scratch,
            bin: crate::preservation::toolchain_bin(),
        })
    }

    fn build(pin: &Pin, build: &Path) -> Result<(), Unavailable> {
        let limit = Duration::from_secs(pin.timeout_seconds * 4);
        let failed = |step: &str, detail: String| Unavailable::BuildFailed {
            step: step.to_owned(),
            detail,
        };
        let mut lake = toolchain_command("lake", &build.join("lean"));
        lake.args(["build", pin.lean_target.as_str()]);
        let ran = run(lake, limit).map_err(|detail| failed("lake build", detail))?;
        if ran.status != Some(0) {
            return Err(failed(
                "lake build",
                format!("{:?}: {}", ran.status, last_lines(&ran.stderr)),
            ));
        }
        // The first attempt is offline: the dependencies of the oracle's
        // workspace are often already in the registry cache, and a build
        // that needs no network is a build that cannot be cut short by it.
        let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned());
        let mut last = String::new();
        for offline in [true, false] {
            let mut command = Command::new(&cargo);
            command
                .args(["build", "--release", "--locked", "-p", pin.cargo_package.as_str()])
                .current_dir(build.join("rust"))
                .env("CARGO_TARGET_DIR", build.join("target"))
                .env("RUSTUP_TOOLCHAIN", &pin.rust_toolchain)
                .env("CARGO_INCREMENTAL", "0")
                .env_remove("RUSTFLAGS")
                .env_remove("CARGO_ENCODED_RUSTFLAGS");
            if offline {
                command.arg("--offline");
            }
            match run(command, limit) {
                Ok(ran) if ran.status == Some(0) => return Ok(()),
                Ok(ran) => last = format!("{:?}: {}", ran.status, last_lines(&ran.stderr)),
                Err(detail) => last = detail,
            }
        }
        Err(failed("cargo build", last))
    }

    /// The pin this oracle was acquired by.
    #[must_use]
    pub fn pin(&self) -> &Pin {
        &self.pin
    }

    fn limit(&self) -> Duration {
        Duration::from_secs(self.pin.timeout_seconds)
    }

    /// Export the closure of `root` of `module` from the compiled Lean
    /// modules in `library`, and generate its crate named `crate_name`.
    ///
    /// # Errors
    ///
    /// [`Unavailable::ToolFailed`] when a tool fails other than by refusing
    /// the program. A refusal is an answer, [`Verdict::Refused`], and carries
    /// the oracle's own words.
    pub fn verdict(
        &self,
        library: &Path,
        module: &str,
        root: &str,
        crate_name: &str,
    ) -> Result<Verdict, Unavailable> {
        let work = tempfile::Builder::new()
            .prefix("oracle-")
            .tempdir_in(&self.scratch)
            .map_err(|error| Unavailable::ToolFailed {
                step: "scratch".into(),
                detail: error.to_string(),
            })?;
        let tool = |step: &str, detail: String| Unavailable::ToolFailed {
            step: step.to_owned(),
            detail,
        };
        let search = std::env::join_paths([library]).map_err(|e| tool("LEAN_PATH", e.to_string()))?;
        let mut export = Command::new(&self.exporter);
        export
            .args(["--module", module, "--root", root, "--ir-module", "Oracle", "--out"])
            .arg(work.path().join("export"))
            .env("LEAN_PATH", &search)
            .env("PATH", self.path_with_toolchain());
        let ran = run(export, self.limit()).map_err(|detail| tool("prod-export", detail))?;
        if ran.status != Some(0) {
            let all = format!("{}\n{}", ran.stderr, ran.stdout);
            return match refusal(&all, EXPORT_REFUSAL) {
                Some(message) => Ok(Verdict::Refused {
                    stage: Stage::Export,
                    message,
                }),
                None => Err(tool(
                    "prod-export",
                    format!("exit {:?}: {}", ran.status, last_lines(&all)),
                )),
            };
        }
        let ir_path = work.path().join("export/kernel.ir");
        let ir = std::fs::read_to_string(&ir_path)
            .map_err(|error| tool("prod-export", format!("kernel.ir: {error}")))?;
        if !ir.contains("(def ") {
            return Err(tool(
                "prod-export",
                "the exported module defines nothing".to_owned(),
            ));
        }
        std::fs::write(work.path().join("README.md"), "oracle differential\n")
            .map_err(|error| tool("scratch", error.to_string()))?;
        let licence = work.path().join("LICENSE");
        std::fs::write(&licence, "none\n").map_err(|error| tool("scratch", error.to_string()))?;
        let package_dir = work.path().join("package");
        let mut generate = Command::new(&self.prod);
        generate
            .arg("cargo")
            .arg(&ir_path)
            .arg("--output")
            .arg(&package_dir)
            .args(["--name", crate_name, "--version", "0.1.0", "--description", "oracle"])
            .args(["--repository", "https://example.invalid/oracle"])
            .args(["--homepage", "https://example.invalid/oracle"])
            .arg("--readme")
            .arg(work.path().join("README.md"))
            .arg("--license-mit")
            .arg(&licence)
            .arg("--license-apache")
            .arg(&licence)
            .env("RUSTC_WRAPPER", "");
        let ran = run(generate, self.limit()).map_err(|detail| tool("prod cargo", detail))?;
        if ran.status != Some(0) {
            let all = format!("{}\n{}", ran.stderr, ran.stdout);
            return match refusal(&all, CODEGEN_REFUSAL) {
                Some(message) => Ok(Verdict::Refused {
                    stage: Stage::Codegen,
                    message,
                }),
                None => Err(tool(
                    "prod cargo",
                    format!("exit {:?}: {}", ran.status, last_lines(&all)),
                )),
            };
        }
        let mut package = BTreeMap::new();
        for entry in walkdir::WalkDir::new(&package_dir) {
            let entry = entry.map_err(|error| tool("prod cargo", error.to_string()))?;
            if entry.file_type().is_file() {
                let relative = entry
                    .path()
                    .strip_prefix(&package_dir)
                    .map_err(|error| tool("prod cargo", error.to_string()))?
                    .components()
                    .map(|part| part.as_os_str().to_string_lossy().into_owned())
                    .collect::<Vec<_>>()
                    .join("/");
                let bytes = std::fs::read(entry.path())
                    .map_err(|error| tool("prod cargo", error.to_string()))?;
                package.insert(relative, bytes);
            }
        }
        if !package.contains_key("src/lib.rs") || !package.contains_key("Cargo.toml") {
            return Err(tool(
                "prod cargo",
                "the generated package has no src/lib.rs and Cargo.toml".to_owned(),
            ));
        }
        Ok(Verdict::Accepted { ir, package })
    }

    fn path_with_toolchain(&self) -> std::ffi::OsString {
        std::env::join_paths(
            std::iter::once(self.bin.clone())
                .chain(std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default())),
        )
        .expect("a search path")
    }
}

/// The oracle's stated reason, when `output` holds its refusal marker.
fn refusal(output: &str, marker: &str) -> Option<String> {
    output.lines().find_map(|line| {
        line.find(marker).map(|at| {
            line[at + marker.len()..]
                .trim()
                .chars()
                .take(400)
                .collect::<String>()
        })
    })
}

/// The kind of an oracle refusal, from its words: the part of the message that
/// does not name a program, so that two programs refused for one reason have
/// one kind.
#[must_use]
pub fn refusal_kind(stage: Stage, message: &str) -> String {
    let kind = match stage {
        Stage::Export => {
            if message.contains("LocalFunctionError.escaping") {
                "escaping-local-function"
            } else if message.contains("opaque lowering") {
                "opaque-lowering"
            } else if message.contains("unresolved external call") {
                "unresolved-external-call"
            } else if message.contains("LocalFunctionError") {
                "unsupported-local-function"
            } else {
                "export-refused"
            }
        }
        Stage::Codegen => {
            let head = message.split([':', '(']).next().unwrap_or_default().trim();
            return format!("codegen-{head}");
        }
    };
    kind.to_owned()
}
