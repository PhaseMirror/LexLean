//! The Atlas provenance gate (ADR-PML-058, §27.9).
//!
//! `cargo xtask atlas-prov [check | run | cross]` binds the Atlas label plane
//! to the migration checkpoint recorded in
//! `examples/uor-atlas/provenance.toml`. The three modes:
//!
//! * `check` (default, the `vv` recipe): the ledger is well-formed and its
//!   recorded facts re-derive from the local git object database (no
//!   network); the sample labels are registered and declared exactly once by
//!   the native Atlas source; every label-shaped citation in repository
//!   documentation resolves to a live, withdrawn, or non-denotable register
//!   label; and the compiler-semantics drift from the checkpoint is reported.
//! * `run`: everything `check` does, plus the byte-equality half of
//!   `conformance_vr_19` re-derived from the recorded native-source tree:
//!   the live corpus is a subset of the checkpoint tree, every surviving
//!   module is byte-identical to its checkpoint blob, and every module the
//!   corpus no longer carries declared only labels the registers withhold or
//!   the live corpus re-declares. The elaboration half (the ≈1,020 s Lean
//!   replay) needs the pinned toolchain and is deferred to CI.
//! * `cross --foundry <dir> [--out <path>]`: samples the shared S-label
//!   plane against the Foundry formal sources and reports
//!   `ATLAS-LABEL-CONFLICT` records, exiting non-zero when the Foundry side
//!   cites an Atlas-family label the register neither owns nor withholds.
//!
//! The gate is deliberately crude — it reads source, git, and the register,
//! finds the defect, and fails naming the rule — matching the repository
//! audit style. It computes no new compiler-semantics digest and touches no
//! file inside the embedded normative data set, so it cannot re-seal an
//! unverified claim by bumping the digest.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::audit::{atlas_source_cores, is_label_shaped, parse_coredata};
use crate::Fail;

const LEDGER_RELATIVE: &str = "examples/uor-atlas/provenance.toml";
const LOCK_RELATIVE: &str = "examples/uor-atlas/lexlean.lock";
const REGISTER_RELATIVE: &str = "language/uor/atlas-registers.toml";
const SOURCE_ROOT_RELATIVE: &str = "examples/uor-atlas/src";
const LEDGER_SPEC: &str = "lexlean/atlas-providence/1";

/// The executive surface of the gate: parse the requested mode and run it.
pub fn atlas_prov(root: &Path, args: &[String]) -> Result<(), Fail> {
    let mode = args
        .iter()
        .find(|argument| !argument.starts_with("--"))
        .map(String::as_str)
        .unwrap_or("check");
    match mode {
        "check" => check(root),
        "run" => run(root),
        "cross" => cross(root, args),
        other => Err(Fail::from(format!(
            "atlas-prov: unknown mode `{other}`; the modes are `check` | `run` | `cross`"
        ))),
    }
}

/// The ledger, decoded and validated. Validation happens at parse time so a
/// malformed ledger is a gate failure rather than a silently trusted input.
struct Ledger {
    checkpoint: BTreeMap<String, String>,
    identities: BTreeMap<String, String>,
    sample: Vec<String>,
    native_modules: u64,
    input_lean_files: u64,
    input_lean_modules: u64,
    elapsed_seconds: f64,
}

const CHECKPOINT_HEX_FIELDS: &[(&str, usize)] = &[
    ("commit", 40),
    ("native_source_tree", 40),
    ("input_lean_tree", 40),
];
const IDENTITY_HEX_FIELDS: &[(&str, usize)] = &[("compiler_semantics", 64)];

