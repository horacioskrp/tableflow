//! Field type system: resolve the requested language and turn a submission
//! value into an export cell.
//!
//! Phase 2 covers header labels per language and `select_one` value→label
//! mapping; other common types (text/integer/decimal/date/time/dateTime/geo)
//! pass their value through as a string. `select_multiple` expansion and
//! geo/media specifics arrive in later phases.

use serde_json::Value;
use tableflow_core::{Field, Version};

/// Resolve the translation index for a requested language.
///
/// `None` means "names mode": an unspecified/untranslated export, or a language
/// not declared by this version — headers use field names and choice values
/// stay raw.
#[must_use]
pub fn lang_index(version: &Version, lang: Option<&str>) -> Option<usize> {
    let lang = lang?;
    version.translations.iter().position(|t| t == lang)
}

/// The column header for a field: its label in the resolved language, else its
/// name.
#[must_use]
pub fn header(field: &Field, lang: Option<usize>) -> String {
    match lang.and_then(|i| field.labels.get(i)) {
        Some(label) if !label.is_empty() => label.clone(),
        _ => field.name.clone(),
    }
}

/// Format a submission value as a CSV cell for a field.
///
/// For `select_one` with a resolved language, the stored choice name is
/// rendered as its label; otherwise the raw value is used.
#[must_use]
pub fn cell(
    version: &Version,
    field: &Field,
    value: Option<&Value>,
    lang: Option<usize>,
) -> String {
    let raw = scalar(value);
    if field.kind == "select_one" && !raw.is_empty() {
        if let (Some(index), Some(list)) = (lang, field.list_name.as_deref()) {
            if let Some(label) = version.choice_label(list, &raw, index) {
                return label.to_owned();
            }
        }
    }
    raw
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
