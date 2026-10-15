//! The `authoring` suite: AU-01..AU-04, the language-1.2 authoring notation
//! (SPEC.md §17.21).
//!
//! The notation is in bijection with the canonical JSON of the closed schema,
//! so the cases state that bijection (AU-01), what it buys (a module in the
//! notation is the module its data form is, AU-02), what it refuses (AU-03),
//! and that it is the only way the product writes a module (AU-04). Every
//! committed semantic module is a case's input, not a sample of them.

use std::collections::BTreeSet;

use lexlean::artifact::canonical_json::Json;
use lexlean::surface::lex::lex;
use lexlean::surface::semantic::{
    parse_at, parse_module, print_at, print_module, schema_samples, Position,
};
use lexlean::{FormatRequest, Selection};

use crate::support::{self, repo_root, P};

/// Run the case for one authoring ID.
///
/// # Panics
///
/// When the case's assertion fails, or for an unwired ID.
pub fn run(id: &str) {
    match id {
        "AU-01" => round_trip(),
        "AU-02" => equivalence(),
        "AU-03" => fails_closed(),
        "AU-04" => one_surface(),
        other => panic!("no authoring case is wired for {other}"),
    }
}

const SPEC: &str = "lexlean/semantic-module/2";

// ---------------------------------------------------------------------------
// The committed corpus.
// ---------------------------------------------------------------------------

/// Every semantic module of a committed source, as canonical JSON, with the
/// file it is in. A module that is not well formed is a negative fixture's
/// business and is left out.
fn corpus() -> Vec<(String, Json)> {
    let root = repo_root();
    let mut modules = Vec::new();
    for entry in walkdir::WalkDir::new(root.as_std_path())
        .into_iter()
        .filter_entry(|entry| {
            let name = entry.file_name().to_string_lossy();
            name != "target" && name != ".git" && name != ".lexlean"
        })
        .flatten()
    {
        let path = entry.path();
        if !path.to_string_lossy().ends_with(".lex.tex") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(path) else {
            continue;
        };
        if let Ok(value) = support::try_module_json(&text) {
            if let Ok(json) = Json::parse(value.to_string().as_bytes()) {
                modules.push((path.display().to_string(), json));
            }
        }
    }
    modules
}

// ---------------------------------------------------------------------------
// AU-01.
// ---------------------------------------------------------------------------

/// A value written and read at a position is the value, and what is written
/// from what is read is what was written.
fn assert_round_trip(value: &Json, position: Position, what: &str) -> String {
    let text = print_at(value, position);
    let back = parse_at(&text, position).unwrap_or_else(|error| panic!("{what}: {error}\n{text}"));
    assert_eq!(&back, value, "{what}: read back differently\n{text}");
    assert_eq!(
        print_at(&back, position),
        text,
        "{what}: the notation is not its own canonical form"
    );
    text
}

