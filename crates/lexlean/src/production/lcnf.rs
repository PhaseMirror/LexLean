//! Named-root extraction through Lean's compiler front end (SPEC.md §22.10).
//!
//! Lean is the authority for how a verified generated declaration compiles:
//! the pinned adapter `language/lcnf-1.2/extract.lean` asks Lean for the
//! base-phase LCNF of every definition a production root reaches and prints
//! it, and nothing else. This module owns everything LexLean decides about
//! that answer: the driver that pins the adapter and probes every authority
//! signature, the closed reading of the adapter's output, the fail-closed
//! rejection classes, the comparison against the production-eligibility
//! closure, and the canonical compiler input with its content identity. No
//! optimization policy lives on either side of the boundary; the adapter runs
//! no LCNF pass after translation.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use super::ModuleReport;
use crate::artifact::content_id::Sha256Digest;

/// The embedded adapter, hashed into the language-1.2 compiler semantics ID.
pub const ADAPTER_PATH: &str = "language/lcnf-1.2/extract.lean";

/// The embedded authority registry, hashed into the same ID.
pub const AUTHORITY_PATH: &str = "language/lcnf-1.2/authority.toml";

/// The tag of the adapter's raw output.
pub const EXTRACTION_SPEC: &str = "lexlean/lcnf-extraction/1";

/// The tag of the canonical compiler input.
pub const INPUT_SPEC: &str = "lexlean/compiler-input/1";

/// The Lean namespaces of the fixed runtimes every generated module may
/// carry, below the module's own name.
pub const RUNTIME_NAMESPACES: [&str; 2] = ["LexLeanRuntime", "LexLeanCollections"];

/// One Lean operation the adapter calls.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthorityCall {
    pub name: String,
    pub signature: String,
    pub source: String,
    pub source_sha256: String,
    pub role: String,
}

/// One Lean data type the adapter matches exhaustively.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthorityType {
    pub name: String,
    pub source: String,
    pub source_sha256: String,
    pub constructors: Vec<String>,
}

/// The closed compiler-front-end interface.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Authority {
    pub spec: String,
    pub authority: String,
    pub lean_version: String,
    pub lean_githash: String,
    pub call: Vec<AuthorityCall>,
    #[serde(rename = "type")]
    pub types: Vec<AuthorityType>,
}

fn embedded_text(path: &str) -> Result<&'static str, String> {
    crate::embedded::FILES
        .iter()
        .find(|(candidate, _)| *candidate == path)
        .and_then(|(_, bytes)| std::str::from_utf8(bytes).ok())
        .ok_or_else(|| format!("embedded `{path}` is missing"))
}

/// The embedded adapter source.
///
/// # Errors
///
/// Returns the reason the embedded adapter is missing.
pub fn adapter() -> Result<&'static str, String> {
    embedded_text(ADAPTER_PATH)
}

/// The embedded authority registry.
///
/// # Errors
///
/// Returns the reason the embedded registry is missing or malformed; the
/// conformance suite parses it, so a malformed registry never ships.
pub fn authority() -> Result<&'static Authority, String> {
    static AUTHORITY: OnceLock<Result<Authority, String>> = OnceLock::new();
    AUTHORITY
        .get_or_init(|| {
            let text = embedded_text(AUTHORITY_PATH)?;
            let authority: Authority =
                toml::from_str(text).map_err(|error| format!("{AUTHORITY_PATH}: {error}"))?;
            if authority.spec != "lexlean/lcnf-authority/1" {
                return Err(format!(
                    "{AUTHORITY_PATH}: unsupported spec `{}`",
                    authority.spec
                ));
            }
            let mut seen = BTreeSet::new();
            for call in &authority.call {
                if !seen.insert(call.name.clone()) {
                    return Err(format!("{AUTHORITY_PATH}: duplicate call `{}`", call.name));
                }
            }
            Ok(authority)
        })
        .as_ref()
        .map_err(Clone::clone)
}

