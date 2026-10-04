# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/), and the project aims to adhere
to [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added

- Phase 0 scaffolding: Cargo workspace (edition 2024) with the pipeline crates
  (`tableflow-core`, `-content`, `-schema`, `-export`), the `tableflow` facade
  and the `tableflow-cli` binary; CI (`fmt` / `clippy -D warnings` / `build` /
  `test`), a Docker build wrapper, `.gitattributes` and dual MIT/Apache-2.0
  licensing.
- Phase 1 walking skeleton: parse a single version's `content.survey` into
  fields (`tableflow-core`) and export a flat, single-version form's submissions
  to CSV (`tableflow-export::to_csv`, `tableflow::export_csv`) — `;`-separated,
  every field double-quoted (quotes doubled), default header of field names,
  matching the reference exporter. Data-driven conformance test with a committed
  golden (`simple_flat`).
- Phase 2 field types & translations: the model now carries per-translation
  labels and choice lists (`tableflow-core`); `tableflow-schema` resolves the
  requested language (`lang`) to label headers and renders `select_one` values
  as their choice label, falling back to names/raw values in "names mode"
  (unspecified / untranslated / unknown language). `export_csv` gains a `lang`
  argument. Goldens: `translated_{default,en,fr}`.
- Phase 3a group flattening: fields inside non-repeat groups flatten into the
  main table, keeping their short name/label as the column header while reading
  values by their full submission path (`group/field`). Goldens:
  `grouped_{default,en}`.
