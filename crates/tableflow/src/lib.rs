//! tableflow: turn form submissions into tabular exports.
//!
//! End-to-end facade over the pipeline. Phase 1–2 expose a flat, single-version
//! CSV export in a chosen language; richer formats and structure arrive later.
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
//! // Names mode (no language): headers are field names, choice value is raw.
//! assert_eq!(
//!     tableflow::export_csv(&version, &submissions, None),
//!     "\"name\";\"col\"\n\"Alice\";\"r\"",
//! );
//! // French: headers and the choice value are translated.
//! assert_eq!(
//!     tableflow::export_csv(&version, &submissions, Some("Français")),
//!     "\"Votre nom\";\"Couleur\"\n\"Alice\";\"Rouge\"",
//! );
//! ```

use serde_json::Value;

#[doc(inline)]
pub use tableflow_core::{Choice, ChoiceList, Field, Version, parse_version};

/// Parse a version schema and export its submissions as CSV in language `lang`
/// (`None` = names mode).
#[must_use]
pub fn export_csv(version_schema: &Value, submissions: &[Value], lang: Option<&str>) -> String {
    let version = parse_version(version_schema);
    tableflow_export::to_csv(&version, submissions, lang)
}
