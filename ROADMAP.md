# Roadmap

`tableflow` is built phase by phase, each gated by a **GO** check: a phase is
done only when its whole slice of the golden corpus matches the reference
exporter (after format-aware normalization) and CI is green.

| Phase | Theme | Status |
| ----- | ----------------------------------------------------- | ------ |
| 0 | Workspace, CI, Docker & conformance harness | ✅ done |
| 1 | Walking skeleton — one version → CSV of a flat form | ✅ done |
| 2 | Field type system & translations | ✅ done |
| 3 | Groups & repeats → linked tables | ✅ done |
| 4 | `select_multiple` expansion | ✅ done |
| 5 | Multi-version field canvas | ✅ done |
| 6 | XLSX output & header options | ✅ done |
| 7 | GeoJSON / KML, SPSS labels, attachments | ☐ |
| 8 | Automatic per-field report | ☐ |

## Phase 0 — Workspace & harness ✅

- [x] Cargo workspace (edition 2024), pipeline crates, CLI scaffold
- [x] `.gitattributes` (LF), CI (`fmt` / `clippy -D warnings` / `build` / `test`)
- [x] Docker build wrapper (`scripts/docker-dev.ps1`)
- [x] Conformance harness: a data-driven runner (`csv_fixtures_match_reference`)
      that compares each fixture's export against its committed golden
- [x] **GO:** green `build` / `test` / `clippy` / `fmt`

## Phase 1 — Walking skeleton

A flat, single-version, single-language form with simple question types exports
to CSV, matching the reference exporter.

- [x] Load one version's content into the model (fields)
- [x] Map submissions onto the field canvas; emit CSV rows (`;`-separated,
      every field quoted, quotes doubled; default header = field names)
- [x] **GO:** the simplest form's CSV golden matches (`simple_flat` fixture)

## Phase 2 — Field types & translations

- [x] Per-type value formatting: scalar passthrough (text/int/decimal/date/
      time/dateTime/geo); `select_one` value → choice label
- [x] `lang` selection → label headers and translated choice values; names mode
      (unspecified/untranslated/unknown language) keeps names and raw values
- [x] **GO:** translated CSV goldens match (`translated_{default,en,fr}`)

## Phase 3 — Groups & repeats ✅

- [x] 3a — Non-repeat groups: fields flatten into the main table, keeping their
      short name/label as header and reading values by full submission path
      (`grouped_{default,en}`)
- [x] 3b — Each repeat becomes its own table (`Section` model + `export_tables`),
      linked by `_index` / `_parent_table_name` / `_parent_index`; multi-section
      conformance via a framed `.tables` golden (`repeat_tables`)
- [x] **GO:** grouped and repeatable goldens match
- [ ] Deferred: `hierarchy_in_labels` (group-label-prefixed headers) and nested
      repeats beyond one level

## Phase 4 — `select_multiple` ✅

- [x] `both` / `summary` / `details`: a joined summary column (names or labels)
      and/or one `field/choice` boolean column per option; headers and summary
      values honor the language
- [x] **GO:** multiple-select goldens match for all three modes
      (`selmulti_{both,summary,details}`, `selmulti_en_both`)
- [ ] Deferred: `or_other` synthetic option/column

## Phase 5 — Multiple versions ✅

- [x] `merge_versions` builds one column canvas from several versions: the
      newest listed version's fields lead, then each older version appends only
      its fields whose names are not yet present
- [x] `export_csv_versions` facade; submissions fill the columns they carry and
      leave the rest blank, so rows from any version coexist
- [x] **GO:** multi-version goldens match (`multiversion`, `multiversion_reversed`)
- [ ] Deferred: cross-version repeat merging, and per-version field paths that
      differ for the same field name (only the main section is merged)

## Phase 6 — XLSX & header options ✅

- [x] `.xlsx` output (`tableflow-xlsx`, `rust_xlsxwriter`): one worksheet per
      exported table (main + repeats), every cell written as text
      (`xls_types_as_text` default); `tableflow::export_xlsx` returns bytes
- [x] Excel worksheet-name rules: forbidden chars `[]:*?/\` and edge apostrophes
      → `_`, truncate to 31 chars with ellipsis, de-duplicate with ` (n)`
- [x] **GO:** XLSX goldens match cell-by-cell — workbook read back and compared
      to the reference grid (`simple_flat`, `translated_fr`, `repeat_tables`,
      `selmulti_both`, as `.xlsx.json`)
- [ ] Deferred: `xls_types_as_text=false` (native cell types), HXL tag header
      rows, `include_media_url`, copy / filter fields

## Phase 7 — Geo / SPSS / attachments

- [ ] GeoJSON and KML; SPSS value-label files; base64 attachments
- [ ] **GO:** geo / SPSS / attachment goldens match

## Phase 8 — Automatic report

- [ ] Per-field statistics (counts, frequencies, numeric summaries)
- [ ] **GO:** auto-report goldens match
