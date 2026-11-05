//! The demeaning kernel (release 0.6.0, step 17) against the exact projection: the model matrix
//! with explicit indicators solved by the kernel's own QR, on generated panels with one, two and
//! three effects, unbalanced, with gaps and singletons, and values to 1e12; and the properties
//! that a column constant within an effect's groups demeans to zero, that the demeaned columns
//! are orthogonal to every indicator, that the row order changes nothing, and that equal
//! weights give the unweighted result bit for bit.

mod common;

use proptest::prelude::*;
use zeff_kernels::Matrix;
use zeff_kernels::demean::{Effect, Options, demean, without_singletons};
use zeff_kernels::qr::least_squares;

fn property_cases(usual: u32) -> u32 {
    std::env::var("ZEFF_PROPERTY_MULTIPLIER")
        .ok()
        .and_then(|m| m.parse::<u32>().ok())
        .map_or(usual, |m| usual.saturating_mul(m))
}

/// A small deterministic generator for the reference panels.
struct SplitMix64(u64);
impl SplitMix64 {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    fn unit(&mut self) -> f64 {
        (self.next() >> 11) as f64 / (1u64 << 53) as f64
    }
    fn below(&mut self, n: usize) -> usize {
        (self.unit() * n as f64) as usize % n
    }
}

/// The indicator columns of the kept rows: every group of the first effect, and every group but
/// the first of each further effect (the projection onto their span).
fn indicators(effects: &[Vec<usize>], rows: &[usize]) -> Matrix {
    let mut columns: Vec<Vec<f64>> = Vec::new();
    for (e, effect) in effects.iter().enumerate() {
        let mut groups: Vec<usize> = rows.iter().map(|&r| effect[r]).collect();
        groups.sort_unstable();
        groups.dedup();
        for (i, &g) in groups.iter().enumerate() {
            if e > 0 && i == 0 {
                continue;
            }
            columns.push(
                rows.iter()
                    .map(|&r| if effect[r] == g { 1.0 } else { 0.0 })
                    .collect(),
            );
        }
    }
    Matrix::from_columns(&columns)
}

/// The residual of `y` on the indicators, through the kernel's QR: the exact projection.
fn projected_out(x: &Matrix, y: &[f64]) -> Vec<f64> {
    let fit = least_squares(x, y, 1e-10, 256);
    (0..y.len())
        .map(|i| {
            let fitted: f64 = (0..x.cols())
                .map(|j| fit.coefficients[j].unwrap_or(0.0) * x.get(i, j))
                .sum();
            y[i] - fitted
        })
        .collect()
}

fn effects_of<'a>(groups: &'a [Vec<usize>], counts: &[usize]) -> Vec<Effect<'a>> {
    groups
        .iter()
        .zip(counts)
        .map(|(g, &n)| Effect {
            group_of_row: g,
            groups: n,
        })
        .collect()
}

