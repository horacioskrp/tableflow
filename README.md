# tableflow

[![CI](https://github.com/horacioskrp/tableflow/actions/workflows/ci.yml/badge.svg)](https://github.com/horacioskrp/tableflow/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/tableflow.svg)](https://crates.io/crates/tableflow)
[![license](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](#license)

`tableflow` turns [XLSForm](https://xlsform.org/) **submissions** into tabular
exports — CSV, XLSX, GeoJSON and a per-field summary report — resolving labels
and translations, flattening repeat groups into linked tables, and reconciling
multiple form versions into a single coherent set of columns. It is the
data-export counterpart to a form compiler: where a compiler turns a form
*definition* into a fillable form, `tableflow` turns the *collected answers*
into analyzable tables.

```toml
[dependencies]
tableflow = "0.1"
serde_json = "1"
```

## What it does

Given one or more form **versions** (XLSForm content) and a stream of
**submissions**, it produces:

- **CSV / XLSX** exports, one table per section, with repeat groups flattened
  into linked child tables (`_index` / `_parent_table_name` / `_parent_index`).
- **Label & translation** resolution (`lang`): coded choice values rendered as
  human-readable labels; hierarchical (`group/field`) headers on request.
- **`select_multiple`** as a joined summary cell and/or one boolean column per
  choice (`both` / `summary` / `details`), including `or_other`.
- **Multi-version** reconciliation: fields added, removed or edited across
  versions merged into one stable column canvas (repeats included).
- **GeoJSON**: geo questions as a `FeatureCollection`.
- **Per-field report**: counts, frequencies, percentages and numeric summaries
  (`mean` / `median` / `mode` / `stdev`), with optional `split_by`
  disaggregation.
- Export extras: `copy_fields` (metadata columns), HXL `tag_cols`,
  `filter_fields`, `include_media_url`, `force_index`.

See the [facade crate README](crates/tableflow/README.md) for the full API and a
worked example of every use case (`cargo run -p tableflow --example showcase`).

## Workspace layout

| Crate | Responsibility | crates.io |
| --- | --- | --- |
| `tableflow-core` | Data model: versions, sections, fields + multi-version merge | ✅ |
| `tableflow-schema` | Field type system: submission value → export header(s)/cell(s) | ✅ |
| `tableflow-export` | CSV engine: field canvas, repeat flattening, `Layout` options | ✅ |
| `tableflow-xlsx` | XLSX serializer: one worksheet per exported table | ✅ |
| `tableflow-geojson` | GeoJSON serializer: geo responses to a `FeatureCollection` | ✅ |
| `tableflow-autoreport` | Per-field summary report: counts, frequencies, numeric stats | ✅ |
| `tableflow` | End-to-end facade (depend on this) | ✅ |
| `tableflow-content` | XLSForm content normalization — reserved stub | — (`publish = false`) |
| `tableflow-cli` | Command-line interface — scaffold | — (`publish = false`) |

## Development

```sh
cargo build
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --all --check
```

> **Note for some Windows hosts.** If the native build is blocked (Smart App
> Control / WDAC, or no linker), build inside the Rust Docker image —
> `scripts/docker-dev.ps1` wraps `cargo` with cached volumes:
>
> ```powershell
> ./scripts/docker-dev.ps1 test
> ```

## Conformance

Correctness is defined by matching a **reference exporter** (after format-aware
normalization — byte-for-byte for CSV, cell grids for XLSX, parsed JSON for
GeoJSON and reports) on a golden corpus of forms + submissions; `tableflow` is
never its own oracle. The golden fixtures are committed under
`crates/tableflow/tests/fixtures/`, so running the tests needs no external
tools.

## License

Licensed under either of [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT) at
your option.
