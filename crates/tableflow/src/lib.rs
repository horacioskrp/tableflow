//! tableflow: turn form submissions into tabular exports.
//!
//! End-to-end facade over the pipeline. Phase 1 exposes a flat, single-version
//! CSV export; richer formats and structure arrive in later phases.
//!
//! # Example
//!
//! ```
//! use serde_json::json;
//!
//! let version = json!({ "content": { "survey": [
//!     { "type": "text", "name": "name", "label": "Your name" },
//!     { "type": "integer", "name": "age", "label": "Your age" }
//! ] } });
//! let submissions = [json!({ "name": "Alice", "age": 30 })];
//!
//! let csv = tableflow::export_csv(&version, &submissions);
//! assert_eq!(csv, "\"name\";\"age\"\n\"Alice\";\"30\"");
//! ```

use serde_json::Value;

#[doc(inline)]
pub use tableflow_core::{Field, Version, parse_version};

/// Parse a version schema and export its submissions as CSV.
#[must_use]
pub fn export_csv(version_schema: &Value, submissions: &[Value]) -> String {
    let version = parse_version(version_schema);
    tableflow_export::to_csv(&version, submissions)
}