/// A Lean name literal that survives any segment spelling: the name is
/// rebuilt from its string, so no source quoting rule is involved.
fn name_literal(name: &str) -> String {
    let escaped: String = name
        .chars()
        .flat_map(|character| match character {
            '"' => vec!['\\', '"'],
            '\\' => vec!['\\', '\\'],
            other => vec![other],
        })
        .collect();
    format!("\"{escaped}\".toName")
}

/// The generated extraction module: fixed header, one signature probe per
/// authority call, the pinned adapter, and one command naming the roots.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Driver {
    /// The reserved module name (`LexLeanExtract.X<hex32>`).
    pub name: String,
    pub text: String,
    /// The 1-based line of each call probe, by call name.
    pub probe_lines: BTreeMap<usize, String>,
    /// The 1-based lines the pinned adapter occupies.
    pub adapter_lines: (usize, usize),
}

/// The reserved extraction module name for a semantic ID prefix.
#[must_use]
pub fn driver_name(semantic_hex32: &str) -> String {
    format!("LexLeanExtract.X{semantic_hex32}")
}

/// The runtime namespaces of a set of generated modules.
#[must_use]
pub fn runtime_namespaces(modules: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    for module in modules {
        for namespace in RUNTIME_NAMESPACES {
            out.push(format!("{module}.{namespace}"));
        }
    }
    out
}

/// Generate the extraction module.
///
/// # Errors
///
/// Returns the reason the embedded adapter or registry is unavailable.
pub fn driver(
    semantic_hex32: &str,
    roots: &[String],
    modules: &[String],
) -> Result<Driver, String> {
    let authority = authority()?;
    let adapter = adapter()?;
    let mut text = String::from("module\npublic meta import Lean\n");
    // `import all` exposes the private kernel values, so a proof reached
    // only through a compiler-generated helper is still recorded as erased.
    for module in modules {
        text.push_str(&format!("import all {module}\n"));
    }
    let mut probe_lines = BTreeMap::new();
    for call in &authority.call {
        probe_lines.insert(text.lines().count() + 1, call.name.clone());
        text.push_str(&format!(
            "meta example : {} := @{}\n",
            call.signature, call.name
        ));
    }
    let adapter_start = text.lines().count() + 1;
    text.push_str(adapter);
    if !text.ends_with('\n') {
        text.push('\n');
    }
    let adapter_end = text.lines().count();
    let list = |names: &[String]| {
        format!(
            "#[{}]",
            names
                .iter()
                .map(|name| name_literal(name))
                .collect::<Vec<_>>()
                .join(", ")
        )
    };
    text.push_str(&format!(
        "#eval LexLeanExtract.main {} {} {}\n",
        list(roots),
        list(modules),
        list(&runtime_namespaces(modules))
    ));
    Ok(Driver {
        name: driver_name(semantic_hex32),
        text,
        probe_lines,
        adapter_lines: (adapter_start, adapter_end),
    })
}

/// A rejected extraction: drift of the pinned authority, or a fail-closed
/// rejection of the extracted program.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Rejection {
    /// `LLV7012`: the authority no longer matches its registry.
    Drift(String),
    /// `LLV7011`: the extraction is not an admissible compiler input.
    Rejected(String),
}

/// Classify a failed extraction process from its Lean messages: an error on
/// a probe line or inside the pinned adapter is drift of the authority; an
/// error the adapter raised names an unknown or unresolved constant.
#[must_use]
pub fn classify_failure(driver: &Driver, output: &str) -> Rejection {
    let messages = crate::verify::parse_lean_messages(output);
    for message in &messages {
        if let Some(call) = driver.probe_lines.get(&message.line) {
            return Rejection::Drift(format!(
                "the signature of `{call}` no longer matches the pinned registry: {}",
                message.message
            ));
        }
    }
    if let Some(raised) = output
        .lines()
        .find_map(|line| line.split_once("lexlean-extract: "))
    {
        return Rejection::Rejected(raised.1.trim().to_owned());
    }
    for message in &messages {
        if (driver.adapter_lines.0..=driver.adapter_lines.1).contains(&message.line) {
            return Rejection::Drift(format!(
                "the pinned extraction adapter no longer elaborates: {}",
                message.message
            ));
        }
    }
    Rejection::Rejected(format!(
        "the extraction process failed: {}",
        output.trim_end()
    ))
}

