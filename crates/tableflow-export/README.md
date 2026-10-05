# tableflow-export

The export engine for [`tableflow`](https://crates.io/crates/tableflow): the
field canvas, repeat flattening into linked tables, and CSV serialization.

Most users should depend on the [`tableflow`](https://crates.io/crates/tableflow)
facade instead of this crate directly.

## What it provides

- **`Layout`** — the export options passed to the functions below: `lang`,
  `multiple_select`, `group_sep`, `hierarchy_in_labels`, `copy_fields`,
  `tag_cols`, `filter_fields`, `include_media_url`, `force_index`.
  `Layout::default()` is names mode / `both` / `/` / no extras.
- **`Table`** — one exported table: `name`, `header`, `rows`.
- **`to_csv(version, submissions, &layout) -> String`** — the main section as
  CSV (`;`-separated, every cell double-quoted, inner quotes doubled).
- **`export_tables(version, submissions, title, &layout) -> Vec<Table>`** —
  every section: the main table plus one per repeat, linked by `_index` /
  `_parent_table_name` / `_parent_index`.
- **`tables_to_text(&[Table]) -> String`** — frame the tables as
  `# section: <name>` blocks for golden comparison / inspection.

Field columns/values come from [`tableflow-schema`]; `copy_fields` appends
trailing metadata columns (`_tags` joined, `_validation_status` as uid/label);
`tag_cols` emits HXL tag header rows; `include_media_url` adds `<name>_URL`
columns from each submission's `_attachments`.

```rust
use serde_json::json;
use tableflow_core::parse_version;
use tableflow_export::{to_csv, Layout};

let version = parse_version(&json!({ "content": { "survey": [
    { "type": "text", "name": "a", "label": "A" }
] } }));
let rows = [json!({ "a": "x" })];
assert_eq!(to_csv(&version, &rows, &Layout::default()), "\"a\"\n\"x\"");
```

[`tableflow-schema`]: https://crates.io/crates/tableflow-schema

## License

Licensed under either of Apache-2.0 or MIT at your option.
