# tableflow-content

Placeholder crate for XLSForm **content normalization** (alias replacement and
expansion) in the [`tableflow`](https://crates.io/crates/tableflow) pipeline.

> **Status: stub.** This crate is a reserved slot in the workspace and currently
> has no public API. `tableflow` expects already-expanded content (the shape
> `parse_version` consumes). It is **not published** to crates.io.

Normalization that turns raw XLSForm rows into expanded content (resolving type
aliases, splitting `select_one <list>` types, etc.) will live here when needed;
until then the facade consumes content that is already expanded.

## License

Licensed under either of Apache-2.0 or MIT at your option.