impl Ledger {
    fn parse(text: &str) -> Result<Self, Fail> {
        let data: toml::Value = text.parse().map_err(|error| {
            Fail::from(format!("atlas-prov: {LEDGER_RELATIVE}: {error}"))
        })?;
        let spec = data
            .get("spec")
            .and_then(toml::Value::as_str)
            .ok_or_else(|| Fail::from(format!("atlas-prov: {LEDGER_RELATIVE} has no `spec`")))?;
        if spec != LEDGER_SPEC {
            return Err(Fail::from(format!(
                "atlas-prov: {LEDGER_RELATIVE} spec is `{spec}`; expected `{LEDGER_SPEC}`"
            )));
        }
        let checkpoint = data
            .get("checkpoint")
            .and_then(toml::Value::as_table)
            .ok_or_else(|| Fail::from("atlas-prov: the ledger has no `[checkpoint]` table"))?;
        let identities = data
            .get("checkpoint")
            .and_then(|table| table.get("identities"))
            .and_then(toml::Value::as_table)
            .ok_or_else(|| Fail::from("atlas-prov: the ledger has no `[checkpoint.identities]`"))?;

        let mut checkpoint_out = BTreeMap::new();
        for (field, length) in CHECKPOINT_HEX_FIELDS {
            let value = checkpoint.get(*field).and_then(toml::Value::as_str).ok_or_else(
                || Fail::from(format!("atlas-prov: the checkpoint has no `{field}`")),
            )?;
            if !is_hex(value, *length) {
                return Err(Fail::from(format!(
                    "atlas-prov: `{field}` must be {length} hex digits, found `{value}`"
                )));
            }
            checkpoint_out.insert((*field).to_owned(), (*value).to_owned());
        }

        let native_modules = int_of(checkpoint, "native_modules")?;
        if native_modules == 0 {
            return Err(Fail::from("atlas-prov: the checkpoint records zero native modules"));
        }
        let input_lean_files = int_of(checkpoint, "input_lean_files")?;
        let input_lean_modules = int_of(checkpoint, "input_lean_modules")?;
        if input_lean_files < input_lean_modules {
            return Err(Fail::from(
                "atlas-prov: the input tree has fewer files than the Lean modules it contains",
            ));
        }
        let records_native = int_of(checkpoint, "records_native")?;
        let records_private = int_of(checkpoint, "records_private")?;
        if records_native + records_private != int_of(checkpoint, "records_total")? {
            return Err(Fail::from(
                "atlas-prov: 5,577 = 5,519 native + 58 private is violated: records_native + records_private != records_total",
            ));
        }
        if int_of(checkpoint, "declaration_lines")? < int_of(checkpoint, "declaration_records")? {
            return Err(Fail::from(
                "atlas-prov: a wrapped declaration cannot produce more records than physical lines",
            ));
        }
        let elapsed_seconds = checkpoint
            .get("elapsed_seconds")
            .and_then(toml::Value::as_float)
            .ok_or_else(|| Fail::from("atlas-prov: the checkpoint has no `elapsed_seconds`"))?;
        if elapsed_seconds <= 0.0 {
            return Err(Fail::from("atlas-prov: `elapsed_seconds` is not positive"));
        }

        let mut identities_out = BTreeMap::new();
        for (field, length) in IDENTITY_HEX_FIELDS {
            let value = identities.get(*field).and_then(toml::Value::as_str).ok_or_else(
                || Fail::from(format!("atlas-prov: the identities have no `{field}`")),
            )?;
            if !is_hex(value, *length) {
                return Err(Fail::from(format!(
                    "atlas-prov: identity `{field}` must be {length} hex digits, found `{value}`"
                )));
            }
            identities_out.insert((*field).to_owned(), (*value).to_owned());
        }

        let sample: Vec<String> = data
            .get("sample")
            .and_then(toml::Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(toml::Value::as_str)
                    .map(str::to_owned)
                    .collect()
            })
            .unwrap_or_default();
        if sample.is_empty() {
            return Err(Fail::from("atlas-prov: the ledger records an empty sample; the gate is unarmed"));
        }
        let mut seen = BTreeSet::new();
        for label in &sample {
            if !is_label_shaped(label) {
                return Err(Fail::from(format!(
                    "atlas-prov: the sample label `{label}` is not label-shaped"
                )));
            }
            if !seen.insert(label) {
                return Err(Fail::from(format!(
                    "atlas-prov: the sample contains `{label}` twice; the sample is a set"
                )));
            }
        }

        Ok(Self {
            checkpoint: checkpoint_out,
            identities: identities_out,
            sample,
            native_modules,
            input_lean_files,
            input_lean_modules,
            elapsed_seconds,
        })
    }
}

fn is_hex(text: &str, length: usize) -> bool {
    text.len() == length && text.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn int_of(table: &toml::value::Table, key: &str) -> Result<u64, Fail> {
    table
        .get(key)
        .and_then(toml::Value::as_integer)
        .map(|value| value.max(0) as u64)
        .ok_or_else(|| Fail::from(format!("atlas-prov: the checkpoint has no integer `{key}`")))
}

/// Run `git` in the repository and require success; capture the output.
fn git(root: &Path, args: &[&str]) -> Result<std::process::Output, Fail> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .map_err(|error| format!("atlas-prov: `git {}` failed: {error}", args.join(" ")))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(Fail::from(format!(
            "atlas-prov: `git {}` failed: {}",
            args.join(" "),
            stderr.trim()
        )));
    }
    Ok(output)
}

fn git_lines(root: &Path, args: &[&str]) -> Result<Vec<String>, Fail> {
    let output = git(root, args)?;
    let text = String::from_utf8(output.stdout).map_err(|error| {
        Fail::from(format!(
            "atlas-prov: `git {}` output is not UTF-8: {error}",
            args.join(" ")
        ))
    })?;
    Ok(text.lines().map(str::to_owned).collect())
}

