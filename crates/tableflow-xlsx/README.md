# tableflow-xlsx

XLSX serializer for [`tableflow`](https://crates.io/crates/tableflow): write the
exported [`Table`]s to an `.xlsx` workbook, one worksheet per table.

Most users should depend on the [`tableflow`](https://crates.io/crates/tableflow)
facade (`export_xlsx`) instead of this crate directly.

## What it provides

- **`to_xlsx(&[Table]) -> Result<Vec<u8>, XlsxError>`** — the workbook as bytes.
- **`write_xlsx(&[Table], &Path) -> Result<(), XlsxError>`** — save to a file.

Each [`Table`] becomes one worksheet (the main table plus one per repeat). Every
cell is written as text (matching the reference exporter's default). Worksheet
names follow Excel's rules: the forbidden characters `[]:*?/\` and leading/
trailing apostrophes become `_`, names are truncated to 31 characters with an
ellipsis, and collisions get an incrementing ` (n)` suffix.

```rust,ignore
use tableflow_export::{export_tables, Layout};
use tableflow_xlsx::to_xlsx;

let tables = export_tables(&version, &submissions, "submissions", &Layout::default());
let bytes = to_xlsx(&tables)?;
std::fs::write("export.xlsx", bytes)?;
```

Built on [`rust_xlsxwriter`](https://crates.io/crates/rust_xlsxwriter).

[`Table`]: https://docs.rs/tableflow-export/latest/tableflow_export/struct.Table.html

## License

Licensed under either of Apache-2.0 or MIT at your option.
