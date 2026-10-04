//! Per-field summary report: counts, frequencies and percentages.
//!
//! [`report`] walks the main section's fields and, for each, counts how many
//! submissions answered it (`provided`) or not (`not_provided`) and tallies the
//! answers. Categorical fields (`text`, `select_one`, `select_multiple`,
//! `date`) also carry a `frequency` table and matching `percentage`s:
//!
//! - `text` orders values by descending count (ties keep first-seen order);
//! - `select_one` / `select_multiple` translate each value to its choice label
//!   and order by descending count; a `select_multiple` answer is split into
//!   its chosen options;
//! - `date` orders values chronologically.
//!
//! `select_*` and `date` set `show_graph`; other fields do not. Numeric fields
//! (`integer` / `decimal`) instead carry `mean` / `median` / `mode` / `stdev`
//! (the sum of squared deviations is accumulated in exact rational arithmetic
//! and its square root correctly rounded, matching the reference exporter
//! bit-for-bit); each is `"*"` when undefined (empty data, a lone value, or a
//! non-unique mode).

use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{One, ToPrimitive, Zero};
use serde::Serialize;
use serde_json::{Value, json};
use tableflow_core::{Field, Version};

/// A whole report: how many submissions, and one entry per reported field.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Report {
    /// Number of submissions summarized.
    pub submissions_count: u64,
    /// One entry per field, in survey order (notes excluded).
    pub fields: Vec<FieldReport>,
}

/// One field's entry: its name, display label and statistics.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FieldReport {
    /// Field name.
    pub name: String,
    /// Display label (field name when untranslated).
    pub label: String,
    /// The field's statistics.
    pub stats: Stats,
}

/// A field's statistics. `frequency` / `percentage` are present only for
/// categorical fields.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Stats {
    /// All submissions summarized (`provided + not_provided`).
    pub total_count: u64,
    /// Submissions that left the field blank (missing or null).
    pub not_provided: u64,
    /// Submissions that answered the field.
    pub provided: u64,
    /// Whether a chart is suggested for this field.
    pub show_graph: bool,
    /// Value → count, ordered per the field type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency: Option<Vec<(String, u64)>>,
    /// Value → percentage of `total_count`, aligned with `frequency`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub percentage: Option<Vec<(String, f64)>>,
    /// Median of the answers (numeric fields); `"*"` when undefined.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub median: Option<Value>,
    /// Arithmetic mean (numeric fields); `"*"` when undefined.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mean: Option<Value>,
    /// Single mode (numeric fields); `"*"` when absent or not unique.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<Value>,
    /// Sample standard deviation (numeric fields); `"*"` when undefined.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stdev: Option<Value>,
}

/// Build a [`Report`] over the main section's submissions, labels in `lang`.
#[must_use]
pub fn report(version: &Version, submissions: &[Value], lang: Option<&str>) -> Report {
    let index = tableflow_schema::lang_index(version, lang);
    let main = &version.sections[0];

    let fields = main
        .fields
        .iter()
        .filter(|f| has_stats(f))
        .map(|field| FieldReport {
            name: field.name.clone(),
            label: tableflow_schema::header(field, index),
            stats: field_stats(version, field, submissions, index),
        })
        .collect();

    Report {
        submissions_count: submissions.len() as u64,
        fields,
    }
}

/// Whether a field is summarized at all (everything but notes).
fn has_stats(field: &Field) -> bool {
    field.kind != "note"
}

/// Whether a field carries a frequency table (vs. counts only).
fn is_categorical(field: &Field) -> bool {
    matches!(
        field.kind.as_str(),
        "text" | "select_one" | "select_multiple"
    ) || field.kind == "date"
}

/// Compute one field's statistics across `submissions`.
fn field_stats(
    version: &Version,
    field: &Field,
    submissions: &[Value],
    index: Option<usize>,
) -> Stats {
    if field.kind == "integer" || field.kind == "decimal" {
        return numeric_stats(field, submissions);
    }

    // Tally answers in first-seen order.
    let mut order: Vec<String> = Vec::new();
    let mut counts: Vec<u64> = Vec::new();
    let mut provided = 0;
    let mut not_provided = 0;

    let mut tally = |value: &str| match order.iter().position(|v| v == value) {
        Some(i) => counts[i] += 1,
        None => {
            order.push(value.to_owned());
            counts.push(1);
        }
    };

    for submission in submissions {
        match submission.get(&field.path) {
            None | Some(Value::Null) => not_provided += 1,
            Some(value) => {
                provided += 1;
                let raw = scalar(value);
                if field.kind == "select_multiple" {
                    for choice in raw.split_whitespace() {
                        tally(choice);
                    }
                } else {
                    tally(&raw);
                }
            }
        }
    }

    let total_count = provided + not_provided;
    if !is_categorical(field) {
        return Stats {
            total_count,
            not_provided,
            provided,
            show_graph: false,
            frequency: None,
            percentage: None,
            median: None,
            mean: None,
            mode: None,
            stdev: None,
        };
    }

    let mut pairs: Vec<(String, u64)> = order.into_iter().zip(counts).collect();
    if field.kind == "date" {
        pairs.sort_by(|a, b| a.0.cmp(&b.0));
    } else {
        pairs.sort_by_key(|a| std::cmp::Reverse(a.1));
    }

    let is_select = field.kind == "select_one" || field.kind == "select_multiple";
    let frequency: Vec<(String, u64)> = pairs
        .into_iter()
        .map(|(value, count)| {
            let display = if is_select {
                translate(version, field, &value, index)
            } else {
                value
            };
            (display, count)
        })
        .collect();

    let percentage = frequency
        .iter()
        .map(|(value, count)| (value.clone(), percent(*count, total_count)))
        .collect();

    Stats {
        total_count,
        not_provided,
        provided,
        show_graph: field.kind != "text",
        frequency: Some(frequency),
        percentage: Some(percentage),
        median: None,
        mean: None,
        mode: None,
        stdev: None,
    }
}