/// The register partitions as disjoint sets, plus the set of label families
/// the register actually uses (the citation scanner's scope guard).
fn register_sets(
    root: &Path,
) -> Result<(BTreeSet<String>, BTreeSet<String>, BTreeSet<String>, BTreeSet<String>), Fail> {
    let path = root.join(REGISTER_RELATIVE);
    if !path.exists() {
        return Err(Fail::from(format!(
            "atlas-prov: {REGISTER_RELATIVE} is absent; audit-atlas-registers arms the first row, atlas-prov cannot bind a citation plane without it"
        )));
    }
    let data: toml::Value = std::fs::read_to_string(&path)?.parse()?;
    let mut live = BTreeSet::new();
    for key in ["entry", "ambient"] {
        if let Some(items) = data.get(key).and_then(toml::Value::as_array) {
            live.extend(items.iter().filter_map(toml::Value::as_str).map(str::to_owned));
        }
    }
    let mut withdrawn = BTreeSet::new();
    for key in ["retracted", "superseded"] {
        if let Some(items) = data.get(key).and_then(toml::Value::as_array) {
            withdrawn.extend(
                items
                    .iter()
                    .filter_map(|row| row.get("label").and_then(toml::Value::as_str))
                    .map(str::to_owned),
            );
        }
    }
    let mut non_denotable = BTreeSet::new();
    if let Some(items) = data.get("non_denotable").and_then(toml::Value::as_array) {
        non_denotable.extend(items.iter().filter_map(toml::Value::as_str).map(str::to_owned));
    }
    let mut families = BTreeSet::new();
    for label in live.iter().chain(withdrawn.iter()).chain(non_denotable.iter()) {
        families.insert(family_of(label));
    }
    Ok((live, withdrawn, non_denotable, families))
}

/// The leading run of upper-case letters: the label family (`T57a` -> `T`).
fn family_of(name: &str) -> String {
    name.chars().take_while(char::is_ascii_uppercase).collect()
}

/// The label-shaped names the native Atlas source declares, mapped to their
/// fully qualified declaration, in the direction `audit_atlas_registers`
/// enforces (one label, one declaration).
fn native_label_map(root: &Path) -> Result<BTreeMap<String, String>, Fail> {
    let mut declared: BTreeMap<String, String> = BTreeMap::new();
    for module in atlas_source_cores(root)? {
        for declaration in
            lexlean::backend::core::environment_declarations(&module.core).map_err(|error| {
                Fail::from(format!("atlas-prov: {}: {}", module.path.display(), error.message))
            })?
        {
            let name = declaration
                .name
                .rsplit('.')
                .next()
                .unwrap_or(&declaration.name);
            if !is_label_shaped(name) {
                continue;
            }
            if let Some(first) = declared.insert(name.to_owned(), declaration.name.clone()) {
                return Err(Fail::from(format!(
                    "atlas-prov: `{name}` is declared as both `{first}` and `{}`; one label has one declaration",
                    declaration.name
                )));
            }
        }
    }
    Ok(declared)
}

/// The label-shaped tokens of one line, measured against the line with code
/// spans stripped to spaces (their content kept as prose). Each token carries
/// the escaped gap that follows it and whether it is the suffix of a dotted
/// path (`UorAtlas.Roots.T5x9` yields `T5x9` as a suffix, which is a name
/// inside an identifier, never a citation).
fn label_tokens(line: &str) -> Vec<(String, String, bool)> {
    let mut chars: Vec<char> = line.chars().collect();
    for index in 0..chars.len() {
        if chars[index] == '`' {
            chars[index] = ' ';
        }
    }
    let mut tokens = Vec::new();
    let mut at = 0usize;
    let mut previous_run_end = None;
    while at < chars.len() {
        let start = at;
        let mut end = at;
        while end < chars.len() && chars[end].is_ascii_alphanumeric() {
            end += 1;
        }
        if end > start {
            let token: String = chars[start..end].iter().collect();
            let gap: String = chars[end..]
                .iter()
                .take_while(|ch| !ch.is_ascii_alphanumeric())
                .collect();
            let dotted_suffix = previous_run_end.is_some_and(|previous_end| {
                let between = &chars[previous_end..start];
                between.iter().any(|ch| *ch == '.')
                    && between.iter().all(|ch| ch.is_whitespace() || *ch == '.' || *ch == '`')
            });
            if is_label_shaped(&token) {
                tokens.push((token, gap, dotted_suffix));
            }
            previous_run_end = Some(end);
            at = end;
        } else {
            at += 1;
        }
    }
    tokens
}

