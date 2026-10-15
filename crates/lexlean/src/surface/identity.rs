//! The data form of a source for the compiler-semantics ID (SPEC.md §21.2).
//!
//! A file of the embedded tree that holds a semantic or tooling module in the
//! authoring notation contributes the same module written as data, which is
//! what every source of that module was before the notation existed: the
//! identity of the compiler is the meaning of its normative data, not the
//! form a person writes it in, so moving a source to the notation changes no
//! identity and moving it back changes none either.

/// How far into a source its module environment begins; a source whose
/// environment is not within it holds no module of the kinds below, and is not
/// read further (the embedded tree holds sources of tens of megabytes).
const HEADER: usize = 8192;

/// The bytes of `bytes` with the body of its semantic or tooling module,
/// written in the notation, written as data; `None` when `path` is not a
/// source or holds no such module, or holds one already written as data or
/// that is not well formed (it then contributes its own bytes).
#[must_use]
pub fn data_form(path: &str, bytes: &[u8]) -> Option<Vec<u8>> {
    if !path.ends_with(".lex.tex") {
        return None;
    }
    let head = bytes.get(..HEADER.min(bytes.len()))?;
    let head = match std::str::from_utf8(head) {
        Ok(text) => text,
        Err(error) => std::str::from_utf8(&head[..error.valid_up_to()]).ok()?,
    };
    for kind in KINDS {
        let begin = format!("\\begin{{{}}}", kind.environment);
        let Some(at) = head.find(&begin) else {
            continue;
        };
        let text = std::str::from_utf8(bytes).ok()?;
        let start = at + begin.len();
        let end = text.rfind(&format!("\\end{{{}}}", kind.environment))?;
        let body = text.get(start..end)?;
        if body.trim_start().starts_with(kind.control) {
            return None;
        }
        let spec = (kind.spec)()?;
        let module = (kind.parse)(body, spec).ok()?;
        return Some(
            format!(
                "{}\n{}{{{}}}\n{}",
                &text[..start],
                kind.control,
                module.to_canonical_string(),
                &text[end..]
            )
            .into_bytes(),
        );
    }
    None
}

type Parse = fn(&str, &str) -> Result<crate::artifact::canonical_json::Json, super::SurfaceError>;

/// A kind of module with a notation: the environment it is written in, the
/// control of its data form, the schema tag the language implies, and the
/// reader of its notation.
struct Kind {
    environment: &'static str,
    control: &'static str,
    spec: fn() -> Option<&'static str>,
    parse: Parse,
}

/// The kinds of module with a notation.
const KINDS: [Kind; 2] = [
    Kind {
        environment: "semanticmodule",
        control: "\\semanticdata",
        spec: || crate::ir::semantic::semantic_module_spec(crate::LANGUAGE_1_2),
        parse: super::semantic::parse_module,
    },
    Kind {
        environment: "toolingmodule",
        control: "\\toolingdata",
        spec: || Some(crate::ir::tooling::TOOLING_SPEC),
        parse: super::tooling::parse_module,
    },
];

#[cfg(test)]
mod tests {
    use super::data_form;

    /// A module in the notation contributes its data form, which is the text a
    /// source written as data is; a module written as data, a source that
    /// holds no module, and a text that is not a module contribute nothing.
    #[test]
    fn the_notation_contributes_its_data_form() {
        let notation = "\\begin{lexlean}{M}\n\\title{t}\n\\begin{semanticmodule}\ntheorem{\n  name: t,\n  parameters: [],\n  statement: true,\n  proof: decide{}\n}\n\\end{semanticmodule}\n\\end{lexlean}\n";
        let data = data_form("src/M.lex.tex", notation.as_bytes()).expect("a data form");
        let data = String::from_utf8(data).expect("UTF-8");
        assert!(
            data.contains("\\begin{semanticmodule}\n\\semanticdata{{\"declarations\":[{"),
            "{data}"
        );
        assert!(
            data.ends_with("\\end{semanticmodule}\n\\end{lexlean}\n"),
            "{data}"
        );
        assert_eq!(data_form("src/M.lex.tex", data.as_bytes()), None);
        assert_eq!(data_form("src/M.toml", notation.as_bytes()), None);
        assert_eq!(
            data_form("src/M.lex.tex", b"\\begin{lexlean}{M}\n\\end{lexlean}\n"),
            None
        );
        let broken = notation.replace("name: t", "name: ");
        assert_eq!(data_form("src/M.lex.tex", broken.as_bytes()), None);
    }
}
