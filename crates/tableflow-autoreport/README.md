# tableflow-autoreport

Per-field summary report for [`tableflow`](https://crates.io/crates/tableflow):
counts, frequencies, percentages and numeric summaries.

Most users should depend on the [`tableflow`](https://crates.io/crates/tableflow)
facade (`export_report`) instead of this crate directly.

## What it provides

- **`report(version, submissions, lang, split_by) -> Report`** — a `Report`
  (`submissions_count` + one `FieldReport` per field) that serializes to JSON
  via `serde`.

For every field (notes and analysis fields excluded): `provided` /
`not_provided` / `total_count`. Then, by field class (mirroring the reference
exporter's type → class mapping):

- **categorical** (`text`, `select_*`, `date`, `time`, `barcode`, media, …):
  a `frequency` table and matching `percentage`s. `text`-like types order by
  descending count; `select_*` translate values to choice labels; `date` orders
  chronologically. `select_*` and `date` set `show_graph`.
- **numeric** (`integer` / `decimal` / `range`): `mean` / `median` / `mode` /
  `stdev` (each `"*"` when undefined). The sum of squared deviations is
  accumulated in exact rational arithmetic and its square root correctly
  rounded, so the summaries match the reference bit-for-bit.

With `split_by: Some("field")`, each other field is disaggregated by that
field's values: `values: [[answer, { frequency, percentage }]]` (or per-splitter
`{ median, mean, mode, stdev }` for numeric fields) across the top-5 splitters
plus a `…` bucket.

```rust
use serde_json::json;
use tableflow_core::parse_version;
use tableflow_autoreport::report;

let version = parse_version(&json!({ "content": { "survey": [
    { "type": "text", "name": "city", "label": "City" }
] } }));
let rows = [json!({ "city": "Lome" }), json!({ "city": "Lome" })];
let r = report(&version, &rows, None, None);
assert_eq!(r.submissions_count, 2);
```

## License

Licensed under either of Apache-2.0 or MIT at your option.
