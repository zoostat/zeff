//! The micro-benchmark of open decision #47: one sum and one dot product over ten million
//! doubles, the scalar Neumaier sum of `zeff-numeric` (the shape shipped at 0.1.0 to 0.3.0)
//! against the lane shape of `zeff-kernels` at 2, 4, 8, 16 and 32 lanes, with a plain
//! uncompensated sum as the floor. Run with `cargo bench -p zeff-kernels --bench reduction`.

use std::time::Duration;

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use zeff_kernels::reduce::Lanes;

/// SplitMix64, so that the data are the same on every machine.
fn data(count: usize, mut state: u64) -> Vec<f64> {
    (0..count)
        .map(|_| {
            state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
            let mut z = state;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            z ^= z >> 31;
            (z >> 11) as f64 / 9_007_199_254_740_992.0 - 0.5
        })
        .collect()
}

fn plain_sum(xs: &[f64]) -> f64 {
    xs.iter().fold(0.0, |acc, &x| acc + x)
}

fn plain_dot(xs: &[f64], ys: &[f64]) -> f64 {
    xs.iter().zip(ys).fold(0.0, |acc, (&x, &y)| acc + x * y)
}

fn neumaier_dot(xs: &[f64], ys: &[f64]) -> f64 {
    let products: Vec<f64> = xs.iter().zip(ys).map(|(&x, &y)| x * y).collect();
    zeff_numeric::sum(&products)
}

fn lanes_sum<const N: usize>(xs: &[f64]) -> f64 {
    let mut lanes = Lanes::<N>::new();
    lanes.push_slice(xs);
    lanes.value()
}

fn lanes_dot<const N: usize>(xs: &[f64], ys: &[f64]) -> f64 {
    let mut lanes = Lanes::<N>::new();
    lanes.push_products(xs, ys);
    lanes.value()
}

fn reductions(c: &mut Criterion) {
    let xs = data(10_000_000, 1);
    let ys = data(10_000_000, 2);

    let mut group = c.benchmark_group("sum of ten million");
    group.bench_function("plain (no compensation)", |b| {
        b.iter(|| plain_sum(black_box(&xs)))
    });
    group.bench_function("neumaier scalar (zeff-numeric)", |b| {
        b.iter(|| zeff_numeric::sum(black_box(&xs)))
    });
    group.bench_function("two-sum 2 lanes", |b| {
        b.iter(|| lanes_sum::<2>(black_box(&xs)))
    });
    group.bench_function("two-sum 4 lanes", |b| {
        b.iter(|| lanes_sum::<4>(black_box(&xs)))
    });
    group.bench_function("two-sum 8 lanes", |b| {
        b.iter(|| lanes_sum::<8>(black_box(&xs)))
    });
    group.bench_function("two-sum 16 lanes", |b| {
        b.iter(|| lanes_sum::<16>(black_box(&xs)))
    });
    group.bench_function("two-sum 32 lanes", |b| {
        b.iter(|| lanes_sum::<32>(black_box(&xs)))
    });
    group.finish();

    let mut group = c.benchmark_group("dot product of ten million");
    group.bench_function("plain (no compensation)", |b| {
        b.iter(|| plain_dot(black_box(&xs), black_box(&ys)))
    });
    group.bench_function("neumaier scalar (products then zeff-numeric sum)", |b| {
        b.iter(|| neumaier_dot(black_box(&xs), black_box(&ys)))
    });
    group.bench_function("two-sum 2 lanes", |b| {
        b.iter(|| lanes_dot::<2>(black_box(&xs), black_box(&ys)))
    });
    group.bench_function("two-sum 4 lanes", |b| {
        b.iter(|| lanes_dot::<4>(black_box(&xs), black_box(&ys)))
    });
    group.bench_function("two-sum 8 lanes", |b| {
        b.iter(|| lanes_dot::<8>(black_box(&xs), black_box(&ys)))
    });
    group.bench_function("two-sum 16 lanes", |b| {
        b.iter(|| lanes_dot::<16>(black_box(&xs), black_box(&ys)))
    });
    group.bench_function("two-sum 32 lanes", |b| {
        b.iter(|| lanes_dot::<32>(black_box(&xs), black_box(&ys)))
    });
    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default().warm_up_time(Duration::from_secs(1)).measurement_time(Duration::from_secs(4)).sample_size(20);
    targets = reductions
}
criterion_main!(benches);
