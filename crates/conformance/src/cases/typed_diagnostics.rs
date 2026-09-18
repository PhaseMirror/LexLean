//! CL-21: actual public Engine failures, not message-derived classifications.

use lexlean::{CheckRequest, Diagnostic, DiagnosticDetail, LockRequest, Selection};

use crate::support::{self, P};

fn one(project: &P, code: &str) -> Diagnostic {
    let error = project.check_fails_with(code);
    let diagnostic = error
        .diagnostics
        .iter()
        .find(|d| d.code.as_str() == code)
        .expect("code")
        .clone();
    let snapshot = project
        .engine()
        .snapshot(CheckRequest {
            selection: Selection::Entrypoints,
        })
        .expect_err("snapshot rejects same source");
    assert_eq!(snapshot.diagnostics, error.diagnostics);
    diagnostic
}

fn unchanged_wire(project: &P, fixture: &str, diagnostic: &Diagnostic) {
    let expected = std::fs::read_to_string(support::repo_root().join(format!(
        "tests/negative/{fixture}/expected/diagnostics.json"
    )))
    .expect("committed diagnostics");
    assert_eq!(
        format!("[{}]\n", diagnostic.to_json().to_canonical_string()),
        expected
    );
    let (exit, stdout, stderr) = project.cli(&["--diagnostic-format", "json", "check"]);
    assert_eq!(exit, 1);
    assert!(stderr.is_empty());
    let result: serde_json::Value = serde_json::from_str(&stdout).expect("CLI JSON");
    assert_eq!(
        result["diagnostics"],
        serde_json::from_str::<serde_json::Value>(&expected).expect("expected JSON")
    );
}

fn expect_terms(diagnostic: &Diagnostic, source: &str) {
    let Some(DiagnosticDetail::UnqualifiedCrossPackageTermAmbiguity {
        candidates,
        packages,
        span,
    }) = diagnostic.detail()
    else {
        panic!("actual term collision must carry native detail: {diagnostic:?}");
    };
    assert_eq!(candidates, &["test.dupa::nzz", "test.dupb::nzz"]);
    assert_eq!(packages, &["test.dupa@1.0.0", "test.dupb@1.0.0"]);
    assert_eq!(span.path, "src/Main.lex.tex");
    assert_eq!(&source[span.byte_start..span.byte_end], "nzz");
    assert_eq!(diagnostic.clone().detail(), diagnostic.detail());
}

fn lexical_cases() {
    let ambiguous = P::negative("ambiguous-typed-resolution");
    let diagnostic = one(&ambiguous, "LLP2002");
    expect_terms(&diagnostic, &ambiguous.read("src/Main.lex.tex"));
    unchanged_wire(&ambiguous, "ambiguous-typed-resolution", &diagnostic);
    ambiguous.edit(
        "src/Main.lex.tex",
        "\\useglossary{test.dupa@1.0.0}\n\\useglossary{test.dupb@1.0.0}",
        "\\useglossary{test.dupb@1.0.0}\n\\useglossary{test.dupa@1.0.0}",
    );
    expect_terms(
        &one(&ambiguous, "LLP2002"),
        &ambiguous.read("src/Main.lex.tex"),
    );

    // A separate explicitly qualified argument does not hide the unqualified
    // collision at nzz. Qualifying the colliding term itself resolves it.
    ambiguous.edit("src/Main.lex.tex", "nzz(z)", r"nzz(\lexeme{test.dupa::z})");
    expect_terms(
        &one(&ambiguous, "LLP2002"),
        &ambiguous.read("src/Main.lex.tex"),
    );
    ambiguous.edit("src/Main.lex.tex", "nzz(", r"\lexeme{test.dupa::nzz}(");
    ambiguous.check_ok();
    let snapshot = ambiguous
        .engine()
        .snapshot(CheckRequest {
            selection: Selection::Entrypoints,
        })
        .expect("qualified snapshot");
    assert_eq!(
        snapshot,
        ambiguous
            .engine()
            .snapshot(CheckRequest {
                selection: Selection::Entrypoints
            })
            .expect("repeat snapshot")
    );

    let same_package = P::negative("ambiguous-typed-resolution");
    let other = same_package
        .read("lexicons/test-dupb/entries/nzz.toml")
        .replacen("id = \"nzz\"", "id = \"nzz2\"", 1);
    same_package.write("lexicons/test-dupa/entries/nzz2.toml", &other);
    same_package.edit("src/Main.lex.tex", "\\useglossary{test.dupb@1.0.0}\n", "");
    same_package.relock();
    assert!(
        one(&same_package, "LLP2002").detail().is_none(),
        "one package is not a cross-package collision"
    );

    let segmented = P::negative("ambiguous-lexical-segmentation");
    assert!(one(&segmented, "LLP2002").detail().is_none());

    let prefixed = P::negative("ambiguous-typed-resolution");
    for path in [
        "lexlean.toml",
        "src/Main.lex.tex",
        "lexicons/test-dupb/lexicon.toml",
    ] {
        prefixed.edit(path, "test.dupb", "test.dupa.child");
    }
    prefixed.relock();
    let diagnostic = one(&prefixed, "LLP2002");
    let Some(DiagnosticDetail::UnqualifiedCrossPackageTermAmbiguity { candidates, .. }) =
        diagnostic.detail()
    else {
        panic!("prefix packages still collide")
    };
    assert_eq!(
        candidates,
        &["test.dupa.child::nzz", "test.dupa::nzz"],
        "sort qualified IDs, not their unjoined components"
    );
}