#[test]
fn the_kernel_agrees_with_the_exact_projection_on_generated_panels() {
    let mut rng = SplitMix64(0x0600_0017_5EED_0001);
    // Panels of one, two and three effects: units over periods with gaps, a third effect of a
    // few regions; some units seen once (singletons); values plain and to 1e12.
    for (units, periods, regions, scale) in [
        (12usize, 1usize, 1usize, 1.0),
        (15, 6, 1, 1.0),
        (15, 6, 1, 1e12),
        (20, 8, 3, 1.0),
        (40, 5, 4, 1e12),
    ] {
        let mut unit_of = Vec::new();
        let mut period_of = Vec::new();
        let mut region_of = Vec::new();
        for u in 0..units {
            let stay = if u % 7 == 0 {
                1
            } else {
                2 + rng.below(periods.max(1))
            };
            let start = rng.below(periods.max(1));
            for t in 0..stay {
                unit_of.push(u);
                period_of.push((start + t) % periods.max(1));
                region_of.push(u % regions.max(1));
            }
        }
        let n = unit_of.len();
        let y: Vec<f64> = (0..n).map(|_| (rng.unit() - 0.5) * scale).collect();
        let x: Vec<f64> = (0..n)
            .map(|i| (rng.unit() - 0.5) * scale + unit_of[i] as f64)
            .collect();
        let mut groups = vec![unit_of.clone()];
        let mut counts = vec![units];
        if periods > 1 {
            groups.push(period_of.clone());
            counts.push(periods);
        }
        if regions > 1 {
            groups.push(region_of.clone());
            counts.push(regions);
        }
        let effects = effects_of(&groups, &counts);
        // Tight: the columns themselves within 1e-10 of the exact projection, and below the
        // slopes at the engine's tolerance of 1e-8.
        let tight = Options {
            tolerance: 1e-13,
            max_passes: 100_000,
        };
        let out = demean(&[&y, &x], &effects, None, tight);
        assert!(out.converged, "{units} units: {} passes", out.passes);
        let (kept, dropped) = without_singletons(&effects, n);
        assert_eq!(out.singletons_dropped, dropped);
        assert_eq!(out.rows.len(), kept.len());
        if kept.is_empty() {
            continue;
        }
        // The exact projection on the kept rows, in the kernel's order.
        let x_ind = indicators(&groups, &out.rows);
        for (c, column) in [&y, &x].iter().enumerate() {
            let gathered: Vec<f64> = out.rows.iter().map(|&r| column[r]).collect();
            let exact = projected_out(&x_ind, &gathered);
            let scale_c = gathered.iter().fold(0.0f64, |m, v| m.max(v.abs()));
            for (i, (a, b)) in out.columns[c].iter().zip(&exact).enumerate() {
                assert!(
                    (a - b).abs() <= 1e-10 * scale_c,
                    "{units} units, column {c}, row {i}: {a} against {b}"
                );
            }
        }
        // The estimates from the demeaned columns and from the explicit indicators agree.
        let x_full = Matrix::from_columns(
            &std::iter::once(out.rows.iter().map(|&r| x[r]).collect::<Vec<_>>())
                .chain((0..x_ind.cols()).map(|j| x_ind.column(j).to_vec()))
                .collect::<Vec<_>>(),
        );
        let y_full: Vec<f64> = out.rows.iter().map(|&r| y[r]).collect();
        let full = least_squares(&x_full, &y_full, 1e-10, 256);
        let within = least_squares(
            &Matrix::from_columns(&[out.columns[1].clone()]),
            &out.columns[0],
            1e-10,
            256,
        );
        let (b_full, b_within) = (
            full.coefficients[0].unwrap(),
            within.coefficients[0].unwrap(),
        );
        assert!(
            (b_full - b_within).abs() <= 1e-10 * b_full.abs().max(1e-300),
            "{units} units: {b_full} against {b_within}"
        );
        // At the engine's tolerance the slope agrees within 1e-8 of the exact one.
        let usual = demean(&[&y, &x], &effects, None, Options::default());
        let within_usual = least_squares(
            &Matrix::from_columns(&[usual.columns[1].clone()]),
            &usual.columns[0],
            1e-10,
            256,
        );
        let b_usual = within_usual.coefficients[0].unwrap();
        assert!(
            (b_full - b_usual).abs() <= 1e-8 * b_full.abs().max(1e-300),
            "{units} units at 1e-8: {b_full} against {b_usual} after {} passes",
            usual.passes
        );
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(48)))]

    #[test]
    fn constant_within_groups_demeans_to_zero_and_the_rest_is_orthogonal(
        unit_of in prop::collection::vec(0usize..6, 8..40),
        period_of in prop::collection::vec(0usize..4, 8..40),
        values in prop::collection::vec(-1.0e3f64..1.0e3, 40),
        level_of_unit in prop::collection::vec(-1.0e6f64..1.0e6, 6),
    ) {
        let n = unit_of.len().min(period_of.len());
        let unit_of = unit_of[..n].to_vec();
        let period_of = period_of[..n].to_vec();
        let x: Vec<f64> = values[..n].to_vec();
        let constant: Vec<f64> = unit_of.iter().map(|&u| level_of_unit[u]).collect();
        let groups = vec![unit_of.clone(), period_of.clone()];
        let effects = effects_of(&groups, &[6, 4]);
        let out = demean(&[&x, &constant], &effects, None, Options { tolerance: 1e-12, max_passes: 20_000 });
        prop_assert!(out.converged, "{} passes, change {}", out.passes, out.largest_change);
        let scale = level_of_unit.iter().fold(0.0f64, |m, v| m.max(v.abs())).max(1.0);
        for v in &out.columns[1] {
            prop_assert!(v.abs() <= 1e-12 * scale, "a constant within units left {v}");
        }
        // Orthogonal to every indicator: each group's sum of the demeaned column is zero within
        // 1e-10 of the column's scale times the group size.
        let x_scale = x.iter().fold(0.0f64, |m, v| m.max(v.abs())).max(1.0);
        for (effect, groups_n) in [(&unit_of, 6usize), (&period_of, 4usize)] {
            let mut sums = vec![0.0f64; groups_n];
            let mut sizes = vec![0usize; groups_n];
            for (i, &r) in out.rows.iter().enumerate() {
                sums[effect[r]] += out.columns[0][i];
                sizes[effect[r]] += 1;
            }
            for (g, s) in sums.iter().enumerate() {
                prop_assert!(s.abs() <= 1e-10 * x_scale * sizes[g].max(1) as f64, "group {g} sums to {s}");
            }
        }
    }

    #[test]
    fn the_row_order_changes_nothing_and_equal_weights_are_no_weights(
        unit_of in prop::collection::vec(0usize..5, 6..30),
        period_of in prop::collection::vec(0usize..3, 6..30),
        values in prop::collection::vec(-1.0e3f64..1.0e3, 30),
        shift in 1usize..29,
    ) {
        let n = unit_of.len().min(period_of.len());
        let (unit_of, period_of, x) = (unit_of[..n].to_vec(), period_of[..n].to_vec(), values[..n].to_vec());
        let groups = vec![unit_of.clone(), period_of.clone()];
        let effects = effects_of(&groups, &[5, 3]);
        let options = Options { tolerance: 1e-12, max_passes: 20_000 };
        let out = demean(&[&x], &effects, None, options);
        // The same rows rotated: every original row gets the same value within 1e-12.
        let rotate = |v: &[usize]| -> Vec<usize> { v.iter().cycle().skip(shift % n).take(n).copied().collect() };
        let rotated_groups = vec![rotate(&unit_of), rotate(&period_of)];
        let rotated_x: Vec<f64> = x.iter().cycle().skip(shift % n).take(n).copied().collect();
        let rotated = demean(&[&rotated_x], &effects_of(&rotated_groups, &[5, 3]), None, options);
        prop_assert_eq!(out.rows.len(), rotated.rows.len());
        let scale = x.iter().fold(0.0f64, |m, v| m.max(v.abs())).max(1.0);
        let mut by_row = vec![f64::NAN; n];
        for (i, &r) in out.rows.iter().enumerate() {
            by_row[r] = out.columns[0][i];
        }
        for (i, &r) in rotated.rows.iter().enumerate() {
            let original = (r + shift % n) % n;
            prop_assert!((rotated.columns[0][i] - by_row[original]).abs() <= 1e-12 * scale, "row {original}");
        }
        let weighted = demean(&[&x], &effects, Some(&vec![0.75; n]), options);
        prop_assert_eq!(out.columns, weighted.columns);
    }
}