/// Statistics for a numeric (`integer` / `decimal`) field: counts plus
/// median / mean / mode / stdev. Values that do not parse as the field's number
/// type count as blank. Mirrors the reference exporter's sequential evaluation:
/// an empty dataset leaves every summary `"*"`, a single value leaves
/// `stdev`/`mode` `"*"`, and a non-unique mode leaves `mode` `"*"`.
fn numeric_stats(field: &Field, submissions: &[Value]) -> Stats {
    let integer = field.kind == "integer";
    let mut ints: Vec<i64> = Vec::new();
    let mut floats: Vec<f64> = Vec::new();
    let mut provided = 0;
    let mut not_provided = 0;

    for submission in submissions {
        match submission.get(&field.path) {
            None | Some(Value::Null) => not_provided += 1,
            Some(value) => match parse_number(value, integer) {
                Some(number) => {
                    provided += 1;
                    match number {
                        Number::Int(i) => ints.push(i),
                        Number::Float(f) => floats.push(f),
                    }
                }
                None => not_provided += 1,
            },
        }
    }

    let star = || Value::String("*".to_owned());
    let mut stats = Stats {
        total_count: provided + not_provided,
        not_provided,
        provided,
        show_graph: false,
        frequency: None,
        percentage: None,
        median: Some(star()),
        mean: Some(star()),
        mode: Some(star()),
        stdev: Some(star()),
    };

    // Work on the dataset as f64 (for mean/stdev) while keeping integer values
    // for integer-typed median/mode.
    let data: Vec<f64> = if integer {
        ints.iter().map(|&i| i as f64).collect()
    } else {
        floats.clone()
    };
    let n = data.len();
    if n == 0 {
        return stats;
    }

    let mean = data.iter().sum::<f64>() / n as f64;
    stats.mean = Some(if integer {
        let sum: i128 = ints.iter().map(|&i| i128::from(i)).sum();
        if sum % n as i128 == 0 {
            json!(i64::try_from(sum / n as i128).unwrap_or_default())
        } else {
            json!(mean)
        }
    } else {
        json!(mean)
    });

    stats.median = Some(if integer {
        median_int(&mut ints)
    } else {
        median_float(&mut floats)
    });

    if n < 2 {
        return stats;
    }

    stats.stdev = Some(json!(sample_stdev(&data, mean)));

    stats.mode = Some(if integer {
        unique_mode(&ints).map_or_else(star, |m| json!(m))
    } else {
        unique_mode(&floats).map_or_else(star, |m| json!(m))
    });

    stats
}

/// A parsed numeric value, keeping integers exact for display.
enum Number {
    Int(i64),
    Float(f64),
}

/// Parse `value` as the field's number type, or `None` if it does not parse.
fn parse_number(value: &Value, integer: bool) -> Option<Number> {
    if integer {
        if let Some(i) = value.as_i64() {
            return Some(Number::Int(i));
        }
        if let Some(f) = value.as_f64() {
            return Some(Number::Int(f as i64));
        }
        return value.as_str()?.trim().parse::<i64>().ok().map(Number::Int);
    }
    let number = match value {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => s.trim().parse::<f64>().ok(),
        _ => None,
    }?;
    number.is_finite().then_some(Number::Float(number))
}

/// Median of integer data: the middle value (odd) or the mean of the two middle
/// values (even, always a float — matching the reference).
fn median_int(data: &mut [i64]) -> Value {
    data.sort_unstable();
    let n = data.len();
    if n % 2 == 1 {
        json!(data[n / 2])
    } else {
        json!((data[n / 2 - 1] + data[n / 2]) as f64 / 2.0)
    }
}

/// Median of float data.
fn median_float(data: &mut [f64]) -> Value {
    data.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = data.len();
    if n % 2 == 1 {
        json!(data[n / 2])
    } else {
        json!((data[n / 2 - 1] + data[n / 2]) / 2.0)
    }
}