fn cycle_cases() {
    let cyclic = P::negative("lexicon-cycle");
    let diagnostic = one(&cyclic, "LLR3003");
    assert_eq!(
        diagnostic.detail(),
        Some(&DiagnosticDetail::PackageImportCycle {
            packages: vec!["test.cyca".into(), "test.cycb".into(), "test.cyca".into()],
            importers: vec!["test.cyca@1.0.0".into(), "test.cycb@1.0.0".into()],
        })
    );
    unchanged_wire(&cyclic, "lexicon-cycle", &diagnostic);
    cyclic.edit(
        "lexicons/test-cycb/lexicon.toml",
        ", \"test.cyca@1.0.0\"",
        ", \"lexlean.std.nat@1.0.0\"",
    );
    cyclic.relock();
    cyclic.check_ok();

    cyclic.edit(
        "lexicons/test-cyca/lexicon.toml",
        "test.cycb@1.0.0",
        "test.cyca@1.0.0",
    );
    let error = cyclic
        .engine()
        .lock(LockRequest {
            check_only: false,
            allow_network: false,
        })
        .err()
        .expect("self import rejected during package loading");
    let diagnostic = error
        .diagnostics
        .iter()
        .find(|d| d.code.as_str() == "LLR3003")
        .expect("cycle");
    assert_eq!(
        diagnostic.detail(),
        Some(&DiagnosticDetail::PackageImportCycle {
            packages: vec!["test.cyca".into(), "test.cyca".into()],
            importers: vec!["test.cyca@1.0.0".into()],
        })
    );

    let modules = P::example();
    modules.edit(
        "src/Main.lex.tex",
        "\\title",
        "\\importmodule{Main}\n\\title",
    );
    assert!(
        one(&modules, "LLR3003").detail().is_none(),
        "module cycle is not a package import cycle"
    );
    modules.edit(
        "src/Main.lex.tex",
        "\\importmodule{Main}",
        "\\importmodule{Helper}",
    );
    modules.write(
        "src/Helper.lex.tex",
        &modules
            .read("src/Main.lex.tex")
            .replacen("{Main}", "{Helper}", 1)
            .replace("\\importmodule{Helper}", "\\importmodule{Main}"),
    );
    assert!(
        one(&modules, "LLR3003").detail().is_none(),
        "multi-module cycle is not a package import cycle"
    );

    let denotations = P::example();
    let entry = |name: &str, other: &str| {
        format!(
            r#"spec = "lexlean/entry/1"
id = "{name}"
category = "term-constant"
signature = "(const lexlean.std.nat::nat)"
surface_arity = 0
frame = "atom"
[denotation]
kind = "defined"
value = "(const test.pkg::{other})"
[[form]]
id = "{name}"
channel = "both"
surface = "{name}"
canonical_source = true
features = []
[render]
math = "(operator-name {name})"
"#
        )
    };
    denotations.add_package(
        "lexicons/test-pkg",
        "test.pkg",
        &["lexlean.core@1.0.0", "lexlean.std.nat@1.0.0"],
        &[
            ("cyca.toml", &entry("cyca", "cycb")),
            ("cycb.toml", &entry("cycb", "cyca")),
        ],
    );
    denotations.relock();
    assert!(
        one(&denotations, "LLR3003").detail().is_none(),
        "denotation cycle is not a package import cycle"
    );
}

fn structural_cases() {
    let binder = P::example();
    binder.write("src/Main.lex.tex", "\\begin{lexlean}{Main}\n\\useglossary{lexlean.std.nat@1.0.0}\n\\title{Natural number addition}\n\\begin{section}{basics}\n\\heading{Natural number addition}\n\\parameters{natural number \\(n\\)}\n\\begin{theorem}{param-use}\n\\noaxioms\n\\(n + 0 = n\\).\n\\begin{proof}\nClose the goal by reflexivity.\n\\end{proof}\n\\end{theorem}\n\\end{section}\n\\end{lexlean}\n");
    binder.check_ok();
    binder.add_package(
        "lexicons/test-dupnat",
        "test.dupnat",
        &["lexlean.core@1.0.0"],
        &[("nat2.toml", super::declarations::DUP_NAT_ENTRY)],
    );
    binder.edit(
        "src/Main.lex.tex",
        "\\title",
        "\\useglossary{test.dupnat@1.0.0}\n\\title",
    );
    binder.relock();
    assert!(
        one(&binder, "LLP2002").detail().is_none(),
        "binder alternatives are not linked term alternatives"
    );

    let notation = P::example();
    notation.check_ok();
    let entry = super::grammar::BUMP_ENTRY.replace("surface = \"⊕\"", "surface = \"+\"");
    notation.add_package(
        "lexicons/test-prec",
        "test.prec",
        &["lexlean.core@1.0.0", "lexlean.std.nat@1.0.0"],
        &[("bump.toml", &entry)],
    );
    notation.edit(
        "src/Main.lex.tex",
        "\\title",
        "\\useglossary{test.prec@1.0.0}\n\\title",
    );
    notation.relock();
    assert!(
        one(&notation, "LLP2002").detail().is_none(),
        "operator profiles are not lexical term candidates"
    );
    // With identical profiles these candidates reach the linked-term
    // ambiguity producer; differing operator meanings are still not terms.
    notation.edit(
        "lexicons/test-prec/entries/bump.toml",
        "precedence = 255",
        "precedence = 65",
    );
    notation.edit(
        "lexicons/test-prec/entries/bump.toml",
        "name = \"Nat.add\"",
        "name = \"Nat.sub\"",
    );
    notation.relock();
    assert!(
        one(&notation, "LLP2002").detail().is_none(),
        "linked operator alternatives are not lexical term candidates"
    );
}

