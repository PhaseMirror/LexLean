//! The LexLean-authored conformance-ID register (issue #111, decision D1).
//!
//! `model/src/Registry.lex.tex` is the language-1.1 semantic module that owns
//! the register: a `ConformanceRow` structure and one definition per conformance
//! ID, exactly as `UOR-Foundation/compression`'s `src/Uorc/Registry.lex.tex`
//! owns its claim register. `model/ids.toml` is the reviewable projection of
//! that graph. In the first slice the projection is additive and opt-in
//! (`cargo xtask validate-registry`): hand edits to `model/ids.toml` remain
//! accepted by the existing gates until every open register branch has
//! rebased, at which point the projection becomes the only authoring path.

use std::path::Path;

use serde_json::Value;

use crate::{IdRow, Ids, Level, ModelError};

/// The LexLean module that owns the conformance-ID register.
pub const REGISTRY_PATH: &str = "model/src/Registry.lex.tex";

/// Project the register module into the ID register it authors.
///
/// The result is the full `Ids` value: validating it with [`Ids::check`] and
/// rendering it with [`crate::codegen::render_ids`] must reproduce the
/// committed `model/ids.toml` byte for byte.
pub fn registry_ids(root: &Path) -> Result<Ids, ModelError> {
    let path = root.join(REGISTRY_PATH);
    let declarations = semantic_declarations(&path, "Registry")?;
    validate_registry_declarations(&declarations)?;

    let mut rows = Vec::new();
    for declaration in &declarations {
        if declaration.get("kind").and_then(Value::as_str) != Some("definition") {
            continue;
        }
        let name = declaration
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(|| ModelError::Inconsistent("row definition has no name".to_string()))?;
        let body = declaration.get("body").ok_or_else(|| {
            ModelError::Inconsistent(format!("row definition `{name}` has no body"))
        })?;
        rows.push(IdRow {
            id: row_field(body, name, "id")?,
            level: row_level(body, name)?,
            suite: row_field(body, name, "suite")?,
            statement: row_field(body, name, "statement")?,
        });
    }

    let ids = Ids {
        spec: "lexlean/ids/1".to_string(),
        id: rows,
    };
    ids.check()?;
    Ok(ids)
}

fn row_field(body: &Value, definition: &str, field: &str) -> Result<String, ModelError> {
    body.get("fields")
        .and_then(Value::as_array)
        .and_then(|fields| {
            fields
                .iter()
                .find(|f| f.get("field").and_then(Value::as_str) == Some(field))
        })
        .and_then(|f| f.get("value"))
        .and_then(|v| v.get("value"))
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| {
            ModelError::Inconsistent(format!(
                "row definition `{definition}` has no string field `{field}`"
            ))
        })
}

fn row_level(body: &Value, definition: &str) -> Result<Level, ModelError> {
    let raw = row_field(body, definition, "level")?;
    serde_json::from_value(Value::String(raw)).map_err(|_| {
        ModelError::Inconsistent(format!(
            "row definition `{definition}` has an unknown honesty level"
        ))
    })
}

/// Read the single `\semanticdata{…}` payload of the module at `path`
/// (the parsing shape `UOR-Foundation/compression`'s authority graph uses).
fn semantic_declarations(path: &Path, module: &str) -> Result<Vec<Value>, ModelError> {
    let text = std::fs::read_to_string(path).map_err(|e| ModelError::Io(path.to_path_buf(), e))?;
    let open = format!("\\begin{{lexlean}}{{{module}}}");
    if !text.contains(&open)
        || !text.contains("\\begin{semanticmodule}")
        || !text.contains("\\end{semanticmodule}")
        || !text.contains("\\end{lexlean}")
    {
        return Err(ModelError::Inconsistent(format!(
            "{} is not the expected LexLean semantic module `{module}`",
            path.display()
        )));
    }

    const MARKER: &str = "\\semanticdata{";
    if text.matches(MARKER).count() != 1 {
        return Err(ModelError::Inconsistent(format!(
            "{} must contain exactly one semanticdata value",
            path.display()
        )));
    }
    let start = text.find(MARKER).expect("counted one semanticdata marker") + MARKER.len();
    let tail = &text[start..];
    let end = tail.find("\\end{semanticmodule}").ok_or_else(|| {
        ModelError::Inconsistent(format!(
            "{} has no semanticmodule terminator after semanticdata",
            path.display()
        ))
    })?;
    let wrapped = tail[..end].trim();
    let json = wrapped.strip_suffix('}').ok_or_else(|| {
        ModelError::Inconsistent(format!(
            "{} semanticdata macro has no closing brace",
            path.display()
        ))
    })?;
    let value: Value = serde_json::from_str(json).map_err(|e| {
        ModelError::Inconsistent(format!(
            "{} contains invalid semantic JSON: {e}",
            path.display()
        ))
    })?;
    if value.get("spec").and_then(Value::as_str) != Some("lexlean/semantic-module/1") {
        return Err(ModelError::Inconsistent(format!(
            "{} must use lexlean/semantic-module/1",
            path.display()
        )));
    }
    value
        .get("declarations")
        .and_then(Value::as_array)
        .cloned()
        .ok_or_else(|| {
            ModelError::Inconsistent(format!(
                "{} has no semantic declaration array",
                path.display()
            ))
        })
}

