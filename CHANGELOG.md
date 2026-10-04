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
- Phase 3b repeats → linked tables: the model moves to a `Section` list (main +
  one per repeat); `tableflow-export::export_tables` emits every section with
  `_index` on parents and `_parent_table_name` / `_parent_index` on repeat rows,
  and `tableflow::export_tables_text` renders them framed by section. CSV export
  now emits the main section with its `_index` column when repeats are present.
  Multi-section golden `repeat_tables`. (Deferred: `hierarchy_in_labels`, nested
  repeats beyond one level.)
- Phase 4 `select_multiple` expansion: `tableflow-schema` now maps each field to
  one or more columns via `columns` / `values`, driven by a `MultipleSelect`
  mode (`both` / `summary` / `details`). A joined summary column (choice names,
  or labels in a language) and/or one `field/choice` boolean (`1`/`0`) column
  per option; headers honor the language. `export_csv` / `export_tables_text`
  gain a `multiple_select` argument. Goldens: `selmulti_{both,summary,details}`,
  `selmulti_en_both`. (Deferred: `or_other`.)
- Phase 5 multi-version field canvas: `tableflow-core::merge_versions` folds
  several versions into one export canvas — the newest listed version's fields
  lead, then each older version appends only its not-yet-seen fields — and the
  `tableflow::export_csv_versions` facade exports it, each submission filling the
  columns it carries and leaving the rest blank. Goldens: `multiversion`,
  `multiversion_reversed`. (Deferred: cross-version repeat merging and
  per-version field paths; only the main section is merged.)
