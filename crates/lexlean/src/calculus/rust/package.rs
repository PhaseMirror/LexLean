//! Rust packages (SPEC.md §17.16): a target program rendered in a profile,
//! with exported functions, a Cargo manifest, and provenance binding the
//! bytes to the program, the runtime, and LexLean's compiler semantics.
//!
//! A package is described by a closed manifest (`lexlean/rust-package/1`).
//! Every exported name, parameter passing mode, and declared failure mode is
//! checked against the program before anything is rendered, so a package
//! whose interface misstates the program it realizes is refused, never
//! repaired.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use super::super::{Program, Ty};
use super::ast::{self, Block, Callee, Expr, Ident, ItemDef, Pat, Type};
use super::lower::{index, Lowering};
use super::runtime::{self, Item};
use super::{elements, validate, Profile};
use crate::artifact::content_id::Sha256Digest;

/// The schema tag of a package manifest.
pub const MANIFEST_SPEC: &str = "lexlean/rust-package/1";

/// The schema tag of a package's provenance.
pub const PROVENANCE_SPEC: &str = "lexlean/rust-provenance/1";

/// How an exported function takes one parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Passing {
    /// By value: the caller gives up the value.
    Own,
    /// By shared reference: the function takes its own copy of the handle.
    Borrow,
    /// By copy: only for a type that is `Copy`.
    Copy,
}

/// Whether an exported function can fail.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Errors {
    /// It cannot fail and returns its value.
    None,
    /// It can overflow and returns `R<T>`.
    Overflow,
}

/// One exported function.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Export {
    /// The program function it exports.
    pub function: u64,
    /// Its Rust name.
    pub name: String,
    /// How it takes each parameter.
    pub parameters: Vec<Passing>,
    /// Whether it can fail.
    pub errors: Errors,
}

/// A package manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub spec: String,
    /// The crate name.
    pub name: String,
    /// The crate version.
    pub version: String,
    /// The machine profile: `rust-core` or `rust-std`.
    pub profile: String,
    /// The target program the package realizes.
    pub program: Program,
    /// The exported functions.
    pub exports: Vec<Export>,
    /// The identities of the LexLean semantic objects the program realizes,
    /// as lowercase SHA-256 hex, strictly ascending.
    pub sources: Vec<String>,
}

/// A rendered package: its files by path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Package {
    pub files: BTreeMap<String, Vec<u8>>,
}

fn fail<T>(reason: impl Into<String>) -> Result<T, String> {
    Err(reason.into())
}

/// Every keyword of Rust 2021: strict, reserved, and weak.
const KEYWORDS: &[&str] = &[
    "as",
    "async",
    "await",
    "break",
    "const",
    "continue",
    "crate",
    "dyn",
    "else",
    "enum",
    "extern",
    "false",
    "fn",
    "for",
    "if",
    "impl",
    "in",
    "let",
    "loop",
    "match",
    "mod",
    "move",
    "mut",
    "pub",
    "ref",
    "return",
    "self",
    "Self",
    "static",
    "struct",
    "super",
    "trait",
    "true",
    "try",
    "type",
    "unsafe",
    "use",
    "where",
    "while",
    "abstract",
    "become",
    "box",
    "do",
    "final",
    "macro",
    "override",
    "priv",
    "typeof",
    "unsized",
    "virtual",
    "yield",
    "union",
    "macro_rules",
    "gen",
    "raw",
    "safe",
];

/// Crate names a package may not take: the sysroot crates and the harness.
const RESERVED_CRATES: &[&str] = &["std", "core", "alloc", "proc_macro", "test", "harness"];

fn snake(name: &str) -> bool {
    let mut characters = name.chars();
    characters
        .next()
        .is_some_and(|first| first.is_ascii_lowercase())
        && characters.all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_'
        })
        && !name.contains("__")
        && !name.ends_with('_')
        && name.len() <= 64
}

/// Why an exported name would collide with a name the rendering or the
/// runtime declares, or would not be a plain Rust identifier.
fn collision(name: &str) -> Option<String> {
    if !snake(name) {
        return Some(format!(
            "`{name}` is not a lowercase snake-case Rust identifier of at most 64 characters"
        ));
    }
    if KEYWORDS.contains(&name) {
        return Some(format!("`{name}` is a Rust keyword"));
    }
    let mut characters = name.chars();
    if characters
        .next()
        .is_some_and(|first| ast::GENERATED_PREFIXES.contains(&first))
        && !name[1..].is_empty()
        && name[1..]
            .chars()
            .all(|character| character.is_ascii_digit())
    {
        return Some(format!("`{name}` has the shape of a generated name"));
    }
    if runtime::ROOT_NAMES.contains(&name) {
        return Some(format!("`{name}` is declared by the runtime"));
    }
    let runtime_paths: BTreeSet<String> = Item::all()
        .into_iter()
        .map(|item| {
            let path = item.path();
            path.split("::").next().unwrap_or(&path).to_owned()
        })
        .collect();
    if runtime_paths.contains(name) {
        return Some(format!("`{name}` is declared by the runtime"));
    }
    None
}

