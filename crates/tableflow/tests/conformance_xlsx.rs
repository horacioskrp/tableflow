//! XLSX conformance: for each `tests/fixtures/<name>.xlsx.json` golden, the
//! `.xlsx` that `tableflow::export_xlsx` produces — read back as a sheet/cell
//! grid — must match the grid the reference exporter produces.
//!
//! XLSX files are not byte-reproducible (zip metadata, timestamps), so the
//! golden records the logical grid: `{ "sheets": [{ "name", "rows" }] }`, every
//! cell a string. Rows are normalized to the header width before comparison, so
//! trailing-empty-cell bookkeeping on either side is irrelevant.

use std::fs;
use std::io::Cursor;
use std::path::Path;

use calamine::{Data, Reader, Xlsx, open_workbook_from_rs};
use serde_json::Value;

#[test]
fn xlsx_fixtures_match_reference() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let mut checked = 0;

    for entry in fs::read_dir(&dir).expect("fixtures directory") {
        let path = entry.expect("dir entry").path();
        // Golden files are named `<name>.xlsx.json`.
        if !path.to_string_lossy().ends_with(".xlsx.json") {
            continue;
        }
        let stem = path
            .file_name()
            .and_then(|s| s.to_str())
            .and_then(|s| s.strip_suffix(".xlsx.json"))
            .expect("fixture stem")
            .to_owned();

        let input: Value =
            serde_json::from_str(&fs::read_to_string(dir.join(format!("{stem}.json"))).unwrap())
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
        let mode = input
            .get("multiple_select")
            .and_then(Value::as_str)
            .unwrap_or("both");

        let bytes = tableflow::export_xlsx(&input["version"], &submissions, lang, title, mode, &[])
            .expect("build xlsx");
        let got = grid_of(&bytes);

        let golden: Value =
            serde_json::from_str(&fs::read_to_string(&path).expect("read golden")).unwrap();
        let want = normalize_golden(&golden);

        assert_eq!(got, want, "xlsx grid mismatch for fixture `{stem}`");
        checked += 1;
    }

    assert!(checked >= 1, "no xlsx fixtures found");
}

/// A sheet's name with its rows normalized to the header width.
type Sheet = (String, Vec<Vec<String>>);

/// Read the workbook bytes into `(name, rows)` sheets, rows padded/trimmed to
/// the header (row 0) width.
fn grid_of(bytes: &[u8]) -> Vec<Sheet> {
    let mut workbook: Xlsx<_> =
        open_workbook_from_rs(Cursor::new(bytes.to_vec())).expect("open xlsx");
    let names = workbook.sheet_names().to_owned();

    names
        .into_iter()
        .map(|name| {
            let range = workbook.worksheet_range(&name).expect("sheet range");
            let rows: Vec<Vec<String>> = range
                .rows()
                .map(|row| row.iter().map(cell_text).collect())
                .collect();
            (name, fit_to_header(rows))
        })
        .collect()
}

/// Render a calamine cell as the reference exporter would (text; empty → "").
fn cell_text(cell: &Data) -> String {
    match cell {
        Data::Empty => String::new(),
        Data::String(s) => s.clone(),
        other => other.to_string(),
    }
}

/// Parse the golden JSON into `(name, rows)` sheets, rows fitted to the header.
fn normalize_golden(golden: &Value) -> Vec<Sheet> {
    golden["sheets"]
        .as_array()
        .expect("sheets array")
        .iter()
        .map(|sheet| {
            let name = sheet["name"].as_str().expect("sheet name").to_owned();
            let rows: Vec<Vec<String>> = sheet["rows"]
                .as_array()
                .expect("rows array")
                .iter()
                .map(|row| {
                    row.as_array()
                        .expect("row array")
                        .iter()
                        .map(|c| c.as_str().unwrap_or_default().to_owned())
                        .collect()
                })
                .collect();
            (name, fit_to_header(rows))
        })
        .collect()
}

/// Pad or trim every row to the width of the header row (row 0).
fn fit_to_header(mut rows: Vec<Vec<String>>) -> Vec<Vec<String>> {
    let width = rows.first().map_or(0, Vec::len);
    for row in &mut rows {
        row.resize(width, String::new());
    }
    rows
}
