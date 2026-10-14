//! The kinds of module that have an authoring notation, and where a module
//! is in a source (SPEC.md §17.21).
//!
//! One table serves everything that reads a source's module without linking
//! it: the formatter, the identity of the embedded tree, and the authoring
//! API. A module is the body of one environment, either `\semanticdata{…}`
//! (`\toolingdata{…}`) holding canonical JSON, or the notation of the module.

use crate::artifact::canonical_json::Json;

use super::SurfaceError;

/// Reads the notation of a module under a schema tag.
pub type Parse = fn(&str, &str) -> Result<Json, SurfaceError>;

/// Writes the notation of a module.
pub type Print = fn(&Json) -> Result<String, SurfaceError>;

/// A kind of module with a notation.
pub struct Kind {
    /// The environment the module is written in.
    pub environment: &'static str,
    /// The control of its data form.
    pub control: &'static str,
    /// The schema tag the language implies for a module in the notation.
    pub spec: fn() -> Option<&'static str>,
    /// Reads the notation.
    pub parse: Parse,
    /// Writes the notation.
    pub print: Print,
}

/// Semantic modules (§17.11, §17.12).
pub const SEMANTIC: Kind = Kind {
    environment: "semanticmodule",
    control: "\\semanticdata",
    spec: || crate::ir::semantic::semantic_module_spec(crate::LANGUAGE_1_2),
    parse: super::semantic::parse_module,
    print: super::semantic::print_module,
};

/// Compiler-tooling modules (§17.20).
pub const TOOLING: Kind = Kind {
    environment: "toolingmodule",
    control: "\\toolingdata",
    spec: || Some(crate::ir::tooling::TOOLING_SPEC),
    parse: super::tooling::parse_module,
    print: super::tooling::print_module,
};

/// The kinds, in the order a source is searched for them.
pub const KINDS: [&Kind; 2] = [&SEMANTIC, &TOOLING];

/// Where the module of a source is.
pub struct Located {
    /// The kind of the module.
    pub kind: &'static Kind,
    /// The byte range between the environment's markers: the body.
    pub body: (usize, usize),
    /// The byte range of the JSON between the braces of the control, when the
    /// module is written as data; `None` when it is written in the notation.
    pub data: Option<(usize, usize)>,
}

impl Kind {
    /// Whether `other` is the same kind: kinds are identified by the
    /// environment they are written in.
    #[must_use]
    pub fn is(&self, other: &Kind) -> bool {
        self.environment == other.environment
    }

    /// The environment's opening marker.
    #[must_use]
    pub fn begin(&self) -> String {
        format!("\\begin{{{}}}", self.environment)
    }

    /// The environment's closing marker.
    #[must_use]
    pub fn end(&self) -> String {
        format!("\\end{{{}}}", self.environment)
    }
}

/// Find the module of `text`: the first environment of a kind, to the last
/// closing marker of that kind. `None` when `text` holds none, or holds one
/// that is not closed.
#[must_use]
pub fn locate(text: &str) -> Option<Located> {
    for kind in KINDS {
        let begin = kind.begin();
        let Some(at) = text.find(&begin) else {
            continue;
        };
        let start = at + begin.len();
        let end = text.rfind(&kind.end())?;
        let body = text.get(start..end)?;
        let data = body
            .trim_start()
            .strip_prefix(kind.control)
            .and_then(|rest| {
                let open = rest.trim_start().strip_prefix('{')?;
                let close = open.trim_end().strip_suffix('}')?;
                // The offsets of `close` within `text`.
                let from = close.as_ptr() as usize - text.as_ptr() as usize;
                Some((from, from + close.len()))
            });
        return Some(Located {
            kind,
            body: (start, end),
            data,
        });
    }
    None
}

#[cfg(test)]
mod tests {
    use super::locate;

    /// Both forms are found, with the range of what each holds.
    #[test]
    fn the_module_of_a_source_is_found_in_either_form() {
        let data = "\\begin{lexlean}{M}\n\\begin{semanticmodule}\n\\semanticdata{{\"a\":1}}\n\\end{semanticmodule}\n\\end{lexlean}\n";
        let found = locate(data).expect("a module");
        assert_eq!(found.kind.environment, "semanticmodule");
        let (start, end) = found.data.expect("the data form");
        assert_eq!(&data[start..end], "{\"a\":1}");
        let notation =
            "\\begin{lexlean}{M}\n\\begin{toolingmodule}\n{declarations: []}\n\\end{toolingmodule}\n\\end{lexlean}\n";
        let found = locate(notation).expect("a module");
        assert_eq!(found.kind.environment, "toolingmodule");
        assert!(found.data.is_none());
        assert_eq!(
            &notation[found.body.0..found.body.1],
            "\n{declarations: []}\n"
        );
        assert!(locate("\\begin{lexlean}{M}\n\\end{lexlean}\n").is_none());
        assert!(locate("\\begin{semanticmodule}").is_none());
    }
}
