//! Core data model shared across the tableflow export pipeline.
//!
//! A [`Version`] holds its `translations`, its `choices` and a list of
//! [`Section`]s. Section 0 is the main table; each `repeat` adds another section
//! linked to its parent. Non-repeat groups do not add a section — their fields
//! flatten into the enclosing section, keeping a short name but a full,
//! `/`-joined submission `path`.

use std::collections::HashSet;

use serde_json::Value;

/// A survey field — one logical column of the export.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    /// Field name: the header in "names" mode (short, not group-prefixed).
    pub name: String,
    /// Full submission key: the field name prefixed by enclosing group / repeat
    /// names, joined with `/` (e.g. `household/location`).
    pub path: String,
    /// XLSForm `type` token (e.g. `text`, `integer`, `select_one`).
    pub kind: String,
    /// Labels indexed by [`Version::translations`]; empty when untranslated.
    pub labels: Vec<String>,
    /// Choice list name, for `select_one` / `select_multiple`.
    pub list_name: Option<String>,
}

/// A table of the export: the main section (index 0) or a repeat.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    /// Repeat name; the main section's name is empty (resolved to the export
    /// title at export time).
    pub name: String,
    /// The repeat's full submission path (the key holding the list of rows);
    /// `None` for the main section.
    pub repeat_path: Option<String>,
    /// Index of the parent section, or `None` for the main section.
    pub parent: Option<usize>,
    /// Whether this section contains a repeat (→ gets an `_index` column).
    pub has_children: bool,
    /// Value fields of this section, in order.
    pub fields: Vec<Field>,
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

/// One form version: translations, sections (main + repeats) and choice lists.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Version {
    /// Declared translations, in order; empty for an untranslated form.
    pub translations: Vec<String>,
    /// Sections; index 0 is the main table.
    pub sections: Vec<Section>,
    /// Choice lists from the `choices` sheet.
    pub choices: Vec<ChoiceList>,
}

impl Default for Version {
    fn default() -> Self {
        Version {
            translations: Vec::new(),
            sections: vec![Section {
                name: String::new(),
                repeat_path: None,
                parent: None,
                has_children: false,
                fields: Vec::new(),
            }],
            choices: Vec::new(),
        }
    }
}

impl Version {
    /// The options of choice list `list` (empty if the list is unknown).
    #[must_use]
    pub fn choices_of(&self, list: &str) -> &[Choice] {
        self.choices
            .iter()
            .find(|c| c.name == list)
            .map_or(&[], |c| c.items.as_slice())
    }

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

/// Merge several form versions into one export canvas (main section only).
///
/// The column set is built from the **last** version in `versions` first, then
/// each earlier version contributes only its fields whose names are not yet
/// present — matching the reference exporter's field ordering across versions.
/// Submissions carry their own keys, so values are read by field `path` at
/// export time regardless of which version produced a row.
///
/// Translations and choices are taken from the last version; cross-version
/// repeat merging is out of scope and only the main section is merged.
#[must_use]
pub fn merge_versions(versions: &[Version]) -> Version {
    let Some(last) = versions.last() else {
        return Version::default();
    };

    let mut fields: Vec<Field> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for version in versions.iter().rev() {
        let main = version.sections.first();
        for field in main.map_or(&[][..], |s| s.fields.as_slice()) {
            if seen.insert(field.name.clone()) {
                fields.push(field.clone());
            }
        }
    }

    Version {
        translations: last.translations.clone(),
        sections: vec![Section {
            name: String::new(),
            repeat_path: None,
            parent: None,
            has_children: false,
            fields,
        }],
        choices: last.choices.clone(),
    }
}

/// Parse a version schema into a [`Version`] (translations, sections, choices).
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

    let mut version = Version {
        translations,
        ..Version::default()
    };
    parse_sections(get("survey").unwrap_or(&Vec::new()), &mut version.sections);
    version.choices = parse_choices(get("choices").unwrap_or(&Vec::new()));
    version
}

/// Walk the survey rows, assigning fields to the main section and to a new
/// section per repeat; non-repeat groups only extend the submission path.
fn parse_sections(rows: &[Value], sections: &mut Vec<Section>) {
    let mut prefix: Vec<String> = Vec::new();
    let mut stack: Vec<usize> = vec![0];

    for row in rows {
        let kind = row.get("type").and_then(Value::as_str).unwrap_or_default();
        let name = row.get("name").and_then(Value::as_str).unwrap_or_default();

        match kind.replace(' ', "_").as_str() {
            "begin_group" => {
                prefix.push(name.to_owned());
                continue;
            }
            "end_group" => {
                prefix.pop();
                continue;
            }
            "begin_repeat" => {
                prefix.push(name.to_owned());
                let current = *stack.last().expect("non-empty stack");
                let new_index = sections.len();
                sections.push(Section {
                    name: name.to_owned(),
                    repeat_path: Some(prefix.join("/")),
                    parent: Some(current),
                    has_children: false,
                    fields: Vec::new(),
                });
                sections[current].has_children = true;
                stack.push(new_index);
                continue;
            }
            "end_repeat" => {
                prefix.pop();
                stack.pop();
                continue;
            }
            _ => {}
        }

        if name.is_empty() {
            continue;
        }
        let path = if prefix.is_empty() {
            name.to_owned()
        } else {
            format!("{}/{name}", prefix.join("/"))
        };
        let current = *stack.last().expect("non-empty stack");
        sections[current].fields.push(Field {
            name: name.to_owned(),
            path,
            kind: kind.to_owned(),
            labels: labels_of(row.get("label")),
            list_name: list_name_of(row, kind),
        });
    }
}

/// Parse `content.choices` into choice lists, grouped by `list_name`.
fn parse_choices(rows: &[Value]) -> Vec<ChoiceList> {
    let mut choices: Vec<ChoiceList> = Vec::new();
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
    choices
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

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{merge_versions, parse_version};

    #[test]
    fn merge_leads_with_last_version_then_older_new_fields() {
        let v1 = parse_version(&json!({ "content": { "survey": [
            { "type": "text", "name": "a" },
            { "type": "text", "name": "color" },
        ] } }));
        let v2 = parse_version(&json!({ "content": { "survey": [
            { "type": "text", "name": "a2" },
            { "type": "text", "name": "b" },
            { "type": "text", "name": "color" },
        ] } }));

        let names = |versions: &[super::Version]| {
            merge_versions(versions).sections[0]
                .fields
                .iter()
                .map(|f| f.name.clone())
                .collect::<Vec<_>>()
        };

        // Last listed version first, then each older version's new fields.
        assert_eq!(names(&[v1.clone(), v2.clone()]), ["a2", "b", "color", "a"]);
        assert_eq!(names(&[v2, v1]), ["a", "color", "a2", "b"]);
    }

    #[test]
    fn merge_of_empty_selection_is_default() {
        assert_eq!(merge_versions(&[]), super::Version::default());
    }
}
