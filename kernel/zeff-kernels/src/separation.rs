//! Separation in a binary outcome model: whether the maximum of the likelihood exists, decided
//! by a linear program before any iteration (Albert and Anderson 1984; Konis 2007).
//!
//! Write `zᵢ = sᵢ xᵢ`, with `sᵢ = +1` for an outcome of 1 and `−1` for an outcome of 0. The
//! outcomes are separated, completely or quasi-completely, when some direction β ≠ 0 has
//! `zᵢᵀβ ≥ 0` in every row: along it the likelihood rises for ever and the estimates diverge.
//! For a design of full column rank, Stiemke's theorem gives the alternative: the outcomes
//! overlap, and the maximum exists, exactly when some weights `uᵢ > 0` have `Σ uᵢ zᵢ = 0`.
//!
//! The kernel decides between the two with the first phase of the simplex method on
//! `Σ vᵢ zᵢ + S a = −Σ zᵢ`, `v ≥ 0`, `a ≥ 0` (so that `uᵢ = 1 + vᵢ`), minimising the sum of the
//! k artificial variables `a`: a minimum of zero is a set of weights, so the outcomes overlap;
//! a positive minimum is separation, and the simplex multipliers at the minimum, negated, are a
//! direction β with `Zβ ≥ 0`, the dual of the program being Konis's linear program
//! `max Σ zᵢᵀβ` subject to `Zβ ≥ 0` and a bound on β. The coefficients that are not zero in β
//! are those whose estimates diverge.
//!
//! The basis has k rows, so each iteration costs one pass over the rows (the pricing) and a
//! k × k update; the basis inverse is rebuilt from its columns at a fixed interval. The entering
//! column is the most negative reduced cost, the lowest row on a tie, and after a run of steps
//! that do not move, Bland's rule (the lowest row with a negative reduced cost), which cannot
//! cycle. Every operation is a plain IEEE 754 one in a fixed order, so the answer is the same on
//! every machine.

use crate::{LaneSum, Matrix};

/// The answer of the check.
#[derive(Clone, Debug, PartialEq)]
pub enum Separation {
    /// The outcomes overlap: the maximum of the likelihood exists.
    Overlap,
    /// The outcomes are separated along `direction`, one value per column of X in the units of
    /// X; `diverging` lists the columns whose entry is not zero, in order, and `rows` counts the
    /// rows the direction separates (`zᵢᵀβ > 0`).
    Separated {
        direction: Vec<f64>,
        diverging: Vec<usize>,
        rows: usize,
    },
    /// The simplex did not settle within its iteration cap, or lost its pivots to rounding: no
    /// answer, and the fit goes ahead.
    Unsettled,
}

/// A reduced cost below minus this (in the units of the scaled columns) lets its column enter.
const PRICE_TOLERANCE: f64 = 1e-11;
/// A pivot entry at or below this is treated as zero in the ratio test.
const PIVOT_TOLERANCE: f64 = 1e-9;
/// A minimum of the artificial variables above this share of the right-hand side's size is
/// separation; the minimum is of the order of the separated rows' share otherwise.
const SEPARATION_TOLERANCE: f64 = 1e-9;
/// An entry of the direction below this share of its largest is zero.
const SUPPORT_TOLERANCE: f64 = 1e-6;
/// The basis inverse is rebuilt from its columns after this many updates.
const REBUILD_EVERY: usize = 32;
/// Steps that do not move before Bland's rule takes over.
const STALL_LIMIT: usize = 50;