/// An LCNF type.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum LcnfType {
    Erased,
    Any,
    Const {
        name: String,
    },
    App {
        head: Box<LcnfType>,
        arguments: Vec<LcnfType>,
    },
    Arrow {
        domain: Box<LcnfType>,
        codomain: Box<LcnfType>,
    },
    Fvar {
        id: String,
    },
    Bvar {
        index: u64,
    },
    Sort,
    Unsupported {
        expression: String,
    },
}

/// An LCNF argument.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum LcnfArg {
    Erased,
    Fvar {
        id: String,
    },
    Type {
        #[serde(rename = "type")]
        ty: LcnfType,
    },
}

/// An LCNF literal; values are decimal strings.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum LcnfLiteral {
    Nat { value: String },
    String { value: String },
    Uint8 { value: String },
    Uint16 { value: String },
    Uint32 { value: String },
    Uint64 { value: String },
    Usize { value: String },
}

/// An LCNF let value.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum LcnfValue {
    Literal {
        literal: LcnfLiteral,
    },
    Erased,
    Projection {
        type_name: String,
        index: u64,
        value: String,
    },
    Const {
        name: String,
        arguments: Vec<LcnfArg>,
    },
    Apply {
        function: String,
        arguments: Vec<LcnfArg>,
    },
}

/// An LCNF parameter.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LcnfParam {
    pub id: String,
    #[serde(rename = "type")]
    pub ty: LcnfType,
    pub borrow: bool,
}

/// An LCNF local function or join point.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LcnfFunDecl {
    pub id: String,
    pub parameters: Vec<LcnfParam>,
    #[serde(rename = "type")]
    pub ty: LcnfType,
    pub value: LcnfCode,
}

/// An LCNF case alternative.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum LcnfAlt {
    Constructor {
        constructor: String,
        parameters: Vec<LcnfParam>,
        code: LcnfCode,
    },
    Default {
        code: LcnfCode,
    },
}

/// LCNF code.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum LcnfCode {
    Let {
        id: String,
        #[serde(rename = "type")]
        ty: LcnfType,
        value: LcnfValue,
        body: Box<LcnfCode>,
    },
    Fun {
        declaration: Box<LcnfFunDecl>,
        body: Box<LcnfCode>,
    },
    Join {
        declaration: Box<LcnfFunDecl>,
        body: Box<LcnfCode>,
    },
    Jump {
        target: String,
        arguments: Vec<LcnfArg>,
    },
    Cases {
        type_name: String,
        result_type: LcnfType,
        discriminant: String,
        alternatives: Vec<LcnfAlt>,
    },
    Return {
        id: String,
    },
    Unreachable {
        #[serde(rename = "type")]
        ty: LcnfType,
    },
}

/// An LCNF declaration body.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum LcnfDeclValue {
    Code { code: LcnfCode },
    Extern,
}

/// One closure member as the adapter reports it: either a code-generating
/// definition with its LCNF, or a constant that is not one, kept so the
/// host names the reason it is refused.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawDeclaration {
    name: String,
    kind: String,
    computable: bool,
    generates_code: bool,
    #[serde(default)]
    safe: Option<bool>,
    #[serde(default)]
    recursive: Option<bool>,
    #[serde(default)]
    level_parameters: Option<Vec<String>>,
    #[serde(default, rename = "type")]
    ty: Option<LcnfType>,
    #[serde(default)]
    parameters: Option<Vec<LcnfParam>>,
    #[serde(default)]
    value: Option<LcnfDeclValue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LcnfConstructor {
    pub name: String,
    pub index: u64,
    pub parameters: u64,
    pub fields: u64,
    #[serde(rename = "type")]
    pub ty: LcnfType,
}

