//! Canonical identity (SPEC.md §33.1).
//!
//! The identity of a lexeme is the hash of a canonical form, never of raw
//! bytes. Canonicalization here is structural: the source is lexed to a token
//! stream, layout is normalized away, top-level declarations are located and
//! sorted by fully qualified name, and the result is the §21.1 frame encoding
//! under the [`CANONICAL_DOMAIN`](crate::lexeme::CANONICAL_DOMAIN).
//!
//! The limit is deliberate and stated in the entry: this is not semantic
//! equivalence. Two sources that elaborate to the same term but tokenize
//! differently receive different digests. That is a narrower claim than the
//! one a reader might assume, so the entry records the canonicalization
//! identifier and every consumer can see which claim is being made.

use crate::artifact::content_id::{FramedHasher, Sha256Digest};

use super::{CANONICALIZATION, CANONICAL_DOMAIN};

/// One lexical token, reduced to the distinctions the canonical form keeps.
///
/// The token text is the source spelling with layout collapsed; there is no
/// separate category field because the canonical form hashes the token stream
/// itself, and the declaration splitter only needs to recognize keywords and
/// declaration heads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    /// The normalized token text.
    pub text: String,
    /// One-based source line, for diagnostics only; never hashed.
    pub line: usize,
    /// One-based source column, for diagnostics only; never hashed.
    pub column: usize,
}

/// One top-level declaration of a source, with its canonical name and body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Declaration {
    /// The fully qualified name: the namespace path joined to the declared
    /// identifier with `.`, exactly as §33.1 step 3 forms it.
    pub name: String,
    /// The normalized token stream of the declaration body.
    pub body: Vec<Token>,
    /// One-based source line of the declaration's first token.
    pub line: usize,
}

impl Declaration {
    /// This declaration's **body proper**: its canonical token stream with the
    /// declaration head removed.
    ///
    /// The head is the two leading tokens the splitter has already identified,
    /// the keyword and the declared name, so the body proper is exactly
    /// `body[2..]`. §34.2 takes both the reference relation `A[i][j]` and the
    /// token count `t_j` over this stream rather than over `body` as a whole,
    /// because the head contains the declaration's own name and scanning it
    /// would make every atom trivially reference itself.
    ///
    /// This returns `body` itself rather than a slice whenever the head is not
    /// actually there, so a hand-built `Declaration` cannot panic here. The
    /// splitter only ever builds declarations with a head, so the fallback is
    /// unreachable through the canonicalizer.
    #[must_use]
    pub fn body_proper(&self) -> &[Token] {
        self.body.get(2..).unwrap_or(&self.body)
    }
}

/// The result of canonicalizing one source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalSource {
    /// The declarations in ascending UTF-8 byte order of [`Declaration::name`].
    pub declarations: Vec<Declaration>,
}

impl CanonicalSource {
    /// The declaration names in canonical order.
    #[must_use]
    pub fn names(&self) -> Vec<String> {
        self.declarations
            .iter()
            .map(|declaration| declaration.name.clone())
            .collect()
    }

    /// The canonical form of §33.1 step 5: the framed byte string, with no
    /// trailing newline.
    ///
    /// Every frame is length-prefixed by §21.1, so the encoding is injective:
    /// no two distinct declaration sequences can produce the same bytes. That
    /// is what makes the digest collision-resistant for reasons stronger than
    /// raw concatenation, and it is why a verifier that has the entry can
    /// recompute these bytes without re-deriving the boundaries.
    #[must_use]
    pub fn canonical_form(&self, toolchain: &str) -> Vec<u8> {
        let mut frames = FrameWriter::new(CANONICAL_DOMAIN);
        frames.frame("canonicalization", CANONICALIZATION.as_bytes());
        frames.frame("toolchain", toolchain.as_bytes());
        for declaration in &self.declarations {
            frames.frame("declaration-name", declaration.name.as_bytes());
            frames.frame(
                "declaration-body",
                render_tokens(&declaration.body).as_bytes(),
            );
        }
        frames.finish()
    }

    /// The SHA-256 of [`Self::canonical_form`], the entry's `content_digest`.
    #[must_use]
    pub fn content_digest(&self, toolchain: &str) -> Sha256Digest {
        Sha256Digest::of(&self.canonical_form(toolchain))
    }

    /// Whether this canonical form is byte-identical to the one
    /// [`FramedHasher`](crate::artifact::content_id::FramedHasher) digests for
    /// the same recipe.
    ///
    /// The two encoders exist for different consumers --- the hasher for the
    /// compound identifiers of §21.1, the writer for a byte string a third
    /// party can reproduce --- and a divergence between them would silently make
    /// §33.1 unverifiable. The conformance suite calls this so the equality is
    /// checked rather than assumed.
    #[must_use]
    pub fn matches_framed_hasher(&self, toolchain: &str) -> bool {
        let mut hasher = FramedHasher::new(CANONICAL_DOMAIN);
        hasher.frame("canonicalization", CANONICALIZATION.as_bytes());
        hasher.frame("toolchain", toolchain.as_bytes());
        for declaration in &self.declarations {
            hasher.frame("declaration-name", declaration.name.as_bytes());
            hasher.frame(
                "declaration-body",
                render_tokens(&declaration.body).as_bytes(),
            );
        }
        hasher.finish() == self.content_digest(toolchain)
    }
}

