# tableflow

[![CI](https://github.com/horacioskrp/tableflow/actions/workflows/ci.yml/badge.svg)](https://github.com/horacioskrp/tableflow/actions/workflows/ci.yml)
[![license](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](#license)

`tableflow` turns [XLSForm](https://xlsform.org/) **submissions** into tabular
exports — CSV, XLSX and GeoJSON — resolving labels and translations, flattening
repeat groups into linked tables, and reconciling multiple form versions into a
single coherent set of columns. It is the data-export counterpart to a form
compiler: where a compiler turns a form *definition* into a fillable form,
`tableflow` turns the *collected answers* into analyzable tables.

> **Status:** 🏗️ **Phase 0 — scaffolding.** The Cargo workspace, CI and the
> Docker build are in place; the model and export engine land next. See
> [ROADMAP.md](ROADMAP.md).

## What it will do

Given one or more form **versions** (XLSForm content) and a stream of
**submissions**, produce:

- **CSV / XLSX** exports, one table per section, with repeat groups flattened
  into linked child tables (`_index` / `_parent_index`).
- **Label & translation** resolution (`lang`): coded choice values rendered as
  human-readable labels.
- **`select_multiple`** rendered as a joined cell and/or one boolean column per
  choice (`both` / `summary` / `details`).
- **Multi-version** reconciliation: fields added, removed or edited across
  versions merged into one stable column canvas.
- **GeoJSON / KML**, SPSS value labels, and automatic per-field statistics.

## Workspace layout

| Crate | Responsibility |
| --- | --- |
| `tableflow-core` | Data model: form pack, versions, sections, fields, submissions |
| `tableflow-content` | XLSForm content normalization (aliases, expansion) |
| `tableflow-schema` | Field type system: submission value → export cell(s) |
| `tableflow-export` | Export engine: field canvas, repeat flattening, translations |
| `tableflow-xlsx` | XLSX serializer: one worksheet per exported table |
| `tableflow-geojson` | GeoJSON serializer: geo responses to a `FeatureCollection` |
| `tableflow-autoreport` | Per-field summary report: counts, frequencies, percentages |
| `tableflow` | End-to-end facade |
| `tableflow-cli` | Command-line interface |

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

Correctness is defined by matching a **reference exporter** byte-for-byte (after
format-aware normalization) on a golden corpus of forms + submissions;
`tableflow` is never its own oracle. The golden fixtures are committed, so
running the tests needs no external tools. Fixtures arrive with Phase 1.

## License

Licensed under either of [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT) at
your option.