/// A project inductive the closure realizes.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LcnfInductive {
    pub name: String,
    pub parameters: u64,
    pub indices: u64,
    pub recursive: bool,
    pub constructors: Vec<LcnfConstructor>,
}

/// A constant the closure uses but does not define.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LcnfExternal {
    pub name: String,
    pub kind: String,
    pub module: String,
    pub computable: bool,
    pub generates_code: bool,
}

/// One root's closure as Lean's compiler reaches it.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LcnfClosure {
    pub root: String,
    pub declarations: Vec<String>,
    pub erased: Vec<String>,
    pub internal_erased: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LeanIdentity {
    pub version: String,
    pub githash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawExtraction {
    spec: String,
    lean: LeanIdentity,
    roots: Vec<String>,
    closures: Vec<LcnfClosure>,
    declarations: Vec<RawDeclaration>,
    inductives: Vec<LcnfInductive>,
    externals: Vec<LcnfExternal>,
    erased: Vec<String>,
    internal_erased: Vec<String>,
}

/// A verified code-generating definition in canonical form.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LcnfDeclaration {
    pub name: String,
    pub recursive: bool,
    pub level_parameters: Vec<String>,
    #[serde(rename = "type")]
    pub ty: LcnfType,
    pub parameters: Vec<LcnfParam>,
    pub code: LcnfCode,
}

/// The canonical compiler input (`lexlean/compiler-input/1`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CompilerInput {
    pub spec: String,
    pub lean: LeanIdentity,
    pub roots: Vec<String>,
    pub closures: Vec<LcnfClosure>,
    pub declarations: Vec<LcnfDeclaration>,
    pub inductives: Vec<LcnfInductive>,
    pub externals: Vec<LcnfExternal>,
    pub erased: Vec<String>,
}

impl CompilerInput {
    /// The canonical file bytes.
    ///
    /// # Panics
    ///
    /// Panics only if `serde_json` produces text the canonical JSON parser
    /// rejects, which would be an internal invariant failure.
    #[must_use]
    pub fn to_file_bytes(&self) -> Vec<u8> {
        let text = serde_json::to_string(self).expect("compiler input serializes");
        crate::artifact::canonical_json::Json::parse(text.as_bytes())
            .expect("compiler input is JSON")
            .to_file_bytes()
    }

    /// The content identity: the SHA-256 of the canonical bytes.
    #[must_use]
    pub fn id(&self) -> Sha256Digest {
        Sha256Digest::of(&self.to_file_bytes())
    }
}

/// Renames every LCNF free variable to its first-occurrence index, so the
/// canonical input does not depend on Lean's unique-name counter.
#[derive(Default)]
struct Renamer {
    names: BTreeMap<String, String>,
}

impl Renamer {
    fn bind(&mut self, id: &mut String) -> Result<(), String> {
        let fresh = format!("v{}", self.names.len());
        if self.names.insert(id.clone(), fresh.clone()).is_some() {
            return Err(format!("LCNF variable `{id}` is bound twice"));
        }
        *id = fresh;
        Ok(())
    }

    fn reference(&self, id: &mut String) -> Result<(), String> {
        let renamed = self
            .names
            .get(id)
            .ok_or_else(|| format!("LCNF variable `{id}` is used before it is bound"))?;
        id.clone_from(renamed);
        Ok(())
    }

    fn ty(&self, ty: &mut LcnfType) -> Result<(), String> {
        match ty {
            LcnfType::Erased
            | LcnfType::Any
            | LcnfType::Const { .. }
            | LcnfType::Bvar { .. }
            | LcnfType::Sort => Ok(()),
            LcnfType::App { head, arguments } => {
                self.ty(head)?;
                arguments.iter_mut().try_for_each(|argument| self.ty(argument))
            }
            LcnfType::Arrow { domain, codomain } => {
                self.ty(domain)?;
                self.ty(codomain)
            }
            LcnfType::Fvar { id } => self.reference(id),
            LcnfType::Unsupported { expression } => Err(format!(
                "unsupported compiler form: the LCNF type `{expression}` has no closed representation"
            )),
        }
    }