/// The separation check of the outcome `y` (0 and 1) on the design `x`, n × k, of full column
/// rank (the collinear columns already dropped).
pub fn separation(x: &Matrix, y: &[f64]) -> Separation {
    let (n, k) = (x.rows(), x.cols());
    if k == 0 || n == 0 {
        return Separation::Overlap;
    }
    // The columns at unit norm, so that the tolerances are shares of a unit.
    let scales: Vec<f64> = (0..k)
        .map(|j| {
            let mut lanes = LaneSum::new();
            lanes.push_squares(x.column(j));
            let squares = lanes.value();
            if squares > 0.0 {
                1.0 / squares.sqrt()
            } else {
                1.0
            }
        })
        .collect();
    let sign: Vec<f64> = y
        .iter()
        .map(|&v| if v == 1.0 { 1.0 } else { -1.0 })
        .collect();
    // Z by rows, k values per row, contiguous for the pricing pass.
    let mut z = vec![0.0; n * k];
    for j in 0..k {
        let column = x.column(j);
        for i in 0..n {
            z[i * k + j] = sign[i] * column[i] * scales[j];
        }
    }
    // The right-hand side b = −Σ zᵢ, by columns in the fixed lane shape.
    let b: Vec<f64> = (0..k)
        .map(|j| {
            let mut lanes = LaneSum::new();
            for i in 0..n {
                lanes.push(z[i * k + j]);
            }
            -lanes.value()
        })
        .collect();
    let artificial_sign: Vec<f64> = b
        .iter()
        .map(|&v| if v >= 0.0 { 1.0 } else { -1.0 })
        .collect();
    let size = b.iter().fold(1.0_f64, |m, v| m.max(v.abs()));

    // Variables 0..n are the vᵢ, n..n + k the artificials. The basis starts as the artificials.
    let mut basis: Vec<usize> = (n..n + k).collect();
    let column_of = |var: usize, out: &mut [f64]| {
        if var < n {
            out.copy_from_slice(&z[var * k..var * k + k]);
        } else {
            out.iter_mut().for_each(|v| *v = 0.0);
            out[var - n] = artificial_sign[var - n];
        }
    };
    let cost = |var: usize| if var < n { 0.0 } else { 1.0 };
    // B⁻¹ by rows, k × k.
    let mut inverse = vec![0.0; k * k];
    for j in 0..k {
        inverse[j * k + j] = artificial_sign[j];
    }
    let mut values: Vec<f64> = b.iter().map(|v| v.abs()).collect();
    let mut in_basis = vec![false; n + k];
    for &var in &basis {
        in_basis[var] = true;
    }

    let cap = 1_000 + 20 * (n + k);
    let mut updates = 0;
    let mut stalled = 0;
    let mut multipliers = vec![0.0; k];
    let mut entering_column = vec![0.0; k];
    let mut w = vec![0.0; k];
    let mut converged = false;
    for _ in 0..cap {
        // The simplex multipliers y = B⁻ᵀ c_B.
        for (j, m) in multipliers.iter_mut().enumerate() {
            let mut total = 0.0;
            for r in 0..k {
                total += inverse[r * k + j] * cost(basis[r]);
            }
            *m = total;
        }
        // Pricing: the reduced cost of vᵢ is −yᵀzᵢ; an artificial that left does not return.
        let bland = stalled >= STALL_LIMIT;
        let mut entering = None;
        let mut best = -PRICE_TOLERANCE;
        for i in 0..n {
            if in_basis[i] {
                continue;
            }
            let row = &z[i * k..i * k + k];
            let mut product = 0.0;
            for j in 0..k {
                product += multipliers[j] * row[j];
            }
            let reduced = -product;
            if reduced < best {
                entering = Some(i);
                if bland {
                    break;
                }
                best = reduced;
            }
        }
        let Some(q) = entering else {
            converged = true;
            break;
        };
        // The entering column in the basis, w = B⁻¹ a_q, and the ratio test.
        column_of(q, &mut entering_column);
        for (r, slot) in w.iter_mut().enumerate() {
            let mut total = 0.0;
            for j in 0..k {
                total += inverse[r * k + j] * entering_column[j];
            }
            *slot = total;
        }
        let mut leaving: Option<usize> = None;
        let mut ratio = f64::INFINITY;
        for r in 0..k {
            if w[r] > PIVOT_TOLERANCE {
                let candidate = values[r] / w[r];
                let better = match leaving {
                    None => true,
                    Some(l) => candidate < ratio || (candidate == ratio && basis[r] < basis[l]),
                };
                if better {
                    ratio = candidate;
                    leaving = Some(r);
                }
            }
        }
        let Some(r) = leaving else {
            // A first phase is bounded below by zero: no positive entry is rounding.
            return Separation::Unsettled;
        };
        stalled = if ratio == 0.0 { stalled + 1 } else { 0 };
        for (s, value) in values.iter_mut().enumerate() {
            if s != r {
                *value -= ratio * w[s];
                if *value < 0.0 {
                    *value = 0.0;
                }
            }
        }
        values[r] = ratio;
        in_basis[basis[r]] = false;
        basis[r] = q;
        in_basis[q] = true;
        updates += 1;
        if updates % REBUILD_EVERY == 0 {
            match rebuild(&basis, k, &column_of) {
                Some(fresh) => inverse = fresh,
                None => return Separation::Unsettled,
            }
            for (s, value) in values.iter_mut().enumerate() {
                let mut total = 0.0;
                for j in 0..k {
                    total += inverse[s * k + j] * b[j];
                }
                *value = total.max(0.0);
            }
        } else {
            let pivot = w[r];
            for j in 0..k {
                inverse[r * k + j] /= pivot;
            }
            for s in 0..k {
                if s != r && w[s] != 0.0 {
                    let factor = w[s];
                    for j in 0..k {
                        inverse[s * k + j] -= factor * inverse[r * k + j];
                    }
                }
            }
        }
    }
    if !converged {
        return Separation::Unsettled;
    }
    let mut objective = 0.0;
    for (r, &var) in basis.iter().enumerate() {
        objective += cost(var) * values[r];
    }
    if objective <= SEPARATION_TOLERANCE * size {
        return Separation::Overlap;
    }
    // The direction: the multipliers at the minimum, negated, back in the units of X.
    for (j, m) in multipliers.iter_mut().enumerate() {
        let mut total = 0.0;
        for r in 0..k {
            total += inverse[r * k + j] * cost(basis[r]);
        }
        *m = total;
    }
    let scaled: Vec<f64> = multipliers.iter().map(|m| -m).collect();
    let largest = scaled.iter().fold(0.0_f64, |m, v| m.max(v.abs()));
    if largest == 0.0 {
        return Separation::Unsettled;
    }
    let diverging: Vec<usize> = (0..k)
        .filter(|&j| scaled[j].abs() > SUPPORT_TOLERANCE * largest)
        .collect();
    let direction: Vec<f64> = (0..k)
        .map(|j| {
            if diverging.contains(&j) {
                scaled[j] / largest * scales[j]
            } else {
                0.0
            }
        })
        .collect();
    let rows = (0..n)
        .filter(|&i| {
            let row = &z[i * k..i * k + k];
            let mut product = 0.0;
            for j in 0..k {
                if diverging.contains(&j) {
                    product += row[j] * scaled[j] / largest;
                }
            }
            product > SUPPORT_TOLERANCE
        })
        .count();
    Separation::Separated {
        direction,
        diverging,
        rows,
    }
}

