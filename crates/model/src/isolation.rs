//! The oracle-isolation audit (SPEC.md §17.19).
//!
//! The differential oracle is validation evidence. Nothing it is, produces,
//! or is pinned by may be an input to an identity LexLean computes, to a
//! verified artifact, to an attestation, or to an optimality certificate, and
//! removing it must change none of them. Those are all computed from the
//! shipped crate, the language data, the schemas, the compiler project, and
//! the committed examples' records, so the audit reads exactly those and
//! fails on any mention of the oracle: its name, its Lake target, its pin's
//! schema tags and path, the commit it is pinned at, or the digest of its
//! files. It is shared by `cargo xtask validate-model` and by the conformance
//! case that plants each of those in a copy of the tree and observes the
//! audit fail.

use std::path::{Path, PathBuf};

/// What the audit reads, relative to the repository root: the shipped crate
/// (its source is what computes every identity), the language data and the
/// schemas that are digested into the compiler-semantics ID, the compiler
/// project and the committed examples (whose records hold the semantic IDs,
/// build manifests, attestations, and certificates), and the two model files
/// that cite authorities and facts.
pub const SCANNED: [&str; 7] = [
    "crates/lexlean",
    "language",
    "schemas",
    "compiler",
    "examples",
    "model/authorities.toml",
    "model/ledger.toml",
];

/// The pin, relative to the repository root.
pub const PIN: &str = "oracle/lean4-prod.toml";

/// The words that name the oracle, wherever they appear.
pub const NAMES: [&str; 5] = [
    "lean4-prod",
    "lean4_prod",
    "prod-export",
    "oracle/lean4-prod",
    "lexlean/oracle-",
];

/// What a scan covered.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Scanned {
    /// How many files were read.
    pub files: usize,
    /// How many files each scanned root held.
    pub per_root: Vec<(String, usize)>,
}

/// The value of `key = "..."` in the pin's text.
fn pinned(text: &str, key: &str) -> Option<String> {
    text.lines().find_map(|line| {
        line.strip_prefix(key)
            .and_then(|rest| rest.trim_start().strip_prefix('='))
            .map(|rest| rest.trim().trim_matches('"').to_owned())
    })
}

/// Every string that identifies the oracle: its names, and the commit and
/// digest its pin states.
///
/// # Errors
///
/// Returns the reason the pin cannot be read.
pub fn identifiers(root: &Path) -> Result<Vec<String>, String> {
    let text =
        std::fs::read_to_string(root.join(PIN)).map_err(|error| format!("{PIN}: {error}"))?;
    let mut out: Vec<String> = NAMES.iter().map(|name| (*name).to_owned()).collect();
    for key in ["commit", "tree_sha256", "git_tree"] {
        let value = pinned(&text, key).ok_or_else(|| format!("{PIN}: no `{key}`"))?;
        if value.len() < 40 {
            return Err(format!("{PIN}: `{key}` is not a digest"));
        }
        out.push(value);
    }
    Ok(out)
}

/// The first identifier of the oracle that `text` mentions.
#[must_use]
pub fn mention<'a>(text: &str, identifiers: &'a [String]) -> Option<&'a str> {
    identifiers
        .iter()
        .find(|identifier| text.contains(identifier.as_str()))
        .map(String::as_str)
}

fn files_under(path: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    if path.is_file() {
        out.push(path.to_path_buf());
        return Ok(());
    }
    let entries =
        std::fs::read_dir(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let mut paths: Vec<PathBuf> = entries.flatten().map(|entry| entry.path()).collect();
    paths.sort();
    for entry in paths {
        let name = entry.file_name().map(|n| n.to_string_lossy().into_owned());
        if matches!(
            name.as_deref(),
            Some(".lexlean" | ".git" | "target" | ".lake")
        ) {
            continue;
        }
        files_under(&entry, out)?;
    }
    Ok(())
}

/// Read every file under [`SCANNED`] and fail on a mention of the oracle.
///
/// # Errors
///
/// Returns the first file that names the oracle, or the reason a scanned root
/// holds no file: an audit that read nothing has not passed.
pub fn scan(root: &Path) -> Result<Scanned, String> {
    let identifiers = identifiers(root)?;
    let mut scanned = Scanned::default();
    for relative in SCANNED {
        let mut files = Vec::new();
        if root.join(relative).exists() {
            files_under(&root.join(relative), &mut files)?;
        }
        if files.is_empty() {
            return Err(format!(
                "`{relative}` holds no file: an audit of nothing has not passed"
            ));
        }
        let mut count = 0;
        for file in files {
            // A file that is not UTF-8 text cannot name the oracle in words.
            let Ok(text) = std::fs::read_to_string(&file) else {
                continue;
            };
            count += 1;
            if let Some(found) = mention(&text, &identifiers) {
                return Err(format!(
                    "R2: `{}` mentions the differential oracle (`{found}`); the oracle is validation evidence and no input to a semantic ID, build ID, verified artifact, attestation, or optimality certificate (§17.19)",
                    file.strip_prefix(root).unwrap_or(&file).display()
                ));
            }
        }
        scanned.files += count;
        scanned.per_root.push((relative.to_owned(), count));
    }
    Ok(scanned)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_mention_is_found_and_silence_is_not() {
        let identifiers = vec!["lean4-prod".to_owned(), "a".repeat(40)];
        assert_eq!(
            mention("uses lean4-prod here", &identifiers),
            Some("lean4-prod")
        );
        assert_eq!(mention("nothing at all", &identifiers), None);
        assert!(mention(&format!("sha {}", "a".repeat(40)), &identifiers).is_some());
    }

    #[test]
    fn the_pin_is_read_by_key() {
        let text = "spec = \"x\"\ncommit = \"abc\"\ncommit_other = \"zzz\"\n";
        assert_eq!(pinned(text, "commit").as_deref(), Some("abc"));
    }
}