/// Whether the gap between two label tokens is only quoting, spacing, dashes,
/// or dots — and carries at least one dash or dot: a spelled range such as
/// `S1`-`S85`, `T10a..T10c`, or `F8`–`F11`. A bare space between two
/// same-family tokens is prose, not a range, and resolves both labels.
fn is_range_gap(gap: &str) -> bool {
    !gap.is_empty()
        && gap.chars().any(|ch| ch == '-' || ch == '\u{2013}' || ch == '\u{2014}' || ch == '.')
        && gap
            .chars()
            .all(|ch| ch.is_whitespace() || ch == '`' || ch == '-' || ch == '\u{2013}' || ch == '\u{2014}' || ch == '.')
}

/// Scan one markdown document for label-shaped citations that the register
/// neither owns nor withholds. Fenced (quoted gate output) blocks and spelled
/// ranges are exempt. Returns every cited live label.
fn scan_citations(
    display: &str,
    text: &str,
    live: &BTreeSet<String>,
    withdrawn: &BTreeSet<String>,
    non_denotable: &BTreeSet<String>,
    families: &BTreeSet<String>,
) -> Result<BTreeSet<String>, Fail> {
    let mut cited = BTreeSet::new();
    let mut fenced = false;
    for (line_index, line) in text.lines().enumerate() {
        if line.trim_start().starts_with("```") {
            fenced = !fenced;
            continue;
        }
        if fenced {
            continue;
        }
        let tokens = label_tokens(line);
        if tokens.is_empty() {
            continue;
        }
        let mut ranged = BTreeSet::new();
        for at in 0..tokens.len() {
            let (token, gap, _) = &tokens[at];
            let partner = (at..tokens.len())
                .skip(1)
                .find(|next| family_of(&tokens[*next].0) == family_of(token));
            if let Some(next) = partner {
                if is_range_gap(gap) {
                    ranged.insert(at);
                    ranged.insert(next);
                }
            }
        }
        for (at, (token, _, dotted_suffix)) in tokens.iter().enumerate() {
            if ranged.contains(&at) || *dotted_suffix {
                continue;
            }
            if !families.contains(&family_of(token)) {
                continue;
            }
            if live.contains(token) {
                cited.insert(token.clone());
                continue;
            }
            if withdrawn.contains(token) || non_denotable.contains(token) {
                continue;
            }
            return Err(Fail::from(format!(
                "R2: {display}:{}: `{token}` is cited but the Atlas register neither owns nor withholds it; a document citation cannot float outside the register plane",
                line_index + 1
            )));
        }
    }
    Ok(cited)
}

/// The markdown documents the citation scanner reads: the root documents,
/// the ADR mirror tree, and the Atlas example's own documentation.
fn documentation_files(root: &Path) -> Result<Vec<PathBuf>, Fail> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(root).into_iter().flatten().flatten() {
        if entry.path().is_file() && entry.path().extension().is_some_and(|e| e == "md") {
            files.push(entry.path());
        }
    }
    for dir in ["docs", "examples/uor-atlas"] {
        let base = root.join(dir);
        if !base.exists() {
            continue;
        }
        for entry in walkdir::WalkDir::new(&base)
            .follow_links(false)
            .into_iter()
            .flatten()
        {
            if !entry.file_type().is_file()
                || entry.path().extension().is_none_or(|extension| extension != "md")
            {
                continue;
            }
            let relative = entry
                .path()
                .strip_prefix(root)
                .expect("documentation under the audit root");
            if relative.components().any(|component| component.as_os_str() == "expected") {
                continue;
            }
            files.push(entry.path().to_path_buf());
        }
    }
    files.sort();
    files.dedup();
    if files.is_empty() {
        return Err(Fail::from("atlas-prov: no document to scan; the citation gate is unarmed"));
    }
    Ok(files)
}

/// The compiler-semantics digest the committed lock records, if any.
fn lock_compiler_semantics(root: &Path) -> Result<Option<String>, Fail> {
    let path = root.join(LOCK_RELATIVE);
    if !path.exists() {
        return Ok(None);
    }
    let data: toml::Value = std::fs::read_to_string(&path)?.parse()?;
    Ok(data
        .get("compiler_semantics")
        .and_then(toml::Value::as_str)
        .map(str::to_owned))
}

