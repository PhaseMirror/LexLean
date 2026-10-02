//! The production-eligibility exhaustiveness audit (SPEC.md §17.13).
//!
//! Production eligibility is decided by mapping every semantic construct to
//! its registry row. A default branch there would admit a construct that
//! nobody classified, so the analysis source may name constructs only
//! explicitly: no wildcard arm, no rest pattern, no `if let`, `while let`,
//! `let`-`else`, or `matches!`, and every variant of every audited IR enum
//! named at least once. The audit reads source text, so it is shared by
//! `cargo xtask validate-model` and the conformance case that plants each
//! forbidden form and observes the audit fail.

use std::fmt::Write as _;

/// The analysis source the audit reads, relative to the repository root.
pub const ELIGIBILITY_SOURCE: &str = "crates/lexlean/src/production/eligibility.rs";

/// The IR source whose enums the analysis must cover.
pub const SEMANTIC_SOURCE: &str = "crates/lexlean/src/ir/semantic.rs";

/// The IR enums whose every variant the analysis names explicitly.
pub const AUDITED_ENUMS: [&str; 5] = [
    "SemanticType",
    "SemanticTerm",
    "SemanticPrimitive",
    "SemanticDeclaration",
    "SemanticInteger",
];

/// The variants of `pub enum <name>` in Rust source, in declaration order.
///
/// # Errors
///
/// Returns a reason when the enum is absent or declares no variant.
pub fn enum_variants(source: &str, name: &str) -> Result<Vec<String>, String> {
    let header = format!("pub enum {name} {{");
    let mut lines = source.lines().skip_while(|line| line.trim() != header);
    if lines.next().is_none() {
        return Err(format!("{SEMANTIC_SOURCE}: `pub enum {name}` is absent"));
    }
    let mut variants = Vec::new();
    for line in lines {
        if line == "}" {
            break;
        }
        // Variants sit at exactly one level of indentation; fields, doc
        // comments, and attributes do not start with an uppercase letter
        // there.
        let Some(rest) = line.strip_prefix("    ") else {
            continue;
        };
        if rest.starts_with(' ') {
            continue;
        }
        let identifier: String = rest
            .chars()
            .take_while(|character| character.is_ascii_alphanumeric() || *character == '_')
            .collect();
        if identifier
            .chars()
            .next()
            .is_some_and(|first| first.is_ascii_uppercase())
        {
            variants.push(identifier);
        }
    }
    if variants.is_empty() {
        return Err(format!(
            "{SEMANTIC_SOURCE}: `pub enum {name}` declares no variant"
        ));
    }
    Ok(variants)
}

/// The code of one line with string and character literals blanked and any
/// line comment removed, so a forbidden spelling in prose or data does not
/// count and a forbidden spelling in code cannot hide behind a quote.
fn code_of(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut chars = line.chars().peekable();
    let mut in_string = false;
    while let Some(character) = chars.next() {
        if in_string {
            match character {
                '\\' => {
                    chars.next();
                }
                '"' => {
                    in_string = false;
                    out.push('"');
                }
                _ => out.push(' '),
            }
            continue;
        }
        match character {
            '"' => {
                in_string = true;
                out.push('"');
            }
            '/' if chars.peek() == Some(&'/') => break,
            _ => out.push(character),
        }
    }
    out
}

/// The forbidden pattern forms, each with the reason it is forbidden.
fn forbidden(code: &str) -> Option<&'static str> {
    let compact: String = code.split_whitespace().collect::<Vec<_>>().join(" ");
    let tight: String = code
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect();
    if tight.contains("_=>") && (compact.contains(" _ =>") || compact.starts_with("_ =>")) {
        return Some("a wildcard arm");
    }
    if tight.contains("..}")
        || tight.contains("..)")
        || tight.contains("..]")
        || tight.contains(",..")
    {
        return Some("a rest pattern");
    }
    if compact.contains("if let ") {
        return Some("an `if let` with an implicit default");
    }
    if compact.contains("while let ") {
        return Some("a `while let` with an implicit default");
    }
    if compact.contains("matches!(") {
        return Some("a `matches!` with an implicit default");
    }
    let let_else = compact.match_indices("let ").any(|(start, _)| {
        let word_start = start == 0
            || !compact[..start].ends_with(|c: char| c.is_ascii_alphanumeric() || c == '_');
        let statement = compact[start..].split(';').next().unwrap_or_default();
        word_start && statement.contains(" else {")
    });
    if let_else {
        return Some("a `let`-`else` with an implicit default");
    }
    None
}

