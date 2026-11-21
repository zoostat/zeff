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
}
