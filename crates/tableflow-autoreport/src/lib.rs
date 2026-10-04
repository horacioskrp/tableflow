//! Per-field summary report: counts, frequencies and percentages.
//!
//! [`report`] walks the main section's fields and, for each, counts how many
//! submissions answered it (`provided`) or not (`not_provided`) and tallies the
//! answers. Categorical fields (`text`, `select_one`, `select_multiple`,
//! `date`) also carry a `frequency` table and matching `percentage`s:
//!
//! - `text` orders values by descending count (ties keep first-seen order);
//! - `select_one` / `select_multiple` translate each value to its choice label
//!   and order by descending count; a `select_multiple` answer is split into
//!   its chosen options;
//! - `date` orders values chronologically.
//!
//! `select_*` and `date` set `show_graph`; other fields do not. Numeric
//! summaries (mean / median / mode / stdev for `integer` / `decimal`) are not
//! yet produced, so those fields report counts only.

use serde::Serialize;
use serde_json::Value;
use tableflow_core::{Field, Version};

/// A whole report: how many submissions, and one entry per reported field.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Report {
    /// Number of submissions summarized.
    pub submissions_count: u64,
    /// One entry per field, in survey order (notes excluded).
    pub fields: Vec<FieldReport>,
}

/// One field's entry: its name, display label and statistics.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FieldReport {
    /// Field name.
    pub name: String,
    /// Display label (field name when untranslated).
    pub label: String,
    /// The field's statistics.
    pub stats: Stats,
}

/// A field's statistics. `frequency` / `percentage` are present only for
/// categorical fields.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Stats {
    /// All submissions summarized (`provided + not_provided`).
    pub total_count: u64,
    /// Submissions that left the field blank (missing or null).
    pub not_provided: u64,
    /// Submissions that answered the field.
    pub provided: u64,
    /// Whether a chart is suggested for this field.
    pub show_graph: bool,
    /// Value → count, ordered per the field type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency: Option<Vec<(String, u64)>>,
    /// Value → percentage of `total_count`, aligned with `frequency`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub percentage: Option<Vec<(String, f64)>>,
}

/// Build a [`Report`] over the main section's submissions, labels in `lang`.
#[must_use]
pub fn report(version: &Version, submissions: &[Value], lang: Option<&str>) -> Report {
    let index = tableflow_schema::lang_index(version, lang);
    let main = &version.sections[0];

    let fields = main
        .fields
        .iter()
        .filter(|f| has_stats(f))
        .map(|field| FieldReport {
            name: field.name.clone(),
            label: tableflow_schema::header(field, index),
            stats: field_stats(version, field, submissions, index),
        })
        .collect();

    Report {
        submissions_count: submissions.len() as u64,
        fields,
    }
}

/// Whether a field is summarized at all (everything but notes).
fn has_stats(field: &Field) -> bool {
    field.kind != "note"
}

/// Whether a field carries a frequency table (vs. counts only).
fn is_categorical(field: &Field) -> bool {
    matches!(
        field.kind.as_str(),
        "text" | "select_one" | "select_multiple"
    ) || field.kind == "date"
}

/// Compute one field's statistics across `submissions`.
fn field_stats(
    version: &Version,
    field: &Field,
    submissions: &[Value],
    index: Option<usize>,
) -> Stats {
    // Tally answers in first-seen order.
    let mut order: Vec<String> = Vec::new();
    let mut counts: Vec<u64> = Vec::new();
    let mut provided = 0;
    let mut not_provided = 0;

    let mut tally = |value: &str| match order.iter().position(|v| v == value) {
        Some(i) => counts[i] += 1,
        None => {
            order.push(value.to_owned());
            counts.push(1);
        }
    };

    for submission in submissions {
        match submission.get(&field.path) {
            None | Some(Value::Null) => not_provided += 1,
            Some(value) => {
                provided += 1;
                let raw = scalar(value);
                if field.kind == "select_multiple" {
                    for choice in raw.split_whitespace() {
                        tally(choice);
                    }
                } else {
                    tally(&raw);
                }
            }
        }
    }

    let total_count = provided + not_provided;
    if !is_categorical(field) {
        return Stats {
            total_count,
            not_provided,
            provided,
            show_graph: false,
            frequency: None,
            percentage: None,
        };
    }

    let mut pairs: Vec<(String, u64)> = order.into_iter().zip(counts).collect();
    if field.kind == "date" {
        pairs.sort_by(|a, b| a.0.cmp(&b.0));
    } else {
        pairs.sort_by_key(|a| std::cmp::Reverse(a.1));
    }

    let is_select = field.kind == "select_one" || field.kind == "select_multiple";
    let frequency: Vec<(String, u64)> = pairs
        .into_iter()
        .map(|(value, count)| {
            let display = if is_select {
                translate(version, field, &value, index)
            } else {
                value
            };
            (display, count)
        })
        .collect();

    let percentage = frequency
        .iter()
        .map(|(value, count)| (value.clone(), percent(*count, total_count)))
        .collect();

    Stats {
        total_count,
        not_provided,
        provided,
        show_graph: field.kind != "text",
        frequency: Some(frequency),
        percentage: Some(percentage),
    }
}

/// Translate a choice value to its label in `index`, or keep it as-is.
fn translate(version: &Version, field: &Field, value: &str, index: Option<usize>) -> String {
    let list = field.list_name.as_deref();
    let label = list
        .zip(index)
        .and_then(|(list, i)| version.choice_label(list, value, i));
    label.unwrap_or(value).to_owned()
}

/// `value` as a percentage of `total`, rounded to two decimals (half to even).
fn percent(value: u64, total: u64) -> f64 {
    if total == 0 {
        return 0.0;
    }
    let raw = (value as f64) * 100.0 / (total as f64);
    (raw * 100.0).round_ties_even() / 100.0
}

/// A submission value as a plain string (empty for null/absent/compound).
fn scalar(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{percent, report};
    use tableflow_core::parse_version;

    #[test]
    fn percentages_round_to_two_decimals() {
        assert_eq!(percent(3, 4), 75.0);
        assert_eq!(percent(1, 3), 33.33);
        assert_eq!(percent(0, 0), 0.0);
    }

    #[test]
    fn text_frequency_orders_by_descending_count() {
        let version = parse_version(&json!({ "content": { "survey": [
            { "type": "text", "name": "city" },
        ] } }));
        let subs = [
            json!({ "city": "Kara" }),
            json!({ "city": "Lome" }),
            json!({ "city": "Lome" }),
            json!({}),
        ];
        let stats = &report(&version, &subs, None).fields[0].stats;
        assert_eq!(stats.provided, 3);
        assert_eq!(stats.not_provided, 1);
        assert_eq!(
            stats.frequency.as_deref(),
            Some(&[("Lome".to_owned(), 2), ("Kara".to_owned(), 1)][..])
        );
    }
}
