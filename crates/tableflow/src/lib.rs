//! tableflow: turn form submissions into tabular exports.
//!
//! End-to-end facade over the pipeline. Phase 1–5 expose CSV and multi-section
//! exports in a chosen language, with `select_multiple` expansion and a
//! multi-version field canvas ([`export_csv_versions`]); richer formats arrive
//! later.
//!
//! `multiple_select` is `"both"` (a joined summary column plus one boolean
//! column per choice), `"summary"`, or `"details"`.
//!
//! # Example
//!
//! ```
//! use serde_json::json;
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
//!     tableflow::export_csv(&version, &submissions, None, "both"),
//!     "\"name\";\"col\"\n\"Alice\";\"r\"",
//! );
//! assert_eq!(
//!     tableflow::export_csv(&version, &submissions, Some("Français"), "both"),
//!     "\"Votre nom\";\"Couleur\"\n\"Alice\";\"Rouge\"",
//! );
//! ```

use serde_json::Value;

#[doc(inline)]
pub use tableflow_core::{
    Choice, ChoiceList, Field, Section, Version, merge_versions, parse_version,
};
#[doc(inline)]
pub use tableflow_export::{MultipleSelect, Table};

/// Parse a version schema and export the main section's submissions as CSV in
/// language `lang` (`None` = names mode). `multiple_select` is
/// `"both"` / `"summary"` / `"details"`.
#[must_use]
pub fn export_csv(
    version_schema: &Value,
    submissions: &[Value],
    lang: Option<&str>,
    multiple_select: &str,
) -> String {
    let version = parse_version(version_schema);
    tableflow_export::to_csv(
        &version,
        submissions,
        lang,
        MultipleSelect::parse(multiple_select),
    )
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
    lang: Option<&str>,
    multiple_select: &str,
) -> String {
    let versions: Vec<Version> = version_schemas.iter().map(parse_version).collect();
    let merged = merge_versions(&versions);
    tableflow_export::to_csv(
        &merged,
        submissions,
        lang,
        MultipleSelect::parse(multiple_select),
    )
}

/// Parse a version schema and export every section (main + repeats) as a framed
/// multi-section text, with `title` naming the main table.
#[must_use]
pub fn export_tables_text(
    version_schema: &Value,
    submissions: &[Value],
    lang: Option<&str>,
    title: &str,
    multiple_select: &str,
) -> String {
    let version = parse_version(version_schema);
    let tables = tableflow_export::export_tables(
        &version,
        submissions,
        lang,
        title,
        MultipleSelect::parse(multiple_select),
    );
    tableflow_export::tables_to_text(&tables)
}