/// The inverse of the basis matrix from its columns, by Gauss–Jordan elimination with partial
/// pivoting (the lowest row on a tie), by rows; `None` for a basis that has lost its rank.
fn rebuild(basis: &[usize], k: usize, column_of: &dyn Fn(usize, &mut [f64])) -> Option<Vec<f64>> {
    // The basis matrix by rows: entry (j, r) is component j of the r-th basic column.
    let mut a = vec![0.0; k * k];
    let mut column = vec![0.0; k];
    for (r, &var) in basis.iter().enumerate() {
        column_of(var, &mut column);
        for j in 0..k {
            a[j * k + r] = column[j];
        }
    }
    // Solve B X = I for X = B⁻¹, with the rows of B⁻¹ indexed by the basis positions.
    let mut inverse = vec![0.0; k * k];
    for j in 0..k {
        inverse[j * k + j] = 1.0;
    }
    for c in 0..k {
        let mut pivot_row = c;
        let mut largest = a[c * k + c].abs();
        for row in c + 1..k {
            let value = a[row * k + c].abs();
            if value > largest {
                largest = value;
                pivot_row = row;
            }
        }
        if largest <= 1e-14 {
            return None;
        }
        if pivot_row != c {
            for j in 0..k {
                a.swap(c * k + j, pivot_row * k + j);
                inverse.swap(c * k + j, pivot_row * k + j);
            }
        }
        let pivot = a[c * k + c];
        for j in 0..k {
            a[c * k + j] /= pivot;
            inverse[c * k + j] /= pivot;
        }
        for row in 0..k {
            if row != c {
                let factor = a[row * k + c];
                if factor != 0.0 {
                    for j in 0..k {
                        a[row * k + j] -= factor * a[c * k + j];
                        inverse[row * k + j] -= factor * inverse[c * k + j];
                    }
                }
            }
        }
    }
    // Row c of the eliminated system is the c-th unknown, the coefficient of basic column c:
    // the rows of `inverse` are already indexed by basis position.
    Some(inverse)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn design(columns: &[Vec<f64>]) -> Matrix {
        Matrix::from_columns(columns)
    }

    #[test]
    fn overlapping_outcomes_have_a_maximum() {
        let x = design(&[vec![1.0; 6], vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]]);
        let y = [0.0, 1.0, 0.0, 1.0, 1.0, 0.0];
        assert_eq!(separation(&x, &y), Separation::Overlap);
    }

    #[test]
    fn complete_separation_names_the_regressor() {
        let x = design(&[vec![1.0; 6], vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]]);
        let y = [0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
        match separation(&x, &y) {
            Separation::Separated {
                diverging, rows, ..
            } => {
                assert!(diverging.contains(&1), "{diverging:?}");
                assert!(rows >= 1);
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn quasi_separation_by_an_indicator() {
        // x = 1 only where y = 1; where x = 0 the outcomes overlap.
        let x = design(&[vec![1.0; 8], vec![1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]]);
        let y = [1.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0];
        match separation(&x, &y) {
            Separation::Separated {
                diverging,
                direction,
                rows,
            } => {
                assert_eq!(diverging, vec![1]);
                assert!(direction[1] > 0.0);
                assert_eq!(rows, 2);
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn the_answer_is_the_same_twice() {
        let x = design(&[
            vec![1.0; 10],
            (0..10).map(|i| (i * 7 % 10) as f64).collect(),
            (0..10).map(|i| (i * 3 % 5) as f64).collect(),
        ]);
        let y: Vec<f64> = (0..10)
            .map(|i| if i % 3 == 0 { 1.0 } else { 0.0 })
            .collect();
        assert_eq!(separation(&x, &y), separation(&x, &y));
    }

    // The tests below are of release 0.9.0, step 42c: the first weekly run of the mutation job
    // on this file (45437619962, 24 November 2026) left some hundred and fifty changes of it
    // that the four tests above, and every fit of `glm`, let pass.

    /// Whole numbers in a fixed order, the same on every machine.
    struct Draws(u64);

    impl Draws {
        fn next(&mut self) -> u64 {
            self.0 = self
                .0
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            self.0 >> 33
        }

        fn between(&mut self, low: i64, high: i64) -> i64 {
            low + (self.next() % (high - low + 1) as u64) as i64
        }
    }

    fn dot_whole(a: &[i64], b: &[i64]) -> i64 {
        a.iter().zip(b).map(|(p, q)| p * q).sum()
    }

    fn cross(a: &[i64], b: &[i64]) -> Vec<i64> {
        vec![
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0],
        ]
    }

    /// The directions an edge of the cone `Zβ ≥ 0` can have, in whole numbers. For a design of
    /// full column rank the cone holds no line, so if it holds anything but zero it has an
    /// edge, and an edge in k dimensions is at right angles to k − 1 independent rows: the
    /// candidates are 1 for one column, the normal of each row for two and the cross product
    /// of each pair of rows for three, each with either sign.
    fn candidates(z: &[Vec<i64>]) -> Vec<Vec<i64>> {
        let mut found = Vec::new();
        match z[0].len() {
            1 => found.push(vec![1]),
            2 => {
                for row in z {
                    found.push(vec![row[1], -row[0]]);
                }
            }
            3 => {
                for (i, a) in z.iter().enumerate() {
                    for b in &z[i + 1..] {
                        found.push(cross(a, b));
                    }
                }
            }
            other => panic!("no search for {other} columns"),
        }
        found.retain(|c| c.iter().any(|&v| v != 0));
        found
    }

    /// Whether the rows are of full column rank, in whole numbers.
    fn full_rank(z: &[Vec<i64>]) -> bool {
        match z[0].len() {
            1 => true,
            2 => z
                .iter()
                .any(|a| z.iter().any(|b| a[0] * b[1] != a[1] * b[0])),
            _ => candidates(z)
                .iter()
                .any(|c| z.iter().any(|row| dot_whole(row, c) != 0)),
        }
    }

    /// Separation decided by trying every candidate direction, exactly.
    fn separated_by_search(z: &[Vec<i64>]) -> bool {
        candidates(z).iter().any(|c| {
            [1, -1]
                .iter()
                .any(|&sign| z.iter().all(|row| sign * dot_whole(row, c) >= 0))
        })
    }

    /// What a verdict of separation promises of its direction: it is not zero, no row lies on
    /// the wrong side of it, `diverging` names its entries that are not zero, its largest
    /// entry is 1 in the units of the columns at unit norm, and `rows` counts the rows
    /// strictly on the right side.
    fn check_direction(x: &[Vec<f64>], y: &[f64], verdict: &Separation, what: &str) {
        let Separation::Separated {
            direction,
            diverging,
            rows,
        } = verdict
        else {
            panic!("{what}: {verdict:?}");
        };
        let n = y.len();
        assert_eq!(direction.len(), x.len(), "{what}");
        let named: Vec<usize> = (0..x.len()).filter(|&j| direction[j] != 0.0).collect();
        assert_eq!(&named, diverging, "{what}");
        assert!(!named.is_empty(), "{what}");
        let largest = (0..x.len())
            .map(|j| {
                let norm = x[j].iter().map(|v| v * v).sum::<f64>().sqrt();
                (direction[j] * norm).abs()
            })
            .fold(0.0_f64, f64::max);
        assert!((largest - 1.0).abs() < 1e-12, "{what}: largest {largest}");
        let mut strictly = 0;
        for i in 0..n {
            let sign = if y[i] == 1.0 { 1.0 } else { -1.0 };
            let side: f64 = (0..x.len()).map(|j| sign * x[j][i] * direction[j]).sum();
            assert!(side >= -1e-9, "{what}: row {i} at {side}");
            if side > SUPPORT_TOLERANCE {
                strictly += 1;
            }
        }
        assert!(strictly >= 1, "{what}");
        assert_eq!(*rows, strictly, "{what}");
    }

    #[test]
    fn the_verdict_is_that_of_a_search_over_every_direction() {
        // Small designs in whole numbers with many ties, where separation, quasi-separation
        // and overlap are all common, each decided twice: by the simplex, and by trying every
        // direction an edge of the cone can have.
        let mut draws = Draws(20_261_125);
        let (mut separated, mut overlapping) = ([0usize; 4], [0usize; 4]);
        let mut tried = 0;
        while tried < 1_500 {
            let k = 1 + (tried % 3);
            let n = draws.between(k as i64 + 1, 12) as usize;
            let mut columns: Vec<Vec<i64>> = vec![vec![1; n]];
            for _ in 1..k {
                columns.push((0..n).map(|_| draws.between(-3, 3)).collect());
            }
            // Half of the outcomes follow a cut of the regressors with one row turned, so
            // that the two answers are about as common; the others are as they come.
            let cut = draws.between(-2, 2);
            let turned = draws.between(0, n as i64 - 1) as usize;
            let by_cut = draws.between(0, 1) == 1;
            let outcomes: Vec<i64> = (0..n)
                .map(|i| {
                    if by_cut {
                        let score: i64 = columns[1..].iter().map(|c| c[i]).sum();
                        i64::from((score > cut) != (i == turned && draws.between(0, 2) == 0))
                    } else {
                        draws.between(0, 1)
                    }
                })
                .collect();
            let z: Vec<Vec<i64>> = (0..n)
                .map(|i| {
                    let sign = if outcomes[i] == 1 { 1 } else { -1 };
                    columns.iter().map(|c| sign * c[i]).collect()
                })
                .collect();
            if !full_rank(&z) {
                continue;
            }
            tried += 1;
            let x: Vec<Vec<f64>> = columns
                .iter()
                .map(|c| c.iter().map(|&v| v as f64).collect())
                .collect();
            let y: Vec<f64> = outcomes.iter().map(|&v| v as f64).collect();
            let verdict = separation(&design(&x), &y);
            let what = format!("case {tried}: x {columns:?}, y {outcomes:?}");
            if separated_by_search(&z) {
                check_direction(&x, &y, &verdict, &what);
                separated[k] += 1;
            } else {
                assert_eq!(verdict, Separation::Overlap, "{what}");
                overlapping[k] += 1;
            }
        }
        // Both answers were met often with two and with three columns.
        for k in 2..=3 {
            assert!(separated[k] >= 100, "{separated:?}");
            assert!(overlapping[k] >= 100, "{overlapping:?}");
        }
    }

    /// A design of `k` columns and whole entries: a constant and `k − 1` regressors.
    fn drawn_rows(draws: &mut Draws, n: usize, k: usize) -> Vec<Vec<i64>> {
        (0..n)
            .map(|_| {
                let mut row = vec![1];
                row.extend((1..k).map(|_| draws.between(-5, 5)));
                row
            })
            .collect()
    }

    fn by_columns(rows: &[Vec<i64>]) -> Vec<Vec<f64>> {
        (0..rows[0].len())
            .map(|j| rows.iter().map(|row| row[j] as f64).collect())
            .collect()
    }

    /// Seeds and numbers of columns for designs on which the simplex takes more steps than
    /// the interval at which it rebuilds its basis, some of them several times over.
    fn long_runs() -> Vec<(u64, usize)> {
        let mut runs = Vec::new();
        for (index, k) in [14usize, 24, 33, 40, 48, 64].into_iter().enumerate() {
            for repeat in 0..4 {
                runs.push((10 * index as u64 + repeat, k));
            }
        }
        runs
    }

    #[test]
    fn a_long_run_of_the_simplex_finds_the_overlap_it_was_built_with() {
        // Many columns, so that the simplex takes more steps than the interval at which it
        // rebuilds its basis. The outcomes overlap by construction: with weights of 1 and 2 on
        // the rows of outcome 0 and of 1 on those of outcome 1, the weighted sums of the two
        // groups are equal in every column, which is the alternative of Stiemke's theorem.
        for (seed, k) in long_runs() {
            let mut draws = Draws(seed);
            let zeros = drawn_rows(&mut draws, 60 + 2 * k, k);
            let weights: Vec<i64> = (0..zeros.len()).map(|i| 1 + (i % 2) as i64).collect();
            let total: i64 = weights.iter().sum();
            let mut ones = drawn_rows(&mut draws, total as usize - 1, k);
            let balance: Vec<i64> = (0..k)
                .map(|j| {
                    let of_zeros: i64 = zeros.iter().zip(&weights).map(|(r, w)| w * r[j]).sum();
                    let of_ones: i64 = ones.iter().map(|r| r[j]).sum();
                    of_zeros - of_ones
                })
                .collect();
            assert_eq!(balance[0], 1);
            ones.push(balance);
            let mut rows = zeros.clone();
            rows.extend(ones.iter().cloned());
            let y: Vec<f64> = (0..rows.len())
                .map(|i| if i < zeros.len() { 0.0 } else { 1.0 })
                .collect();
            let verdict = separation(&design(&by_columns(&rows)), &y);
            assert_eq!(verdict, Separation::Overlap, "{k} columns");
        }
    }

    #[test]
    fn a_long_run_of_the_simplex_finds_the_direction_it_was_built_with() {
        // The outcome is 1 where a drawn combination of the columns is above zero, so the
        // outcomes are separated along it; rows on the cut make the separation quasi-complete.
        for (seed, k) in long_runs() {
            let mut draws = Draws(100 + seed);
            let rows = drawn_rows(&mut draws, 220 + 4 * k, k);
            let along: Vec<i64> = (0..k).map(|_| draws.between(-3, 3)).collect();
            let y: Vec<f64> = rows
                .iter()
                .map(|row| if dot_whole(row, &along) > 0 { 1.0 } else { 0.0 })
                .collect();
            assert!(y.contains(&1.0) && y.contains(&0.0));
            let x = by_columns(&rows);
            let verdict = separation(&design(&x), &y);
            check_direction(&x, &y, &verdict, &format!("{k} columns"));
        }
    }

    /// A design of `k` columns in which each drawn row is met twice, once with each outcome.
    fn mirrored(k: usize) -> (Vec<Vec<f64>>, Vec<f64>) {
        let mut draws = Draws(7);
        let half = drawn_rows(&mut draws, 80 + 2 * k, k);
        let mut rows = half.clone();
        rows.extend(half.iter().cloned());
        let y: Vec<f64> = (0..rows.len())
            .map(|i| if i < half.len() { 0.0 } else { 1.0 })
            .collect();
        (by_columns(&rows), y)
    }

    #[test]
    fn every_row_met_with_both_outcomes_overlaps() {
        // Each row twice, once with each outcome: the right-hand side of the program is zero
        // and every step of the simplex is one that does not move. The outcomes overlap, with
        // equal weights. With 26 and 30 columns there are more such steps than the run after
        // which the lowest row enters (Bland's rule), several hundred of them under it.
        for k in [12, 26, 30] {
            let (x, y) = mirrored(k);
            assert_eq!(
                separation(&design(&x), &y),
                Separation::Overlap,
                "{k} columns"
            );
        }
    }

    #[test]
    fn a_design_with_no_step_that_moves_is_never_called_separated() {
        // A limit found with this test (release 0.9.0, step 42c; docs/known_issues.md): with
        // forty columns and more of drawn whole numbers, each row met with both outcomes, the
        // hundreds of steps that do not move leave reduced costs that are rounding and lie
        // between the tolerance for entering and the tolerance for a pivot, and the simplex
        // ends without an answer. The fit then goes ahead unchecked, which is right here,
        // since the outcomes overlap. What must not be given is a verdict of separation.
        for k in [40, 70] {
            let (x, y) = mirrored(k);
            let verdict = separation(&design(&x), &y);
            assert!(
                matches!(verdict, Separation::Overlap | Separation::Unsettled),
                "{k} columns: {verdict:?}"
            );
        }
    }

    #[test]
    fn matched_pairs_with_a_dummy_for_each_pair_overlap() {
        // The design in which such balance is met in practice: a case and a control in each
        // pair, a constant, a dummy for every pair but the first and a regressor. Every
        // column but the regressor's has a right-hand side of zero.
        for pairs in [10usize, 60, 200] {
            let mut draws = Draws(pairs as u64);
            let n = 2 * pairs;
            let mut x: Vec<Vec<f64>> = vec![vec![1.0; n]];
            for pair in 1..pairs {
                x.push(
                    (0..n)
                        .map(|i| if i / 2 == pair { 1.0 } else { 0.0 })
                        .collect(),
                );
            }
            x.push((0..n).map(|_| draws.between(-5, 5) as f64).collect());
            let y: Vec<f64> = (0..n).map(|i| (i % 2) as f64).collect();
            assert_eq!(
                separation(&design(&x), &y),
                Separation::Overlap,
                "{pairs} pairs"
            );
        }
    }

    #[test]
    fn no_rows_or_no_columns_overlap() {
        assert_eq!(
            separation(&Matrix::from_fn(0, 2, |_, _| 0.0), &[]),
            Separation::Overlap
        );
        assert_eq!(
            separation(&Matrix::from_fn(3, 0, |_, _| 0.0), &[0.0, 1.0, 1.0]),
            Separation::Overlap
        );
    }

    #[test]
    fn a_constant_alone_separates_outcomes_that_are_all_alike() {
        let x = vec![vec![1.0; 5]];
        for outcome in [0.0, 1.0] {
            let y = [outcome; 5];
            let verdict = separation(&design(&x), &y);
            check_direction(&x, &y, &verdict, "all alike");
            let Separation::Separated {
                direction, rows, ..
            } = verdict
            else {
                unreachable!()
            };
            // Towards the outcome met: up for ones, down for zeros, in the units of the column.
            let unit = 1.0 / 5.0_f64.sqrt();
            assert_eq!(direction, vec![if outcome == 1.0 { unit } else { -unit }]);
            assert_eq!(rows, 5);
        }
    }

    /// The basis whose r-th column is `columns[r]`, inverted by `rebuild`.
    fn rebuilt(columns: &[Vec<f64>]) -> Option<Vec<f64>> {
        let k = columns.len();
        let basis: Vec<usize> = (0..k).collect();
        rebuild(&basis, k, &|var, out: &mut [f64]| {
            out.copy_from_slice(&columns[var])
        })
    }

    #[test]
    fn a_rebuilt_inverse_is_the_inverse_of_the_basis() {
        // Bases in whole numbers of one to six columns, some with zeros on the diagonal so
        // that rows must change places: B⁻¹B and BB⁻¹ are the identity, where row r of the
        // inverse belongs to the r-th basic column.
        let mut draws = Draws(8);
        let mut inverted = 0;
        for case in 0..400 {
            let k = 1 + case % 6;
            let mut columns: Vec<Vec<f64>> = (0..k)
                .map(|_| (0..k).map(|_| draws.between(-4, 4) as f64).collect())
                .collect();
            if case % 3 == 0 {
                columns[0][0] = 0.0;
            }
            let Some(inverse) = rebuilt(&columns) else {
                continue;
            };
            inverted += 1;
            for r in 0..k {
                for c in 0..k {
                    let wanted = if r == c { 1.0 } else { 0.0 };
                    let left: f64 = (0..k).map(|j| inverse[r * k + j] * columns[c][j]).sum();
                    let right: f64 = (0..k).map(|j| columns[j][r] * inverse[j * k + c]).sum();
                    assert!((left - wanted).abs() < 1e-9, "case {case}: {columns:?}");
                    assert!((right - wanted).abs() < 1e-9, "case {case}: {columns:?}");
                }
            }
        }
        assert!(inverted >= 250, "{inverted}");
    }

    #[test]
    fn a_rebuilt_inverse_of_a_known_basis() {
        // Columns (0, 2, 1), (1, 0, 0) and (0, 0, 4): the first pivot is in the second row.
        let inverse = rebuilt(&[
            vec![0.0, 2.0, 1.0],
            vec![1.0, 0.0, 0.0],
            vec![0.0, 0.0, 4.0],
        ])
        .unwrap();
        assert_eq!(
            inverse,
            vec![0.0, 0.5, 0.0, 1.0, 0.0, 0.0, 0.0, -0.125, 0.25]
        );
        assert_eq!(rebuilt(&[vec![-4.0]]).unwrap(), vec![-0.25]);
    }

    #[test]
    fn a_basis_that_lost_its_rank_is_not_rebuilt() {
        assert_eq!(rebuilt(&[vec![0.0]]), None);
        assert_eq!(rebuilt(&[vec![1.0, 2.0], vec![2.0, 4.0]]), None);
        assert_eq!(
            rebuilt(&[
                vec![1.0, 0.0, 1.0],
                vec![0.0, 1.0, 1.0],
                vec![1.0, 1.0, 2.0]
            ]),
            None
        );
        // A pivot of one part in 10¹⁴ is the limit; just above it the basis is inverted.
        assert_eq!(rebuilt(&[vec![1e-14]]), None);
        assert!(rebuilt(&[vec![1.0000001e-14]]).is_some());
    }
}