/// Whether a type holds a function value anywhere.
fn holds_function(ty: &Ty) -> bool {
    match ty {
        Ty::Fn { .. } => true,
        Ty::Option { value } | Ty::List { element: value } => holds_function(value),
        Ty::Result {
            ok: left,
            error: right,
        }
        | Ty::Pair { left, right } => holds_function(left) || holds_function(right),
        Ty::Unit
        | Ty::Bool
        | Ty::Nat
        | Ty::Int
        | Ty::Fixed { .. }
        | Ty::String
        | Ty::Bytes
        | Ty::Ordering
        | Ty::Adt { .. } => false,
    }
}

impl Manifest {
    /// Read a manifest from JSON.
    ///
    /// # Errors
    ///
    /// Returns the reason the bytes are not a manifest.
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        crate::artifact::canonical_json::Json::parse(bytes)
            .map_err(|error| format!("package manifest is malformed: {error}"))?;
        serde_json::from_slice(bytes)
            .map_err(|error| format!("package manifest is malformed: {error}"))
    }

    /// The canonical file bytes.
    ///
    /// # Panics
    ///
    /// Panics only if `serde_json` produces text the canonical JSON parser
    /// rejects, which would be an internal invariant failure.
    #[must_use]
    pub fn to_file_bytes(&self) -> Vec<u8> {
        let text = serde_json::to_string(self).expect("manifest serializes");
        crate::artifact::canonical_json::Json::parse(text.as_bytes())
            .expect("manifest is JSON")
            .to_file_bytes()
    }
}

/// The export wrappers of `manifest`, checked against the program.
fn exports(manifest: &Manifest, lowering: &mut Lowering<'_>) -> Result<Vec<ItemDef>, String> {
    let program = lowering.program;
    if manifest.exports.is_empty() {
        return fail("a package exports at least one function");
    }
    let mut names = BTreeSet::new();
    let mut out = Vec::new();
    for export in &manifest.exports {
        let name = &export.name;
        if let Some(reason) = collision(name) {
            return fail(format!("identifier collision: export {reason}"));
        }
        if !names.insert(name.clone()) {
            return fail(format!("identifier collision: `{name}` is exported twice"));
        }
        let function = program
            .functions
            .get(index(export.function)?)
            .ok_or_else(|| {
                format!(
                    "export `{name}` names function {}, which is not declared",
                    export.function
                )
            })?;
        if export.parameters.len() != function.types.len() {
            return fail(format!(
                "export `{name}` passes {} parameters; function {} takes {}",
                export.parameters.len(),
                export.function,
                function.types.len()
            ));
        }
        if function
            .types
            .iter()
            .chain([&function.result])
            .any(holds_function)
        {
            return fail(format!(
                "unsupported type: the boundary of export `{name}` holds a function value, which no Rust caller can construct"
            ));
        }
        let fallible = lowering
            .fallible
            .get(index(export.function)?)
            .copied()
            .unwrap_or(false);
        match (export.errors, fallible) {
            (Errors::None, true) => {
                return fail(format!(
                    "arithmetic mismatch: export `{name}` declares no errors, but function {} can overflow",
                    export.function
                ))
            }
            (Errors::Overflow, false) => {
                return fail(format!(
                    "arithmetic mismatch: export `{name}` declares overflow, but function {} cannot overflow",
                    export.function
                ))
            }
            _ => {}
        }
        let mut parameters = Vec::new();
        let mut args = Vec::new();
        for (position, (passing, ty)) in export.parameters.iter().zip(&function.types).enumerate() {
            let lowered = lowering.ty(ty)?;
            let param = Ident::Param(position as u64);
            let (ty, arg) = match passing {
                Passing::Own => (lowered, Expr::Move(param.clone())),
                Passing::Borrow if lowered.copy() => {
                    (Type::Ref(Box::new(lowered)), Expr::Deref(param.clone()))
                }
                Passing::Borrow => (Type::Ref(Box::new(lowered)), Expr::Clone(param.clone())),
                Passing::Copy if lowered.copy() => (lowered, Expr::Copy(param.clone())),
                Passing::Copy => {
                    return fail(format!(
                        "ownership mismatch: export `{name}` copies parameter {position}, whose type {ty:?} is not Copy"
                    ))
                }
            };
            parameters.push((Pat::Bind(param), ty));
            args.push(arg);
        }
        let result = lowering.ty(&function.result)?;
        out.push(ItemDef::Function {
            name: Ident::Export(name.clone()),
            parameters,
            result: if fallible {
                Type::Fallible(Box::new(result))
            } else {
                result
            },
            body: Block::of(Expr::Call {
                callee: Callee::Function(export.function),
                args,
                propagate: false,
            }),
        });
    }
    Ok(out)
}

fn sha256(bytes: &[u8]) -> String {
    Sha256Digest::of(bytes).to_hex()
}