/// The register module contains exactly one `ConformanceRow` structure over
/// the four register fields, and one record definition per row.
fn validate_registry_declarations(declarations: &[Value]) -> Result<(), ModelError> {
    let mut structure_count = 0usize;
    let mut definition_names = std::collections::BTreeSet::new();

    for declaration in declarations {
        let kind = declaration
            .get("kind")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                ModelError::Inconsistent("registry declaration has no kind".to_string())
            })?;
        let name = declaration
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                ModelError::Inconsistent("registry declaration has no name".to_string())
            })?;
        match kind {
            "structure" if name == "ConformanceRow" => {
                structure_count += 1;
            }
            "definition" if definition_result_name(declaration) == Some("ConformanceRow") => {
                if !definition_names.insert(name.to_string()) {
                    return Err(ModelError::Inconsistent(format!(
                        "Registry.lex.tex repeats row definition `{name}`"
                    )));
                }
            }
            _ => {
                return Err(ModelError::Inconsistent(format!(
                    "Registry.lex.tex contains unprojected declaration `{kind} {name}`"
                )));
            }
        }
    }

    if structure_count != 1 {
        return Err(ModelError::Inconsistent(format!(
            "Registry.lex.tex must contain exactly one ConformanceRow structure, observed {structure_count}"
        )));
    }
    if definition_names.is_empty() {
        return Err(ModelError::Inconsistent(
            "Registry.lex.tex contains no ConformanceRow definitions".to_string(),
        ));
    }
    validate_string_structure(
        declarations,
        "ConformanceRow",
        &["id", "level", "suite", "statement"],
    )?;
    Ok(())
}

fn definition_result_name(declaration: &Value) -> Option<&str> {
    declaration
        .get("result")
        .and_then(|result| result.get("member"))
        .and_then(|member| member.get("name"))
        .and_then(Value::as_str)
}

fn validate_string_structure(
    declarations: &[Value],
    name: &str,
    expected_fields: &[&str],
) -> Result<(), ModelError> {
    let declaration = declarations
        .iter()
        .find(|value| {
            value.get("kind").and_then(Value::as_str) == Some("structure")
                && value.get("name").and_then(Value::as_str) == Some(name)
        })
        .ok_or_else(|| {
            ModelError::Inconsistent(format!("Registry.lex.tex is missing structure `{name}`"))
        })?;

    for key in ["parameters", "type_parameters"] {
        if declaration
            .get(key)
            .and_then(Value::as_array)
            .is_none_or(|values| !values.is_empty())
        {
            return Err(ModelError::Inconsistent(format!(
                "`{name}` must have no {key}"
            )));
        }
    }

    let fields = declaration
        .get("fields")
        .and_then(Value::as_array)
        .ok_or_else(|| ModelError::Inconsistent(format!("`{name}` has no fields")))?;
    let mut observed = Vec::new();
    for field in fields {
        let field_name = field.get("name").and_then(Value::as_str).ok_or_else(|| {
            ModelError::Inconsistent(format!("`{name}` has a field without a name"))
        })?;
        let field_kind = field
            .get("type")
            .and_then(|t| t.get("kind"))
            .and_then(Value::as_str);
        if field_kind != Some("string") {
            return Err(ModelError::Inconsistent(format!(
                "`{name}` field `{field_name}` must be a string"
            )));
        }
        observed.push(field_name);
    }
    if observed != expected_fields {
        return Err(ModelError::Inconsistent(format!(
            "`{name}` must have exactly the fields {expected_fields:?}, observed {observed:?}"
        )));
    }
    Ok(())
}
