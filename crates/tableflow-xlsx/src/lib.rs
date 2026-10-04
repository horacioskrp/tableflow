//! XLSX serializer: write exported [`Table`]s as an `.xlsx` workbook.
//!
//! Each [`Table`] becomes one worksheet — the main table plus one per repeat —
//! named after the table (sanitized to Excel's rules: forbidden characters
//! `[]:*?/\` and leading/trailing `'` become `_`, names are truncated to 31
//! characters and de-duplicated). Every cell is written as text, matching the
//! reference exporter's default (`xls_types_as_text`), so numeric-looking
//! values stay verbatim.
//!
//! [`to_xlsx`] returns the workbook as bytes; [`write_xlsx`] saves it to a path.

use std::path::Path;

use rust_xlsxwriter::{Workbook, XlsxError as WriterError};
use thiserror::Error;

#[doc(inline)]
pub use tableflow_export::Table;

/// Excel's worksheet-name length limit.
const SHEET_NAME_LIMIT: usize = 31;

/// Characters Excel forbids in a worksheet name.
const FORBIDDEN: [char; 7] = ['[', ']', ':', '*', '?', '/', '\\'];

/// An error writing the workbook.
#[derive(Debug, Error)]
pub enum XlsxError {
    /// The underlying writer failed.
    #[error("xlsx writer error: {0}")]
    Writer(#[from] WriterError),
}

/// Serialize `tables` into an in-memory `.xlsx` workbook.
///
/// # Errors
///
/// Returns [`XlsxError`] if the workbook cannot be built.
pub fn to_xlsx(tables: &[Table]) -> Result<Vec<u8>, XlsxError> {
    Ok(build(tables)?.save_to_buffer()?)
}

/// Serialize `tables` and save the workbook to `path`.
///
/// # Errors
///
/// Returns [`XlsxError`] if the workbook cannot be built or written.
pub fn write_xlsx(tables: &[Table], path: &Path) -> Result<(), XlsxError> {
    build(tables)?.save(path)?;
    Ok(())
}

/// Build the workbook: one named worksheet per table, every cell a string.
fn build(tables: &[Table]) -> Result<Workbook, WriterError> {
    let mut workbook = Workbook::new();
    let mut taken: Vec<String> = Vec::new();

    for table in tables {
        let name = sheet_name(&table.name, &taken);
        taken.push(name.clone());

        let sheet = workbook.add_worksheet();
        sheet.set_name(&name)?;

        write_row(sheet, 0, &table.header)?;
        for (offset, row) in table.rows.iter().enumerate() {
            let index = u32::try_from(offset + 1).unwrap_or(u32::MAX);
            write_row(sheet, index, row)?;
        }
    }

    Ok(workbook)
}

/// Write one row of string cells at `row`.
fn write_row(
    sheet: &mut rust_xlsxwriter::Worksheet,
    row: u32,
    cells: &[String],
) -> Result<(), WriterError> {
    for (col, cell) in cells.iter().enumerate() {
        let col = u16::try_from(col).unwrap_or(u16::MAX);
        sheet.write_string(row, col, cell)?;
    }
    Ok(())
}

/// A unique, Excel-valid worksheet name for `raw`, avoiding names in `taken`.
///
/// Forbidden characters and leading/trailing apostrophes become `_`; the name
/// is truncated to 31 characters with an ellipsis, and collisions get an
/// incrementing ` (n)` suffix folded into that ellipsis.
fn sheet_name(raw: &str, taken: &[String]) -> String {
    let sanitized: String = raw
        .chars()
        .map(|c| if FORBIDDEN.contains(&c) { '_' } else { c })
        .collect();
    let sanitized = trim_edge_apostrophes(&sanitized);

    let candidate = ellipsize(&sanitized, "...");
    if !taken.iter().any(|t| t == &candidate) {
        return candidate;
    }
    let mut i = 1;
    loop {
        let candidate = ellipsize(&sanitized, &format!("... ({i})"));
        if !taken.iter().any(|t| t == &candidate) {
            return candidate;
        }
        i += 1;
    }
}

/// Replace a leading or trailing apostrophe with `_` (Excel forbids both).
fn trim_edge_apostrophes(name: &str) -> String {
    let mut chars: Vec<char> = name.chars().collect();
    if chars.first() == Some(&'\'') {
        chars[0] = '_';
    }
    if let Some(last) = chars.last_mut()
        && *last == '\''
    {
        *last = '_';
    }
    chars.into_iter().collect()
}

/// Fit `name` into 31 characters: keep it whole if it fits, else take the
/// leading characters and append `ellipsis` so the result is exactly 31 long.
fn ellipsize(name: &str, ellipsis: &str) -> String {
    let chars: Vec<char> = name.chars().collect();
    if chars.len() <= SHEET_NAME_LIMIT {
        return name.to_owned();
    }
    let keep = SHEET_NAME_LIMIT.saturating_sub(ellipsis.chars().count());
    let head: String = chars.into_iter().take(keep).collect();
    format!("{head}{ellipsis}")
}

#[cfg(test)]
mod tests {
    use super::{ellipsize, sheet_name};

    #[test]
    fn short_names_pass_through() {
        assert_eq!(sheet_name("members", &[]), "members");
    }

    #[test]
    fn forbidden_characters_become_underscore() {
        assert_eq!(sheet_name("a/b:c*d", &[]), "a_b_c_d");
    }

    #[test]
    fn long_names_are_ellipsized_to_31() {
        let name = sheet_name("This string has more than 31 characters!", &[]);
        assert_eq!(name, "This string has more than 31...");
        assert_eq!(name.chars().count(), 31);
    }

    #[test]
    fn collisions_get_a_numbered_suffix() {
        let long = "This string has more than 31 characters!";
        let first = ellipsize(long, "...");
        let second = sheet_name(long, std::slice::from_ref(&first));
        assert_eq!(second, "This string has more tha... (1)");
        assert_eq!(second.chars().count(), 31);
    }
}
