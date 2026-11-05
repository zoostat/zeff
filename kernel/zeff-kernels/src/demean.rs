//! Fixed-effects demeaning (Language Reference, section 7.2, "Fixed effects" and "Rows"; Engine
//! and Project Specification, section 8.3, points 3 and 10; release 0.6.0, step 17): the columns
//! of a model with one or several fixed effects projected out by alternating projections with
//! Irons–Tuck acceleration, as fixest does, to a tolerance on the largest change.
//!
//! The caller hands over each effect as the group of every row (a category, an Int, a text or
//! a date column, or an interaction of two, grouped by the column kernel's identity) and the
//! columns to demean; the kernel drops singletons until none remains, orders the rows once by
//! the effect with the most groups so that its group sums run over contiguous rows, gathers
//! every column into one scratch buffer, and sweeps the effects in their written order.
//!
//! The arithmetic is fixed: the group sums of the largest effect use the reduction shape of
//! `reduce` over each group's contiguous run, the other effects' sums are compensated two-sum
//! accumulators per group fed in row order, and a group mean is the compensated sum over the
//! count (the weighted sum over the weights' sum under weights). No operation depends on the
//! target, so the four targets give the same bits. A constant weight is no weight: the weighted
//! path is taken only when the weights differ, so equal weights give the unweighted result bit
//! for bit.

use crate::reduce::{LaneSum, sum};

/// One fixed effect: the group of each row, as positions `0..groups`.
#[derive(Debug, Clone)]
pub struct Effect<'a> {
    pub group_of_row: &'a [usize],
    pub groups: usize,
}

/// How the projections run.
#[derive(Debug, Clone, Copy)]
pub struct Options {
    /// Convergence: the largest change of a sweep, relative to the column's largest absolute
    /// value, below which the projections stop (section 7.2: 1e-8).
    pub tolerance: f64,
    /// The most sweeps over the effects before the kernel gives up.
    pub max_passes: usize,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            tolerance: 1e-8,
            max_passes: 10_000,
        }
    }
}

/// The columns with the fixed effects projected out.
#[derive(Debug, Clone)]
pub struct Demeaned {
    /// The rows kept, as positions in the caller's data, in the order the columns hold them:
    /// sorted once by the effect with the most groups, stable within a group.
    pub rows: Vec<usize>,
    /// Each column demeaned, one value per kept row, in the order of `rows`.
    pub columns: Vec<Vec<f64>>,
    /// The rows dropped as singletons, over every round.
    pub singletons_dropped: usize,
    /// The sweeps over the effects that moved the data; the sweep that confirmed convergence
    /// by changing nothing is not counted.
    pub passes: usize,
    /// Whether the largest change fell below the tolerance.
    pub converged: bool,
    /// The largest relative change of the last sweep.
    pub largest_change: f64,
}

