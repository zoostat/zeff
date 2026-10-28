//! The benchmark suite (testing handbook, section 6): every case at every size, its digest
//! checked against the stored one before it is timed. Run with `cargo bench -p zeff-bench
//! --bench suite`; the nightly job runs it and writes the times to its summary.

use std::hint::black_box;
use std::time::Duration;

use criterion::{Criterion, criterion_group, criterion_main};
use zeff_bench::{SIZES, csv, digests, import, mean_by, mismatches, reg, summary_by};

fn suite(c: &mut Criterion) {
    for &rows in &SIZES {
        let bytes = csv(rows);
        let table = import(&bytes);
        let found = digests(&bytes, &table);
        let wrong = mismatches(rows, &found);
        assert!(
            wrong.is_empty(),
            "a result differs from its stored digest:\n{}",
            wrong.join("\n")
        );
        let mut group = c.benchmark_group(format!("{rows} rows"));
        group.sample_size(10);
        group.measurement_time(Duration::from_secs(if rows >= 10_000_000 {
            20
        } else {
            10
        }));
        group.bench_function("import", |b| b.iter(|| import(black_box(&bytes))));
        group.bench_function("summary by group", |b| {
            b.iter(|| summary_by(black_box(&table)))
        });
        group.bench_function("mean by group", |b| b.iter(|| mean_by(black_box(&table))));
        group.bench_function("reg", |b| b.iter(|| reg(black_box(&table))));
        group.finish();
    }
}

criterion_group!(benches, suite);
criterion_main!(benches);
