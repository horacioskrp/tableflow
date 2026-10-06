//! tableflow: turn form submissions into tabular exports.
//!
//! End-to-end facade over the pipeline. It exposes CSV, multi-section text,
//! `.xlsx` ([`export_xlsx`]) and GeoJSON ([`export_geojson`]) exports in a
//! chosen language — with `select_multiple` expansion and a multi-version field
//! canvas ([`export_csv_versions`]) — plus a per-field summary report
//! ([`export_report`]).
//!
//! A [`Layout`] carries the export options (language, `select_multiple` mode,
//! header layout, copy fields, tag columns); `Layout::default()` is names mode,
//! `both`, group separator `/`, no extras.
//!
//! # Example
//!
//! ```
//! use serde_json::json;
//! use tableflow::Layout;
//!
//! let version = json!({ "content": {
//!     "translations": ["English", "Français"],
//!     "survey": [
//!         { "type": "text", "name": "name", "label": ["Your name", "Votre nom"] },
//!         { "type": "select_one", "select_from_list_name": "c", "name": "col",
//!           "label": ["Colour", "Couleur"] }
//!     ],
//!     "choices": [
//!         { "list_name": "c", "name": "r", "label": ["Red", "Rouge"] }
//!     ]
//! } });
//! let submissions = [json!({ "name": "Alice", "col": "r" })];
//!
//! assert_eq!(
//!     tableflow::export_csv(&version, &submissions, &Layout::default()),
//!     "\"name\";\"col\"\n\"Alice\";\"r\"",
//! );
//! assert_eq!(
//!     tableflow::export_csv(
//!         &version,
//!         &submissions,
//!         &Layout { lang: Some("Français"), ..Layout::default() },
//!     ),
//!     "\"Votre nom\";\"Couleur\"\n\"Alice\";\"Rouge\"",
//! );
//! ```

use serde_json::Value;

#[doc(inline)]
pub use tableflow_core::{
    Choice, ChoiceList, Field, Section, Version, merge_versions, parse_version,
};
#[doc(inline)]
pub use tableflow_export::{Layout, MultipleSelect, Table};
#[doc(inline)]
pub use tableflow_xlsx::XlsxError;

/// Parse a version schema and export the main section's submissions as CSV,
/// per `layout` (language, `select_multiple` mode, header layout and extras).
#[must_use]
pub fn export_csv(version_schema: &Value, submissions: &[Value], layout: &Layout) -> String {
    let version = parse_version(version_schema);
    tableflow_export::to_csv(&version, submissions, layout)
}

/// Stream the main section's submissions as CSV into `writer`, row by row,
/// without buffering the whole export in memory. Same output as [`export_csv`].
///
/// # Errors
///
/// Propagates any error from `writer`.
pub fn export_csv_to<W: std::io::Write>(
    writer: &mut W,
    version_schema: &Value,
    submissions: &[Value],
    layout: &Layout,
) -> std::io::Result<()> {
    let version = parse_version(version_schema);
    tableflow_export::write_csv(writer, &version, submissions, layout)
}

/// Merge several form versions and export the shared main-section canvas as CSV.
///
/// `version_schemas` are given oldest-to-newest; the merged column set leads
/// with the newest version's fields and appends each older version's new
/// fields. Each submission supplies its own keys, so a row from any version
/// fills the columns it has and leaves the rest blank.
#[must_use]
pub fn export_csv_versions(
    version_schemas: &[Value],
    submissions: &[Value],
    layout: &Layout,
) -> String {
    let versions: Vec<Version> = version_schemas.iter().map(parse_version).collect();
    let merged = merge_versions(&versions);
    tableflow_export::to_csv(&merged, submissions, layout)
}

/// Merge several form versions and export every section (main + repeats, merged
/// across versions) as framed multi-section text, with `title` naming the main
/// table.
#[must_use]
pub fn export_tables_text_versions(
    version_schemas: &[Value],
    submissions: &[Value],
    title: &str,
    layout: &Layout,
) -> String {
    let versions: Vec<Version> = version_schemas.iter().map(parse_version).collect();
    let merged = merge_versions(&versions);
    let tables = tableflow_export::export_tables(&merged, submissions, title, layout);
    tableflow_export::tables_to_text(&tables)
}

/// Parse a version schema and export every section (main + repeats) as a framed
/// multi-section text, with `title` naming the main table.
#[must_use]
pub fn export_tables_text(
    version_schema: &Value,
    submissions: &[Value],
    title: &str,
    layout: &Layout,
) -> String {
    let version = parse_version(version_schema);
    let tables = tableflow_export::export_tables(&version, submissions, title, layout);
    tableflow_export::tables_to_text(&tables)
}

/// Parse a version schema and export every section (main + repeats) as an
/// `.xlsx` workbook (one worksheet per table), returned as bytes.
///
/// `title` names the main worksheet. Every cell is written as text.
///
/// # Errors
///
/// Returns [`XlsxError`] if the workbook cannot be built.
pub fn export_xlsx(
    version_schema: &Value,
    submissions: &[Value],
    title: &str,
    layout: &Layout,
) -> Result<Vec<u8>, XlsxError> {
    let version = parse_version(version_schema);
    let tables = tableflow_export::export_tables(&version, submissions, title, layout);
    tableflow_xlsx::to_xlsx(&tables)
}

/// Parse a version schema and export the main section's geo responses as a
/// GeoJSON `FeatureCollection` (serialized), one feature per answered geo
/// question. `title` names the collection; repeats are not included.
#[must_use]
pub fn export_geojson(
    version_schema: &Value,
    submissions: &[Value],
    lang: Option<&str>,
    title: &str,
) -> String {
    let version = parse_version(version_schema);
    tableflow_geojson::to_geojson(&version, submissions, lang, title).to_string()
}

/// Parse a version schema and build a per-field summary report (counts,
/// frequencies, percentages and numeric summaries), serialized as JSON.
///
/// When `split_by` names a field, each other field is disaggregated by that
/// field's values.
#[must_use]
pub fn export_report(
    version_schema: &Value,
    submissions: &[Value],
    lang: Option<&str>,
    split_by: Option<&str>,
) -> String {
    let version = parse_version(version_schema);
    let report = tableflow_autoreport::report(&version, submissions, lang, split_by);
    serde_json::to_string(&report).unwrap_or_default()
}
