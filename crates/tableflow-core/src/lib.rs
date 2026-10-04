//! Core data model shared across the tableflow export pipeline.
//!
//! A [`Version`] is a flat, ordered list of [`Field`]s plus its declared
//! `translations` and `choices`. Labels are stored as a list indexed by the
//! translation order; a field with a single plain label has one entry. Sections
//! (groups/repeats) and the multi-version canvas land in later phases.

use serde_json::Value;

/// A survey field — one logical column of the export.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    /// Field name: the header in "names" mode (short, not group-prefixed).
    pub name: String,
    /// Full submission key: the field name prefixed by any enclosing group /
    /// repeat names, joined with `/` (e.g. `household/location`).
    pub path: String,
    /// XLSForm `type` token (e.g. `text`, `integer`, `select_one`).
    pub kind: String,
    /// Labels indexed by [`Version::translations`]; empty when untranslated.
    pub labels: Vec<String>,
    /// Choice list name, for `select_one` / `select_multiple`.
    pub list_name: Option<String>,
}

/// One option of a choice list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Choice {
    /// Stored value (the submission uses this).
    pub name: String,
    /// Labels indexed by [`Version::translations`].
    pub labels: Vec<String>,
}

/// A named list of choices.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChoiceList {
    /// List name referenced by `select_*` fields.
    pub name: String,
    /// Options in declaration order.
    pub items: Vec<Choice>,
}

/// One form version: translations, fields (flat) and choice lists.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Version {
    /// Declared translations, in order; empty for an untranslated form.
    pub translations: Vec<String>,
    /// Fields in survey order.
    pub fields: Vec<Field>,
    /// Choice lists from the `choices` sheet.
    pub choices: Vec<ChoiceList>,
}

impl Version {
    /// The label of choice `name` in list `list` for translation `index`.
    #[must_use]
    pub fn choice_label(&self, list: &str, name: &str, index: usize) -> Option<&str> {
        self.choices
            .iter()
            .find(|c| c.name == list)?
            .items
            .iter()
            .find(|item| item.name == name)?
            .labels
            .get(index)
            .map(String::as_str)
    }
}

/// Parse a version schema into a [`Version`].
///
/// Reads `content.translations`, `content.survey` (rows with a `name`; labels
/// as a string or a per-translation list; `select_from_list_name` or a
/// `select_one <list>` type) and `content.choices`.
#[must_use]
pub fn parse_version(schema: &Value) -> Version {
    let content = schema.get("content");
    let get = |key| content.and_then(|c| c.get(key)).and_then(Value::as_array);

    let translations = get("translations")
        .map(|arr| {
            arr.iter()
                .map(|t| t.as_str().unwrap_or_default().to_owned())
                .collect()
        })
        .unwrap_or_default();

    let mut fields = Vec::new();
    let mut prefix: Vec<String> = Vec::new();
    if let Some(rows) = get("survey") {
        for row in rows {
            let kind = row.get("type").and_then(Value::as_str).unwrap_or_default();
            // Structural markers nest the submission path; they are not fields.
            match kind.replace(' ', "_").as_str() {
                "begin_group" | "begin_repeat" => {
                    if let Some(name) = row.get("name").and_then(Value::as_str) {
                        prefix.push(name.to_owned());
                    }
                    continue;
                }
                "end_group" | "end_repeat" => {
                    prefix.pop();
                    continue;
                }
                _ => {}
            }

            let name = row.get("name").and_then(Value::as_str).unwrap_or_default();
            if name.is_empty() {
                continue;
            }
            let path = if prefix.is_empty() {
                name.to_owned()
            } else {
                format!("{}/{name}", prefix.join("/"))
            };
            fields.push(Field {
                name: name.to_owned(),
                path,
                kind: kind.to_owned(),
                labels: labels_of(row.get("label")),
                list_name: list_name_of(row, kind),
            });
        }
    }

    let mut choices: Vec<ChoiceList> = Vec::new();
    if let Some(rows) = get("choices") {
        for row in rows {
            let Some(list) = row.get("list_name").and_then(Value::as_str) else {
                continue;
            };
            let choice = Choice {
                name: row
                    .get("name")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_owned(),
                labels: labels_of(row.get("label")),
            };
            match choices.iter_mut().find(|c| c.name == list) {
                Some(existing) => existing.items.push(choice),
                None => choices.push(ChoiceList {
                    name: list.to_owned(),
                    items: vec![choice],
                }),
            }
        }
    }

    Version {
        translations,
        fields,
        choices,
    }
}

/// Normalize a `label` cell (absent / string / per-translation list) into a
/// list of labels.
fn labels_of(value: Option<&Value>) -> Vec<String> {
    match value {
        Some(Value::String(s)) => vec![s.clone()],
        Some(Value::Array(items)) => items
            .iter()
            .map(|v| v.as_str().unwrap_or_default().to_owned())
            .collect(),
        _ => Vec::new(),
    }
}

/// The choice-list name for a select field: `select_from_list_name`, or the
/// second token of a `select_one <list>` / `select_multiple <list>` type.
fn list_name_of(row: &Value, kind: &str) -> Option<String> {
    if let Some(list) = row.get("select_from_list_name").and_then(Value::as_str) {
        return Some(list.to_owned());
    }
    let mut parts = kind.split_whitespace();
    match parts.next() {
        Some("select_one" | "select_multiple") => parts.next().map(str::to_owned),
        _ => None,
    }
}
