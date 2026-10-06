# tableflow-schema

Field type system for [`tableflow`](https://crates.io/crates/tableflow): turn a
field and a submission value into the export header(s) and cell(s).

Most users should depend on the [`tableflow`](https://crates.io/crates/tableflow)
facade instead of this crate directly.

## What it provides

- **`MultipleSelect`** (`Both` / `Summary` / `Details`) with `parse(&str)`.
- **`lang_index(version, lang) -> Option<usize>`** — resolve a language name to
  its index in `content.translations` (`None` = names mode).
- **`header(field, lang)`** / **`header_path(field, lang, group_sep, hierarchy)`**
  — a field's column header, optionally prefixed by its enclosing groups'
  labels (`hierarchy_in_labels`).
- **`columns(version, field, lang, mode, group_sep, hierarchy)`** — the header
  columns a field contributes. A `select_multiple` expands into a summary
  column and/or one `field/choice` boolean column per option (plus `field/other`
  for `or_other`).
- **`values(version, field, value, lang, mode)`** — the matching cell values.
  `select_one` renders its choice label; `select_multiple` expands to the
  summary and booleans.
- **`cell(version, field, value, lang)`** — a single formatted cell for a
  non-`select_multiple` field.

```rust
use serde_json::json;
use tableflow_core::parse_version;
use tableflow_schema::{columns, MultipleSelect};

let version = parse_version(&json!({ "content": {
    "survey": [{ "type": "select_multiple", "select_from_list_name": "c", "name": "x", "label": "X" }],
    "choices": [{ "list_name": "c", "name": "a" }, { "list_name": "c", "name": "b" }]
} }));
let field = &version.sections[0].fields[0];
assert_eq!(
    columns(&version, field, None, MultipleSelect::Both, "/", false),
    vec!["x", "x/a", "x/b"],
);
```

## License

Licensed under either of Apache-2.0 or MIT at your option.
