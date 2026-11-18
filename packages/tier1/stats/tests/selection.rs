//! Order statistics of the pairwise differences and of the Walsh averages without the pairs
//! being stored (release 0.8.0, step 20d): on small samples every one of them is, bit for bit,
//! the one of the pairs sorted and stored; and the statistic by one merge is the statistic by
//! ranking.

use zeff_numeric::association::ranks;
use zeff_numeric::order::median;
use zeff_stats::selection::{Pairs, rank_sum_at, signed_rank_at};

fn property_cases(usual: u32) -> u32 {
    std::env::var("ZEFF_PROPERTY_MULTIPLIER")
        .ok()
        .and_then(|m| m.parse::<u32>().ok())
        .map_or(usual, |m| usual.saturating_mul(m))
}

/// A sample of `n` values of one of several kinds: spread over a range, on a coarse grid (so
/// that many are tied), with zeros of both signs, very large, or very small.
fn sample(stream: &mut zeff_numeric::random::Stream, n: usize, kind: usize) -> Vec<f64> {
    (0..n)
        .map(|_| {
            let u = stream.uniform();
            match kind {
                0 => 200.0 * u - 100.0,
                1 => (10.0 * u).floor() - 5.0,
                2 => {
                    if u < 0.3 {
                        0.0
                    } else if u < 0.5 {
                        -0.0
                    } else {
                        (4.0 * u).floor() - 2.0
                    }
                }
                3 => (2.0 * u - 1.0) * 1e300,
                _ => (2.0 * u - 1.0) * 1e-310,
            }
        })
        .collect()
}

fn sorted(mut values: Vec<f64>) -> Vec<f64> {
    values.sort_by(f64::total_cmp);
    values
}

fn tie_sum(r: &[f64]) -> f64 {
    let sorted = sorted(r.to_vec());
    let mut total = 0.0;
    let mut i = 0;
    while i < sorted.len() {
        let mut j = i;
        while j + 1 < sorted.len() && sorted[j + 1] == sorted[i] {
            j += 1;
        }
        let t = (j - i + 1) as f64;
        total += t * t * t - t;
        i = j + 1;
    }
    total
}

#[test]
fn a_selected_order_statistic_is_the_one_of_the_sorted_pairs() {
    let cases = property_cases(60);
    for case in 0..cases {
        let mut stream = zeff_numeric::random::Stream::new(4100 + u64::from(case), 1, 0);
        let kind = case as usize % 5;
        let m = 1 + (stream.uniform() * 30.0) as usize;
        let n = 1 + (stream.uniform() * 30.0) as usize;
        let xs = sorted(sample(&mut stream, m, kind));
        let ys = sorted(sample(&mut stream, n, (kind + case as usize / 5) % 5));
        // The differences.
        let mut stored: Vec<f64> = Vec::new();
        for x in &xs {
            for y in &ys {
                stored.push(x - y);
            }
        }
        let stored = sorted(stored);
        let pairs = Pairs::Differences(&xs, &ys);
        assert_eq!(pairs.total(), stored.len() as u64);
        for (index, value) in stored.iter().enumerate() {
            let selected = pairs.select(index as u64 + 1).unwrap();
            assert_eq!(
                selected.to_bits(),
                value.to_bits(),
                "case {case}: difference {} of {}: {selected:e} against {value:e}",
                index + 1,
                stored.len()
            );
        }
        assert_eq!(
            pairs.median().unwrap().to_bits(),
            median(&stored).unwrap().to_bits(),
            "case {case}"
        );
        assert_eq!(pairs.select(0), None);
        assert_eq!(pairs.select(stored.len() as u64 + 1), None);
        // How many are at or below a value, at every value and between them.
        for value in stored.iter().step_by(7) {
            assert_eq!(
                pairs.at_most(*value),
                stored.iter().filter(|d| **d <= *value).count() as u64,
                "case {case}"
            );
        }
        // The Walsh averages.
        let mut stored: Vec<f64> = Vec::new();
        for i in 0..xs.len() {
            for j in i..xs.len() {
                stored.push((xs[i] + xs[j]) / 2.0);
            }
        }
        let stored = sorted(stored);
        let pairs = Pairs::Walsh(&xs);
        assert_eq!(pairs.total(), stored.len() as u64);
        for (index, value) in stored.iter().enumerate() {
            let selected = pairs.select(index as u64 + 1).unwrap();
            assert_eq!(
                selected.to_bits(),
                value.to_bits(),
                "case {case}: Walsh average {} of {}: {selected:e} against {value:e}",
                index + 1,
                stored.len()
            );
        }
        assert_eq!(
            pairs.median().unwrap().to_bits(),
            median(&stored).unwrap().to_bits(),
            "case {case}"
        );
    }
    // No pair: nothing to select.
    assert_eq!(Pairs::Differences(&[], &[1.0]).median(), None);
    assert_eq!(Pairs::Walsh(&[]).median(), None);
    assert_eq!(Pairs::Walsh(&[3.5]).median(), Some(3.5));
}

#[test]
fn the_statistic_by_one_merge_is_the_statistic_by_ranking() {
    let cases = property_cases(60);
    for case in 0..cases {
        let mut stream = zeff_numeric::random::Stream::new(5200 + u64::from(case), 1, 0);
        let kind = case as usize % 3;
        let m = 1 + (stream.uniform() * 40.0) as usize;
        let n = 1 + (stream.uniform() * 40.0) as usize;
        let x = sample(&mut stream, m, kind);
        let y = sample(&mut stream, n, kind);
        let (xs, ys) = (sorted(x.clone()), sorted(y.clone()));
        // The shifts the interval's search tries: values of the data, their differences, and
        // values between.
        let shifts = [0.0, x[0] - y[0], x[0], -1.5, 0.25, (x[0] - y[0]) / 2.0];
        for d in shifts {
            // The rank sum: the pooled sample ranked, in its own order.
            let pooled: Vec<f64> = x.iter().map(|v| v - d).chain(y.iter().copied()).collect();
            let r = ranks(&pooled);
            let by_ranking = (r[..m].iter().sum::<f64>(), tie_sum(&r));
            let merged = rank_sum_at(&xs, &ys, d);
            assert_eq!(
                (merged.0.to_bits(), merged.1.to_bits()),
                (by_ranking.0.to_bits(), by_ranking.1.to_bits()),
                "case {case}, shift {d}: {merged:?} against {by_ranking:?}"
            );
            // The signed ranks: the values that are not zero, their sizes ranked.
            let kept: Vec<f64> = x.iter().map(|v| v - d).filter(|v| *v != 0.0).collect();
            let r = ranks(&kept.iter().map(|v| v.abs()).collect::<Vec<_>>());
            let above: f64 = r
                .iter()
                .zip(&kept)
                .filter(|(_, v)| **v > 0.0)
                .map(|(rank, _)| *rank)
                .sum();
            let merged = signed_rank_at(&xs, d);
            assert_eq!(merged.0, kept.len() as u64, "case {case}, shift {d}");
            // A sum of no rank is written -0 by the one and 0 by the other, and is the same
            // zero in the statistic, which subtracts from it.
            assert_eq!(
                (merged.1.to_bits(), merged.2.to_bits()),
                ((above + 0.0).to_bits(), tie_sum(&r).to_bits()),
                "case {case}, shift {d}"
            );
        }
    }
}
