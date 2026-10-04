//! The export engine.
//!
//! [`to_csv`] emits the main section only (matching the reference exporter's
//! `to_csv`); [`export_tables`] emits every section — the main table plus one
//! per repeat — linked by `_index` / `_parent_index` / `_parent_table_name`.
//! Output is `;`-separated with every field double-quoted (inner quotes
//! doubled); headers and `select_one` values resolve through [`tableflow_schema`].

use serde_json::Value;
use tableflow_core::{Section, Version};

/// One exported table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table {
    /// Section name (the export title for the main table, else the repeat name).
    pub name: String,
    /// Column headers (value fields, then any auto columns).
    pub header: Vec<String>,
    /// Data rows, each aligned with `header`.
    pub rows: Vec<Vec<String>>,
}

/// Export the main section's submissions as CSV in language `lang`.
#[must_use]
pub fn to_csv(version: &Version, submissions: &[Value], lang: Option<&str>) -> String {
    let index = tableflow_schema::lang_index(version, lang);
    let main = &version.sections[0];

    let mut header: Vec<String> = main
        .fields
        .iter()
        .map(|field| tableflow_schema::header(field, index))
        .collect();
    if main.has_children {
        header.push("_index".to_owned());
    }

    let mut lines = vec![format_line(&header)];
    for (position, submission) in submissions.iter().enumerate() {
        let mut row: Vec<String> = main
            .fields
            .iter()
            .map(|field| tableflow_schema::cell(version, field, submission.get(&field.path), index))
            .collect();
        if main.has_children {
            row.push((position + 1).to_string());
        }
        lines.push(format_line(&row));
    }
    lines.join("\n")
}

/// Export every section as a [`Table`]. `title` names the main table (and is
/// the `_parent_table_name` of its direct repeat rows).
#[must_use]
pub fn export_tables(
    version: &Version,
    submissions: &[Value],
    lang: Option<&str>,
    title: &str,
) -> Vec<Table> {
    let index = tableflow_schema::lang_index(version, lang);

    let mut tables: Vec<Table> = version
        .sections
        .iter()
        .map(|section| Table {
            name: section_name(section, title),
            header: section_header(section, index),
            rows: Vec::new(),
        })
        .collect();

    let mut counters = vec![0usize; version.sections.len()];
    for submission in submissions {
        emit_section(
            version,
            0,
            submission,
            None,
            index,
            &mut counters,
            &mut tables,
        );
    }
    tables
}

/// Serialize tables into a framed multi-section text for golden comparison:
/// a `# section: <name>` marker, then the section's CSV block.
#[must_use]
pub fn tables_to_text(tables: &[Table]) -> String {
    let mut lines = Vec::new();
    for table in tables {
        lines.push(format!("# section: {}", table.name));
        lines.push(format_line(&table.header));
        for row in &table.rows {
            lines.push(format_line(row));
        }
    }
    lines.join("\n")
}

/// Emit one row for `section` from `data`, then recurse into its child repeats.
fn emit_section(
    version: &Version,
    section_index: usize,
    data: &Value,
    parent: Option<(&str, usize)>,
    lang: Option<usize>,
    counters: &mut [usize],
    tables: &mut [Table],
) {
    counters[section_index] += 1;
    let my_index = counters[section_index];
    let section = &version.sections[section_index];

    let mut row: Vec<String> = section
        .fields
        .iter()
        .map(|field| tableflow_schema::cell(version, field, data.get(&field.path), lang))
        .collect();
    if section.has_children {
        row.push(my_index.to_string());
    }
    if let Some((parent_name, parent_index)) = parent {
        row.push(parent_name.to_owned());
        row.push(parent_index.to_string());
    }
    tables[section_index].rows.push(row);

    let my_name = tables[section_index].name.clone();
    for child_index in 0..version.sections.len() {
        let child = &version.sections[child_index];
        if child.parent != Some(section_index) {
            continue;
        }
        let Some(repeat_path) = &child.repeat_path else {
            continue;
        };
        if let Some(Value::Array(items)) = data.get(repeat_path) {
            for item in items {
                emit_section(
                    version,
                    child_index,
                    item,
                    Some((&my_name, my_index)),
                    lang,
                    counters,
                    tables,
                );
            }
        }
    }
}

/// A section's display name: the export title for the main section, else the
/// repeat name.
fn section_name(section: &Section, title: &str) -> String {
    if section.parent.is_none() {
        title.to_owned()
    } else {
        section.name.clone()
    }
}

/// A section's header: value-field headers, then auto columns.
fn section_header(section: &Section, lang: Option<usize>) -> Vec<String> {
    let mut header: Vec<String> = section
        .fields
        .iter()
        .map(|field| tableflow_schema::header(field, lang))
        .collect();
    if section.has_children {
        header.push("_index".to_owned());
    }
    if section.parent.is_some() {
        header.push("_parent_table_name".to_owned());
        header.push("_parent_index".to_owned());
    }
    header
}

/// Quote every cell with `"`, double inner quotes, and join with `;`.
fn format_line(cells: &[String]) -> String {
    let escaped: Vec<String> = cells.iter().map(|c| c.replace('"', "\"\"")).collect();
    format!("\"{}\"", escaped.join("\";\""))
}
