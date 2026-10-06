//! Prints every export for a small running example — used to keep the README's
//! use-case outputs honest. Run with `cargo run -p tableflow --example showcase`.

use serde_json::json;
use tableflow::{Layout, MultipleSelect};

fn main() {
    // A translated form: a text field, a numeric field, a select_one and a
    // select_multiple, each with English + French labels and choice labels.
    let version = json!({ "content": {
        "translations": ["English", "Français"],
        "survey": [
            { "type": "text", "name": "name", "label": ["Name", "Nom"] },
            { "type": "integer", "name": "age", "label": ["Age", "Âge"] },
            { "type": "select_one", "select_from_list_name": "col",
              "name": "fav", "label": ["Favourite", "Préférée"] },
            { "type": "select_multiple", "select_from_list_name": "lng",
              "name": "langs", "label": ["Languages", "Langues"] }
        ],
        "choices": [
            { "list_name": "col", "name": "red", "label": ["Red", "Rouge"] },
            { "list_name": "col", "name": "blue", "label": ["Blue", "Bleu"] },
            { "list_name": "lng", "name": "en", "label": ["English", "Anglais"] },
            { "list_name": "lng", "name": "fr", "label": ["French", "Français"] }
        ]
    } });
    let submissions = [
        json!({ "name": "Alice", "age": 30, "fav": "red", "langs": "en fr" }),
        json!({ "name": "Bob", "age": 25, "fav": "blue", "langs": "fr" }),
    ];

    show(
        "CSV — names mode (both)",
        tableflow::export_csv(&version, &submissions, &Layout::default()),
    );

    show(
        "CSV — French labels (both)",
        tableflow::export_csv(
            &version,
            &submissions,
            &Layout {
                lang: Some("Français"),
                ..Layout::default()
            },
        ),
    );

    show(
        "CSV — select_multiple = summary",
        tableflow::export_csv(
            &version,
            &submissions,
            &Layout {
                multiple_select: MultipleSelect::Summary,
                ..Layout::default()
            },
        ),
    );

    show(
        "CSV — copy_fields (_id, _submission_time)",
        tableflow::export_csv(
            &version,
            &[
                json!({ "name": "Alice", "age": 30, "fav": "red", "langs": "en",
                      "_id": 7, "_submission_time": "2024-01-02T03:04:05" }),
            ],
            &Layout {
                copy_fields: &["_id", "_submission_time"],
                ..Layout::default()
            },
        ),
    );

    // Groups + a repeat → linked tables.
    let repeated = json!({ "content": { "survey": [
        { "type": "text", "name": "hh", "label": "Household" },
        { "type": "begin_repeat", "name": "members", "label": "Members" },
        { "type": "text", "name": "mname", "label": "Member name" },
        { "type": "end_repeat" }
    ] } });
    let repeated_subs = [json!({
        "hh": "Doe",
        "members": [{ "members/mname": "Al" }, { "members/mname": "Bea" }]
    })];
    show(
        "Repeats — multi-section text",
        tableflow::export_tables_text(&repeated, &repeated_subs, "household", &Layout::default()),
    );

    // GeoJSON.
    let geo = json!({ "content": { "survey": [
        { "type": "text", "name": "site", "label": "Site" },
        { "type": "geopoint", "name": "loc", "label": "Location" }
    ] } });
    let geo_subs = [json!({ "site": "Alpha", "loc": "40.7128 -74.0060 10 5" })];
    show(
        "GeoJSON",
        tableflow::export_geojson(&geo, &geo_subs, None, "sites"),
    );

    // Per-field report.
    show(
        "Report (JSON)",
        tableflow::export_report(&version, &submissions, None, None),
    );

    // Multiple versions: v2 adds `email`.
    let v1 = json!({ "content": { "survey": [
        { "type": "text", "name": "name", "label": "Name" }
    ] } });
    let v2 = json!({ "content": { "survey": [
        { "type": "text", "name": "name", "label": "Name" },
        { "type": "text", "name": "email", "label": "Email" }
    ] } });
    let versioned = [
        json!({ "name": "Old" }),
        json!({ "name": "New", "email": "n@x.io" }),
    ];
    show(
        "Multiple versions — merged CSV",
        tableflow::export_csv_versions(&[v1, v2], &versioned, &Layout::default()),
    );
}

fn show(title: &str, output: String) {
    println!("\n===== {title} =====\n{output}");
}