/// Sample standard deviation of `data`, computed like the reference exporter:
/// the sum of squared deviations from `mean` is accumulated in exact rational
/// arithmetic, then its square root is correctly rounded to `f64`.
fn sample_stdev(data: &[f64], mean: f64) -> f64 {
    let xbar = ratio_of_f64(mean);
    let mut ss = BigRational::zero();
    for &x in data {
        let deviation = ratio_of_f64(x) - &xbar;
        ss += &deviation * &deviation;
    }
    let variance = ss / BigRational::from_integer(BigInt::from(data.len() - 1));
    float_sqrt_of_frac(variance.numer(), variance.denom())
}

/// The exact value of a finite `f64` as a rational.
fn ratio_of_f64(x: f64) -> BigRational {
    let bits = x.to_bits();
    let sign = if bits >> 63 == 1 { -1 } else { 1 };
    let raw_exponent = ((bits >> 52) & 0x7ff) as i64;
    let raw_mantissa = bits & 0x000f_ffff_ffff_ffff;
    let (mantissa, exponent) = if raw_exponent == 0 {
        (raw_mantissa, -1074)
    } else {
        (raw_mantissa | 0x0010_0000_0000_0000, raw_exponent - 1075)
    };
    let numerator = BigInt::from(mantissa) * sign;
    if exponent >= 0 {
        BigRational::from_integer(numerator << (exponent as usize))
    } else {
        BigRational::new(numerator, BigInt::one() << ((-exponent) as usize))
    }
}

/// Correctly-rounded `f64` square root of `n / m` (with `n >= 0`, `m > 0`), a
/// port of CPython's `statistics._float_sqrt_of_frac`.
fn float_sqrt_of_frac(n: &BigInt, m: &BigInt) -> f64 {
    if n.is_zero() {
        return 0.0;
    }
    const SQRT_BIT_WIDTH: i64 = 109;
    let q = (n.bits() as i64 - m.bits() as i64 - SQRT_BIT_WIDTH).div_euclid(2);
    if q >= 0 {
        let scaled = m << (2 * q) as usize;
        let numerator = integer_sqrt_of_frac_rto(n, &scaled) << (q as usize);
        numerator.to_f64().unwrap_or(f64::INFINITY)
    } else {
        let numerator = integer_sqrt_of_frac_rto(&(n << (-2 * q) as usize), m);
        let denominator = BigInt::one() << ((-q) as usize);
        numerator.to_f64().unwrap_or(f64::INFINITY) / denominator.to_f64().unwrap_or(f64::INFINITY)
    }
}

/// Integer square root of `n / m`, rounded to odd (so a later division rounds
/// correctly).
fn integer_sqrt_of_frac_rto(n: &BigInt, m: &BigInt) -> BigInt {
    let a = (n / m).sqrt();
    if &(&a * &a) * m != *n {
        a | BigInt::one()
    } else {
        a
    }
}

/// The single most frequent value (first-seen order on ties), or `None` when
/// more than one value shares the top frequency.
fn unique_mode<T: PartialEq + Copy>(values: &[T]) -> Option<T> {
    let mut order: Vec<T> = Vec::new();
    let mut counts: Vec<u64> = Vec::new();
    for &v in values {
        match order.iter().position(|x| *x == v) {
            Some(i) => counts[i] += 1,
            None => {
                order.push(v);
                counts.push(1);
            }
        }
    }
    let max = *counts.iter().max()?;
    let mut modes = order.iter().zip(&counts).filter(|(_, c)| **c == max);
    let first = *modes.next()?.0;
    match modes.next() {
        Some(_) => None,
        None => Some(first),
    }
}

/// Translate a choice value to its label in `index`, or keep it as-is.
fn translate(version: &Version, field: &Field, value: &str, index: Option<usize>) -> String {
    let list = field.list_name.as_deref();
    let label = list
        .zip(index)
        .and_then(|(list, i)| version.choice_label(list, value, i));
    label.unwrap_or(value).to_owned()
}

/// `value` as a percentage of `total`, rounded to two decimals (half to even).
fn percent(value: u64, total: u64) -> f64 {
    if total == 0 {
        return 0.0;
    }
    let raw = (value as f64) * 100.0 / (total as f64);
    (raw * 100.0).round_ties_even() / 100.0
}

/// A submission value as a plain string (empty for null/absent/compound).
fn scalar(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{percent, report};
    use tableflow_core::parse_version;

    #[test]
    fn percentages_round_to_two_decimals() {
        assert_eq!(percent(3, 4), 75.0);
        assert_eq!(percent(1, 3), 33.33);
        assert_eq!(percent(0, 0), 0.0);
    }

    #[test]
    fn text_frequency_orders_by_descending_count() {
        let version = parse_version(&json!({ "content": { "survey": [
            { "type": "text", "name": "city" },
        ] } }));
        let subs = [
            json!({ "city": "Kara" }),
            json!({ "city": "Lome" }),
            json!({ "city": "Lome" }),
            json!({}),
        ];
        let stats = &report(&version, &subs, None).fields[0].stats;
        assert_eq!(stats.provided, 3);
        assert_eq!(stats.not_provided, 1);
        assert_eq!(
            stats.frequency.as_deref(),
            Some(&[("Lome".to_owned(), 2), ("Kara".to_owned(), 1)][..])
        );
    }
}
