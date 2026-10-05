//! The export engine.
//!
//! [`to_csv`] emits the main section only (matching the reference exporter's
//! `to_csv`); [`export_tables`] emits every section — the main table plus one
//! per repeat — linked by `_index` / `_parent_index` / `_parent_table_name`.
//! Each field contributes one or more columns via [`tableflow_schema`]
//! (`select_multiple` expands per [`MultipleSelect`]). Output is `;`-separated
//! with every field double-quoted (inner quotes doubled).

use serde_json::Value;
use tableflow_core::{Section, Version};
pub use tableflow_schema::MultipleSelect;

/// One exported table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table {
    /// Section name (the export title for the main table, else the repeat name).
    pub name: String,
    /// Column headers (expanded value columns, then any auto columns).
    pub header: Vec<String>,
    /// Data rows, each aligned with `header`.
    pub rows: Vec<Vec<String>>,
}

/// Export options: language, `select_multiple` mode, header layout and extras.
#[derive(Clone, Copy)]
pub struct Layout<'a> {
    /// Language for labels (`None` = names mode).
    pub lang: Option<&'a str>,
    /// How `select_multiple` fields expand.
    pub multiple_select: MultipleSelect,
    /// Separator between hierarchy levels and expansion columns.
    pub group_sep: &'a str,
    /// Prefix each header with its enclosing groups' labels.
    pub hierarchy_in_labels: bool,
    /// Extra submission keys appended as trailing main-section columns.
    pub copy_fields: &'a [&'a str],
    /// Tag columns (e.g. `hxl`) emitted as header rows after the labels.
    pub tag_cols: &'a [&'a str],
    /// When set, keep only the fields whose names are listed (survey order).
    pub filter_fields: Option<&'a [&'a str]>,
    /// Append a `<name>_URL` column after each media field (image/audio/video/
    /// file), filled from the submission's `_attachments`.
    pub include_media_url: bool,
    /// Add an `_index` column to every section, even without repeats.
    pub force_index: bool,
}

impl Default for Layout<'_> {
    fn default() -> Self {
        Layout {
            lang: None,
            multiple_select: MultipleSelect::Both,
            group_sep: "/",
            hierarchy_in_labels: false,
            copy_fields: &[],
            tag_cols: &[],
            filter_fields: None,
            include_media_url: false,
            force_index: false,
        }
    }
}

/// Media field types that carry an attachment.
const MEDIA_TYPES: [&str; 6] = [
    "image",
    "audio",
    "video",
    "file",
    "background-audio",
    "audit",
];

/// Export the main section's submissions as CSV per `layout`.
#[must_use]
pub fn to_csv(version: &Version, submissions: &[Value], layout: &Layout) -> String {
    let index = tableflow_schema::lang_index(version, layout.lang);
    let main = &version.sections[0];

    let mut header = field_columns(version, main, index, layout);
    header.extend(layout.copy_fields.iter().map(|&name| name.to_owned()));
    if main.has_children || layout.force_index {
        header.push("_index".to_owned());
    }

    let mut lines = vec![format_line(&header)];
    for tag_row in tag_rows(version, main, index, layout) {
        lines.push(format_line(&tag_row));
    }
    for (position, submission) in submissions.iter().enumerate() {
        let mut row = field_values(version, main, submission, index, layout);
        row.extend(copy_values(layout.copy_fields, submission, index));
        if main.has_children || layout.force_index {
            row.push((position + 1).to_string());
        }
        lines.push(format_line(&row));
    }
    lines.join("\n")
}

/// The tag header rows (one per `tag_col` that any field carries) for a
/// section's value columns. Each field's tag sits at its first value column,
/// with blanks for its expansion columns.
fn tag_rows(
    version: &Version,
    section: &Section,
    index: Option<usize>,
    layout: &Layout,
) -> Vec<Vec<String>> {
    let mut rows = Vec::new();
    for &col in layout.tag_cols {
        let mut row = Vec::new();
        let mut any = false;
        for field in included_fields(section, layout) {
            let value = tag_value(&field.tags, col);
            any |= !value.is_empty();
            let mut width = tableflow_schema::columns(
                version,
                field,
                index,
                layout.multiple_select,
                layout.group_sep,
                layout.hierarchy_in_labels,
            )
            .len();
            if layout.include_media_url && is_media(field) {
                width += 1;
            }
            row.push(value);
            row.extend(std::iter::repeat_n(String::new(), width.saturating_sub(1)));
        }
        if any {
            rows.push(row);
        }
    }
    rows
}

/// The value of tag column `col` for a field: the parts of tags matching
/// `<col>:<value>` joined (no separator for `hxl`, a space otherwise).
fn tag_value(tags: &[String], col: &str) -> String {
    let prefix = format!("{col}:");
    let separator = if col == "hxl" { "" } else { " " };
    tags.iter()
        .filter_map(|tag| tag.strip_prefix(&prefix))
        .collect::<Vec<_>>()
        .join(separator)
}