/// The checkpoint commit and both trees, re-derived from the local git
/// object database: presence, reachability, and tree membership counts.
fn verify_checkpoint_objects(root: &Path, ledger: &Ledger) -> Result<(), Fail> {
    let commit = ledger.checkpoint.get("commit").expect("validated above");
    let native = ledger
        .checkpoint
        .get("native_source_tree")
        .expect("validated above");
    let input = ledger
        .checkpoint
        .get("input_lean_tree")
        .expect("validated above");
    git(root, &["cat-file", "-e", commit])?;
    let ancestor = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["merge-base", "--is-ancestor", commit, "HEAD"])
        .status()
        .map_err(|error| format!("atlas-prov: `git merge-base` failed: {error}"))?;
    if !ancestor.success() {
        return Err(Fail::from(format!(
            "atlas-prov: checkpoint commit {commit} is not an ancestor of HEAD; the migration proved nothing about a tree that no longer reaches it"
        )));
    }
    git(root, &["cat-file", "-e", &format!("{native}^{{tree}}")])?;
    git(root, &["cat-file", "-e", &format!("{input}^{{tree}}")])?;
    let native_rows = git_lines(root, &["ls-tree", "-r", native])?;
    if native_rows.len() as u64 != ledger.native_modules {
        return Err(Fail::from(format!(
            "atlas-prov: the native-source tree now holds {} modules; the ledger records {}",
            native_rows.len(),
            ledger.native_modules
        )));
    }
    let input_rows = git_lines(root, &["ls-tree", "-r", input])?;
    let lean = input_rows
        .iter()
        .filter(|line| line.ends_with(".lean"))
        .count() as u64;
    if input_rows.len() as u64 != ledger.input_lean_files || lean != ledger.input_lean_modules {
        return Err(Fail::from(format!(
            "atlas-prov: the input tree now holds {} files ({lean} Lean); the ledger records {} files ({} Lean)",
            input_rows.len(),
            ledger.input_lean_files,
            ledger.input_lean_modules
        )));
    }
    Ok(())
}

/// `check`: the `vv` gate. Fast, whole, and offline.
fn check(root: &Path) -> Result<(), Fail> {
    let ledger_text = std::fs::read_to_string(root.join(LEDGER_RELATIVE))
        .map_err(|error| Fail::from(format!("atlas-prov: {LEDGER_RELATIVE}: {error}")))?;
    let ledger = Ledger::parse(&ledger_text)?;
    verify_checkpoint_objects(root, &ledger)?;

    let (live, withdrawn, non_denotable, families) = register_sets(root)?;
    let declared = native_label_map(root)?;
    for label in &ledger.sample {
        if !live.contains(label) {
            return Err(Fail::from(format!(
                "atlas-prov: sample label `{label}` is not a live Atlas register label; a sample the corpus does not own cannot bind the plane"
            )));
        }
        match declared.get(label) {
            Some(qualified) => println!("atlas-prov-check: sample `{label}` is {qualified}"),
            None => {
                return Err(Fail::from(format!(
                    "atlas-prov: sample label `{label}` has no native Atlas declaration; the register owns a label the corpus withholds"
                )));
            }
        }
    }

    let docs = documentation_files(root)?;
    let mut total_cited = BTreeSet::new();
    for path in &docs {
        let text = std::fs::read_to_string(path)?;
        let display = path
            .strip_prefix(root)
            .unwrap_or(path)
            .display()
            .to_string();
        total_cited.extend(scan_citations(
            &display, &text, &live, &withdrawn, &non_denotable, &families,
        )?);
    }

    let checkpoint = ledger
        .identities
        .get("compiler_semantics")
        .expect("validated above");
    match lock_compiler_semantics(root)? {
        Some(current) if current == *checkpoint => {
            println!("atlas-prov-check: compiler-semantics pinned at the checkpoint ({checkpoint})");
        }
        Some(current) => {
            println!(
                "atlas-prov-check: compiler-semantics drifted from the checkpoint: checkpoint {checkpoint}, committed lock {current}; the byte-equality anchor below re-derives regardless"
            );
        }
        None => {
            println!("atlas-prov-check: no committed lock; the compiler-semantics delta is not computed");
        }
    }

    println!(
        "atlas-prov-check: ledger `{LEDGER_SPEC}` valid; checkpoint commit and both trees re-derived from local git; {} sample labels declared once by the native source; {} distinct label citations resolved across {} documentation file(s) (R2, R4)",
        ledger.sample.len(),
        total_cited.len(),
        docs.len()
    );
    Ok(())
}

