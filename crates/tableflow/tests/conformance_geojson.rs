//! GeoJSON conformance: for each `tests/fixtures/<name>.geojson` golden, the
//! `FeatureCollection` that `tableflow::export_geojson` produces must match the
//! reference exporter's — compared as parsed JSON (key order and number
//! formatting are irrelevant).

use std::fs;
use std::path::Path;

use serde_json::Value;

#[test]
fn geojson_fixtures_match_reference() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let mut checked = 0;

    for entry in fs::read_dir(&dir).expect("fixtures directory") {
        let path = entry.expect("dir entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("geojson") {
            continue;
        }
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
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
        let title = input
            .get("title")
            .and_then(Value::as_str)
            .unwrap_or("submissions");

        let got: Value = serde_json::from_str(&tableflow::export_geojson(
            &input["version"],
            &submissions,
            lang,
            title,
        ))
        .expect("parse produced geojson");
        let want: Value = serde_json::from_str(&fs::read_to_string(&path).expect("read golden"))
            .expect("parse golden geojson");

        assert_eq!(got, want, "geojson mismatch for fixture `{stem}`");
        checked += 1;
    }

    assert!(checked >= 1, "no geojson fixtures found");
}
