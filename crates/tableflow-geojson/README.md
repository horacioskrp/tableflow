# tableflow-geojson

GeoJSON serializer for [`tableflow`](https://crates.io/crates/tableflow): turn a
form's geo responses into a `FeatureCollection`.

Most users should depend on the [`tableflow`](https://crates.io/crates/tableflow)
facade (`export_geojson`) instead of this crate directly.

## What it provides

- **`to_geojson(version, submissions, lang, title) -> serde_json::Value`** —
  one `Feature` per answered geo question in the main section.

Geometry mapping (XForm → GeoJSON):

- `geopoint` / `gps` → `Point`
- `geotrace` → `LineString`
- `geoshape` → `Polygon`

XForm orders a coordinate `latitude longitude altitude accuracy`; GeoJSON wants
`[longitude, latitude, altitude]`, so they are swapped and the accuracy dropped.
Polygon rings are closed and wound counter-clockwise (RFC 7946 right-hand rule).
A feature's `properties` carry the other non-geo, non-empty fields (summary
formatted). Submissions with missing or unparseable geo data are skipped.

```rust
use serde_json::json;
use tableflow_core::parse_version;
use tableflow_geojson::to_geojson;

let version = parse_version(&json!({ "content": { "survey": [
    { "type": "geopoint", "name": "loc", "label": "Location" }
] } }));
let rows = [json!({ "loc": "40.7128 -74.0060 10 5" })];
let fc = to_geojson(&version, &rows, None, "submissions");
assert_eq!(fc["features"][0]["geometry"]["type"], "Point");
```

## License

Licensed under either of Apache-2.0 or MIT at your option.
