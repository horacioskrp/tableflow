# tableflow-cli

Command-line interface for [`tableflow`](https://crates.io/crates/tableflow):
read a form definition and submissions, write a tabular export.

> **Status: scaffold.** Argument parsing and export dispatch are not implemented
> yet, so this crate is **not published** to crates.io. Use the
> [`tableflow`](https://crates.io/crates/tableflow) library for now.

Planned: `tableflow --format csv|xlsx|geojson|report <version.json> <submissions.json>`,
dispatching to the library's `export_*` functions with the usual `Layout`
options (language, `select_multiple` mode, `copy_fields`, `tag_cols`, …).

## License

Licensed under either of Apache-2.0 or MIT at your option.
