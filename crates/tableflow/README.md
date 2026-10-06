# tableflow

Turn form submissions into tabular exports — **CSV**, **XLSX**, **GeoJSON** and a
per-field **summary report** — from an XLSForm-style definition and a list of
JSON submissions.

`tableflow` is the end-to-end facade over a small pipeline of focused crates
([`tableflow-core`], [`tableflow-schema`], [`tableflow-export`],
[`tableflow-xlsx`], [`tableflow-geojson`], [`tableflow-autoreport`]). In almost
all cases this is the only crate you need to depend on.

```toml
[dependencies]
tableflow = "0.1"
serde_json = "1"
```

## Inputs

- **A version schema**: `{ "content": { "survey": [...], "choices": [...], "translations": [...] } }`
  — the expanded XLSForm content (survey rows, choice lists, declared
  translations). Passed as a `serde_json::Value`.
- **Submissions**: a slice of `serde_json::Value` objects, each mapping a
  field's path (e.g. `"age"`, `"group/city"`) to its answer. Repeat answers are
  arrays of nested objects.

## Quick start

```rust
use serde_json::json;
use tableflow::Layout;

let version = json!({ "content": {
    "survey": [
        { "type": "text", "name": "name", "label": "Name" },
        { "type": "select_one", "select_from_list_name": "c", "name": "col", "label": "Colour" }
    ],
    "choices": [ { "list_name": "c", "name": "r", "label": "Red" } ]
} });
let submissions = [json!({ "name": "Alice", "col": "r" })];

// CSV, names mode (field names as headers, raw choice values)
assert_eq!(
    tableflow::export_csv(&version, &submissions, &Layout::default()),
    "\"name\";\"col\"\n\"Alice\";\"r\"",
);
```

## Export functions

| Function | Output |
| --- | --- |
| `export_csv(version, submissions, &layout) -> String` | CSV of the **main section** |
| `export_tables_text(version, submissions, title, &layout) -> String` | every section (main + repeats) as framed multi-section text |
| `export_xlsx(version, submissions, title, &layout) -> Result<Vec<u8>, XlsxError>` | `.xlsx` workbook (one worksheet per table), as bytes |
| `export_geojson(version, submissions, lang, title) -> String` | a GeoJSON `FeatureCollection` |
| `export_report(version, submissions, lang, split_by) -> String` | per-field summary report (JSON) |
| `export_csv_versions(&[version], submissions, &layout) -> String` | CSV of a canvas **merged across versions** |
| `export_tables_text_versions(&[version], submissions, title, &layout) -> String` | merged multi-section text across versions |

CSV cells are `;`-separated, every cell double-quoted (inner quotes doubled).

## `Layout` options

`Layout` carries the export options; `Layout::default()` is names mode, `both`,
group separator `/`, no extras.

```rust
use tableflow::{Layout, MultipleSelect};

let layout = Layout {
    lang: Some("Français"),            // None = names mode
    multiple_select: MultipleSelect::Both, // Both | Summary | Details
    group_sep: "/",                    // separates hierarchy + expansion columns
    hierarchy_in_labels: false,        // prefix headers with enclosing group labels
    copy_fields: &["_id", "_submission_time"], // extra trailing metadata columns
    tag_cols: &["hxl"],                // HXL tag header rows
    filter_fields: None,               // Some(&["a","b"]) to keep only some fields
    include_media_url: false,          // add <name>_URL columns for media fields
    force_index: false,                // add an _index column to every section
};
```

## Use cases

### Translated labels

`lang: Some("<translation>")` emits label headers and translated `select_one` /
`select_multiple` values; `None` keeps field names and raw values.

### Groups and repeats

Non-repeat groups flatten into the main table (values read by full path). Each
`begin_repeat` becomes its own table, linked to its parent by `_index` /
`_parent_table_name` / `_parent_index`. Use `export_tables_text` (or
`export_xlsx`, which writes one worksheet per table) to get every section.

### `select_multiple`

`MultipleSelect::Both` emits a joined summary column **and** one `field/choice`
boolean column per option; `Summary` or `Details` pick one. `or_other` adds a
`field/other` boolean plus a companion `<name>_other` free-text column.

### XLSX

```rust,ignore
let bytes = tableflow::export_xlsx(&version, &submissions, "submissions", &Layout::default())?;
std::fs::write("export.xlsx", bytes)?;
```

One worksheet per table; every cell is written as text (worksheet names follow
Excel's rules: forbidden characters and edge apostrophes become `_`, truncated
to 31 chars and de-duplicated).

### GeoJSON

```rust,ignore
let fc = tableflow::export_geojson(&version, &submissions, None, "submissions");
```

One `Feature` per answered geo question — `geopoint`/`gps` → `Point`,
`geotrace` → `LineString`, `geoshape` → `Polygon` (coordinates `[lon, lat, alt]`,
rings wound counter-clockwise). Other non-empty fields become `properties`.

### Summary report

```rust,ignore
let report = tableflow::export_report(&version, &submissions, None, None);
// split_by: pass Some("field") to disaggregate every field by that field's values
```

Counts (`provided` / `not_provided` / `total_count`) for every field; frequency
and percentage tables for categorical fields; `mean`/`median`/`mode`/`stdev` for
numeric fields (`integer` / `decimal` / `range`), matching the reference
exporter bit-for-bit.

### Multiple versions

When a form evolves, pass all version schemas (oldest-to-newest). The column set
is merged — the newest version's fields lead, older versions append their
not-yet-seen fields — and every section (main + repeats) is merged by repeat
path. Each submission fills the columns it carries and leaves the rest blank.

## License

Licensed under either of Apache-2.0 or MIT at your option.