    fn arg(&self, arg: &mut LcnfArg) -> Result<(), String> {
        match arg {
            LcnfArg::Erased => Ok(()),
            LcnfArg::Fvar { id } => self.reference(id),
            LcnfArg::Type { ty } => self.ty(ty),
        }
    }

    fn params(&mut self, params: &mut [LcnfParam]) -> Result<(), String> {
        for param in params {
            self.ty(&mut param.ty)?;
            self.bind(&mut param.id)?;
        }
        Ok(())
    }

    fn fun(&mut self, declaration: &mut LcnfFunDecl) -> Result<(), String> {
        self.bind(&mut declaration.id)?;
        self.params(&mut declaration.parameters)?;
        self.ty(&mut declaration.ty)?;
        self.code(&mut declaration.value)
    }

    fn code(&mut self, code: &mut LcnfCode) -> Result<(), String> {
        match code {
            LcnfCode::Let {
                id,
                ty,
                value,
                body,
            } => {
                self.ty(ty)?;
                match value {
                    LcnfValue::Literal { .. } | LcnfValue::Erased => {}
                    LcnfValue::Projection { value, .. } => self.reference(value)?,
                    LcnfValue::Const { arguments, .. } => {
                        arguments.iter_mut().try_for_each(|arg| self.arg(arg))?;
                    }
                    LcnfValue::Apply {
                        function,
                        arguments,
                    } => {
                        self.reference(function)?;
                        arguments.iter_mut().try_for_each(|arg| self.arg(arg))?;
                    }
                }
                self.bind(id)?;
                self.code(body)
            }
            LcnfCode::Fun { declaration, body } | LcnfCode::Join { declaration, body } => {
                self.fun(declaration)?;
                self.code(body)
            }
            LcnfCode::Jump { target, arguments } => {
                self.reference(target)?;
                arguments.iter_mut().try_for_each(|arg| self.arg(arg))
            }
            LcnfCode::Cases {
                result_type,
                discriminant,
                alternatives,
                ..
            } => {
                self.ty(result_type)?;
                self.reference(discriminant)?;
                for alternative in alternatives {
                    match alternative {
                        LcnfAlt::Constructor {
                            parameters, code, ..
                        } => {
                            self.params(parameters)?;
                            self.code(code)?;
                        }
                        LcnfAlt::Default { code } => self.code(code)?,
                    }
                }
                Ok(())
            }
            LcnfCode::Return { id } => self.reference(id),
            LcnfCode::Unreachable { ty } => self.ty(ty),
        }
    }
}

/// Every constant a type mentions.
fn type_constants(ty: &LcnfType, out: &mut BTreeSet<String>) {
    match ty {
        LcnfType::Const { name } => {
            out.insert(name.clone());
        }
        LcnfType::App { head, arguments } => {
            type_constants(head, out);
            for argument in arguments {
                type_constants(argument, out);
            }
        }
        LcnfType::Arrow { domain, codomain } => {
            type_constants(domain, out);
            type_constants(codomain, out);
        }
        LcnfType::Erased
        | LcnfType::Any
        | LcnfType::Fvar { .. }
        | LcnfType::Bvar { .. }
        | LcnfType::Sort
        | LcnfType::Unsupported { .. } => {}
    }
}

fn arg_constants(arg: &LcnfArg, out: &mut BTreeSet<String>) {
    match arg {
        LcnfArg::Type { ty } => type_constants(ty, out),
        LcnfArg::Erased | LcnfArg::Fvar { .. } => {}
    }
}

fn param_constants(params: &[LcnfParam], out: &mut BTreeSet<String>) {
    for param in params {
        type_constants(&param.ty, out);
    }
}