/// The §21.1 frame encoding emitted as bytes rather than streamed into a
/// hasher: `u32be(len(label)) || label || u64be(len(bytes)) || bytes`, after a
/// domain prefix and its zero terminator.
///
/// The byte-string form exists because a third party has to reproduce the
/// canonical form exactly, not merely reach the same digest: the browser
/// verifier of §33.8 recomputes these bytes, and the bootstrap entry publishes
/// them as an artifact.
#[derive(Debug, Clone)]
pub struct FrameWriter {
    out: Vec<u8>,
}

impl FrameWriter {
    /// Start a recipe with its domain-separation prefix, `domain` and a zero
    /// terminator.
    #[must_use]
    pub fn new(domain: &str) -> Self {
        let mut out = Vec::new();
        out.extend_from_slice(domain.as_bytes());
        out.push(0u8);
        Self { out }
    }

    /// Append one `frame(label, bytes)` of §21.1.
    pub fn frame(&mut self, label: &str, bytes: &[u8]) {
        let label_len = u32::try_from(label.len()).expect("a frame label fits u32");
        self.out.extend_from_slice(&label_len.to_be_bytes());
        self.out.extend_from_slice(label.as_bytes());
        let payload_len = u64::try_from(bytes.len()).expect("a frame payload fits u64");
        self.out.extend_from_slice(&payload_len.to_be_bytes());
        self.out.extend_from_slice(bytes);
    }

    /// The framed bytes, with no trailing newline.
    #[must_use]
    pub fn finish(self) -> Vec<u8> {
        self.out
    }
}

/// Render a token stream as the canonical single-space-joined text.
///
/// Two adjacent tokens that would otherwise merge when concatenated stay
/// separated by that one space, which is why the body is rendered as text and
/// then framed rather than framed token by token: the frame length would mask
/// a difference the text form would expose.
#[must_use]
pub fn render_tokens(tokens: &[Token]) -> String {
    let mut out = String::new();
    for (index, token) in tokens.iter().enumerate() {
        if index > 0 {
            out.push(' ');
        }
        out.push_str(&token.text);
    }
    out
}

/// The declaration keywords of §33.1 step 3, all four characters or fewer, so
/// the match is exact rather than a prefix.
const DECLARATION_KEYWORDS: [&str; 9] = [
    "abbrev",
    "theorem",
    "example",
    "inductive",
    "instance",
    "structure",
    "axiom",
    "class",
    "def",
];

/// A cursor over a source that tracks the display coordinates §20.1 reports in
/// diagnostics and that the canonical form deliberately ignores.
struct Cursor<'source> {
    source: &'source str,
    at: usize,
    line: usize,
    column: usize,
}

impl<'source> Cursor<'source> {
    fn new(source: &'source str) -> Self {
        Self {
            source,
            at: 0,
            line: 1,
            column: 1,
        }
    }

    /// The next scalar, advancing the display coordinates.
    fn bump(&mut self) -> Option<char> {
        let rest = self.source.get(self.at..)?;
        let ch = rest.chars().next()?;
        self.at += ch.len_utf8();
        if ch == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        Some(ch)
    }

    /// Whether the rest of the source starts with `prefix`.
    fn looking_at(&self, prefix: &str) -> bool {
        self.source[self.at..].starts_with(prefix)
    }

    /// Consume `prefix` if it is next.
    fn eat(&mut self, prefix: &str) -> bool {
        if !self.looking_at(prefix) {
            return false;
        }
        for _ in 0..prefix.chars().count() {
            let _ = self.bump();
        }
        true
    }
}

/// Whether a scalar starts a Lean identifier.
///
/// Lean's own rule admits any non-ASCII scalar into a name, so the predicate
/// keeps a Unicode name whole instead of splitting it into punctuation that
/// would then differ between two equivalent spellings.
fn is_identifier_start(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphabetic() || (!ch.is_ascii() && ch.is_alphanumeric())
}

/// Whether a scalar continues a Lean identifier.
fn is_identifier_continue(ch: char) -> bool {
    is_identifier_start(ch) || ch.is_ascii_digit() || ch == '!' || ch == '?' || ch == '\''
}

