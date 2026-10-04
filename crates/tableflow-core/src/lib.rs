//! Core data model shared across the tableflow export pipeline.
//!
//! Phase 1 covers a flat, single-version form: a [`Version`] is an ordered list
//! of [`Field`]s. Sections (groups/repeats), the multi-version field canvas and
//! the full field type system land in later phases.

use serde_json::Value;

/// A survey field — one column of the export.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    /// Field name: the submission key and, by default (no language), the
    /// export header.
    pub name: String,
    /// XLSForm `type` token (e.g. `text`, `integer`, `decimal`).
    pub kind: String,
    /// Default-language label, if the survey row carries one.
    pub label: Option<String>,
}

/// One form version: its fields in document order (flat, Phase 1).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Version {
    /// Fields in survey order.
    pub fields: Vec<Field>,
}

/// Parse a version schema (`{ "content": { "survey": [ … ] } }`) into a
/// [`Version`]. Rows without a `name` are skipped.
#[must_use]
pub fn parse_version(schema: &Value) -> Version {
    let rows = schema
        .get("content")
        .and_then(|content| content.get("survey"))
        .and_then(Value::as_array);

    let mut fields = Vec::new();
    if let Some(rows) = rows {
        for row in rows {
            let name = row.get("name").and_then(Value::as_str).unwrap_or_default();
            if name.is_empty() {
                continue;
            }
            fields.push(Field {
                name: name.to_owned(),
                kind: row
                    .get("type")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_owned(),
                label: row.get("label").and_then(Value::as_str).map(str::to_owned),
            });
        }
    }
    Version { fields }
}
