# tableflow-cli

Command-line interface for [`tableflow`](https://crates.io/crates/tableflow):
read a form version and its submissions (both JSON), write a tabular export.

```sh
cargo install tableflow-cli
```

The installed binary is named `tableflow`.

## Usage

```text
tableflow [OPTIONS] <SCHEMA> <SUBMISSIONS>
```

- `<SCHEMA>` — the form version schema, `{ "content": { "survey": [...], ... } }`.
- `<SUBMISSIONS>` — a JSON array of submission objects.

```sh
# Per-field report
tableflow --format report survey.json submissions.json

# XLSX workbook to a file
tableflow --format xlsx -o out.xlsx survey.json submissions.json

# French labels, select_multiple as a summary column only
tableflow --lang Français --multiple-select summary survey.json submissions.json
```

## Options

| Flag | Meaning |
| --- | --- |
| `--format csv\|tables\|xlsx\|geojson\|report` | output format (default `csv`) |
| `--lang <LANG>` | language for labels (field names when omitted) |
| `--multiple-select both\|summary\|details` | `select_multiple` expansion (default `both`) |
| `--title <TITLE>` | main table / collection name (default `submissions`) |
| `--group-sep <SEP>` | hierarchy / expansion separator (default `/`) |
| `--hierarchy` | prefix headers with enclosing group labels |
| `--copy-field <NAME>` | append a submission key as a trailing column (repeatable) |
| `--tag-col <NAME>` | emit a tag header row, e.g. `hxl` (repeatable) |
| `--filter-field <NAME>` | keep only these fields (repeatable) |
| `--media-url` | add a `<name>_URL` column per media field |
| `--force-index` | add an `_index` column to every section |
| `--split-by <FIELD>` | disaggregate the report by a field (report format) |
| `-o, --output <PATH>` | write to a file instead of stdout |

Output goes to stdout unless `-o` is given (use `-o` for `xlsx`).

## License

Licensed under either of Apache-2.0 or MIT at your option.
