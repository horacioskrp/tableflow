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

use std::collections::HashMap;

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
    /// Disaggregated breakdown (when a `split_by` field is given): each answer
    /// paired with its `{frequency, percentage}` across the splitter values.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub values: Option<Vec<(String, Value)>>,
}

/// A `Stats` with every optional field empty.
impl Stats {
    fn counts(total_count: u64, not_provided: u64, provided: u64, show_graph: bool) -> Self {
        Stats {
            total_count,
            not_provided,
            provided,
            show_graph,
            frequency: None,
            percentage: None,
            median: None,
            mean: None,
            mode: None,
            stdev: None,
            values: None,
        }
    }
}

/// Build a [`Report`] over the main section's submissions, labels in `lang`.
///
/// When `split_by` names a main-section field, each other field is disaggregated
/// by that field's values (the split field itself is omitted).
#[must_use]
pub fn report(
    version: &Version,
    submissions: &[Value],
    lang: Option<&str>,
    split_by: Option<&str>,
) -> Report {
    let index = tableflow_schema::lang_index(version, lang);
    let main = &version.sections[0];
    let split = split_by.and_then(|name| main.fields.iter().find(|f| f.name == name));

    let fields = main
        .fields
        .iter()
        .filter(|f| has_stats(f))
        .filter(|f| split.is_none_or(|s| s.name != f.name))
        .map(|field| FieldReport {
            name: field.name.clone(),
            label: tableflow_schema::header(field, index),
            stats: match split {
                Some(split) => disaggregated_stats(version, field, split, submissions, index),
                None => field_stats(version, field, submissions, index),
            },
        })
        .collect();

    Report {
        submissions_count: submissions.len() as u64,
        fields,
    }
}

/// The tokens a field contributes for one submission (`None` when unanswered);
/// a `select_multiple` yields one per chosen option.
fn field_tokens(field: &Field, submission: &Value) -> Option<Vec<String>> {
    match submission.get(&field.path) {
        None | Some(Value::Null) => None,
        Some(value) => {
            let raw = scalar(value);
            Some(if field.kind == "select_multiple" {
                raw.split_whitespace().map(str::to_owned).collect()
            } else {
                vec![raw]
            })
        }
    }
}

/// A sentinel splitter key for submissions missing the split value.
const MISSING_SPLITTER: &str = "\u{0}none";

/// Counts string values while preserving first-seen order, in O(1) per update
/// (a hash index over an insertion-ordered list). Keeps the frequency tallies
/// linear rather than quadratic on high-cardinality fields.
#[derive(Default)]
struct Counter {
    index: HashMap<String, usize>,
    order: Vec<String>,
    counts: Vec<u64>,
}

impl Counter {
    fn add(&mut self, value: &str) {
        if let Some(&i) = self.index.get(value) {
            self.counts[i] += 1;
        } else {
            let i = self.order.len();
            self.index.insert(value.to_owned(), i);
            self.order.push(value.to_owned());
            self.counts.push(1);
        }
    }

    /// `(value, count)` pairs in first-seen order.
    fn into_pairs(self) -> Vec<(String, u64)> {
        self.order.into_iter().zip(self.counts).collect()
    }
}

/// The split field's values, most common first (capped at 5), and whether an
/// ellipsis bucket is needed (there were five or more distinct values).
fn top_splitters(split: &Field, submissions: &[Value]) -> (Vec<String>, bool) {
    let mut counter = Counter::default();
    for submission in submissions {
        let Some(value) = submission.get(&split.path) else {
            continue;
        };
        if value.is_null() {
            continue;
        }
        counter.add(&scalar(value));
    }
    let mut pairs = counter.into_pairs();
    pairs.sort_by_key(|s| std::cmp::Reverse(s.1));
    let top: Vec<String> = pairs.into_iter().take(5).map(|(s, _)| s).collect();
    let add_ellipsis = top.len() == 5;
    (top, add_ellipsis)
}

/// The split value of a submission, or the missing-splitter sentinel.
fn splitter_key(split: &Field, submission: &Value) -> String {
    match submission.get(&split.path) {
        Some(value) if !value.is_null() => scalar(value),
        _ => MISSING_SPLITTER.to_owned(),
    }
}