/// `run`: the ADR Decision (1) re-derivation. Everything `check` verifies,
/// plus the byte-equality half of `conformance_vr_19` from the recorded
/// native-source tree, entirely offline.
fn run(root: &Path) -> Result<(), Fail> {
    check(root)?;
    let ledger_text = std::fs::read_to_string(root.join(LEDGER_RELATIVE))?;
    let ledger = Ledger::parse(&ledger_text)?;
    let native = ledger
        .checkpoint
        .get("native_source_tree")
        .expect("validated above");

    // The native-source tree: blob identity per module path.
    let mut tree: BTreeMap<String, String> = BTreeMap::new();
    for line in git_lines(root, &["ls-tree", "-r", native])? {
        let (meta, path) = line.split_once('\t').ok_or_else(|| {
            Fail::from(format!("atlas-prov: unparsable tree row `{line}`"))
        })?;
        let blob = meta
            .rsplit(' ')
            .next()
            .ok_or_else(|| Fail::from(format!("atlas-prov: unparsable tree row `{line}`")))?;
        tree.insert(path.to_owned(), blob.to_owned());
    }

    let source_root = root.join(SOURCE_ROOT_RELATIVE);
    let mut corpus: Vec<PathBuf> = Vec::new();
    for entry in walkdir::WalkDir::new(&source_root)
        .follow_links(false)
        .into_iter()
        .flatten()
    {
        if entry.file_type().is_file()
            && entry
                .path()
                .file_name()
                .is_some_and(|name| name.to_string_lossy().ends_with(".lex.tex"))
        {
            corpus.push(entry.path().to_path_buf());
        }
    }
    corpus.sort();

    let mut preserved = 0usize;
    let mut diverged: Vec<String> = Vec::new();
    let mut added: Vec<String> = Vec::new();
    for path in &corpus {
        let relative = path
            .strip_prefix(&source_root)
            .expect("corpus entry under the source root")
            .to_string_lossy()
            .replace('\\', "/");
        let Some(checkpoint_blob) = tree.get(&relative) else {
            added.push(relative);
            continue;
        };
        let current = git_lines(root, &["hash-object", path.to_str().expect("utf-8 corpus path")])?
            .into_iter()
            .next()
            .ok_or_else(|| Fail::from(format!("atlas-prov: `git hash-object` printed nothing for {}", path.display())))?;
        if current == *checkpoint_blob {
            preserved += 1;
        } else {
            diverged.push(relative);
        }
    }
    if !added.is_empty() {
        return Err(Fail::from(format!(
            "P2: atlas-prov: corpus module(s) {} have no twin in the checkpoint tree; a module that appears after the checkpoint moves the corpus off the single bound and needs a new checkpoint, not a silent extension",
            added.join(", ")
        )));
    }
    if !diverged.is_empty() {
        return Err(Fail::from(format!(
            "P2: atlas-prov: {} corpus module(s) diverged from their checkpoint blob: {}; byte-equality is the checkpoint's claim, so a surviving module that changed must be re-triangulated before it is bound",
            diverged.len(),
            diverged.join(", ")
        )));
    }

    // Every checkpoint module the corpus no longer carries must be accounted
    // for: parse its checkpoint payload and require that each label it
    // declared is withheld by the register or re-declared live by the corpus
    // (the same obligation `audit_atlas_registers` enforces forward).
    let (_, withdrawn, _, _) = register_sets(root)?;
    let declared = native_label_map(root)?;
    let mut dropped: Vec<String> = Vec::new();
    for (relative, blob) in &tree {
        if corpus
            .iter()
            .any(|path| {
                path.strip_prefix(&source_root)
                    .expect("corpus entry under the source root")
                    .to_string_lossy()
                    .replace('\\', "/")
                    == *relative
            })
        {
            continue;
        }
        let output = git(root, &["show", &format!("{native}:{relative}")])?;
        let text = String::from_utf8(output.stdout)
            .map_err(|error| Fail::from(format!("atlas-prov: {relative}: {error}")))?;
        let module = parse_coredata(&text)
            .map_err(|fail| Fail::from(format!("atlas-prov: {relative}: {fail}")))?;
        for declaration in lexlean::backend::core::environment_declarations(&module)
            .map_err(|error| Fail::from(format!("atlas-prov: {relative}: {}", error.message)))?
        {
            let name = declaration
                .name
                .rsplit('.')
                .next()
                .unwrap_or(&declaration.name);
            if !is_label_shaped(name) || withdrawn.contains(name) || declared.contains_key(name) {
                continue;
            }
            return Err(Fail::from(format!(
                "P2: atlas-prov: the checkpoint module `{relative}` (blob {blob}) declared `{name}`, which the registers neither withhold nor the corpus re-declares; dropping a module cannot withdraw a label silently"
            )));
        }
        dropped.push(relative.clone());
    }

    println!(
        "atlas-prov-run: {preserved} corpus modules byte-identical to their checkpoint blobs, {} added, {} diverged, {} dropped module(s) accounted by the register",
        added.len(),
        diverged.len(),
        dropped.len()
    );

    // The elaboration half needs the pinned toolchain; report the boundary
    // honestly rather than pretending the repository alone can replay it.
    let home = std::env::var("HOME").unwrap_or_default();
    let toolchain = Path::new(&home).join(".elan/toolchains/leanprover--lean4---v4.32.1/bin");
    if toolchain.join("lean").is_file() && toolchain.join("leanchecker").is_file() {
        println!(
            "atlas-prov-run: pinned toolchain {} present; run `just verify-examples` for the full normalized-record replay",
            toolchain.display()
        );
    } else {
        println!(
            "atlas-prov-run: the elaboration half of `conformance_vr_19` ({} s at the checkpoint) is deferred here: the pinned toolchain (leanprover/lean4:v4.32.1 + leanchecker) is not installed; it runs as `just vv` on CI and as the pre-bump boundary in VerificationVerification.md",
            ledger.elapsed_seconds
        );
    }
    Ok(())
}

