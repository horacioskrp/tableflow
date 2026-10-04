//! The export engine.
//!
//! Phase 1: CSV of a flat, single-version form. Output matches the reference
//! exporter's `to_csv` — `;`-separated, **every** field double-quoted, inner
//! quotes doubled — and the default (no language) header uses field names.

use serde_json::Value;
use tableflow_core::Version;

/// Export a flat, single-version form's submissions as CSV.
///
/// Lines are joined with `\n` and there is no trailing newline, matching the
/// reference exporter's line generator.
#[must_use]
pub fn to_csv(version: &Version, submissions: &[Value]) -> String {
    let header: Vec<String> = version.fields.iter().map(|f| f.name.clone()).collect();
    let mut lines = vec![format_line(&header)];
    for submission in submissions {
        let row: Vec<String> = version
            .fields
            .iter()
            .map(|field| cell(submission.get(&field.name)))
            .collect();
        lines.push(format_line(&row));
    }
    lines.join("\n")
}

/// Format a submission value as a CSV cell (Phase 1 scalar subset): strings
/// verbatim, numbers stringified, missing/null as empty.
fn cell(value: Option<&Value>) -> String {
    match value {
        None | Some(Value::Null) => String::new(),
        Some(Value::String(s)) => s.clone(),
        Some(Value::Number(n)) => n.to_string(),
        Some(Value::Bool(b)) => b.to_string(),
        Some(other) => other.to_string(),
    }
}

/// Quote every cell with `"`, double inner quotes, and join with `;`.
fn format_line(cells: &[String]) -> String {
    let escaped: Vec<String> = cells.iter().map(|c| c.replace('"', "\"\"")).collect();
    format!("\"{}\"", escaped.join("\";\""))
}
