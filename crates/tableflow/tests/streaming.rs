//! `export_csv_to` (streaming) must produce exactly what `export_csv` returns.

use serde_json::json;
use tableflow::Layout;

#[test]
fn streaming_csv_matches_in_memory() {
    let version = json!({ "content": {
        "survey": [
            { "type": "text", "name": "name", "label": "Name" },
            { "type": "select_multiple", "select_from_list_name": "c", "name": "cols", "label": "Colours" }
        ],
        "choices": [
            { "list_name": "c", "name": "r", "label": "Red" },
            { "list_name": "c", "name": "b", "label": "Blue" }
        ]
    } });
    let submissions = [
        json!({ "name": "Al \"ice\"", "cols": "r b" }),
        json!({ "name": "Bob", "cols": "r" }),
        json!({}),
    ];
    let layout = Layout::default();

    let in_memory = tableflow::export_csv(&version, &submissions, &layout);

    let mut buffer = Vec::new();
    tableflow::export_csv_to(&mut buffer, &version, &submissions, &layout).expect("stream csv");

    assert_eq!(String::from_utf8(buffer).expect("utf-8"), in_memory);
}
