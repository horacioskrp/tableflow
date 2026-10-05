//! Field type system: resolve the requested language and turn a submission
//! value into export column(s).
//!
//! Most fields contribute one column. A `select_one` renders its value as a
//! choice label (in a language) or the raw name. A `select_multiple` expands
//! per [`MultipleSelect`]: a joined summary column and/or one boolean column per
//! choice. Other common types pass their value through as a string.

use serde_json::Value;
use tableflow_core::{Choice, Field, Version};

/// How `select_multiple` fields expand into columns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MultipleSelect {
    /// A joined summary column plus one boolean column per choice.
    Both,
    /// Only the joined summary column.
    Summary,
    /// Only the per-choice boolean columns.
    Details,
}

impl MultipleSelect {
    /// Parse `both` / `summary` / `details` (anything else → `Both`).
    #[must_use]
    pub fn parse(value: &str) -> Self {
        match value {
            "summary" => Self::Summary,
            "details" => Self::Details,
            _ => Self::Both,
        }
    }

    fn has_summary(self) -> bool {
        matches!(self, Self::Both | Self::Summary)
    }

    fn has_details(self) -> bool {
        matches!(self, Self::Both | Self::Details)
    }
}

/// Resolve the translation index for a requested language (`None` = names mode:
/// an unspecified/untranslated export or a language this version does not
/// declare — names and raw values).
#[must_use]
pub fn lang_index(version: &Version, lang: Option<&str>) -> Option<usize> {
    let lang = lang?;
    version.translations.iter().position(|t| t == lang)
}

/// The header for a field: its label in the resolved language, else its name.
#[must_use]
pub fn header(field: &Field, lang: Option<usize>) -> String {
    match lang.and_then(|i| field.labels.get(i)) {
        Some(label) if !label.is_empty() => label.clone(),
        _ => field.name.clone(),
    }
}

/// The header for a field, hierarchical when `hierarchy` is set: each enclosing
/// group's label (or name) then the field's own, joined by `group_sep`.
#[must_use]
pub fn header_path(field: &Field, lang: Option<usize>, group_sep: &str, hierarchy: bool) -> String {
    if !hierarchy {
        return header(field, lang);
    }
    let mut parts: Vec<String> = field
        .group_path
        .iter()
        .map(|group| match lang.and_then(|i| group.labels.get(i)) {
            Some(label) if !label.is_empty() => label.clone(),
            _ => group.name.clone(),
        })
        .collect();
    parts.push(header(field, lang));
    parts.join(group_sep)
}

/// The column headers a field contributes.
#[must_use]
pub fn columns(
    version: &Version,
    field: &Field,
    lang: Option<usize>,
    mode: MultipleSelect,
    group_sep: &str,
    hierarchy: bool,
) -> Vec<String> {
    let base = header_path(field, lang, group_sep, hierarchy);
    if field.kind != "select_multiple" {
        return vec![base];
    }
    let mut cols = Vec::new();
    if mode.has_summary() {
        cols.push(base.clone());
    }
    if mode.has_details() {
        for choice in choices(version, field) {
            cols.push(format!("{base}{group_sep}{}", choice_header(choice, lang)));
        }
        if field.or_other {
            cols.push(format!("{base}{group_sep}other"));
        }
    }
    cols
}

/// The cell value(s) a field contributes for a submission value.
#[must_use]
pub fn values(
    version: &Version,
    field: &Field,
    value: Option<&Value>,
    lang: Option<usize>,
    mode: MultipleSelect,
) -> Vec<String> {
    if field.kind != "select_multiple" {
        return vec![cell(version, field, value, lang)];
    }
    let raw = scalar(value);
    let selected: Vec<&str> = raw.split_whitespace().collect();
    let mut out = Vec::new();
    if mode.has_summary() {
        let joined = selected
            .iter()
            .map(|name| choice_value(version, field, name, lang))
            .collect::<Vec<_>>()
            .join(" ");
        out.push(joined);
    }
    if mode.has_details() {
        for choice in choices(version, field) {
            let present = selected.contains(&choice.name.as_str());
            out.push(if present { "1" } else { "0" }.to_owned());
        }
        if field.or_other {
            let present = selected.contains(&"other");
            out.push(if present { "1" } else { "0" }.to_owned());
        }
    }
    out
}

/// Format a single (non-`select_multiple`) field value as a cell.
#[must_use]
pub fn cell(
    version: &Version,
    field: &Field,
    value: Option<&Value>,
    lang: Option<usize>,
) -> String {
    let raw = scalar(value);
    if field.kind == "select_one" && !raw.is_empty() {
        return choice_value(version, field, &raw, lang);
    }
    raw
}

/// Render one choice name as its label (in a language) or the raw name.
fn choice_value(version: &Version, field: &Field, name: &str, lang: Option<usize>) -> String {
    if let (Some(index), Some(list)) = (lang, field.list_name.as_deref()) {
        if let Some(label) = version.choice_label(list, name, index) {
            return label.to_owned();
        }
    }
    name.to_owned()
}

/// A choice's header: its label in the resolved language, else its name.
fn choice_header(choice: &Choice, lang: Option<usize>) -> String {
    match lang.and_then(|i| choice.labels.get(i)) {
        Some(label) if !label.is_empty() => label.clone(),
        _ => choice.name.clone(),
    }
}

/// The options of a select field's list.
fn choices<'a>(version: &'a Version, field: &Field) -> &'a [Choice] {
    field
        .list_name
        .as_deref()
        .map_or(&[], |list| version.choices_of(list))
}

/// Stringify a scalar submission value (missing/null → empty).
fn scalar(value: Option<&Value>) -> String {
    match value {
        None | Some(Value::Null) => String::new(),
        Some(Value::String(s)) => s.clone(),
        Some(Value::Number(n)) => n.to_string(),
        Some(Value::Bool(b)) => b.to_string(),
        Some(other) => other.to_string(),
    }
}