/// The rows that remain once every singleton is dropped, repeatedly, until no effect has a
/// group of one row among them; with the count dropped.
pub fn without_singletons(effects: &[Effect<'_>], rows: usize) -> (Vec<usize>, usize) {
    let mut kept: Vec<usize> = (0..rows).collect();
    loop {
        let mut drop = vec![false; kept.len()];
        for effect in effects {
            let mut sizes = vec![0usize; effect.groups];
            for &r in &kept {
                sizes[effect.group_of_row[r]] += 1;
            }
            for (i, &r) in kept.iter().enumerate() {
                if sizes[effect.group_of_row[r]] == 1 {
                    drop[i] = true;
                }
            }
        }
        if !drop.iter().any(|d| *d) {
            break;
        }
        kept = kept
            .iter()
            .zip(&drop)
            .filter(|(_, d)| !**d)
            .map(|(r, _)| *r)
            .collect();
    }
    (kept.clone(), rows - kept.len())
}

/// The kept rows ordered by the effect with the most groups, stably, and the position of that
/// effect.
fn ordered(effects: &[Effect<'_>], kept: &[usize]) -> (Vec<usize>, usize) {
    let largest = (0..effects.len())
        .max_by_key(|&e| effects[e].groups)
        .unwrap_or(0);
    let mut order = kept.to_vec();
    if !effects.is_empty() {
        order.sort_by_key(|&r| effects[largest].group_of_row[r]);
    }
    (order, largest)
}

/// The groups of one effect over the kept rows in their order: each row's group relabelled to
/// `0..count` in order of first appearance, and the rows of each group.
struct Grouped {
    group_of_row: Vec<usize>,
    rows_of_group: Vec<Vec<usize>>,
    /// The groups' runs when the rows are sorted by this effect: `Some((start, end))` per group.
    runs: Option<Vec<(usize, usize)>>,
}

fn grouped(effect: &Effect<'_>, order: &[usize], contiguous: bool) -> Grouped {
    let mut label = vec![usize::MAX; effect.groups];
    let mut group_of_row = Vec::with_capacity(order.len());
    let mut rows_of_group: Vec<Vec<usize>> = Vec::new();
    for (i, &r) in order.iter().enumerate() {
        let g = effect.group_of_row[r];
        if label[g] == usize::MAX {
            label[g] = rows_of_group.len();
            rows_of_group.push(Vec::new());
        }
        group_of_row.push(label[g]);
        rows_of_group[label[g]].push(i);
    }
    let runs = contiguous.then(|| {
        rows_of_group
            .iter()
            .map(|rows| (rows[0], rows[rows.len() - 1] + 1))
            .collect()
    });
    Grouped {
        group_of_row,
        rows_of_group,
        runs,
    }
}

/// The group means of `x` under `effect`, the largest effect's over its contiguous runs in the
/// reduction shape, the others' by compensated accumulators in row order.
fn group_means(x: &[f64], weights: Option<&[f64]>, effect: &Grouped, scratch: &mut Vec<f64>) {
    let groups = effect.rows_of_group.len();
    scratch.clear();
    scratch.resize(groups, 0.0);
    match (&effect.runs, weights) {
        (Some(runs), None) => {
            for (g, &(start, end)) in runs.iter().enumerate() {
                scratch[g] = sum(&x[start..end]) / (end - start) as f64;
            }
        }
        (Some(runs), Some(w)) => {
            for (g, &(start, end)) in runs.iter().enumerate() {
                let mut wx = LaneSum::new();
                let mut ww = LaneSum::new();
                for i in start..end {
                    wx.push(w[i] * x[i]);
                    ww.push(w[i]);
                }
                scratch[g] = wx.value() / ww.value();
            }
        }
        (None, None) => {
            let mut sums = vec![0.0f64; groups];
            let mut errors = vec![0.0f64; groups];
            for (i, &g) in effect.group_of_row.iter().enumerate() {
                let (s, e) = two_sum(sums[g], x[i]);
                sums[g] = s;
                errors[g] += e;
            }
            for g in 0..groups {
                scratch[g] = (sums[g] + errors[g]) / effect.rows_of_group[g].len() as f64;
            }
        }
        (None, Some(w)) => {
            let mut sums = vec![0.0f64; groups];
            let mut errors = vec![0.0f64; groups];
            let mut wsums = vec![0.0f64; groups];
            let mut werrors = vec![0.0f64; groups];
            for (i, &g) in effect.group_of_row.iter().enumerate() {
                let (s, e) = two_sum(sums[g], w[i] * x[i]);
                sums[g] = s;
                errors[g] += e;
                let (s, e) = two_sum(wsums[g], w[i]);
                wsums[g] = s;
                werrors[g] += e;
            }
            for g in 0..groups {
                scratch[g] = (sums[g] + errors[g]) / (wsums[g] + werrors[g]);
            }
        }
    }
}

/// Knuth's two-sum: `a + b` rounded, and the rounding error exactly.
#[inline(always)]
fn two_sum(a: f64, b: f64) -> (f64, f64) {
    let s = a + b;
    let bb = s - a;
    let error = (a - (s - bb)) + (b - bb);
    (s, error)
}

/// One sweep: every column less its group means under each effect in turn.
fn sweep(
    buffer: &mut [f64],
    n: usize,
    k: usize,
    effects: &[Grouped],
    weights: Option<&[f64]>,
    means: &mut Vec<f64>,
) {
    for effect in effects {
        for c in 0..k {
            let column = &mut buffer[c * n..(c + 1) * n];
            group_means(column, weights, effect, means);
            for (i, x) in column.iter_mut().enumerate() {
                *x -= means[effect.group_of_row[i]];
            }
        }
    }
}

/// The columns with the fixed effects projected out, the rows kept in the kernel's order.
pub fn demean(
    columns: &[&[f64]],
    effects: &[Effect<'_>],
    weights: Option<&[f64]>,
    options: Options,
) -> Demeaned {
    let rows = columns.first().map_or(0, |c| c.len());
    for column in columns {
        assert_eq!(column.len(), rows, "every column has one value per row");
    }
    let (kept, singletons_dropped) = without_singletons(effects, rows);
    let (order, largest) = ordered(effects, &kept);
    let n = order.len();
    let k = columns.len();
    // One scratch buffer, column-major, every column gathered in the rows' order.
    let mut buffer: Vec<f64> = Vec::with_capacity(n * k);
    for column in columns {
        buffer.extend(order.iter().map(|&r| column[r]));
    }
    let scales: Vec<f64> = (0..k)
        .map(|c| {
            buffer[c * n..(c + 1) * n]
                .iter()
                .fold(0.0f64, |m, x| m.max(x.abs()))
                .max(f64::MIN_POSITIVE)
        })
        .collect();
    // A constant weight is no weight.
    let weights_in_order: Option<Vec<f64>> = weights.and_then(|w| {
        let gathered: Vec<f64> = order.iter().map(|&r| w[r]).collect();
        let first = gathered.first().copied().unwrap_or(1.0);
        gathered.iter().any(|x| *x != first).then_some(gathered)
    });
    let w = weights_in_order.as_deref();
    let groups: Vec<Grouped> = effects
        .iter()
        .enumerate()
        .map(|(e, effect)| grouped(effect, &order, e == largest))
        .collect();
    let mut means: Vec<f64> = Vec::new();
    // `passes` counts the sweeps that moved the data; the sweep that finds no change confirms
    // convergence and is not counted, so one effect, and two balanced effects, take one pass.
    let mut sweeps = 0usize;
    let mut converged = effects.is_empty() || n == 0;
    let mut largest_change = 0.0f64;
    let change_of = |before: &[f64], after: &[f64]| -> f64 {
        let mut worst = 0.0f64;
        for c in 0..k {
            for i in 0..n {
                let d = (after[c * n + i] - before[c * n + i]).abs() / scales[c];
                worst = worst.max(d);
            }
        }
        worst
    };
    if !converged {
        let mut previous = buffer.clone();
        let mut gx = vec![0.0; n * k];
        sweep(&mut buffer, n, k, &groups, w, &mut means);
        sweeps += 1;
        // Irons–Tuck acceleration over whole sweeps: from x, the sweeps give Gx and GGx; the
        // next point is GGx − ((ΔG − ΔX)·ΔG / ‖ΔG − ΔX‖²) ΔG, with ΔX = Gx − x and ΔG = GGx − Gx,
        // as fixest accelerates its projections. A sweep that changes nothing ends the loop.
        loop {
            previous.copy_from_slice(&buffer);
            sweep(&mut buffer, n, k, &groups, w, &mut means);
            sweeps += 1;
            largest_change = change_of(&previous, &buffer);
            if largest_change < options.tolerance || sweeps >= options.max_passes {
                converged = largest_change < options.tolerance;
                break;
            }
            gx.copy_from_slice(&buffer);
            sweep(&mut buffer, n, k, &groups, w, &mut means);
            sweeps += 1;
            largest_change = change_of(&gx, &buffer);
            if largest_change < options.tolerance || sweeps >= options.max_passes {
                converged = largest_change < options.tolerance;
                break;
            }
            let (mut num, mut den) = (0.0f64, 0.0f64);
            for i in 0..n * k {
                let dx = gx[i] - previous[i];
                let dg = buffer[i] - gx[i];
                let diff = dg - dx;
                num += diff * dg;
                den += diff * diff;
            }
            if den > 0.0 && num.is_finite() {
                let step = num / den;
                for i in 0..n * k {
                    let dg = buffer[i] - gx[i];
                    buffer[i] -= step * dg;
                }
            }
        }
    }
    let passes = if converged && sweeps > 0 {
        sweeps - 1
    } else {
        sweeps
    };
    let columns_out: Vec<Vec<f64>> = (0..k)
        .map(|c| buffer[c * n..(c + 1) * n].to_vec())
        .collect();
    Demeaned {
        rows: order,
        columns: columns_out,
        singletons_dropped,
        passes,
        converged,
        largest_change,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The weighted means in row order (the branch of an effect that is not the largest) keep
    /// the rounding errors of both accumulators: a group of 2⁵³, 1 and 1 has the mean
    /// (2⁵³ + 2)/3 and not 2⁵³/3, and weights of 2⁵⁴, 2 and 2 are summed to 2⁵⁴ + 4 and not
    /// 2⁵⁴. The expected values are written out by an independent compensated sum.
    #[test]
    fn weighted_means_in_row_order_keep_their_compensation() {
        fn compensated(values: &[f64]) -> f64 {
            let (mut sum, mut error) = (0.0f64, 0.0f64);
            for &v in values {
                let s = sum + v;
                let bv = s - sum;
                error += (sum - (s - bv)) + (v - bv);
                sum = s;
            }
            sum + error
        }
        fn naive(values: &[f64]) -> f64 {
            values.iter().fold(0.0, |a, b| a + b)
        }
        let big = 9_007_199_254_740_992.0f64; // 2⁵³
        let x = [big, 1.0, 1.0, 5.0, 7.0, 9.0];
        let w = [1.0, 1.0, 1.0, 2.0 * big, 2.0, 2.0];
        let effect = Grouped {
            group_of_row: vec![0, 0, 0, 1, 1, 1],
            rows_of_group: vec![vec![0, 1, 2], vec![3, 4, 5]],
            runs: None,
        };
        let mut scratch = Vec::new();
        group_means(&x, Some(&w), &effect, &mut scratch);
        let wx1 = [5.0 * 2.0 * big, 7.0 * 2.0, 9.0 * 2.0];
        let w1 = [2.0 * big, 2.0, 2.0];
        let expected0 = compensated(&[big, 1.0, 1.0]) / compensated(&[1.0, 1.0, 1.0]);
        let expected1 = compensated(&wx1) / compensated(&w1);
        assert_eq!(scratch, vec![expected0, expected1]);
        // The compensation shows: the naive sums give other numbers (the first group's
        // values, the second group's weights).
        assert_ne!(expected0, naive(&[big, 1.0, 1.0]) / 3.0);
        assert_ne!(expected1, naive(&wx1) / naive(&w1));
    }

    #[test]
    fn one_effect_is_centring_by_group_in_one_sweep() {
        let g = [0usize, 0, 1, 1, 1];
        let x = [1.0, 3.0, 10.0, 20.0, 30.0];
        let out = demean(
            &[&x],
            &[Effect {
                group_of_row: &g,
                groups: 2,
            }],
            None,
            Options::default(),
        );
        assert_eq!(out.rows, [0, 1, 2, 3, 4]);
        assert_eq!(out.columns[0], [-1.0, 1.0, -10.0, 0.0, 10.0]);
        assert_eq!(out.singletons_dropped, 0);
        assert!(out.converged);
        assert_eq!(out.passes, 1);
    }

    #[test]
    fn two_balanced_effects_converge_in_one_pass() {
        // Two units over two periods, balanced: the two projections commute.
        let unit = [0usize, 0, 1, 1];
        let period = [0usize, 1, 0, 1];
        let x = [1.0, 2.0, 4.0, 8.0];
        let out = demean(
            &[&x],
            &[
                Effect {
                    group_of_row: &unit,
                    groups: 2,
                },
                Effect {
                    group_of_row: &period,
                    groups: 2,
                },
            ],
            None,
            Options::default(),
        );
        assert!(out.converged);
        assert_eq!(out.passes, 1);
        // x − unit mean − period mean + grand mean.
        let expected = [
            1.0 - 1.5 - 2.5 + 3.75,
            2.0 - 1.5 - 5.0 + 3.75,
            4.0 - 6.0 - 2.5 + 3.75,
            8.0 - 6.0 - 5.0 + 3.75,
        ];
        for (a, b) in out.columns[0].iter().zip(&expected) {
            assert!((a - b).abs() < 1e-12, "{a} against {b}");
        }
    }

    #[test]
    fn singletons_are_dropped_until_none_remains() {
        // Unit 2 has one row; dropping it leaves period 2 with one row, dropped in turn.
        let unit = [0usize, 0, 1, 1, 2];
        let period = [0usize, 1, 0, 1, 2];
        let x = [1.0, 2.0, 3.0, 4.0, 5.0];
        let (kept, dropped) = without_singletons(
            &[
                Effect {
                    group_of_row: &unit,
                    groups: 3,
                },
                Effect {
                    group_of_row: &period,
                    groups: 3,
                },
            ],
            5,
        );
        assert_eq!(kept, [0, 1, 2, 3]);
        assert_eq!(dropped, 1);
        let period2 = [0usize, 1, 0, 2, 1];
        let (kept, dropped) = without_singletons(
            &[
                Effect {
                    group_of_row: &unit,
                    groups: 3,
                },
                Effect {
                    group_of_row: &period2,
                    groups: 3,
                },
            ],
            5,
        );
        // Row 4 (unit 2) goes; then period 1 holds row 1 alone and goes; then unit 0 holds row 0
        // alone and goes; then period 0 holds row 2 alone and goes; then unit 1 holds row 3 alone.
        assert_eq!(kept, Vec::<usize>::new());
        assert_eq!(dropped, 5);
        let _ = x;
    }

    #[test]
    fn equal_weights_are_no_weights() {
        let g = [0usize, 0, 1, 1, 1];
        let x = [1.0, 3.0, 10.0, 20.0, 30.5];
        let effects = [Effect {
            group_of_row: &g,
            groups: 2,
        }];
        let plain = demean(&[&x], &effects, None, Options::default());
        let weighted = demean(&[&x], &effects, Some(&[2.5; 5]), Options::default());
        assert_eq!(plain.columns, weighted.columns);
    }
}