/// Disaggregated statistics for a numeric field, broken down by `split`: each
/// splitter paired with its `{median, mean, mode, stdev}`.
fn disaggregated_numeric(
    version: &Version,
    field: &Field,
    split: &Field,
    submissions: &[Value],
    index: Option<usize>,
) -> Stats {
    let integer = field.kind == "integer";
    let mut provided = 0;
    let mut not_provided = 0;
    let mut key_order: Vec<String> = Vec::new();
    let mut key_ints: Vec<Vec<i64>> = Vec::new();
    let mut key_floats: Vec<Vec<f64>> = Vec::new();

    for submission in submissions {
        match submission
            .get(&field.path)
            .and_then(|v| parse_number(v, integer))
        {
            Some(number) => {
                provided += 1;
                let key = splitter_key(split, submission);
                let i = match key_order.iter().position(|k| k == &key) {
                    Some(i) => i,
                    None => {
                        key_order.push(key);
                        key_ints.push(Vec::new());
                        key_floats.push(Vec::new());
                        key_order.len() - 1
                    }
                };
                match number {
                    Number::Int(n) => key_ints[i].push(n),
                    Number::Float(n) => key_floats[i].push(n),
                }
            }
            None => not_provided += 1,
        }
    }

    let (top, add_ellipsis) = top_splitters(split, submissions);
    let summary = |ints: Vec<i64>, floats: Vec<f64>| {
        let (median, mean, mode, stdev) = numeric_summary(ints, floats, integer);
        json!({ "median": median, "mean": mean, "mode": mode, "stdev": stdev })
    };

    let mut values: Vec<(String, Value)> = Vec::new();
    for splitter in &top {
        let (ints, floats) = key_order
            .iter()
            .position(|k| k == splitter)
            .map(|i| (key_ints[i].clone(), key_floats[i].clone()))
            .unwrap_or_default();
        values.push((
            translate(version, split, splitter, index),
            summary(ints, floats),
        ));
    }
    if add_ellipsis {
        let mut ints = Vec::new();
        let mut floats = Vec::new();
        for (i, key) in key_order.iter().enumerate() {
            if !top.contains(key) {
                ints.extend(&key_ints[i]);
                floats.extend(&key_floats[i]);
            }
        }
        values.push(("...".to_owned(), summary(ints, floats)));
    }

    let mut stats = Stats::counts(provided + not_provided, not_provided, provided, false);
    stats.values = Some(values);
    stats
}

/// Disaggregated statistics for `field`, broken down by the values of `split`.
fn disaggregated_stats(
    version: &Version,
    field: &Field,
    split: &Field,
    submissions: &[Value],
    index: Option<usize>,
) -> Stats {
    let class = classify(&field.kind);
    if class == Class::Numeric {
        return disaggregated_numeric(version, field, split, submissions, index);
    }

    let mut provided = 0;
    let mut not_provided = 0;
    let mut value_index: HashMap<String, usize> = HashMap::new();
    let mut value_order: Vec<String> = Vec::new();
    let mut value_metrics: Vec<HashMap<String, u64>> = Vec::new();

    for submission in submissions {
        let key = splitter_key(split, submission);
        match field_tokens(field, submission) {
            None => not_provided += 1,
            Some(tokens) => {
                provided += 1;
                for token in tokens {
                    let i = *value_index.entry(token.clone()).or_insert_with(|| {
                        value_order.push(token);
                        value_metrics.push(HashMap::new());
                        value_order.len() - 1
                    });
                    *value_metrics[i].entry(key.clone()).or_insert(0) += 1;
                }
            }
        }
    }

    let total_count = provided + not_provided;
    if class == Class::Base {
        return Stats::counts(total_count, not_provided, provided, false);
    }

    let (top, add_ellipsis) = top_splitters(split, submissions);
    let is_select = class == Class::Select;

    let mut values: Vec<(String, Value, u64)> = Vec::new();
    for (i, raw_value) in value_order.iter().enumerate() {
        let metrics = &value_metrics[i];
        let mut frequency: Vec<(String, u64)> = Vec::new();
        let mut percentage: Vec<(String, f64)> = Vec::new();
        for splitter in &top {
            let count = *metrics.get(splitter).unwrap_or(&0);
            let label = translate(version, split, splitter, index);
            frequency.push((label.clone(), count));
            percentage.push((label, percent(count, total_count)));
        }
        if add_ellipsis {
            let remaining: u64 = metrics
                .iter()
                .filter(|(key, _)| !top.contains(key))
                .map(|(_, count)| count)
                .sum();
            frequency.push(("...".to_owned(), remaining));
            percentage.push(("...".to_owned(), percent(remaining, total_count)));
        }
        let sum: u64 = frequency.iter().map(|(_, count)| count).sum();
        let display = if is_select {
            translate(version, field, raw_value, index)
        } else {
            raw_value.clone()
        };
        values.push((
            display,
            json!({ "frequency": frequency, "percentage": percentage }),
            sum,
        ));
    }

    if class == Class::Date {
        values.sort_by(|a, b| a.0.cmp(&b.0));
    } else {
        values.sort_by_key(|v| std::cmp::Reverse(v.2));
    }

    let show_graph = matches!(class, Class::Select | Class::Date);
    let mut stats = Stats::counts(total_count, not_provided, provided, show_graph);
    stats.values = Some(values.into_iter().map(|(d, s, _)| (d, s)).collect());
    stats
}

/// How a field is summarized, mirroring the reference's type→class mapping.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Class {
    /// Numeric summaries (mean/median/mode/stdev).
    Numeric,
    /// Choice field: frequency with choice-label values, `show_graph`.
    Select,
    /// `date`: frequency ordered chronologically, `show_graph`.
    Date,
    /// Text-like field: frequency ordered by descending count, no graph.
    Text,
    /// Counts only (geo, `today`/`datetime`/`start`/`end`, unknown types).
    Base,
}