/// Lex a Lean source into the normalized token stream of §33.1 steps 1 and 2.
///
/// Line comments, nestable block comments, and string literals are each handled
/// as a unit, so a keyword inside a string or a comment is never read as a
/// declaration head and a `--` inside a string never starts a comment.
#[must_use]
pub fn lex(source: &str) -> Vec<Token> {
    let mut cursor = Cursor::new(source);
    let mut tokens: Vec<Token> = Vec::new();

    while let Some(ch) = cursor.bump() {
        if ch.is_whitespace() {
            continue;
        }
        let line = cursor.line;
        let column = cursor.column.saturating_sub(1).max(1);

        if ch == '-' && cursor.looking_at("-") {
            while let Some(next) = cursor.bump() {
                if next == '\n' {
                    break;
                }
            }
            continue;
        }
        if ch == '/' && cursor.looking_at("*") {
            let _ = cursor.eat("*");
            let mut depth = 1usize;
            while depth > 0 {
                if cursor.eat("*/") {
                    depth -= 1;
                } else if cursor.eat("/*") {
                    depth += 1;
                } else if cursor.bump().is_none() {
                    // An unterminated comment ends the token stream; the
                    // canonical form of a truncated source is still
                    // deterministic, which is what hashing requires.
                    break;
                }
            }
            continue;
        }
        if ch == '"' {
            let mut text = String::from('"');
            while let Some(next) = cursor.bump() {
                text.push(next);
                match next {
                    '\\' => {
                        if let Some(escaped) = cursor.bump() {
                            text.push(escaped);
                        }
                    }
                    '"' => break,
                    _ => {}
                }
            }
            tokens.push(Token { text, line, column });
            continue;
        }
        if is_identifier_start(ch) {
            let mut text = String::from(ch);
            while let Some(next) = cursor
                .source
                .get(cursor.at..)
                .and_then(|rest| rest.chars().next())
            {
                if !is_identifier_continue(next) {
                    break;
                }
                let _ = cursor.bump();
                text.push(next);
            }
            tokens.push(Token { text, line, column });
            continue;
        }
        tokens.push(Token {
            text: ch.to_string(),
            line,
            column,
        });
    }
    tokens
}

/// Split a token stream into the top-level declarations of §33.1 step 3 and
/// name each by its namespace path.
///
/// A `namespace` or `section` opens a path component and the matching `end`
/// pops it, so a declaration's fully qualified name is the path it sits under.
/// A declaration extends to the next declaration head, so that two sources
/// differing only in whether they wrote `theorem foo` on one line or three hash
/// one declaration body.
///
/// # Errors
/// Returns the offending line and reason when two declarations resolve to the
/// same fully qualified name, which §33.1 forbids because the canonical form
/// would then depend on declaration order rather than on names.
pub fn split_declarations(tokens: &[Token]) -> Result<Vec<Declaration>, (String, usize)> {
    let mut declarations = Vec::new();
    let mut prefix: Vec<String> = Vec::new();
    let mut index = 0usize;

    while index < tokens.len() {
        let token = &tokens[index];
        match token.text.as_str() {
            "section" => {
                if let Some(name) = tokens.get(index + 1) {
                    prefix.push(name.text.clone());
                }
                index += 2;
                continue;
            }
            "end" => {
                prefix.pop();
                index += 1;
                continue;
            }
            "namespace" => {
                if let Some(name) = tokens.get(index + 1) {
                    prefix.push(name.text.clone());
                }
                index += 2;
                continue;
            }
            _ => {}
        }
        if !DECLARATION_KEYWORDS.contains(&token.text.as_str()) {
            index += 1;
            continue;
        }
        let Some(name_token) = tokens.get(index + 1) else {
            return Err((
                "a declaration keyword is not followed by a name".to_owned(),
                token.line,
            ));
        };
        let mut name = String::new();
        for part in &prefix {
            name.push_str(part);
            name.push('.');
        }
        name.push_str(&name_token.text);

        let body_start = index;
        index += 2;
        while index < tokens.len() {
            let next = &tokens[index];
            if next.text == "section" || next.text == "end" || next.text == "namespace" {
                break;
            }
            if DECLARATION_KEYWORDS.contains(&next.text.as_str())
                && tokens.get(index + 1).is_some_and(|after| {
                    after.text != "["
                        && !matches!(
                            after.text.as_str(),
                            "where" | "extends" | "extends?" | "where?" | "deriving"
                        )
                })
            {
                break;
            }
            index += 1;
        }
        declarations.push(Declaration {
            name,
            body: tokens[body_start..index].to_vec(),
            line: tokens[body_start].line,
        });
    }

    declarations.sort_by(|left, right| left.name.as_bytes().cmp(right.name.as_bytes()));
    for pair in declarations.windows(2) {
        if pair[0].name == pair[1].name {
            return Err((
                format!("two declarations are named `{}`", pair[0].name),
                pair[1].line,
            ));
        }
    }
    Ok(declarations)
}

/// Canonicalize one Lean source (SPEC.md §33.1).
///
/// # Errors
/// Returns the offending line and reason when two declarations share a fully
/// qualified name.
pub fn canonicalize(source: &str) -> Result<CanonicalSource, (String, usize)> {
    let tokens = lex(source);
    let declarations = split_declarations(&tokens)?;
    Ok(CanonicalSource { declarations })
}
