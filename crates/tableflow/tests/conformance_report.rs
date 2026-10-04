//! Report conformance: for each `tests/fixtures/<name>.report.json` golden, the
//! per-field summary that `tableflow::export_report` produces must match the
//! reference exporter's autoreport — compared as parsed JSON.
//!
//! Only the covered field types appear in these fixtures: numeric summaries
//! (mean / median / mode / stdev) are not yet produced, so `integer` /
//! `decimal` fields are excluded here.

use std::fs;
use std::path::Path;

use serde_json::Value;

#[test]
fn report_fixtures_match_reference() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let mut checked = 0;

    for entry in fs::read_dir(&dir).expect("fixtures directory") {
        let path = entry.expect("dir entry").path();
        if !path.to_string_lossy().ends_with(".report.json") {
            continue;
        }
        let stem = path
            .file_name()
            .and_then(|s| s.to_str())
            .and_then(|s| s.strip_suffix(".report.json"))
            .expect("fixture stem")
            .to_owned();

        let input: Value = serde_json::from_str(
            &fs::read_to_string(dir.join(format!("{stem}.json"))).expect("read fixture json"),
        )
        .expect("parse fixture json");
        let submissions: Vec<Value> = input["submissions"]
            .as_array()
            .expect("submissions array")
            .clone();
        let lang = input.get("lang").and_then(Value::as_str);

        let got: Value = serde_json::from_str(&tableflow::export_report(
            &input["version"],
            &submissions,
            lang,
        ))
        .expect("parse produced report");
        let want: Value =
            serde_json::from_str(&fs::read_to_string(&path).expect("read golden")).unwrap();

        assert_eq!(got, want, "report mismatch for fixture `{stem}`");
        checked += 1;
    }

    assert!(checked >= 1, "no report fixtures found");
}
