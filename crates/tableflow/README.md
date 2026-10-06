# tableflow

Turn form submissions into tabular exports — **CSV**, **XLSX**, **GeoJSON** and a
per-field **summary report** — from an XLSForm-style definition and a list of
JSON submissions.

`tableflow` is the end-to-end facade over a small pipeline of focused crates
(`tableflow-core`, `-schema`, `-export`, `-xlsx`, `-geojson`, `-autoreport`). In
almost all cases this is the only crate you need to depend on.

```toml
[dependencies]
tableflow = "0.1"
serde_json = "1"
```

## Table of contents

- [Inputs](#inputs)
- [Quick start](#quick-start)
- [Export functions](#export-functions)
- [`Layout` options](#layout-options)
- [Use cases](#use-cases)
  - [1. Names vs. translated labels](#1-names-vs-translated-labels)
  - [2. `select_multiple` expansion](#2-select_multiple-expansion)
  - [3. `or_other`](#3-or_other)
  - [4. Groups and repeats → linked tables](#4-groups-and-repeats--linked-tables)
  - [5. XLSX workbook](#5-xlsx-workbook)
  - [6. GeoJSON](#6-geojson)
  - [7. Per-field summary report](#7-per-field-summary-report)
  - [8. Disaggregated report (`split_by`)](#8-disaggregated-report-split_by)
  - [9. Metadata columns (`copy_fields`)](#9-metadata-columns-copy_fields)
  - [10. HXL tag header rows (`tag_cols`)](#10-hxl-tag-header-rows-tag_cols)
  - [11. Keep a subset of fields (`filter_fields`)](#11-keep-a-subset-of-fields-filter_fields)
  - [12. Media URLs (`include_media_url`)](#12-media-urls-include_media_url)
  - [13. Group-prefixed headers (`hierarchy_in_labels`)](#13-group-prefixed-headers-hierarchy_in_labels)
  - [14. Force an `_index` column (`force_index`)](#14-force-an-_index-column-force_index)
  - [15. Multiple form versions](#15-multiple-form-versions)
- [Crates](#crates)
- [License](#license)

## Inputs

Every function takes two `serde_json::Value` inputs:

- **A version schema** — the expanded XLSForm content:

  ```jsonc
  { "content": {
      "survey":       [ /* field rows: {type, name, label, …} */ ],
      "choices":      [ /* {list_name, name, label} */ ],
      "translations": [ "English", "Français" ]   // optional
  } }
  ```

  A field's `label` is a string, or an array aligned with `translations`.
  `select_*` fields reference a choice list via `select_from_list_name` (or the
  `select_one <list>` type form). Non-repeat `begin_group`/`end_group` only
  prefix the submission path; `begin_repeat`/`end_repeat` open a linked table.

- **Submissions** — a slice of objects mapping each field's path to its answer
  (`"age"`, `"group/city"`, …). A repeat's answers are an array of nested
  objects keyed by full path.

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

assert_eq!(
    tableflow::export_csv(&version, &submissions, &Layout::default()),
    "\"name\";\"col\"\n\"Alice\";\"r\"",
);
```

CSV cells are `;`-separated and every cell is double-quoted (inner quotes
doubled).

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

## `Layout` options

`Layout` carries the options for the CSV / tables / XLSX exports.
`Layout::default()` is names mode, `both`, group separator `/`, no extras.

```rust
use tableflow::{Layout, MultipleSelect};

let layout = Layout {
    lang: Some("Français"),                     // None = names mode
    multiple_select: MultipleSelect::Both,      // Both | Summary | Details
    group_sep: "/",                             // hierarchy + expansion separator
    hierarchy_in_labels: false,                 // prefix headers with group labels
    copy_fields: &["_id", "_submission_time"],  // extra trailing columns
    tag_cols: &["hxl"],                         // HXL tag header rows
    filter_fields: None,                        // Some(&["a","b"]) keeps only those
    include_media_url: false,                   // add <name>_URL columns
    force_index: false,                         // add _index to every section
};
```

`export_geojson` and `export_report` take their `lang` (and `split_by`)
directly rather than a `Layout`.

---

## Use cases

The first examples share this translated form — a text field, a numeric field,
a `select_one` and a `select_multiple`, each with English + French labels:

```rust
use serde_json::json;

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
        { "list_name": "col", "name": "red",  "label": ["Red", "Rouge"] },
        { "list_name": "col", "name": "blue", "label": ["Blue", "Bleu"] },
        { "list_name": "lng", "name": "en",   "label": ["English", "Anglais"] },
        { "list_name": "lng", "name": "fr",   "label": ["French", "Français"] }
    ]
} });
let submissions = [
    json!({ "name": "Alice", "age": 30, "fav": "red",  "langs": "en fr" }),
    json!({ "name": "Bob",   "age": 25, "fav": "blue", "langs": "fr" }),
];
```

### 1. Names vs. translated labels

`lang: None` keeps field names as headers and raw values; `lang: Some("…")`
emits label headers and translated `select_*` values.

```rust,ignore
tableflow::export_csv(&version, &submissions, &Layout::default());
```
```text
"name";"age";"fav";"langs";"langs/en";"langs/fr"
"Alice";"30";"red";"en fr";"1";"1"
"Bob";"25";"blue";"fr";"0";"1"
```

```rust,ignore
tableflow::export_csv(&version, &submissions,
    &Layout { lang: Some("Français"), ..Layout::default() });
```
```text
"Nom";"Âge";"Préférée";"Langues";"Langues/Anglais";"Langues/Français"
"Alice";"30";"Rouge";"Anglais Français";"1";"1"
"Bob";"25";"Bleu";"Français";"0";"1"
```

### 2. `select_multiple` expansion

`MultipleSelect::Both` (default) emits a joined summary column **and** one
`field/choice` boolean (`1`/`0`) column per option. `Summary` keeps only the
joined column; `Details` keeps only the booleans.

```rust,ignore
tableflow::export_csv(&version, &submissions,
    &Layout { multiple_select: MultipleSelect::Summary, ..Layout::default() });
```
```text
"name";"age";"fav";"langs"
"Alice";"30";"red";"en fr"
"Bob";"25";"blue";"fr"
```

### 3. `or_other`

A select declared with `or_other` (inline `select_* <list> or_other`, the
`select_*_or_other` type, or an `or_other` column) adds — for
`select_multiple` — a `field/other` boolean set when the answer includes the
`other` token, and for either select kind a companion `<name>_other` free-text
column right after it.

### 4. Groups and repeats → linked tables

Non-repeat groups flatten into the main table (values read by full path). Each
`begin_repeat` becomes its own table, linked by `_index` /
`_parent_table_name` / `_parent_index`. Use `export_tables_text` (or
`export_xlsx`) to get every section:

```rust,ignore
let version = json!({ "content": { "survey": [
    { "type": "text", "name": "hh", "label": "Household" },
    { "type": "begin_repeat", "name": "members", "label": "Members" },
    { "type": "text", "name": "mname", "label": "Member name" },
    { "type": "end_repeat" }
] } });
let submissions = [json!({
    "hh": "Doe",
    "members": [{ "members/mname": "Al" }, { "members/mname": "Bea" }]
})];
tableflow::export_tables_text(&version, &submissions, "household", &Layout::default());
```
```text
# section: household
"hh";"_index"
"Doe";"1"
# section: members
"mname";"_parent_table_name";"_parent_index"
"Al";"household";"1"
"Bea";"household";"1"
```

### 5. XLSX workbook

```rust,ignore
let bytes = tableflow::export_xlsx(&version, &submissions, "submissions", &Layout::default())?;
std::fs::write("export.xlsx", bytes)?;
```

One worksheet per table (main + each repeat); every cell is written as text.
Worksheet names follow Excel's rules (forbidden characters and edge apostrophes
→ `_`, truncated to 31 chars, de-duplicated).

### 6. GeoJSON

One `Feature` per answered geo question — `geopoint`/`gps` → `Point`,
`geotrace` → `LineString`, `geoshape` → `Polygon`. Coordinates are reordered to
`[lon, lat, alt]` (accuracy dropped); polygon rings are closed and wound
counter-clockwise. Other non-empty fields become `properties`.

```rust,ignore
let version = json!({ "content": { "survey": [
    { "type": "text", "name": "site", "label": "Site" },
    { "type": "geopoint", "name": "loc", "label": "Location" }
] } });
let submissions = [json!({ "site": "Alpha", "loc": "40.7128 -74.0060 10 5" })];
tableflow::export_geojson(&version, &submissions, None, "sites");
```
```json
{"type":"FeatureCollection","name":"sites","features":[
  {"type":"Feature",
   "geometry":{"type":"Point","coordinates":[-74.006,40.7128,10.0]},
   "properties":{"site":"Alpha"}}]}
```

### 7. Per-field summary report

Counts for every field; a `frequency`/`percentage` table for categorical fields
(`text`, `select_*`, `date`, `time`, media, …); `mean`/`median`/`mode`/`stdev`
for numeric fields (`integer`/`decimal`/`range`). Numeric summaries match the
reference exporter bit-for-bit; `mode` is `"*"` when not unique.

```rust,ignore
tableflow::export_report(&version, &submissions, None, None);
```
```jsonc
{ "submissions_count": 2, "fields": [
  { "name": "name", "label": "name", "stats": {
      "total_count": 2, "not_provided": 0, "provided": 2, "show_graph": false,
      "frequency": [["Alice",1],["Bob",1]], "percentage": [["Alice",50.0],["Bob",50.0]] } },
  { "name": "age", "label": "age", "stats": {
      "total_count": 2, "not_provided": 0, "provided": 2, "show_graph": false,
      "median": 27.5, "mean": 27.5, "mode": "*", "stdev": 3.5355339059327378 } },
  { "name": "fav", "label": "fav", "stats": {
      "total_count": 2, "not_provided": 0, "provided": 2, "show_graph": true,
      "frequency": [["red",1],["blue",1]], "percentage": [["red",50.0],["blue",50.0]] } },
  { "name": "langs", "label": "langs", "stats": {
      "total_count": 2, "not_provided": 0, "provided": 2, "show_graph": true,
      "frequency": [["fr",2],["en",1]], "percentage": [["fr",100.0],["en",50.0]] } }
] }
```

### 8. Disaggregated report (`split_by`)

`split_by: Some("field")` breaks every other field down by that field's values
(the split field itself is omitted): `values: [[answer, { frequency, percentage }]]`
across the split field's top-5 values (plus a `…` bucket). Numeric fields get
per-splitter `{ median, mean, mode, stdev }` instead.

```rust,ignore
tableflow::export_report(&version, &submissions, None, Some("fav"));
```

### 9. Metadata columns (`copy_fields`)

Append extra submission keys as trailing columns of the main section. `_tags`
(a list) joins with `, `; `_validation_status` (an object) renders as its uid
(names mode) or label; others are the scalar value, blank when absent.

```rust,ignore
tableflow::export_csv(&version, &submissions,
    &Layout { copy_fields: &["_id", "_submission_time"], ..Layout::default() });
```
```text
"name";"age";"fav";"langs";"langs/en";"langs/fr";"_id";"_submission_time"
"Alice";"30";"red";"en";"1";"0";"7";"2024-01-02T03:04:05"
```

### 10. HXL tag header rows (`tag_cols`)

Emit a header row per tag column (e.g. `hxl`) right after the labels, taking
each field's value from its `tags` (`["hxl:#code"]`). For a field tagged
`hxl:#qcode`, the `hxl` row carries `#qcode` under that field's first column and
blanks under its expansion columns.

```rust,ignore
let layout = Layout { tag_cols: &["hxl"], ..Layout::default() };
```

### 11. Keep a subset of fields (`filter_fields`)

```rust,ignore
let layout = Layout { filter_fields: Some(&["name", "age"]), ..Layout::default() };
// → only the `name` and `age` columns, in survey order.
```

### 12. Media URLs (`include_media_url`)

For each media field (`image`/`audio`/`video`/`file`/`background-audio`/`audit`),
append a `<name>_URL` column filled from the submission's `_attachments`
(matched by file name).

```rust,ignore
let layout = Layout { include_media_url: true, ..Layout::default() };
```

### 13. Group-prefixed headers (`hierarchy_in_labels`)

Prefix each header with its enclosing groups' labels (or names), joined by
`group_sep` — which also separates a `select_multiple`'s expansion columns. With
`group_sep: ":"`, a field `c` in group `g` renders as `Groupe:Couleur` (labels)
or `g:c` (names).

```rust,ignore
let layout = Layout { hierarchy_in_labels: true, lang: Some("Français"), ..Layout::default() };
```

### 14. Force an `_index` column (`force_index`)

By default `_index` appears only on sections that contain a repeat. Set
`force_index: true` to add it to every section.

```rust,ignore
let layout = Layout { force_index: true, ..Layout::default() };
```

### 15. Multiple form versions

When a form evolves, pass all version schemas (oldest-to-newest). The column set
is merged across every section (matched by repeat path): the newest version's
fields lead, then each older version appends its not-yet-seen fields. Each
submission fills the columns it carries and leaves the rest blank.

```rust,ignore
let v1 = json!({ "content": { "survey": [
    { "type": "text", "name": "name", "label": "Name" }
] } });
let v2 = json!({ "content": { "survey": [
    { "type": "text", "name": "name", "label": "Name" },
    { "type": "text", "name": "email", "label": "Email" }
] } });
let submissions = [ json!({ "name": "Old" }), json!({ "name": "New", "email": "n@x.io" }) ];
tableflow::export_csv_versions(&[v1, v2], &submissions, &Layout::default());
```
```text
"name";"email"
"Old";""
"New";"n@x.io"
```

Use `export_tables_text_versions` for forms whose repeats also change between
versions.

## Crates

| Crate | Responsibility |
| --- | --- |
| [`tableflow-core`](https://crates.io/crates/tableflow-core) | data model + multi-version merge |
| [`tableflow-schema`](https://crates.io/crates/tableflow-schema) | field type system → header(s) / cell(s) |
| [`tableflow-export`](https://crates.io/crates/tableflow-export) | CSV engine, repeat flattening, `Layout` |
| [`tableflow-xlsx`](https://crates.io/crates/tableflow-xlsx) | XLSX serializer |
| [`tableflow-geojson`](https://crates.io/crates/tableflow-geojson) | GeoJSON serializer |
| [`tableflow-autoreport`](https://crates.io/crates/tableflow-autoreport) | per-field summary report |
| `tableflow` | end-to-end facade (this crate) |

A runnable version of every example above lives in
[`examples/showcase.rs`](examples/showcase.rs):
`cargo run -p tableflow --example showcase`.

## License

Licensed under either of Apache-2.0 or MIT at your option.
