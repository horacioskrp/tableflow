# Roadmap

`tableflow` is built phase by phase, each gated by a **GO** check: a phase is
done only when its whole slice of the golden corpus matches the reference
exporter (after format-aware normalization) and CI is green.

| Phase | Theme | Status |
| ----- | ----------------------------------------------------- | ------ |
| 0 | Workspace, CI, Docker & conformance harness | 🚧 in progress |
| 1 | Walking skeleton — one version → CSV of a flat form | ☐ |
| 2 | Field type system & translations | ☐ |
| 3 | Groups & repeats → linked tables | ☐ |
| 4 | `select_multiple` expansion | ☐ |
| 5 | Multi-version field canvas | ☐ |
| 6 | XLSX output & header options | ☐ |
| 7 | GeoJSON / KML, SPSS labels, attachments | ☐ |
| 8 | Automatic per-field report | ☐ |

## Phase 0 — Workspace & harness 🚧

- [x] Cargo workspace (edition 2024), pipeline crates, CLI scaffold
- [x] `.gitattributes` (LF), CI (`fmt` / `clippy -D warnings` / `build` / `test`)
- [x] Docker build wrapper (`scripts/docker-dev.ps1`)
- [ ] Conformance harness: a data-driven runner that compares a generated
      export against its golden after format-aware normalization
- [ ] **GO:** green `build` / `test` / `clippy` / `fmt`

## Phase 1 — Walking skeleton

A flat, single-version, single-language form with simple question types exports
to CSV, matching the reference exporter.

- [ ] Load one version's content into the model (sections + fields)
- [ ] Map submissions onto the field canvas; emit CSV rows
- [ ] **GO:** the simplest form's CSV golden matches

## Phase 2 — Field types & translations

- [ ] Per-type value formatting (text/int/decimal/date/time/geo/calculate/…)
- [ ] `lang` selection; render coded choice values as labels
- [ ] **GO:** typed and translated CSV goldens match

## Phase 3 — Groups & repeats

- [ ] Each repeat group becomes its own table, linked by `_index` /
      `_parent_index`; `group_sep` and `hierarchy_in_labels`
- [ ] **GO:** grouped / (nested) repeatable goldens match

## Phase 4 — `select_multiple`

- [ ] `both` / `summary` / `details`: joined cell and/or per-choice booleans;
      `or_other`
- [ ] **GO:** multiple-select goldens match for all three modes

## Phase 5 — Multiple versions

- [ ] Merge fields added / removed / edited across versions into one column
      canvas; version-id handling
- [ ] **GO:** multi-version goldens match

## Phase 6 — XLSX & header options

- [ ] XLSX output; `xls_types_as_text`; media URLs; HXL tag columns; copy /
      filter fields
- [ ] **GO:** XLSX goldens match (cell-by-cell)

## Phase 7 — Geo / SPSS / attachments

- [ ] GeoJSON and KML; SPSS value-label files; base64 attachments
- [ ] **GO:** geo / SPSS / attachment goldens match

## Phase 8 — Automatic report

- [ ] Per-field statistics (counts, frequencies, numeric summaries)
- [ ] **GO:** auto-report goldens match