/// The copy-field cell values for one submission.
fn copy_values(copy_fields: &[&str], data: &Value, index: Option<usize>) -> Vec<String> {
    copy_fields
        .iter()
        .map(|&name| copy_value(name, data, index))
        .collect()
}

/// One copy field's value: `_tags` joined by `", "`, `_validation_status` as
/// its uid (names mode) or label (a language), otherwise the scalar; empty when
/// absent.
fn copy_value(name: &str, data: &Value, index: Option<usize>) -> String {
    let Some(value) = data.get(name) else {
        return String::new();
    };
    match name {
        "_tags" => value.as_array().map_or_else(String::new, |items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(", ")
        }),
        "_validation_status" => match value.as_object() {
            Some(status) => {
                let key = if index.is_none() { "uid" } else { "label" };
                status
                    .get(key)
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_owned()
            }
            None => scalar(value),
        },
        _ => scalar(value),
    }
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

/// Export every section as a [`Table`]. `title` names the main table (and is
/// the `_parent_table_name` of its direct repeat rows).
#[must_use]
pub fn export_tables(
    version: &Version,
    submissions: &[Value],
    title: &str,
    layout: &Layout,
) -> Vec<Table> {
    let index = tableflow_schema::lang_index(version, layout.lang);

    let mut tables: Vec<Table> = version
        .sections
        .iter()
        .map(|section| Table {
            name: section_name(section, title),
            header: section_header(version, section, index, layout),
            rows: tag_rows(version, section, index, layout),
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
            layout,
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

/// The expanded value columns of a section (fields only, no auto columns).
fn field_columns(
    version: &Version,
    section: &Section,
    lang: Option<usize>,
    layout: &Layout,
) -> Vec<String> {
    let mut cols = Vec::new();
    for field in included_fields(section, layout) {
        cols.extend(tableflow_schema::columns(
            version,
            field,
            lang,
            layout.multiple_select,
            layout.group_sep,
            layout.hierarchy_in_labels,
        ));
        if layout.include_media_url && is_media(field) {
            let base = tableflow_schema::header_path(
                field,
                lang,
                layout.group_sep,
                layout.hierarchy_in_labels,
            );
            cols.push(format!("{base}_URL"));
        }
    }
    cols
}

/// The expanded value cells of a section for one data object (fields only).
fn field_values(
    version: &Version,
    section: &Section,
    data: &Value,
    lang: Option<usize>,
    layout: &Layout,
) -> Vec<String> {
    let mut vals = Vec::new();
    for field in included_fields(section, layout) {
        vals.extend(tableflow_schema::values(
            version,
            field,
            data.get(&field.path),
            lang,
            layout.multiple_select,
        ));
        if layout.include_media_url && is_media(field) {
            vals.push(media_url(
                data,
                &scalar(data.get(&field.path).unwrap_or(&Value::Null)),
            ));
        }
    }
    vals
}

/// Whether a field carries a media attachment.
fn is_media(field: &tableflow_core::Field) -> bool {
    MEDIA_TYPES.contains(&field.kind.as_str())
}

/// The section's fields kept by `layout.filter_fields` (all when unset).
fn included_fields<'a>(
    section: &'a Section,
    layout: &'a Layout,
) -> impl Iterator<Item = &'a tableflow_core::Field> {
    section.fields.iter().filter(move |field| {
        layout
            .filter_fields
            .is_none_or(|set| set.contains(&field.name.as_str()))
    })
}

/// The download URL for a media `value`, matched by file name in the
/// submission's `_attachments`; empty when absent.
fn media_url(data: &Value, value: &str) -> String {
    if value.is_empty() {
        return String::new();
    }
    let Some(attachments) = data.get("_attachments").and_then(Value::as_array) else {
        return String::new();
    };
    for attachment in attachments {
        let name = attachment
            .get("filename")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let basename = name.rsplit('/').next().unwrap_or(name);
        if name == value || basename == value {
            return attachment
                .get("download_url")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned();
        }
    }
    String::new()
}

/// Emit one row for `section` from `data`, then recurse into its child repeats.
#[expect(
    clippy::too_many_arguments,
    reason = "export state threaded explicitly"
)]
fn emit_section(
    version: &Version,
    section_index: usize,
    data: &Value,
    parent: Option<(&str, usize)>,
    lang: Option<usize>,
    layout: &Layout,
    counters: &mut [usize],
    tables: &mut [Table],
) {
    counters[section_index] += 1;
    let my_index = counters[section_index];
    let section = &version.sections[section_index];

    let mut row = field_values(version, section, data, lang, layout);
    if section.parent.is_none() {
        row.extend(copy_values(layout.copy_fields, data, lang));
    }
    if section.has_children || layout.force_index {
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
                    layout,
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

/// A section's header: expanded value columns, then auto columns.
fn section_header(
    version: &Version,
    section: &Section,
    lang: Option<usize>,
    layout: &Layout,
) -> Vec<String> {
    let mut header = field_columns(version, section, lang, layout);
    if section.parent.is_none() {
        header.extend(layout.copy_fields.iter().map(|&name| name.to_owned()));
    }
    if section.has_children || layout.force_index {
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