/// Every constant code mentions: its callees, constructors, matched and
/// projected types, and the types it states.
fn code_constants(code: &LcnfCode, out: &mut BTreeSet<String>) {
    match code {
        LcnfCode::Let {
            ty, value, body, ..
        } => {
            type_constants(ty, out);
            match value {
                LcnfValue::Literal { .. } | LcnfValue::Erased => {}
                LcnfValue::Projection { type_name, .. } => {
                    out.insert(type_name.clone());
                }
                LcnfValue::Const { name, arguments } => {
                    out.insert(name.clone());
                    for argument in arguments {
                        arg_constants(argument, out);
                    }
                }
                LcnfValue::Apply { arguments, .. } => {
                    for argument in arguments {
                        arg_constants(argument, out);
                    }
                }
            }
            code_constants(body, out);
        }
        LcnfCode::Fun { declaration, body } | LcnfCode::Join { declaration, body } => {
            param_constants(&declaration.parameters, out);
            type_constants(&declaration.ty, out);
            code_constants(&declaration.value, out);
            code_constants(body, out);
        }
        LcnfCode::Jump { arguments, .. } => {
            for argument in arguments {
                arg_constants(argument, out);
            }
        }
        LcnfCode::Cases {
            type_name,
            result_type,
            alternatives,
            ..
        } => {
            out.insert(type_name.clone());
            type_constants(result_type, out);
            for alternative in alternatives {
                match alternative {
                    LcnfAlt::Constructor {
                        constructor,
                        parameters,
                        code,
                    } => {
                        out.insert(constructor.clone());
                        param_constants(parameters, out);
                        code_constants(code, out);
                    }
                    LcnfAlt::Default { code } => code_constants(code, out),
                }
            }
        }
        LcnfCode::Return { .. } => {}
        LcnfCode::Unreachable { ty } => type_constants(ty, out),
    }
}

/// The definitions of a set of eligibility reports, per qualified root.
fn eligibility_closures(
    reports: &[&ModuleReport],
) -> BTreeMap<String, (BTreeSet<String>, BTreeSet<String>)> {
    let mut out = BTreeMap::new();
    for report in reports {
        for root in &report.roots {
            let declarations: BTreeSet<String> = root
                .runtime
                .iter()
                .map(|member| member.declaration.clone())
                .collect();
            out.insert(root.root.clone(), (declarations, root.erased.clone()));
        }
    }
    out
}

