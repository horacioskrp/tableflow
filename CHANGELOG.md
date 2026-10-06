# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/), and the project aims to adhere
to [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added

- Streaming CSV: `tableflow_export::write_csv` and the `tableflow::export_csv_to`
  facade write the main-section CSV row by row into any `std::io::Write`, so a
  large export is never fully buffered in memory. `to_csv` / `export_csv` now
  delegate to it (output unchanged — rows `\n`-separated, no trailing newline).

### Performance

- Report frequency tallies are now linear: the per-field value counter,
  `top_splitters`, and the disaggregation value index use a hash index over an
  insertion-ordered list instead of a linear scan per value (previously O(n²) on
  high-cardinality fields). Output is unchanged (first-seen order preserved). A
  `criterion` benchmark harness (`benches/export.rs`) covers CSV and report
  exports at 1k/10k/50k submissions.

## [0.1.1] - 2026-10-06

No code changes. First version published to crates.io: `v0.1.0` was tagged
before the release workflow existed, so publishing starts at `0.1.1`.

## [0.1.0] - 2026-10-05

First release: CSV, multi-section text, XLSX, GeoJSON and per-field report
exports from an XLSForm-style definition and JSON submissions, conformance-
tested against the reference exporter.

### Fixed

- Report field classification now mirrors the reference's type→class mapping, a
  differential-audit fix: `time`, `barcode`, `acknowledge`, `calculate`, `rank`,
  `select_multiple_from_file`, `select_one_external`, `cascading_select` and the
  media types (`image`/`audio`/`video`/`file`/`background-audio`/`audit`) are now
  **categorical** (frequency/percentage, no graph); `range` is **numeric**
  (mean/median/mode/stdev); analysis/NLP fields (`qual*`, `transcript`,
  `translation`) are excluded like notes. Previously these all fell back to
  counts-only. New golden `report_types`.
- `include_media_url` now recognizes `background-audio` and `audit` as media
  fields (was only `image`/`audio`/`video`/`file`). New golden `media_types`.
- GeoJSON now emits a `Point` for the legacy `gps` type (alias of `geopoint`).
  New golden `geo_gps`.

### Added

- `force_index` export option: adds an `_index` column to every section even
  without repeats (`force_index` fixture).

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
- Phase 6 XLSX output: a new `tableflow-xlsx` crate (backed by
  `rust_xlsxwriter`) writes the exported tables to an `.xlsx` workbook — one
  worksheet per table (main + each repeat), every cell as text
  (`xls_types_as_text` default) — via `to_xlsx` (bytes) / `write_xlsx` (file).
  Worksheet names follow Excel's rules: forbidden characters `[]:*?/\` and edge
  apostrophes become `_`, names truncate to 31 characters with an ellipsis and
  de-duplicate with a ` (n)` suffix. The `tableflow::export_xlsx` facade returns
  the workbook bytes. XLSX conformance reads the workbook back (via `calamine`)
  and compares its sheet/cell grid to the reference exporter's; goldens
  `simple_flat`, `translated_fr`, `repeat_tables`, `selmulti_both` (as
  `.xlsx.json`). (Deferred: native cell types, HXL tag header rows, media URLs,
  copy / filter fields.)
- Phase 7 GeoJSON output: a new `tableflow-geojson` crate builds a
  `FeatureCollection` with one `Feature` per answered geo question in the main
  section — `geopoint`→`Point`, `geotrace`→`LineString`, `geoshape`→`Polygon`.
  Coordinates are swapped from XForm's `lat lon alt [acc]` to GeoJSON's
  `[lon, lat, alt]` (accuracy dropped); polygon rings are closed and wound
  counter-clockwise (RFC 7946 right-hand rule); the remaining non-geo, non-empty
  fields become summary-formatted `properties`. Submissions with missing or
  malformed geo data are skipped. The `tableflow::export_geojson` facade returns
  the serialized collection; conformance compares parsed JSON (goldens
  `geo_points`, `geo_shape_cw`). (Deferred: KML, SPSS value-label files, base64
  attachments.)
- Phase 8 per-field report: a new `tableflow-autoreport` crate summarizes every
  main-section field (notes excluded) — `provided` / `not_provided` /
  `total_count`, and for categorical fields a `frequency` table with matching
  `percentage`s (rounded to two decimals). `text` orders by descending count
  (ties keep first-seen order); `select_one` / `select_multiple` translate each
  value to its choice label and order by descending count (a `select_multiple`
  answer is split into its options); `date` orders chronologically; `select_*`
  and `date` set `show_graph`. The `tableflow::export_report` facade returns the
  serialized report; conformance compares parsed JSON (goldens `report_counts`,
  `report_translated`). (Deferred: `split_by` disaggregation.)
- Numeric report summaries: `integer` / `decimal` fields now carry `mean` /
  `median` / `mode` / `stdev` (each `"*"` when undefined — empty data, a lone
  value, or a non-unique mode). The sum of squared deviations is accumulated in
  exact rational arithmetic (`num-rational` / `num-bigint`) and its square root
  correctly rounded (a port of CPython's `statistics._float_sqrt_of_frac`), so
  the summaries match the reference exporter bit-for-bit. New goldens
  `report_numeric`, `report_numeric_edge`.
- `copy_fields` export option: extra submission keys (e.g. `_id`, `_uuid`,
  `_submission_time`, `_tags`, `_validation_status`) are appended as trailing
  columns of the main section. `_tags` lists join with `, `; `_validation_status`
  objects render as their uid (names mode) or label (a language); others are the
  scalar value, blank when absent. Threaded through `to_csv` / `export_tables`
  and the `export_csv` / `export_tables_text` / `export_xlsx` facades. New
  golden `copy_fields`.
- Cross-version repeat merging: `merge_versions` now merges every section (main
  and each repeat), matched across versions by `repeat_path`, remapping parents
  and recomputing `has_children`. The new `export_tables_text_versions` facade
  exports the merged multi-section canvas, so a repeat that gains or loses
  fields between versions exports as one linked table. New golden
  `multiversion_repeat`. (Still deferred: per-version field paths that differ
  for the same field name.)
- `filter_fields` export option: when set, only the listed fields are exported
  (in survey order). New golden `filter_fields`.
- `include_media_url` export option: a `<name>_URL` column is appended after each
  media field (image/audio/video/file), filled from the submission's
  `_attachments` (matched by file name). New golden `media_url`.
- `hierarchy_in_labels` export option: headers are prefixed by their enclosing
  groups' labels (or names in names mode), joined by `group_sep`, which also
  separates the `select_multiple` expansion columns. The model now records each
  field's `group_path` (`tableflow-core::GroupLabel`), and `tableflow-schema`
  gains `header_path`. The export functions now take a single
  `tableflow::Layout` (language, `select_multiple` mode, `group_sep`,
  `hierarchy_in_labels`, `copy_fields`, `tag_cols`) in place of their separate
  option arguments. New goldens `hierarchy_labels`, `hierarchy_sep`.
- `split_by` report disaggregation: `export_report` gains an optional split
  field; each other field's stats become `values: [[answer, {frequency,
  percentage}]]` broken down across the split field's top-5 values (with a
  trailing `…` bucket when it has more), the split field itself omitted.
  New golden `report_split`.
- Numeric `split_by`: `integer` / `decimal` fields disaggregated by a split
  field now carry per-splitter `{median, mean, mode, stdev}` (the same exact
  numeric summary, computed per group). New golden `report_split_numeric`.
- Nested repeats: a repeat inside a repeat now exports as its own linked table,
  with each level's `_index` / `_parent_table_name` / `_parent_index` resolved
  through the section tree (already supported by the model; locked by the new
  `nested_repeats` golden).
- HXL tag header rows (`tag_cols` export option): a field's `tags` (e.g.
  `hxl:#code`) are parsed onto the model, and for each requested tag column a
  header row is emitted right after the labels (CSV, multi-section text and
  XLSX), with each field's tag value at its first value column and blanks for
  its expansion columns. New golden `hxl_tags`.
- `or_other` selects: a field typed `select_* <list> or_other` (or
  `select_*_or_other`, or an `or_other` column) now carries an `or_other` flag
  on the model. `select_multiple` gains an `/other` details column (set when the
  value includes the `other` token), and either select kind gains a companion
  `<name>_other` free-text column right after it. New golden `selmulti_or_other`.
