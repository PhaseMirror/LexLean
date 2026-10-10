//! The certification library's environment (SPEC.md §17.18): the hand-written
//! Lean library under `compiler/proofs/`, compiled with the pinned Lean beside
//! the compiler project's generated modules and the shipped calculus modules,
//! replayed through `leanchecker`, and audited declaration by declaration
//! against its registry.
//!
//! The library is repository evidence, not a shipped file: it quantifies over
//! the evaluator and the certificate checker, which LexLean's proof language
//! cannot state. What it proves is therefore checked here, against the
//! generated modules the compiler project verifies (their text is the
//! project's rendered build, so a drifted module is the library's problem too).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use serde::Deserialize;

use crate::preservation::{joined, pinned_with, shipped};
use crate::support::{self, repo_root, P};

/// The directory of the library, relative to the repository root.
pub const LIBRARY_DIR: &str = "compiler/proofs/LexLeanCertification";

/// The registry of the library's declarations and their axioms.
pub const REGISTRY: &str = "compiler/proofs/library.toml";

/// The registry's tag.
pub const REGISTRY_SPEC: &str = "lexlean/certification-library/1";

/// The library's modules, in dependency order.
pub const MODULES: [&str; 13] = [
    "Trials",
    "Tail",
    "Factor",
    "Schemas",
    "Facts",
    "PremiseSound",
    "Positions",
    "Lower",
    "Frontier",
    "Representation",
    "Normality",
    "Check",
    "Soundness",
];

/// The generated modules the library is stated against, with the target
/// modules they import, in dependency order.
pub const GENERATED: [&str; 6] = [
    "LexLeanTarget.TargetSyntax",
    "LexLeanTarget.TargetSemantics",
    "LexLeanTarget.Gnaf",
    "LexLeanTarget.GnafFixtures",
    "LexLeanTarget.Certification",
    "LexLeanTarget.GnafCertificates",
];

/// The axioms every library declaration may reach.
pub const AXIOMS: [&str; 3] = ["Classical.choice", "Quot.sound", "propext"];

/// The registry.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Registry {
    /// The registry's tag.
    pub spec: String,
    /// The pinned toolchain the axioms were measured under.
    pub toolchain: String,
    /// The library's modules, in dependency order.
    pub modules: Vec<String>,
    /// The registered declarations.
    #[serde(default)]
    pub declaration: Vec<Declaration>,
}

/// One registered declaration: its module, name, and exact axioms.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Declaration {
    /// The module declaring it.
    pub module: String,
    /// Its qualified name.
    pub name: String,
    /// The exact axioms it reaches.
    pub axioms: Vec<String>,
}

/// A module of the library by its name.
#[must_use]
pub fn module_name(stem: &str) -> String {
    format!("LexLeanCertification.{stem}")
}

/// The text of the library module `stem`.
///
/// # Panics
///
/// Panics when the file is unreadable.
#[must_use]
pub fn library_text(stem: &str) -> String {
    std::fs::read_to_string(
        repo_root()
            .join(format!("{LIBRARY_DIR}/{stem}.lean"))
            .as_std_path(),
    )
    .unwrap_or_else(|error| panic!("{stem}: {error}"))
}

/// The registry.
///
/// # Panics
///
/// Panics when it is unreadable or malformed.
#[must_use]
pub fn registry() -> Registry {
    let text = std::fs::read_to_string(repo_root().join(REGISTRY).as_std_path())
        .unwrap_or_else(|error| panic!("{REGISTRY}: {error}"));
    toml::from_str(&text).unwrap_or_else(|error| panic!("{REGISTRY}: {error}"))
}

/// The top-level declarations of a library module's text: structures,
/// inductives, definitions, and theorems, qualified by the library's
/// namespace.
#[must_use]
pub fn declarations(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(|line| {
            let mut words = line.split_whitespace();
            match words.next() {
                Some("theorem" | "def" | "structure" | "inductive" | "abbrev") => words
                    .next()
                    .map(|name| format!("LexLeanCertification.{name}")),
                _ => None,
            }
        })
        .collect()
}

/// One Lean source to compile.
pub struct Source {
    /// The Lean module name.
    pub module: String,
    /// Its text.
    pub text: String,
}

fn path_of(module: &str) -> String {
    format!("{}.lean", module.replace('.', "/"))
}