fn importer_cases() {
    let project = P::negative("lexicon-cycle");
    let entry = project.read("lexicons/test-cyca/entries/cyca.toml");
    for (id, imports) in [
        (
            "test.parent",
            vec![
                "lexlean.core@1.0.0",
                "lexlean.std.nat@1.0.0",
                "test.cyca@1.0.0",
            ],
        ),
        (
            "test.unrelated",
            vec!["lexlean.core@1.0.0", "lexlean.std.nat@1.0.0"],
        ),
    ] {
        project.add_package(
            &format!("lexicons/{id}"),
            id,
            &imports,
            &[("cyca.toml", &entry)],
        );
    }
    project.relock();
    let expected = vec!["test.cyca@1.0.0", "test.cycb@1.0.0", "test.parent@1.0.0"];
    let assert_importers = |diagnostic: &Diagnostic, expected: &[&str]| {
        let Some(DiagnosticDetail::PackageImportCycle { importers, .. }) = diagnostic.detail()
        else {
            panic!("package cycle detail")
        };
        assert_eq!(
            importers, expected,
            "only proven exact-version importers reach the cycle"
        );
    };
    assert_importers(&one(&project, "LLR3003"), &expected);
    // Project declarations are canonically sorted. Perturb the actual loaded
    // graph at its owning internal boundary instead, not that public syntax.
    let loaded = lexlean::project::Project::load(&project.root.join("lexlean.toml")).unwrap();
    let mut packages = lexlean::lock::resolve_packages(&loaded, false)
        .unwrap_or_else(|error| panic!("package load: {error:?}"))
        .packages;
    for _ in 0..packages.len() {
        packages.rotate_left(1);
        let errors = lexlean::lexicon::resolve::Closure::build(
            packages.clone(),
            lexlean::lexicon::load_token_registry().unwrap(),
            lexlean::lexicon::load_bootstrap_for("1.0").unwrap(),
            128,
        )
        .unwrap_err();
        assert_importers(
            errors
                .iter()
                .find(|d| d.code.as_str() == "LLR3003")
                .unwrap(),
            &expected,
        );
    }

    // Same provenance for the earlier package-local self-import failure;
    // successful parent edges are retained without deferring that failure.
    project.edit(
        "lexicons/test-cyca/lexicon.toml",
        "test.cycb@1.0.0",
        "test.cyca@1.0.0",
    );
    let failure = || {
        project
            .engine()
            .lock(LockRequest {
                check_only: false,
                allow_network: false,
            })
            .err()
            .expect("self-import failure")
    };
    let error = failure();
    let diagnostic = error
        .diagnostics
        .iter()
        .find(|d| d.code.as_str() == "LLR3003")
        .unwrap();
    assert_importers(diagnostic, &expected);

    project.edit(
        "lexicons/test.parent/lexicon.toml",
        "test.cyca@1.0.0",
        "test.cyca@2.0.0",
    );
    let error = failure();
    let diagnostic = error
        .diagnostics
        .iter()
        .find(|d| d.code.as_str() == "LLR3003")
        .unwrap();
    assert_importers(diagnostic, &["test.cyca@1.0.0", "test.cycb@1.0.0"]);
    project.edit(
        "lexicons/test-cyca/lexicon.toml",
        "test.cyca@1.0.0",
        "test.cyca@2.0.0",
    );
    let error = failure();
    assert!(
        error
            .diagnostics
            .iter()
            .filter(|d| d.code.as_str() == "LLR3003")
            .all(|d| d.detail().is_none()),
        "wrong-version self import is not a proven exact-version cycle"
    );
}

pub(super) fn run() {
    // Constructor-based downstream source compatibility, with native metadata
    // unavailable to this separate crate's diagnostic producers.
    let diagnostic = Diagnostic::new(lexlean::code!("LLP2002"), "consumer diagnostic");
    assert!(diagnostic.detail().is_none());
    lexical_cases();
    cycle_cases();
    structural_cases();
    importer_cases();
}