/// `cross`: sample the shared label plane against the Foundry formal
/// sources. Every live label the Foundry side also cites becomes a bound
/// pair; an Atlas-family label the Foundry cites that this register neither
/// owns nor withholds is an `ATLAS-LABEL-CONFLICT` and fails the gate.
fn cross(root: &Path, args: &[String]) -> Result<(), Fail> {
    let foundry = flag_value(args, "--foundry")
        .ok_or_else(|| Fail::from("atlas-prov cross: `--foundry <dir>` is required"))?;
    let foundry_root = Path::new(&foundry);
    if !foundry_root.is_dir() {
        return Err(Fail::from(format!(
            "atlas-prov cross: the Foundry root {} is not a directory",
            foundry_root.display()
        )));
    }

    let ledger_text = std::fs::read_to_string(root.join(LEDGER_RELATIVE))?;
    let ledger = Ledger::parse(&ledger_text)?;
    let (live, withdrawn, non_denotable, families) = register_sets(root)?;
    let declared = native_label_map(root)?;

    // The shared plane: the ledger sample plus every live label cited in this
    // repository's documentation.
    let docs = documentation_files(root)?;
    let mut shared: BTreeSet<String> = ledger.sample.iter().cloned().collect();
    for path in &docs {
        let text = std::fs::read_to_string(path)?;
        let display = path
            .strip_prefix(root)
            .unwrap_or(path)
            .display()
            .to_string();
        shared.extend(scan_citations(
            &display, &text, &live, &withdrawn, &non_denotable, &families,
        )?);
    }

    // Index the Foundry side: every Atlas-family label token in every file.
    let mut foundry_files = Vec::new();
    for entry in walkdir::WalkDir::new(foundry_root)
        .follow_links(false)
        .into_iter()
        .flatten()
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let extension = entry
            .path()
            .extension()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        if ["md", "lean", "txt", "toml"].contains(&extension.as_str()) {
            foundry_files.push(entry.path().to_path_buf());
        }
    }
    foundry_files.sort();
    let mut hits: BTreeMap<String, Vec<PathBuf>> = BTreeMap::new();
    let mut foreign: Vec<(String, PathBuf)> = Vec::new();
    for path in &foundry_files {
        let Ok(text) = std::fs::read_to_string(path) else {
            continue;
        };
        let tokens = label_tokens(&text);
        for (token, _, dotted_suffix) in tokens
            .iter()
            .filter(|(token, _, _)| families.contains(&family_of(token)))
        {
            if *dotted_suffix || withdrawn.contains(token) || non_denotable.contains(token) {
                continue;
            }
            if live.contains(token) {
                hits.entry(token.clone()).or_default().push(path.clone());
            } else {
                foreign.push((token.clone(), path.clone()));
            }
        }
    }
    foreign.sort();
    foreign.dedup();

    let mut conflicts = 0usize;
    let mut records = Vec::new();
    for (token, path) in &foreign {
        conflicts += 1;
        records.push(serde_json::json!({
            "label": token,
            "status": "ATLAS-LABEL-CONFLICT",
            "foundry_file": path.strip_prefix(foundry_root).unwrap_or(path).display().to_string(),
            "detail": "the Foundry side cites an Atlas-family label the register neither owns nor withholds (ADR-0119)"
        }));
    }
    for label in &shared {
        let qualified = declared.get(label).cloned().unwrap_or_default();
        let files: Vec<PathBuf> = hits.get(label).cloned().unwrap_or_default();
        let status = if qualified.is_empty() {
            "ATLAS-LABEL-CONFLICT"
        } else if files.is_empty() {
            "PENDING-FOUNDRY"
        } else {
            "PRESENT"
        };
        if status == "ATLAS-LABEL-CONFLICT" {
            conflicts += 1;
        }
        records.push(serde_json::json!({
            "label": label,
            "status": status,
            "native_declaration": qualified,
            "foundry_hits": files
                .iter()
                .map(|path| {
                    path.strip_prefix(foundry_root).unwrap_or(path).display().to_string()
                })
                .collect::<Vec<String>>(),
        }));
    }

    let report = serde_json::to_string_pretty(&records)?;
    if let Some(out) = flag_value(args, "--out") {
        std::fs::write(Path::new(&out), format!("{report}\n"))
            .map_err(|error| Fail::from(format!("atlas-prov cross: {}: {error}", out)))?;
        println!("atlas-prov-cross: {} records written to {}", records.len(), out);
    } else {
        println!("{report}");
    }
    let present = records
        .iter()
        .filter(|record| record["status"] == "PRESENT")
        .count();
    let pending = records
        .iter()
        .filter(|record| record["status"] == "PENDING-FOUNDRY")
        .count();
    if conflicts > 0 {
        return Err(Fail::from(format!(
            "ATLAS-LABEL-CONFLICT: {conflicts} record(s) must resolve to zero before the next `just vv` read of the register (ADR-PML-058, ADR-0119)"
        )));
    }
    println!(
        "atlas-prov-cross: {} shared label pairs checked against {} Foundry files; {present} present, {pending} pending, 0 conflicts",
        shared.len(),
        foundry_files.len()
    );
    Ok(())
}

