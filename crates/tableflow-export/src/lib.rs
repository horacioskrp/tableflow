//! The export engine.
//!
//! Phase 1–2: CSV of a flat, single-version form, in a chosen language. Output
//! matches the reference exporter's `to_csv` — `;`-separated, **every** field
//! double-quoted (inner quotes doubled). Headers and `select_one` values are
//! resolved through [`tableflow_schema`].

use serde_json::Value;
use tableflow_core::Version;

/// Export a flat, single-version form's submissions as CSV in language `lang`
/// (`None` = names mode: field-name headers and raw choice values).
///
/// Lines are joined with `\n` with no trailing newline, matching the reference
/// exporter's line generator.
#[must_use]
pub fn to_csv(version: &Version, submissions: &[Value], lang: Option<&str>) -> String {
    let index = tableflow_schema::lang_index(version, lang);

    let header: Vec<String> = version
        .fields
        .iter()
        .map(|field| tableflow_schema::header(field, index))
        .collect();
    let mut lines = vec![format_line(&header)];

    for submission in submissions {
        let row: Vec<String> = version
            .fields
            .iter()
            .map(|field| tableflow_schema::cell(version, field, submission.get(&field.path), index))
            .collect();
        lines.push(format_line(&row));
    }
    lines.join("\n")
}

/// Quote every cell with `"`, double inner quotes, and join with `;`.
fn format_line(cells: &[String]) -> String {
    let escaped: Vec<String> = cells.iter().map(|c| c.replace('"', "\"\"")).collect();
    format!("\"{}\"", escaped.join("\";\""))
}
