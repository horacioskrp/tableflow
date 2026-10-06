# tableflow-core

Core data model for [`tableflow`](https://crates.io/crates/tableflow): the
in-memory representation of a parsed form version and the multi-version merge.

Most users should depend on the [`tableflow`](https://crates.io/crates/tableflow)
facade instead of this crate directly.

## What it provides

- **`Field`** — one logical column: `name`, full submission `path`, `kind`
  (XLSForm type), per-translation `labels`, `list_name`, `or_other`, `tags`,
  and `group_path` (enclosing groups, for hierarchical headers).
- **`Section`** — a table: the main section (index 0) plus one per repeat,
  linked by `parent` / `repeat_path` / `has_children`.
- **`Choice` / `ChoiceList`** — choice lists for `select_*` fields.
- **`GroupLabel`** — an enclosing group/repeat (name + labels).
- **`Version`** — a parsed form version: `translations`, `sections`, `choices`.
- **`parse_version(&Value) -> Version`** — parse an XLSForm content schema
  (`content.survey` / `content.choices` / `content.translations`). Non-repeat
  groups flatten into the enclosing section (keeping a full `/`-joined path);
  each `begin_repeat` opens a new linked section.
- **`merge_versions(&[Version]) -> Version`** — fold several versions into one
  export canvas across every section, matched by `repeat_path`: the newest
  listed version's fields lead, then each older version contributes only its
  not-yet-seen fields.

```rust
use serde_json::json;
use tableflow_core::parse_version;

let version = parse_version(&json!({ "content": { "survey": [
    { "type": "text", "name": "a", "label": "A" }
] } }));
assert_eq!(version.sections[0].fields[0].name, "a");
```

## License

Licensed under either of Apache-2.0 or MIT at your option.
