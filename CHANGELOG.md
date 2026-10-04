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