/// Numeric field types.
const NUMERIC_TYPES: [&str; 3] = ["integer", "decimal", "range"];
/// Choice field types.
const SELECT_TYPES: [&str; 3] = ["select_one", "select_one_from_file", "select_multiple"];
/// Text-like field types (frequency, no translation), per the reference.
const TEXT_TYPES: [&str; 15] = [
    "text",
    "barcode",
    "acknowledge",
    "calculate",
    "time",
    "rank",
    "select_multiple_from_file",
    "select_one_external",
    "cascading_select",
    "video",
    "image",
    "audio",
    "file",
    "background-audio",
    "audit",
];

/// Classify a field by its type.
fn classify(kind: &str) -> Class {
    if NUMERIC_TYPES.contains(&kind) {
        Class::Numeric
    } else if SELECT_TYPES.contains(&kind) {
        Class::Select
    } else if kind == "date" {
        Class::Date
    } else if TEXT_TYPES.contains(&kind) {
        Class::Text
    } else {
        Class::Base
    }
}

/// Whether a field is summarized at all: notes and analysis (qual/NLP) fields
/// carry no stats in the reference.
fn has_stats(field: &Field) -> bool {
    let kind = field.kind.as_str();
    kind != "note" && !kind.starts_with("qual") && kind != "transcript" && kind != "translation"
}

/// Compute one field's statistics across `submissions`.
fn field_stats(
    version: &Version,
    field: &Field,
    submissions: &[Value],
    index: Option<usize>,
) -> Stats {
    let class = classify(&field.kind);
    if class == Class::Numeric {
        return numeric_stats(field, submissions);
    }

    // Tally answers in first-seen order.
    let mut counter = Counter::default();
    let mut provided = 0;
    let mut not_provided = 0;

    for submission in submissions {
        match submission.get(&field.path) {
            None | Some(Value::Null) => not_provided += 1,
            Some(value) => {
                provided += 1;
                let raw = scalar(value);
                if field.kind == "select_multiple" {
                    for choice in raw.split_whitespace() {
                        counter.add(choice);
                    }
                } else {
                    counter.add(&raw);
                }
            }
        }
    }

    let total_count = provided + not_provided;
    if class == Class::Base {
        return Stats::counts(total_count, not_provided, provided, false);
    }

    let mut pairs: Vec<(String, u64)> = counter.into_pairs();
    if class == Class::Date {
        pairs.sort_by(|a, b| a.0.cmp(&b.0));
    } else {
        pairs.sort_by_key(|a| std::cmp::Reverse(a.1));
    }

    let frequency: Vec<(String, u64)> = pairs
        .into_iter()
        .map(|(value, count)| {
            let display = if class == Class::Select {
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

    let show_graph = matches!(class, Class::Select | Class::Date);
    let mut stats = Stats::counts(total_count, not_provided, provided, show_graph);
    stats.frequency = Some(frequency);
    stats.percentage = Some(percentage);
    stats
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

    let (median, mean, mode, stdev) = numeric_summary(ints, floats, integer);
    let mut stats = Stats::counts(provided + not_provided, not_provided, provided, false);
    stats.median = Some(median);
    stats.mean = Some(mean);
    stats.mode = Some(mode);
    stats.stdev = Some(stdev);
    stats
}

/// The `(median, mean, mode, stdev)` of a numeric dataset, each `"*"` when
/// undefined. Mirrors the reference's sequential evaluation: empty data leaves
/// all `"*"`; a lone value leaves `stdev`/`mode` `"*"`; a non-unique mode leaves
/// `mode` `"*"`.
fn numeric_summary(
    mut ints: Vec<i64>,
    mut floats: Vec<f64>,
    integer: bool,
) -> (Value, Value, Value, Value) {
    let star = || Value::String("*".to_owned());
    let (mut median, mut mean, mut mode, mut stdev) = (star(), star(), star(), star());

    let data: Vec<f64> = if integer {
        ints.iter().map(|&i| i as f64).collect()
    } else {
        floats.clone()
    };
    let n = data.len();
    if n == 0 {
        return (median, mean, mode, stdev);
    }

    let mean_f = data.iter().sum::<f64>() / n as f64;
    mean = if integer {
        let sum: i128 = ints.iter().map(|&i| i128::from(i)).sum();
        if sum % n as i128 == 0 {
            json!(i64::try_from(sum / n as i128).unwrap_or_default())
        } else {
            json!(mean_f)
        }
    } else {
        json!(mean_f)
    };

    median = if integer {
        median_int(&mut ints)
    } else {
        median_float(&mut floats)
    };

    if n < 2 {
        return (median, mean, mode, stdev);
    }

    stdev = json!(sample_stdev(&data, mean_f));
    mode = if integer {
        unique_mode(&ints).map_or_else(star, |m| json!(m))
    } else {
        unique_mode(&floats).map_or_else(star, |m| json!(m))
    };
    (median, mean, mode, stdev)
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
        let stats = &report(&version, &subs, None, None).fields[0].stats;
        assert_eq!(stats.provided, 3);
        assert_eq!(stats.not_provided, 1);
        assert_eq!(
            stats.frequency.as_deref(),
            Some(&[("Lome".to_owned(), 2), ("Kara".to_owned(), 1)][..])
        );
    }
}