/// The Cargo manifest of a package. Its lint table is the package's lint
/// gate: rustc's warnings and every Clippy lint of the default set are
/// denied, except those whose advice would make the rendering depart from
/// the calculus. `type_complexity` asks for type aliases, but a generated
/// type is exactly its calculus type and an alias would be a name with no
/// calculus counterpart; `manual_unwrap_or` and `manual_unwrap_or_default`
/// ask for an idiom in place of the program's own match.
fn cargo_toml(manifest: &Manifest) -> String {
    format!(
        "[package]\nname = \"{}\"\nversion = \"{}\"\nedition = \"2021\"\npublish = false\n\n[lib]\npath = \"src/lib.rs\"\n\n[lints.rust]\nunsafe_code = \"forbid\"\nwarnings = \"deny\"\n\n[lints.clippy]\nall = {{ level = \"deny\", priority = -1 }}\ntype_complexity = \"allow\"\nmanual_unwrap_or = \"allow\"\nmanual_unwrap_or_default = \"allow\"\n",
        manifest.name, manifest.version
    )
}

/// A package's checked crate, its canonical program, and its profile.
///
/// # Errors
///
/// As [`package`].
pub fn lower(manifest: &Manifest) -> Result<(ast::Crate, Program, Profile), String> {
    if manifest.spec != MANIFEST_SPEC {
        return fail(format!(
            "package manifest has spec `{}`, expected `{MANIFEST_SPEC}`",
            manifest.spec
        ));
    }
    if !snake(&manifest.name)
        || KEYWORDS.contains(&manifest.name.as_str())
        || RESERVED_CRATES.contains(&manifest.name.as_str())
    {
        return fail(format!(
            "identifier collision: `{}` is not an available crate name",
            manifest.name
        ));
    }
    let version_parts: Vec<&str> = manifest.version.split('.').collect();
    if version_parts.len() != 3
        || version_parts.iter().any(|part| {
            part.is_empty()
                || !part.chars().all(|character| character.is_ascii_digit())
                || (part.len() > 1 && part.starts_with('0'))
        })
    {
        return fail(format!(
            "`{}` is not a version of three canonical decimals",
            manifest.version
        ));
    }
    let profile = Profile::named(&manifest.profile)
        .ok_or_else(|| format!("`{}` is not a Rust profile", manifest.profile))?;
    if manifest.sources.iter().any(|source| {
        source.len() != 64
            || !source
                .chars()
                .all(|character| character.is_ascii_digit() || ('a'..='f').contains(&character))
    }) {
        return fail("every source is a lowercase SHA-256 hex identity");
    }
    if manifest.sources.windows(2).any(|pair| pair[0] >= pair[1]) {
        return fail("sources are strictly ascending");
    }
    let program = manifest.program.canonical()?;
    let mut lowering = Lowering::new(&program, profile).map_err(|reason| {
        if reason.contains("requires heap allocation") {
            format!("hidden allocation: {reason}")
        } else {
            reason
        }
    })?;
    let mut krate = lowering.lower().map_err(|reason| {
        if reason.contains("requires heap allocation") {
            format!("hidden allocation: {reason}")
        } else {
            reason
        }
    })?;
    krate.items.extend(exports(manifest, &mut lowering)?);
    validate::validate(&krate)?;
    let mut realized = elements(&program, &lowering);
    realized.insert("export".to_owned());
    validate::correspond(&krate, &realized)?;
    Ok((krate, program, profile))
}

/// Render a package.
///
/// # Errors
///
/// Returns the reason the manifest is malformed, names an invalid program,
/// misstates the program's interface (an identifier collision, an ownership
/// mismatch, an unsupported boundary type, or an arithmetic mismatch), or
/// needs heap allocation its profile does not provide.
pub fn package(manifest: &Manifest) -> Result<Package, String> {
    let (krate, program, profile) = lower(manifest)?;
    let library = ast::print(&krate).into_bytes();
    let cargo = cargo_toml(manifest).into_bytes();
    let runtime = match profile {
        Profile::Core => runtime::CORE.to_owned(),
        Profile::Std => format!("{}{}", runtime::CORE, runtime::STD),
    };
    let provenance = serde_json::json!({
        "spec": PROVENANCE_SPEC,
        "name": manifest.name,
        "version": manifest.version,
        "profile": profile.target(),
        "program": program.id()?.to_hex(),
        "compiler_semantics": crate::compiler_semantics_id_for(crate::LANGUAGE_1_2).to_hex(),
        "runtime": sha256(runtime.as_bytes()),
        "sources": manifest.sources,
        "exports": manifest.exports.iter().map(|export| serde_json::json!({
            "name": export.name,
            "function": export.function,
        })).collect::<Vec<_>>(),
        "files": {
            "Cargo.toml": sha256(&cargo),
            "src/lib.rs": sha256(&library),
        },
    });
    let provenance =
        crate::artifact::canonical_json::Json::parse(provenance.to_string().as_bytes())
            .map_err(|error| format!("provenance is not canonical JSON: {error}"))?
            .to_file_bytes();
    let mut files = BTreeMap::new();
    files.insert("Cargo.toml".to_owned(), cargo);
    files.insert("src/lib.rs".to_owned(), library);
    files.insert("provenance.json".to_owned(), provenance);
    Ok(Package { files })
}
