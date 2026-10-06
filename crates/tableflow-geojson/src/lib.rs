//! GeoJSON serializer: geo responses to a `FeatureCollection`.
//!
//! [`to_geojson`] builds one `Feature` per answered geo question across the
//! main section's submissions (repeats are not included), matching the
//! reference exporter's flattened GeoJSON:
//!
//! - `geopoint` → `Point`, `geotrace` → `LineString`, `geoshape` → `Polygon`;
//! - XForm orders each coordinate `latitude longitude altitude accuracy`;
//!   GeoJSON wants `[longitude, latitude, altitude]`, so they are swapped and
//!   the accuracy is dropped;
//! - a polygon's ring is closed and wound counter-clockwise (the right-hand
//!   rule, per RFC 7946);
//! - a feature's `properties` carry the other (non-geo, non-empty) fields,
//!   formatted as a summary.
//!
//! Submissions with missing or unparseable geo data are skipped.

use serde_json::{Map, Value, json};
use tableflow_core::{Field, Version};
use tableflow_schema::MultipleSelect;

/// XForm geo data types, mapped to GeoJSON geometry types (`gps` is a legacy
/// alias for `geopoint`).
const GEO_TYPES: [&str; 4] = ["geopoint", "gps", "geotrace", "geoshape"];

/// Build a GeoJSON `FeatureCollection` from `submissions`.
///
/// `title` names the collection; `lang` selects the property labels' language
/// (`None` keeps field names).
#[must_use]
pub fn to_geojson(
    version: &Version,
    submissions: &[Value],
    lang: Option<&str>,
    title: &str,
) -> Value {
    let index = tableflow_schema::lang_index(version, lang);
    let main = &version.sections[0];
    let geo_fields: Vec<&Field> = main.fields.iter().filter(|f| is_geo(f)).collect();

    let mut features = Vec::new();
    for submission in submissions {
        for field in &geo_fields {
            let Some(response) = submission.get(&field.path).and_then(Value::as_str) else {
                continue;
            };
            let Some(geometry) = geometry(&field.kind, response) else {
                continue;
            };
            features.push(json!({
                "type": "Feature",
                "geometry": geometry,
                "properties": properties(version, main, submission, index),
            }));
        }
    }

    json!({
        "type": "FeatureCollection",
        "name": title,
        "features": features,
    })
}

/// Whether a field is a geo question.
fn is_geo(field: &Field) -> bool {
    GEO_TYPES.contains(&field.kind.as_str())
}

/// The non-geo, non-empty fields of `section` as summary-formatted properties.
fn properties(
    version: &Version,
    section: &tableflow_core::Section,
    submission: &Value,
    index: Option<usize>,
) -> Map<String, Value> {
    let mut props = Map::new();
    for field in &section.fields {
        if is_geo(field) {
            continue;
        }
        let value = submission.get(&field.path);
        let cells = tableflow_schema::values(version, field, value, index, MultipleSelect::Summary);
        let Some(cell) = cells.into_iter().next() else {
            continue;
        };
        if cell.is_empty() {
            continue;
        }
        props.insert(tableflow_schema::header(field, index), Value::String(cell));
    }
    props
}

/// Build a GeoJSON `geometry` for a geo `kind` and its raw `response`, or
/// `None` if the response is missing points or malformed.
fn geometry(kind: &str, response: &str) -> Option<Value> {
    match kind {
        "geopoint" | "gps" => {
            let point = parse_point(response)?;
            Some(json!({ "type": "Point", "coordinates": point }))
        }
        "geotrace" => {
            let points = parse_points(response)?;
            if points.len() < 2 {
                return None;
            }
            Some(json!({ "type": "LineString", "coordinates": points }))
        }
        "geoshape" => {
            let mut ring = parse_points(response)?;
            if ring.len() < 4 || ring.first() != ring.last() {
                return None;
            }
            wind_counter_clockwise(&mut ring);
            Some(json!({ "type": "Polygon", "coordinates": [ring] }))
        }
        _ => None,
    }
}

/// Parse one `latitude longitude altitude [accuracy]` point into
/// `[longitude, latitude, altitude]`, dropping the accuracy.
fn parse_point(point: &str) -> Option<[f64; 3]> {
    let nums: Vec<f64> = point
        .split_whitespace()
        .map(|c| c.parse::<f64>().ok())
        .collect::<Option<Vec<_>>>()?;
    if nums.len() < 3 {
        return None;
    }
    Some([nums[1], nums[0], nums[2]])
}

/// Parse a `;`-separated list of points, failing if any point is malformed.
fn parse_points(response: &str) -> Option<Vec<[f64; 3]>> {
    response
        .split(';')
        .map(|p| parse_point(p.trim()))
        .collect::<Option<Vec<_>>>()
}

/// Reverse `ring` unless it already winds counter-clockwise (positive signed
/// area over longitude/latitude), per RFC 7946's right-hand rule.
fn wind_counter_clockwise(ring: &mut [[f64; 3]]) {
    let mut area = 0.0;
    for pair in ring.windows(2) {
        let [a, b] = [pair[0], pair[1]];
        area += a[0] * b[1] - b[0] * a[1];
    }
    if area < 0.0 {
        ring.reverse();
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_point, wind_counter_clockwise};

    #[test]
    fn point_swaps_lat_lon_and_drops_accuracy() {
        assert_eq!(
            parse_point("40.7128 -74.0060 10 5"),
            Some([-74.006, 40.7128, 10.0])
        );
    }

    #[test]
    fn clockwise_ring_is_reversed() {
        // A clockwise square (over lon/lat) becomes counter-clockwise.
        let mut ring = [
            [0.0, 0.0, 0.0],
            [0.0, 2.0, 0.0],
            [2.0, 2.0, 0.0],
            [2.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
        ];
        wind_counter_clockwise(&mut ring);
        assert_eq!(
            ring,
            [
                [0.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [2.0, 2.0, 0.0],
                [0.0, 2.0, 0.0],
                [0.0, 0.0, 0.0],
            ]
        );
    }
}