fn flag_value(args: &[String], name: &str) -> Option<String> {
    args.iter()
        .position(|argument| argument == name)
        .and_then(|at| args.get(at + 1))
        .cloned()
}

#[cfg(test)]
mod tests {
    use super::{is_hex, is_range_gap, label_tokens, family_of, Ledger};

    const VALID: &str = r#"
spec = "lexlean/atlas-providence/1"

sample = ["S4", "S37", "S38", "S43"]

[checkpoint]
commit = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
native_source_tree = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
input_lean_tree = "cccccccccccccccccccccccccccccccccccccccc"
native_modules = 66
input_lean_files = 71
input_lean_modules = 68
records_total = 5577
records_native = 5519
records_private = 58
elapsed_seconds = 1019.88
ga_run = 32799526580
ga_job = 97657516155
attestation = "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"
artifact = 9549590620
artifact_sha256 = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"
normalized_records = 267
declaration_records = 5519
declaration_lines = 5521

[checkpoint.identities]
compiler_semantics = "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
"#;

    #[test]
    fn a_well_formed_ledger_parses() {
        Ledger::parse(VALID).expect("the valid ledger parses");
    }

    #[test]
    fn a_40_hex_field_and_the_semantics_identity_are_distinct_shapes() {
        assert!(is_hex("ffffffffffffffffffffffffffffffffffffffff", 40));
        assert!(is_hex(&"f".repeat(64), 64));
        assert!(!is_hex(&"f".repeat(63), 64));
    }

    #[test]
    fn the_arithmetic_invariant_is_enforced() {
        let tampered = VALID.replace("records_total = 5577", "records_total = 5578");
        assert!(
            Ledger::parse(&tampered).is_err(),
            "5,577 = 5,519 + 58 is enforced"
        );
    }

    #[test]
    fn the_sample_is_a_set() {
        let duplicated = VALID.replacen("\"S43\"]", "\"S43\", \"S43\"]", 1);
        assert!(Ledger::parse(&duplicated).is_err());
    }

    #[test]
    fn label_tokens_split_on_non_alphanumerics_and_keep_backticked_content() {
        let tokens = label_tokens("see `S43` and S37-S38 (a range) plus T57a.");
        let shaped: Vec<(String, String)> = tokens
            .iter()
            .map(|(token, gap, _)| (token.clone(), gap.clone()))
            .collect();
        assert_eq!(
            shaped,
            vec![
                (String::from("S43"), String::from("  ")),
                (String::from("S37"), String::from("-")),
                (String::from("S38"), String::from(" (")),
                (String::from("T57a"), String::from(".")),
            ]
        );
        assert!(!tokens.iter().any(|(token, _, _)| token == "see"));
    }

    #[test]
    fn dotted_path_suffixes_are_not_citations() {
        let tokens = label_tokens("`UorAtlas.Roots.T5x9`, a declaration the native source does not make.");
        let t5x9 = tokens.iter().find(|(token, _, _)| token == "T5x9").expect("T5x9 is tokenized");
        assert!(t5x9.2, "a label-shaped name following a dotted identifier is a path suffix, not a citation");
    }

    #[test]
    fn range_gaps_are_only_dashes_and_dots() {
        assert!(is_range_gap("-"));
        assert!(is_range_gap("`-`"));
        assert!(is_range_gap(".."));
        assert!(!is_range_gap(" "));
        assert!(!is_range_gap("and"));
    }

    #[test]
    fn families_are_the_leading_upper_case_run() {
        assert_eq!(family_of("T57a"), "T");
        assert_eq!(family_of("S43"), "S");
        assert_eq!(family_of("BC1"), "BC");
    }
}