/// Audit the eligibility analysis source against the IR source.
///
/// # Errors
///
/// Returns every violation found, one per line.
pub fn audit_eligibility(eligibility: &str, semantic: &str) -> Result<(), String> {
    let mut report = String::new();
    for (index, line) in eligibility.lines().enumerate() {
        let code = code_of(line);
        if let Some(reason) = forbidden(&code) {
            let _ = writeln!(
                report,
                "{ELIGIBILITY_SOURCE}:{}: {reason} would classify constructs by default",
                index + 1
            );
        }
    }
    let code: String = eligibility
        .lines()
        .map(code_of)
        .collect::<Vec<_>>()
        .join("\n");
    for name in AUDITED_ENUMS {
        for variant in enum_variants(semantic, name)? {
            let path = format!("{name}::{variant}");
            let named = code.match_indices(&path).any(|(start, _)| {
                !code[start + path.len()..]
                    .chars()
                    .next()
                    .is_some_and(|next| next.is_ascii_alphanumeric() || next == '_')
            });
            if !named {
                let _ = writeln!(
                    report,
                    "{ELIGIBILITY_SOURCE}: `{path}` has no explicit production disposition"
                );
            }
        }
    }
    if report.is_empty() {
        Ok(())
    } else {
        Err(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const IR: &str = "pub enum SemanticType {\n    Nat,\n    List { element: Box<Self> },\n}\npub enum SemanticTerm {\n    Var { name: String },\n}\npub enum SemanticPrimitive {\n    Length,\n}\npub enum SemanticDeclaration {\n    Theorem {\n        name: String,\n    },\n}\npub enum SemanticInteger {\n    Int,\n}\n";

    const CLEAN: &str = "fn f(t: &SemanticType) { match t { SemanticType::Nat => {} SemanticType::List { element: _ } => {} } }\n// SemanticTerm::Var SemanticPrimitive::Length SemanticDeclaration::Theorem\nconst K: [&str; 1] = [\"_ => .. }\"];\nfn g() { SemanticTerm::Var; SemanticPrimitive::Length; SemanticDeclaration::Theorem; SemanticInteger::Int; }\n";

    #[test]
    fn variants_are_read_at_one_indentation_level() {
        assert_eq!(enum_variants(IR, "SemanticType").unwrap(), ["Nat", "List"]);
        assert_eq!(
            enum_variants(IR, "SemanticDeclaration").unwrap(),
            ["Theorem"]
        );
    }

    #[test]
    fn explicit_source_passes_and_comments_or_strings_do_not_count() {
        audit_eligibility(CLEAN, IR).unwrap();
    }

    #[test]
    fn every_default_form_and_every_unnamed_variant_fails() {
        for planted in [
            "fn h(t: &SemanticType) { match t { _ => {} } }",
            "fn h(t: &SemanticTerm) { match t { SemanticTerm::Var { .. } => {} } }",
            "fn h(t: &SemanticType) { if let SemanticType::Nat = t {} }",
            "fn h(t: &SemanticType) -> bool { matches!(t, SemanticType::Nat) }",
            "fn h(t: Option<u8>) { while let Some(_) = t {} }",
            "fn h(t: Option<u8>) { let Some(_) = t else { return }; }",
        ] {
            let source = format!("{CLEAN}{planted}\n");
            assert!(audit_eligibility(&source, IR).is_err(), "{planted}");
        }
        let missing = CLEAN.replace("SemanticInteger::Int", "SemanticInteger");
        let error = audit_eligibility(&missing, IR).unwrap_err();
        assert!(error.contains("SemanticInteger::Int"), "{error}");
    }
}