/// The residual of `y` on the indicators under weights `w`: the weighted projection, through
/// the kernel's QR on the rows scaled by the square root of their weights.
fn projected_out_weighted(x: &Matrix, y: &[f64], w: &[f64]) -> Vec<f64> {
    let roots: Vec<f64> = w.iter().map(|v| v.sqrt()).collect();
    let scaled_columns: Vec<Vec<f64>> = (0..x.cols())
        .map(|j| (0..y.len()).map(|i| x.get(i, j) * roots[i]).collect())
        .collect();
    let scaled = Matrix::from_columns(&scaled_columns);
    let scaled_y: Vec<f64> = y.iter().zip(&roots).map(|(v, r)| v * r).collect();
    projected_out(&scaled, &scaled_y)
        .iter()
        .zip(&roots)
        .map(|(v, r)| v / r)
        .collect()
}

/// Two effects under weights, the second with fewer groups so that its means are taken by the
/// compensated accumulators in row order: the kernel agrees with the weighted projection.
#[test]
fn weights_with_two_effects_agree_with_the_weighted_projection() {
    let mut rng = SplitMix64(0x5EED_0623);
    let n = 120;
    let first: Vec<usize> = (0..n).map(|i| i % 12).collect();
    let second: Vec<usize> = (0..n).map(|_| rng.below(4)).collect();
    let x: Vec<f64> = (0..n).map(|_| rng.unit() * 20.0 - 10.0).collect();
    let y: Vec<f64> = (0..n)
        .map(|i| 3.0 * first[i] as f64 - 2.0 * second[i] as f64 + rng.unit())
        .collect();
    let w: Vec<f64> = (0..n).map(|_| 0.5 + rng.unit() * 3.0).collect();
    let groups = vec![first.clone(), second.clone()];
    let effects = effects_of(&groups, &[12, 4]);
    let result = demean(&[&x, &y], &effects, Some(&w), Options::default());
    assert!(result.converged);
    assert_eq!(result.singletons_dropped, 0);
    let rows = result.rows.clone();
    let indicators = indicators(&groups, &rows);
    for (c, column) in [&x, &y].iter().enumerate() {
        let ordered: Vec<f64> = rows.iter().map(|&r| column[r]).collect();
        let ordered_w: Vec<f64> = rows.iter().map(|&r| w[r]).collect();
        let exact = projected_out_weighted(&indicators, &ordered, &ordered_w);
        for (i, (a, b)) in result.columns[c].iter().zip(&exact).enumerate() {
            assert!(
                (a - b).abs() <= 1e-8 * (1.0 + b.abs()),
                "column {c}, row {i}: {a} vs {b}"
            );
        }
    }
}

/// No effects: the columns come back as given, converged, in no pass.
#[test]
fn no_effects_change_nothing_in_no_pass() {
    let x = [1.0, 2.5, -3.0, 4.0];
    let result = demean(&[&x], &[], None, Options::default());
    assert_eq!(result.columns, vec![x.to_vec()]);
    assert_eq!(result.rows, vec![0, 1, 2, 3]);
    assert!(result.converged);
    assert_eq!(result.passes, 0);
    assert_eq!(result.singletons_dropped, 0);
}