fn sorted_unique(values: &[String]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

/// Whether an external constant is one LexLean may hand to a compiler: a
/// computable constant of Lean's own `Init` library, or a member of a
/// generated module's fixed runtime namespaces.
fn admissible_external(external: &LcnfExternal, modules: &BTreeSet<String>) -> Result<(), String> {
    let runtime = modules.contains(&external.module)
        && RUNTIME_NAMESPACES.iter().any(|namespace| {
            external
                .name
                .starts_with(&format!("{}.{namespace}.", external.module))
        });
    let core = external.module == "Init" || external.module.starts_with("Init.");
    if !runtime && !core {
        return Err(format!(
            "unresolved dependency: `{}` ({} from `{}`) is neither a Lean core constant nor a LexLean runtime member",
            external.name, external.kind, external.module
        ));
    }
    match external.kind.as_str() {
        "inductive" | "constructor" => Ok(()),
        "definition" if external.computable && external.generates_code => Ok(()),
        "definition" => Err(format!(
            "`{}` is noncomputable or generates no code, so no compiler can realize it",
            external.name
        )),
        other => Err(format!(
            "`{}` is {} and has no executable content",
            external.name,
            article(other)
        )),
    }
}

fn article(kind: &str) -> String {
    match kind {
        "axiom" | "opaque" | "unsafe-definition" | "inductive" => format!("an {kind}"),
        other => format!("a {other}"),
    }
}

/// Validate the adapter's output and produce the canonical compiler input.
///
/// `roots` are the qualified production roots in request order, `modules`
/// the generated modules, and `reports` the eligibility reports whose
/// closures Lean's must equal.
///
/// # Errors
///
/// Returns the first drift or rejection found; nothing partial is produced.
pub fn compiler_input(
    output: &str,
    roots: &[String],
    modules: &[String],
    reports: &[&ModuleReport],
) -> Result<CompilerInput, Rejection> {
    let authority = authority().map_err(Rejection::Drift)?;
    let payload = output.trim_end_matches('\n');
    if payload.contains('\n') {
        return Err(Rejection::Rejected(
            "the extraction printed more than its single JSON record".to_owned(),
        ));
    }
    let raw: RawExtraction = serde_json::from_str(payload).map_err(|error| {
        Rejection::Rejected(format!("the extraction record is malformed: {error}"))
    })?;
    if raw.spec != EXTRACTION_SPEC {
        return Err(Rejection::Rejected(format!(
            "the extraction record has spec `{}`",
            raw.spec
        )));
    }
    if raw.lean.version != authority.lean_version || raw.lean.githash != authority.lean_githash {
        return Err(Rejection::Drift(format!(
            "Lean reports version {} at {}, but the authority is pinned to {} at {}",
            raw.lean.version, raw.lean.githash, authority.lean_version, authority.lean_githash
        )));
    }
    if raw.roots != roots {
        return Err(Rejection::Rejected(format!(
            "the extraction answered roots {:?} for requested roots {roots:?}",
            raw.roots
        )));
    }
    let module_set: BTreeSet<String> = modules.iter().cloned().collect();
    let mut declarations = Vec::new();
    let mut defined = BTreeSet::new();
    for declaration in raw.declarations {
        let name = declaration.name.clone();
        if declaration.kind == "theorem" {
            return Err(Rejection::Rejected(format!(
                "proof-as-runtime dependency: the theorem `{name}` is in the runtime closure"
            )));
        }
        if declaration.kind != "definition" {
            return Err(Rejection::Rejected(format!(
                "`{name}` is {}, not a definition the compiler can realize",
                article(&declaration.kind)
            )));
        }
        if !declaration.computable {
            return Err(Rejection::Rejected(format!("`{name}` is noncomputable")));
        }
        if !declaration.generates_code {
            return Err(Rejection::Rejected(format!("`{name}` generates no code")));
        }
        let (
            Some(safe),
            Some(recursive),
            Some(level_parameters),
            Some(ty),
            Some(parameters),
            Some(value),
        ) = (
            declaration.safe,
            declaration.recursive,
            declaration.level_parameters,
            declaration.ty,
            declaration.parameters,
            declaration.value,
        )
        else {
            return Err(Rejection::Rejected(format!(
                "the extraction record of `{name}` lacks its code"
            )));
        };
        if !safe {
            return Err(Rejection::Rejected(format!("`{name}` is unsafe")));
        }
        let LcnfDeclValue::Code { code } = value else {
            return Err(Rejection::Rejected(format!(
                "unsupported compiler form: `{name}` is implemented externally"
            )));
        };
        let mut renamer = Renamer::default();
        let mut ty = ty;
        let mut parameters = parameters;
        let mut code = code;
        renamer
            .ty(&mut ty)
            .and_then(|()| renamer.params(&mut parameters))
            .and_then(|()| renamer.code(&mut code))
            .map_err(|reason| Rejection::Rejected(format!("`{name}`: {reason}")))?;
        if !defined.insert(name.clone()) {
            return Err(Rejection::Rejected(format!("`{name}` is extracted twice")));
        }
        declarations.push(LcnfDeclaration {
            name,
            recursive,
            level_parameters,
            ty,
            parameters,
            code,
        });
    }
    declarations.sort_by(|left, right| left.name.cmp(&right.name));
    for external in &raw.externals {
        admissible_external(external, &module_set).map_err(Rejection::Rejected)?;
    }
    let mut inductives = raw.inductives;
    for inductive in &mut inductives {
        for constructor in &mut inductive.constructors {
            Renamer::default()
                .ty(&mut constructor.ty)
                .map_err(|reason| {
                    Rejection::Rejected(format!("`{}`: {reason}", constructor.name))
                })?;
        }
    }
    inductives.sort_by(|left, right| left.name.cmp(&right.name));
    let mut externals = raw.externals;
    externals.sort_by(|left, right| left.name.cmp(&right.name));

    // Lean's closure of every root must be exactly the eligibility
    // analysis's: a member only Lean reaches is a dependency the analysis
    // dropped, and a member only the analysis reaches is one Lean does not
    // compile, such as a proof treated as runtime.
    let expected = eligibility_closures(reports);
    let mut union = BTreeSet::new();
    let mut closures = raw.closures;
    closures.sort_by(|left, right| left.root.cmp(&right.root));
    if closures.len() != roots.len() {
        return Err(Rejection::Rejected(format!(
            "the extraction reports {} closures for {} roots",
            closures.len(),
            roots.len()
        )));
    }
    for closure in &closures {
        if !sorted_unique(&closure.declarations) || !sorted_unique(&closure.erased) {
            return Err(Rejection::Rejected(format!(
                "the closure of `{}` is not sorted and unique",
                closure.root
            )));
        }
        let Some((runtime, erased)) = expected.get(&closure.root) else {
            return Err(Rejection::Rejected(format!(
                "unknown root: `{}` is not a production root of this project",
                closure.root
            )));
        };
        let lean: BTreeSet<String> = closure.declarations.iter().cloned().collect();
        if let Some(dropped) = lean.difference(runtime).next() {
            return Err(Rejection::Rejected(format!(
                "dropped dependency: Lean's compiler reaches `{dropped}` from `{}`, but the production-eligibility closure does not",
                closure.root
            )));
        }
        if let Some(phantom) = runtime.difference(&lean).next() {
            return Err(Rejection::Rejected(format!(
                "the production-eligibility closure of `{}` realizes `{phantom}`, which Lean's compiler does not reach",
                closure.root
            )));
        }
        let lean_erased: BTreeSet<String> = closure.erased.iter().cloned().collect();
        if &lean_erased != erased {
            return Err(Rejection::Rejected(format!(
                "the proof-only dependencies of `{}` differ: Lean erases {lean_erased:?}, the eligibility analysis {erased:?}",
                closure.root
            )));
        }
        if let Some(runtime_proof) = lean.intersection(&lean_erased).next() {
            return Err(Rejection::Rejected(format!(
                "proof-as-runtime dependency: `{runtime_proof}` is both erased and realized"
            )));
        }
        union.extend(lean);
    }
    if union != defined {
        let missing: Vec<&String> = union.symmetric_difference(&defined).collect();
        return Err(Rejection::Rejected(format!(
            "the extracted declarations and the root closures disagree on {missing:?}"
        )));
    }
    // Completeness: every constant the extracted code names is a closure
    // member, a project inductive or one of its constructors, or a recorded
    // external, so no computational dependency is left implicit.
    let mut known: BTreeSet<String> = defined.clone();
    for inductive in &inductives {
        known.insert(inductive.name.clone());
        known.extend(
            inductive
                .constructors
                .iter()
                .map(|constructor| constructor.name.clone()),
        );
    }
    known.extend(externals.iter().map(|external| external.name.clone()));
    for declaration in &declarations {
        let mut used = BTreeSet::new();
        type_constants(&declaration.ty, &mut used);
        param_constants(&declaration.parameters, &mut used);
        code_constants(&declaration.code, &mut used);
        if let Some(missing) = used.difference(&known).next() {
            return Err(Rejection::Rejected(format!(
                "dropped dependency: `{}` uses `{missing}`, which the extraction neither defines nor records",
                declaration.name
            )));
        }
    }
    let mut erased = raw.erased;
    erased.sort();
    let _ = raw.internal_erased;
    Ok(CompilerInput {
        spec: INPUT_SPEC.to_owned(),
        lean: raw.lean,
        roots: roots.to_vec(),
        closures,
        declarations,
        inductives,
        externals,
        erased,
    })
}
