//! Benchmarks for the hot export paths on a high-cardinality corpus.
//!
//! `cargo bench -p tableflow`. The report benchmarks exercise the frequency
//! tally, which this corpus (mostly distinct `text` values) makes
//! cardinality-sensitive.
#![expect(
    missing_docs,
    reason = "criterion's macros generate undocumented public items"
)]

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use serde_json::{Value, json};
use std::hint::black_box;
use tableflow::Layout;

/// A form with a high-cardinality text field, a select_one and a numeric field.
fn version() -> Value {
    json!({ "content": {
        "survey": [
            { "type": "text", "name": "city", "label": "City" },
            { "type": "select_one", "select_from_list_name": "g", "name": "grp", "label": "Group" },
            { "type": "integer", "name": "age", "label": "Age" }
        ],
        "choices": [
            { "list_name": "g", "name": "a", "label": "A" },
            { "list_name": "g", "name": "b", "label": "B" }
        ]
    } })
}

/// `n` submissions: `city` almost all distinct (worst case for the tally),
/// `grp` a two-value split, `age` a spread of integers.
fn submissions(n: usize) -> Vec<Value> {
    (0..n)
        .map(|i| {
            json!({
                "city": format!("city-{i}"),
                "grp": if i % 2 == 0 { "a" } else { "b" },
                "age": (i % 80) as i64 + 18
            })
        })
        .collect()
}

fn bench_exports(c: &mut Criterion) {
    let version = version();
    let mut group = c.benchmark_group("exports");
    for &n in &[1_000usize, 10_000, 50_000] {
        let subs = submissions(n);
        group.bench_with_input(BenchmarkId::new("csv", n), &n, |bencher, _| {
            bencher.iter(|| {
                black_box(tableflow::export_csv(&version, &subs, &Layout::default()));
            });
        });
        group.bench_with_input(BenchmarkId::new("report", n), &n, |bencher, _| {
            bencher.iter(|| {
                black_box(tableflow::export_report(&version, &subs, None, None));
            });
        });
        group.bench_with_input(BenchmarkId::new("report_split_by", n), &n, |bencher, _| {
            bencher.iter(|| {
                black_box(tableflow::export_report(&version, &subs, None, Some("grp")));
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_exports);
criterion_main!(benches);
