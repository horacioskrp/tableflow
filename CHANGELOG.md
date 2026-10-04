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