fn round_trip() {
    // Every alternative of the four unions of the schema, with and without
    // its optional members. The samples are read from the schema, so a
    // variant added to it is sampled the day it is added.
    let samples = schema_samples();
    let mut generic = 0usize;
    for (position, label, value) in &samples {
        let text = assert_round_trip(value, *position, label);
        // A readable form is one that is not `kind{…}` spelled out.
        if text.starts_with(&format!(
            "{}{{",
            match value {
                Json::Obj(members) => match members.get("kind") {
                    Some(Json::Str(kind)) => kind.as_str(),
                    _ => "",
                },
                _ => "",
            }
        )) {
            generic += 1;
        }
    }
    assert!(
        samples.len() > 180 && generic > 30 && generic < samples.len(),
        "{} samples, {generic} written generically: the schema moved, or a readable form is lost",
        samples.len()
    );

    // A node that no readable form carries exactly is written generically.
    for (generic_text, position) in [
        (r#"var{name: let}"#, Position::Term),
        (r#"nat{value: "01"}"#, Position::Term),
        (
            r#"add{left: nat{value: "1"}, right: nat{value: "2"}, extra: true}"#,
            Position::Term,
        ),
        (
            r#"named{member: {name: Nat}, arguments: []}"#,
            Position::Type,
        ),
    ] {
        let value = lexlean::surface::generic::parse(generic_text).expect("generic notation");
        let text = assert_round_trip(&value, position, generic_text);
        assert!(text.contains('{'), "{text} is written generically");
    }

    // Every committed semantic module of language 1.2.
    let mut modules = 0usize;
    let mut declarations = 0usize;
    for (path, json) in corpus() {
        let Json::Obj(members) = &json else { continue };
        if members.get("spec") != Some(&Json::Str(SPEC.to_owned())) {
            continue;
        }
        let text = print_module(&json).unwrap_or_else(|error| panic!("{path}: {error}"));
        let back =
            parse_module(&text, SPEC).unwrap_or_else(|error| panic!("{path}: {error}\n{text}"));
        assert_eq!(back, json, "{path}: read back differently");
        assert_eq!(
            print_module(&back).expect("prints"),
            text,
            "{path}: the notation is not its own canonical form"
        );
        modules += 1;
        if let Some(Json::Arr(items)) = members.get("declarations") {
            declarations += items.len();
        }
    }
    // Anti-vacuity: the committed corpus is large, and says so.
    assert!(
        modules >= 250 && declarations >= 4_000,
        "{modules} modules, {declarations} declarations: the corpus the notation is checked on has shrunk"
    );
}

// ---------------------------------------------------------------------------
// AU-02.
// ---------------------------------------------------------------------------

/// Committed language-1.2 examples, which are written in the notation.
const EXAMPLES: [&str; 6] = [
    "language-1.2",
    "collections",
    "higher-order",
    "recursive-data",
    "models",
    "reasoning",
];

/// Every file under `directory`, by `/`-separated relative path.
fn files(directory: &camino::Utf8Path) -> std::collections::BTreeMap<String, Vec<u8>> {
    let mut out = std::collections::BTreeMap::new();
    for entry in walkdir::WalkDir::new(directory.as_std_path())
        .into_iter()
        .flatten()
    {
        if entry.file_type().is_file() {
            let relative = entry
                .path()
                .strip_prefix(directory.as_std_path())
                .expect("under the directory")
                .to_string_lossy()
                .replace('\\', "/");
            out.insert(relative, std::fs::read(entry.path()).expect("read"));
        }
    }
    out
}

/// Write every module of a project as `\semanticdata{…}`: the form a module
/// had before it was written in the notation.
fn convert_to_data(project: &P) {
    for entry in walkdir::WalkDir::new(project.root.join("src").as_std_path())
        .into_iter()
        .flatten()
    {
        let path = entry.path();
        if !path.to_string_lossy().ends_with(".lex.tex") {
            continue;
        }
        let text = std::fs::read_to_string(path).expect("read");
        if text.contains("\\begin{semanticmodule}") {
            std::fs::write(path, support::module_as_data(&text)).expect("write");
        }
    }
}

fn format_all(
    project: &P,
    check_only: bool,
) -> Result<lexlean::FormatResultSet, lexlean::LexLeanError> {
    project.engine().format(FormatRequest {
        selection: Selection::All,
        check_only,
    })
}

fn equivalence() {
    let declaration_words: BTreeSet<&str> = [
        "structure",
        "class",
        "instance",
        "inductive",
        "definition",
        "theorem",
        "artifact",
        "contract",
        "realization",
        "evidence",
        "model",
        "logic",
        "inference_rule",
        "verifier",
        "reasoner",
    ]
    .into_iter()
    .collect();
    for name in EXAMPLES {
        let data = P::copy_example(name);
        let notation = P::copy_example(name);
        convert_to_data(&data);
        // A committed module is written in the notation, and it is canonical
        // as it is: `fmt` leaves it unchanged, in the file and in the check.
        let sources: Vec<String> = walkdir::WalkDir::new(notation.root.join("src").as_std_path())
            .into_iter()
            .flatten()
            .filter(|entry| entry.path().to_string_lossy().ends_with(".lex.tex"))
            .map(|entry| entry.path().display().to_string())
            .collect();
        let before: Vec<String> = sources
            .iter()
            .map(|path| std::fs::read_to_string(path).expect("read"))
            .collect();
        for text in &before {
            assert!(
                !text.contains("\\semanticdata"),
                "{name}: a committed module is written as data"
            );
        }
        notation.fmt_check_ok();
        // The data form is the same module in text `fmt` does not write: it
        // is not canonical, and `fmt` migrates it to the notation, to exactly
        // the committed bytes.
        support::expect_code(
            &format_all(&data, true)
                .err()
                .expect("a module written as data is not canonical"),
            "LLL1003",
        );
        let migrated = P::copy_example(name);
        convert_to_data(&migrated);
        format_all(&migrated, false).expect("fmt migrates the data form");
        for (path, committed) in sources.iter().zip(&before) {
            let relative = std::path::Path::new(path)
                .strip_prefix(notation.root.as_std_path())
                .expect("under the project")
                .to_string_lossy()
                .replace('\\', "/");
            let relative = relative.as_str();
            assert_eq!(
                &migrated.read_raw(relative),
                committed,
                "{name}: fmt did not migrate {relative} to the committed notation"
            );
        }
        // The notation is canonical because `fmt` writes the same text from
        // any layout of it: collapse the layout of one module, and `fmt` is
        // what restores it.
        let (path, canonical) = sources
            .iter()
            .zip(&before)
            .find(|(_, text)| text.contains("\\begin{semanticmodule}"))
            .expect("a semantic module");
        let squeezed = support::squeezed_notation(canonical);
        assert_ne!(&squeezed, canonical, "{name}: the layout did not change");
        std::fs::write(path, &squeezed).expect("write");
        support::expect_code(
            &format_all(&notation, true)
                .err()
                .expect("another layout is not canonical"),
            "LLL1003",
        );
        format_all(&notation, false).expect("fmt writes the notation");
        assert_eq!(
            &std::fs::read_to_string(path).expect("read"),
            canonical,
            "{name}: fmt did not restore the canonical layout"
        );
        notation.fmt_check_ok();
        format_all(&notation, false).expect("fmt writes the notation");
        let after: Vec<String> = sources
            .iter()
            .map(|path| std::fs::read_to_string(path).expect("read"))
            .collect();
        assert_eq!(before, after, "{name}: fmt changed the notation");

        // The same module: the same semantic ID, and the same generated Lean
        // and LaTeX byte for byte.
        let data_checked = data.check_ok();
        let notation_checked = notation.check_ok();
        assert_eq!(
            data_checked.semantic_id, notation_checked.semantic_id,
            "{name}: the notation does not link to its data form's module"
        );
        assert_ne!(
            data_checked.source_id, notation_checked.source_id,
            "{name}: the source did not change"
        );
        let data_built = data.build_ok();
        let notation_built = notation.build_ok();
        let data_dir = data.build_dir(&data_built.build_id.expect("a build id"));
        let notation_dir = notation.build_dir(&notation_built.build_id.expect("a build id"));
        let data_files = files(&data_dir.join("modules"));
        let notation_files = files(&notation_dir.join("modules"));
        assert!(!data_files.is_empty(), "{name}: no generated files");
        assert_eq!(
            data_files, notation_files,
            "{name}: the generated Lean and LaTeX differ"
        );

        // The source maps relate each declaration to its own text in the
        // notation.
        let mut mapped = 0usize;
        for (relative, bytes) in files(&notation_dir.join("maps")) {
            let map: serde_json::Value = serde_json::from_slice(&bytes).expect("a source map");
            let Some(module) = relative
                .strip_suffix(".map.json")
                .map(|stem| format!("src/{}.lex.tex", stem.rsplit('/').next().unwrap_or(stem)))
            else {
                continue;
            };
            let Ok(source) = std::fs::read_to_string(notation.root.join(&module).as_std_path())
            else {
                continue;
            };
            for mapping in map["mappings"].as_array().expect("mappings") {
                let start =
                    usize::try_from(mapping["src_start"].as_u64().expect("start")).expect("fits");
                let end = usize::try_from(mapping["src_end"].as_u64().expect("end")).expect("fits");
                // The module's own pieces map the whole source.
                if mapping["role"] != "declaration" || (start == 0 && end == source.len()) {
                    continue;
                }
                let text = &source[start..end];
                let word = text.split(['{', ' ']).next().unwrap_or_default();
                assert!(
                    declaration_words.contains(word) && text.ends_with('}'),
                    "{name}: `{}` is not a declaration of the notation",
                    &text[..text.len().min(40)]
                );
                mapped += 1;
            }
        }
        assert!(
            mapped > 0,
            "{name}: no declaration is mapped to the notation"
        );
    }

    // Under language 1.1 the notation is refused.
    let project = P::copy_example("semantic-1.1");
    let main = project.read("src/Main.lex.tex");
    let begin = main
        .find("\\begin{semanticmodule}")
        .expect("a semantic module");
    let end = main.rfind("\\end{semanticmodule}").expect("its end");
    project.write(
        "src/Main.lex.tex",
        &format!(
            "{}\\begin{{semanticmodule}}\ntheorem{{name: t, parameters: [], statement: true, proof: decide{{}}}}\n{}",
            &main[..begin],
            &main[end..]
        ),
    );
    project.check_fails_with("LLP2003");
}

// ---------------------------------------------------------------------------
// AU-03.
// ---------------------------------------------------------------------------

const MAIN: &str = "src/Main.lex.tex";
const BEGIN: &str = "\\begin{semanticmodule}\n";
const END: &str = "\\end{semanticmodule}";

/// The body of the notation of a project's `Main`.
fn body(project: &P) -> String {
    let text = project.read_raw(MAIN);
    let start = text.find(BEGIN).expect("begin") + BEGIN.len();
    let end = text.rfind(END).expect("end");
    text[start..end].to_owned()
}

fn put_body(project: &P, body: &str) {
    let text = project.read_raw(MAIN);
    let start = text.find(BEGIN).expect("begin") + BEGIN.len();
    let end = text.rfind(END).expect("end");
    project.write(MAIN, &format!("{}{body}{}", &text[..start], &text[end..]));
}

fn fails_closed() {
    let project = P::copy_example("language-1.2");
    let original = body(&project);
    project.check_ok();

    // Malformed in each way the statement names.
    let named: [(&str, String); 5] = [
        (
            "an unclosed delimiter",
            original.replacen("definition{", "definition{ name: x, parameters: [", 1),
        ),
        (
            "a keyword used as a name",
            original.replacen("let doubled", "let let", 1),
        ),
        (
            "a literal that is not canonical",
            original.replacen("(doubledSuccessor 2)", "0x1", 1),
        ),
        (
            "a member written twice",
            original.replacen(
                "name: doubledSuccessor,",
                "name: doubledSuccessor, name: x,",
                1,
            ),
        ),
        (
            "an unknown kind",
            format!("{original}\nfrobnicate{{name: x}}\n"),
        ),
    ];
    for (what, text) in &named {
        assert_ne!(text, &original, "{what}: the plant changed nothing");
        put_body(&project, text);
        let error = project.check_err();
        assert!(
            !error.diagnostics.is_empty(),
            "{what}: refused without a diagnostic"
        );
        for diagnostic in &error.diagnostics {
            let span = diagnostic
                .primary
                .as_ref()
                .unwrap_or_else(|| panic!("{what}: the diagnostic has no position"));
            assert!(span.line_start >= 1, "{what}: no line");
        }
    }
    put_body(&project, &original);
    project.check_ok();

    // Every single-token deletion, duplication, and replacement of the
    // module is refused with a registered code or links to a module that
    // decodes; none panics.
    let tokens = lex(&original).expect("the module lexes");
    assert!(tokens.len() > 80, "{} tokens", tokens.len());
    let mut refused = 0usize;
    let mut linked = 0usize;
    for token in &tokens {
        let mutants = [
            format!("{}{}", &original[..token.start], &original[token.end..]),
            format!(
                "{}{} {}",
                &original[..token.end],
                &original[token.start..token.end],
                &original[token.end..]
            ),
            format!("{})", &original[..token.start]) + &original[token.end..],
            format!("{}{{{}", &original[..token.start], &original[token.end..]),
        ];
        for mutant in mutants {
            if mutant == original {
                continue;
            }
            put_body(&project, &mutant);
            if project.check_err_or_ok() {
                linked += 1;
            } else {
                refused += 1;
            }
        }
    }
    assert!(
        refused > 100 && linked > 0,
        "{refused} refused, {linked} linked: the sweep did not exercise both outcomes"
    );
    put_body(&project, &original);

    // A numeral with a redundant leading zero is LLL1003 outside a string
    // and is string data inside one.
    put_body(
        &project,
        &original.replacen("(doubledSuccessor 2)", "(doubledSuccessor 02)", 1),
    );
    project.check_fails_with("LLL1003");
    put_body(
        &project,
        &format!(
            "{original}\ndefinition{{name: zeros, parameters: [], result: String, body: \"007\"}}\n"
        ),
    );
    project.check_ok();
}

// ---------------------------------------------------------------------------
// AU-04.
// ---------------------------------------------------------------------------

/// The `language` of the project a source belongs to: that of the nearest
/// ancestor directory with a `lexlean.toml`.
fn project_language(source: &std::path::Path) -> Option<String> {
    for directory in source.ancestors().skip(1) {
        let manifest = directory.join("lexlean.toml");
        if let Ok(text) = std::fs::read_to_string(&manifest) {
            return text
                .lines()
                .find_map(|line| line.strip_prefix("language = "))
                .map(|value| value.trim().trim_matches('"').to_owned());
        }
    }
    None
}

/// The payload of a module written as `\semanticdata{…}` and the body of one
/// written in the notation, as the text between the module's markers.
enum Form<'a> {
    Data(&'a str),
    Notation(&'a str),
}

fn form(text: &str) -> Option<Form<'_>> {
    let start = text.find("\\begin{semanticmodule}")? + "\\begin{semanticmodule}".len();
    let end = text.rfind("\\end{semanticmodule}")?;
    let body = text.get(start..end)?;
    let Some(rest) = body.trim_start().strip_prefix("\\semanticdata") else {
        return Some(Form::Notation(body));
    };
    let payload = rest
        .trim_start()
        .strip_prefix('{')?
        .trim_end()
        .strip_suffix('}')?;
    Some(Form::Data(payload))
}

/// Whether a payload written as data can be written in the notation: it is a
/// module of the schema the notation names, and reads back equal.
fn spellable(payload: &str) -> bool {
    let Ok(json) = Json::parse(payload.as_bytes()) else {
        return false;
    };
    let Json::Obj(members) = &json else {
        return false;
    };
    members.get("spec") == Some(&Json::Str(SPEC.to_owned()))
        && print_module(&json)
            .ok()
            .and_then(|text| parse_module(&text, SPEC).ok())
            .is_some_and(|back| back == json)
}

fn one_surface() {
    let root = repo_root();
    let (mut notation, mut canonical, mut exempt_old, mut exempt_refused) = (0usize, 0usize, 0, 0);
    let mut defects = Vec::new();
    for entry in walkdir::WalkDir::new(root.as_std_path())
        .into_iter()
        .filter_entry(|entry| {
            let name = entry.file_name().to_string_lossy();
            name != "target" && name != ".git" && name != ".lexlean"
        })
        .flatten()
    {
        let path = entry.path();
        if !path.to_string_lossy().ends_with(".lex.tex") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(path) else {
            continue;
        };
        let Some(form) = form(&text) else { continue };
        // `/`-separated on every host, as the repository states paths.
        let shown = path
            .strip_prefix(root.as_std_path())
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/");
        // Only language 1.2 has the notation; 1.0 and 1.1 sources keep the
        // bytes they were released with.
        if project_language(path).as_deref() != Some("1.2") {
            if matches!(form, Form::Data(_)) {
                exempt_old += 1;
            }
            continue;
        }
        let refused = shown.starts_with("tests/negative/");
        match form {
            Form::Data(payload) if refused && !spellable(payload) => exempt_refused += 1,
            Form::Data(_) => defects.push(format!(
                "{shown}: written as data, which the notation can say{}",
                if refused { " (a refused fixture)" } else { "" }
            )),
            Form::Notation(body) => {
                notation += 1;
                // A refused fixture may be refused for its text; a source of
                // the product is its own canonical source.
                let printed = parse_module(body, SPEC)
                    .ok()
                    .and_then(|module| print_module(&module).ok());
                if body
                    .strip_prefix('\n')
                    .is_some_and(|rest| printed.as_deref() == Some(rest))
                {
                    canonical += 1;
                } else if !refused {
                    defects.push(format!("{shown}: not its own canonical source"));
                }
            }
        }
    }
    assert!(
        defects.is_empty(),
        "{} source(s) are not written in the authoring notation:\n{}",
        defects.len(),
        defects.join("\n")
    );
    // Anti-vacuity: the walk reads the product's sources, and the data form
    // it exempts is really there to be exempted.
    assert!(
        notation >= 250 && canonical >= 250,
        "{notation} sources in the notation, {canonical} canonical: the walk did not read the product"
    );
    assert!(
        exempt_old > 0 && exempt_refused > 0,
        "{exempt_old} sources under language 1.0 and 1.1, {exempt_refused} refused fixtures written as data: the exemptions are not exercised"
    );
}