/// Compile `sources` in order, the olean of each found by those after it, in
/// a directory below the build directory named by the hash of the sources and
/// of the search directories' names, so that an unchanged environment is
/// compiled once per machine and never twice in a run. Publication is by
/// renaming a finished directory, as for the shipped environment.
fn cached(label: &str, sources: &[Source], upstream: &[PathBuf]) -> PathBuf {
    let mut digest = String::new();
    for source in sources {
        digest.push_str(&format!("{}\n{}\n", source.module, source.text));
    }
    for directory in upstream {
        digest.push_str(&format!("{}\n", directory.display()));
    }
    digest.push_str(lexlean::LEAN_TOOLCHAIN);
    let key = lexlean::artifact::content_id::Sha256Digest::of(digest.as_bytes()).to_hex();
    let base = std::env::current_exe()
        .ok()
        .and_then(|exe| {
            exe.ancestors()
                .nth(2)
                .map(|debug| debug.join("lexlean-certification-build"))
        })
        .unwrap_or_else(|| std::env::temp_dir().join("lexlean-certification-build"));
    let finished = base.join(format!("{label}-{key}"));
    if finished.join("ready").is_file() {
        return finished.join("build");
    }
    std::fs::create_dir_all(&base).expect("the build's directory");
    let building = base.join(format!("{label}-{key}.{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&building);
    /// Removes the unfinished directory when a module is rejected.
    struct Unfinished(PathBuf);
    impl Drop for Unfinished {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let _unfinished = Unfinished(building.clone());
    let mut search = vec![building.join("build")];
    search.extend(upstream.iter().cloned());
    for source in sources {
        let path = path_of(&source.module);
        let file = building.join("src").join(&path);
        let olean = building
            .join("build")
            .join(path.trim_end_matches(".lean"))
            .with_extension("olean");
        for target in [&file, &olean] {
            std::fs::create_dir_all(target.parent().expect("a parent")).expect("mkdir");
        }
        std::fs::write(&file, &source.text).expect("write");
        let output = pinned_with("lean", &building, &search)
            .arg("-o")
            .arg(&olean)
            .arg(&file)
            .output()
            .expect("lean runs");
        assert!(
            output.status.success(),
            "the module `{}` was rejected by Lean:\n{}",
            source.module,
            joined(&output)
        );
    }
    std::fs::write(building.join("ready"), &key).expect("write");
    if std::fs::rename(&building, &finished).is_err() {
        std::fs::remove_dir_all(&building).expect("remove the duplicate build");
    }
    finished.join("build")
}

/// The compiler project's generated modules this library is stated against,
/// from its rendered build.
fn generated() -> Vec<Source> {
    let project = P::compiler();
    let rendered = support::rendered(&project);
    GENERATED
        .iter()
        .map(|module| {
            let text = rendered
                .modules
                .iter()
                .find(|candidate| candidate.lean_module == *module)
                .unwrap_or_else(|| panic!("the compiler renders {module}"))
                .lean_text
                .clone();
            Source {
                module: (*module).to_owned(),
                text,
            }
        })
        .collect()
}

/// The build directory with the compiler project's generated `Gnaf`,
/// `GnafFixtures`, `Certification`, and `GnafCertificates` compiled over the
/// shipped calculus modules.
///
/// # Panics
///
/// Panics when Lean rejects a generated module.
#[must_use]
pub fn environment() -> &'static PathBuf {
    static ENVIRONMENT: OnceLock<PathBuf> = OnceLock::new();
    ENVIRONMENT.get_or_init(|| {
        let shipped = shipped().build.clone();
        cached("generated", &generated(), &[shipped])
    })
}

/// The build directory with the library compiled over [`environment`].
///
/// # Panics
///
/// Panics when Lean rejects a library module.
#[must_use]
pub fn library() -> &'static PathBuf {
    static LIBRARY: OnceLock<PathBuf> = OnceLock::new();
    LIBRARY.get_or_init(|| {
        let sources: Vec<Source> = MODULES
            .iter()
            .map(|stem| Source {
                module: module_name(stem),
                text: library_text(stem),
            })
            .collect();
        cached(
            "library",
            &sources,
            &[environment().clone(), shipped().build.clone()],
        )
    })
}

/// The directories Lean searches for the library and what it imports.
#[must_use]
pub fn search() -> Vec<PathBuf> {
    vec![
        library().clone(),
        environment().clone(),
        shipped().build.clone(),
    ]
}

/// Compile one extra module over `search`, in a directory of its own below
/// `root`; the module's name is `module` and its output is Lean's.
#[must_use]
pub fn compile(root: &Path, module: &str, text: &str, search: &[PathBuf]) -> std::process::Output {
    let path = path_of(module);
    let file = root.join("src").join(&path);
    let olean = root
        .join("build")
        .join(path.trim_end_matches(".lean"))
        .with_extension("olean");
    for target in [&file, &olean] {
        std::fs::create_dir_all(target.parent().expect("a parent")).expect("mkdir");
    }
    std::fs::write(&file, text).expect("write");
    let mut directories = vec![root.join("build")];
    directories.extend(search.iter().cloned());
    pinned_with("lean", root, &directories)
        .arg("-o")
        .arg(&olean)
        .arg(&file)
        .output()
        .expect("lean runs")
}

/// Replay `module` through `leanchecker`, over the directories of `search`
/// and `root`'s own build.
#[must_use]
pub fn replay(root: &Path, module: &str, search: &[PathBuf]) -> std::process::Output {
    let mut directories = vec![root.join("build")];
    directories.extend(search.iter().cloned());
    pinned_with("leanchecker", root, &directories)
        .arg(module)
        .output()
        .expect("leanchecker runs")
}

/// The axioms a Lean `#print axioms` listing reports, by declaration.
#[must_use]
pub fn printed_axioms(output: &str) -> BTreeMap<String, Vec<String>> {
    let mut out = BTreeMap::new();
    for line in output.lines() {
        let Some(rest) = line.strip_prefix('\'') else {
            continue;
        };
        let Some((name, tail)) = rest.split_once('\'') else {
            continue;
        };
        let tail = tail.trim();
        let axioms = if tail == "does not depend on any axioms" {
            Vec::new()
        } else if let Some(list) = tail.strip_prefix("depends on axioms: [") {
            let mut axioms: Vec<String> = list
                .trim_end_matches(']')
                .split(',')
                .map(|axiom| axiom.trim().to_owned())
                .filter(|axiom| !axiom.is_empty())
                .collect();
            axioms.sort();
            axioms
        } else {
            continue;
        };
        out.insert(name.to_owned(), axioms);
    }
    out
}
