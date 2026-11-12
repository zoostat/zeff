//! The estimated marginal means of a term of a fitted linear model (release 0.7.0, step 19b;
//! open decision #104, option A): for each level of a category, or each cell of an interaction
//! of categories, the model's prediction averaged with equal weight over the levels of every
//! other category of the model, each numeric variable held at its mean. They are what
//! `posthoc` compares after the factorial form of `anova`, as R's `emmeans` and Stata's
//! `pwcompare` do.
//!
//! A marginal mean is a linear function of the coefficients: one weight for each column of
//! the design. A column is a product of parts, so its weight is the product of what each part
//! contributes: a number its held value, a category of the term the weight the column gives
//! the level asked for, and any other category the mean of the column's weights over its
//! levels, which is the equal-weight average over them (zero under the coding by deviations
//! that `anova` uses, so a column that involves another category drops out).
//!
//! The estimate and the variance come from the triangular factor of the fit
//! (`zeff_core::value::ModelFit`), one forward substitution for each mean, without the
//! coefficients or their covariance matrix being formed. A mean that the rows cannot determine
//! (its level meets another category in a combination that no row holds, with their
//! interaction in the model) is marked and left out of the comparisons, as `emmeans` leaves
//! it out.

use zeff_core::value::{ColumnPart, ModelFit};
use zeff_kernels::{dot, norm, sum};

/// Relative to the size of a function's weights, how much of it may lie outside what the rows
/// determine and still be rounding: the tolerance of R's `estimability`, which `emmeans` uses.
pub const ESTIMABLE: f64 = 1e-8;

/// One marginal mean.
#[derive(Debug, Clone, PartialEq)]
pub struct Mean {
    /// The level of each category of the term, by its place among the category's levels.
    pub levels: Vec<usize>,
    /// `low`, or `low:A` for a cell of an interaction, the levels in the order of the term.
    pub label: String,
    pub estimate: f64,
    /// The solution u of R₁ᵀu = L: the variance of the mean is the residual mean square times
    /// uᵀu, and of a difference of two means that times the squared length of u₁ − u₂.
    pub root: Vec<f64>,
    /// Whether the rows determine the mean.
    pub estimable: bool,
}

/// The weight each column of the design has in the marginal mean at `levels` of the categories
/// `factors`.
pub fn weights(fit: &ModelFit, factors: &[usize], levels: &[usize]) -> Vec<f64> {
    fit.columns
        .iter()
        .map(|parts| {
            let mut product = 1.0;
            for part in parts {
                product *= match part {
                    ColumnPart::Number(held) => *held,
                    ColumnPart::Level { factor, weights } => {
                        match factors.iter().position(|f| f == factor) {
                            Some(at) => weights[levels[at]],
                            None => sum(weights) / weights.len() as f64,
                        }
                    }
                };
            }
            product
        })
        .collect()
}

/// The linear function of the coefficients with the weight `l` on each column of the design:
/// its estimate, the solution u its variance is read from, and whether the rows determine it.
pub fn linear(fit: &ModelFit, l: &[f64]) -> (f64, Vec<f64>, bool) {
    let columns = fit.order.len();
    let rank = fit.rank;
    let at = |row: usize, position: usize| fit.triangle[row * columns + position];
    // The weights on the scaled columns, in the order of the factor.
    let scaled: Vec<f64> = fit.order.iter().map(|&c| l[c] * fit.scales[c]).collect();
    let mut root = vec![0.0; rank];
    for i in 0..rank {
        let mut value = scaled[i];
        for (j, earlier) in root.iter().enumerate().take(i) {
            value -= at(j, i) * earlier;
        }
        root[i] = value / at(i, i);
    }
    // A column left out is a combination of the ones before it; the function is determined
    // when its weight on that column is the same combination of its other weights.
    let gaps: Vec<f64> = (rank..columns)
        .map(|position| {
            let mut gap = scaled[position];
            for (i, u) in root.iter().enumerate() {
                gap -= at(i, position) * u;
            }
            gap
        })
        .collect();
    let estimable = norm(&gaps) <= ESTIMABLE * norm(&scaled);
    (dot(&root, &fit.effects), root, estimable)
}

/// A level as a comparison names it, as the one-way form names its groups: the label alone
/// (`low`), and with the category's name when its levels are numbers (`dose=10`), which alone
/// would read as arithmetic in `10 − 20`.
fn level_label(fit: &ModelFit, factor: usize, level: usize) -> String {
    let category = &fit.factors[factor];
    let numbers = category
        .levels
        .iter()
        .all(|label| label.trim().parse::<f64>().is_ok());
    if numbers {
        format!("{}={}", category.name, category.levels[level])
    } else {
        category.levels[level].clone()
    }
}

/// The marginal means of the term made of the categories `factors`: every combination of
/// their levels, the first category changing slowest.
pub fn means(fit: &ModelFit, factors: &[usize]) -> Vec<Mean> {
    let sizes: Vec<usize> = factors
        .iter()
        .map(|&f| fit.factors[f].levels.len())
        .collect();
    let cells: usize = sizes.iter().product();
    (0..cells)
        .map(|cell| {
            let mut levels = vec![0; factors.len()];
            let mut rest = cell;
            for (at, &size) in sizes.iter().enumerate().rev() {
                levels[at] = rest % size;
                rest /= size;
            }
            let label = factors
                .iter()
                .zip(&levels)
                .map(|(&f, &level)| level_label(fit, f, level))
                .collect::<Vec<String>>()
                .join(":");
            let (estimate, root, estimable) = linear(fit, &weights(fit, factors, &levels));
            Mean {
                levels,
                label,
                estimate,
                root,
                estimable,
            }
        })
        .collect()
}

/// The difference of two marginal means, the first minus the second, and its standard error.
pub fn difference(fit: &ModelFit, first: &Mean, second: &Mean) -> (f64, f64) {
    let apart: Vec<f64> = first
        .root
        .iter()
        .zip(&second.root)
        .map(|(a, b)| a - b)
        .collect();
    (
        first.estimate - second.estimate,
        fit.residual_ms.sqrt() * norm(&apart),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use zeff_core::value::{ModelFactor, ModelTerm};

    /// Four rows, two in each level of a category coded +1 and −1, with the outcome `y`: the
    /// two columns are orthogonal and of length 2, so scaled by 1/2 they are orthonormal, the
    /// triangle is the identity and the effects are the columns' products with `y`.
    fn two_groups(y: [f64; 4], residual_ms: f64) -> ModelFit {
        ModelFit {
            outcome: "y".to_string(),
            formula: "y ~ a".to_string(),
            factors: vec![ModelFactor {
                name: "a".to_string(),
                levels: vec!["p".to_string(), "q".to_string()],
            }],
            numbers: Vec::new(),
            terms: vec![ModelTerm {
                label: "a".to_string(),
                factors: vec![0],
                numbers: Vec::new(),
            }],
            columns: vec![
                Vec::new(),
                vec![ColumnPart::Level {
                    factor: 0,
                    weights: vec![1.0, -1.0],
                }],
            ],
            scales: vec![0.5, 0.5],
            order: vec![0, 1],
            rank: 2,
            triangle: vec![1.0, 0.0, 0.0, 1.0],
            effects: vec![
                (y[0] + y[1] + y[2] + y[3]) / 2.0,
                (y[0] + y[1] - y[2] - y[3]) / 2.0,
            ],
            residual_df: 2.0,
            residual_ms,
            n: 4.0,
        }
    }

    #[test]
    fn two_groups_give_their_means_and_the_standard_error_of_the_t_test() {
        let fit = two_groups([1.0, 2.0, 4.0, 7.0], 2.5);
        let means = means(&fit, &[0]);
        assert_eq!(means.len(), 2);
        assert_eq!(means[0].label, "p");
        assert_eq!(means[1].label, "q");
        assert!(means.iter().all(|mean| mean.estimable));
        assert_eq!(means[0].estimate, 1.5);
        assert_eq!(means[1].estimate, 5.5);
        // The variance of a mean of two values is MS/2, of the difference MS·(1/2 + 1/2).
        assert_eq!(means[0].root, vec![0.5, 0.5]);
        let (difference, std_error) = difference(&fit, &means[0], &means[1]);
        assert_eq!(difference, -4.0);
        assert!((std_error - 2.5_f64.sqrt()).abs() <= 1e-15);
    }

    #[test]
    fn a_category_outside_the_term_is_averaged_and_a_number_is_held() {
        // Columns: intercept, a (two levels by deviations), b written in full (three
        // indicators' worth of weights), x held at 3, and a:x.
        let mut fit = two_groups([1.0, 2.0, 4.0, 7.0], 1.0);
        fit.factors.push(ModelFactor {
            name: "b".to_string(),
            levels: vec!["u".to_string(), "v".to_string(), "w".to_string()],
        });
        fit.columns = vec![
            Vec::new(),
            vec![ColumnPart::Level {
                factor: 0,
                weights: vec![1.0, -1.0],
            }],
            vec![ColumnPart::Level {
                factor: 1,
                weights: vec![1.0, 0.0, -1.0],
            }],
            vec![ColumnPart::Level {
                factor: 1,
                weights: vec![0.0, 0.0, 1.0],
            }],
            vec![ColumnPart::Number(3.0)],
            vec![
                ColumnPart::Level {
                    factor: 0,
                    weights: vec![1.0, -1.0],
                },
                ColumnPart::Number(3.0),
            ],
        ];
        // The mean of the second level of a: b averaged, by deviations to nothing and in full
        // to a third.
        assert_eq!(
            weights(&fit, &[0], &[1]),
            vec![1.0, -1.0, 0.0, 1.0 / 3.0, 3.0, -3.0]
        );
        // A cell of a and b, in the order asked for.
        assert_eq!(
            weights(&fit, &[1, 0], &[2, 0]),
            vec![1.0, 1.0, -1.0, 1.0, 3.0, 3.0]
        );
        // Numeric levels carry the name of their category.
        fit.factors[1].levels = vec!["10".to_string(), "20".to_string(), "30.5".to_string()];
        assert_eq!(level_label(&fit, 1, 2), "b=30.5");
        assert_eq!(level_label(&fit, 0, 0), "p");
    }

    #[test]
    fn a_function_the_rows_do_not_determine_is_marked() {
        // A third column that repeats the second, so left out of the fit: its row of the
        // triangle says so (column 3 = column 2).
        let mut fit = two_groups([1.0, 2.0, 4.0, 7.0], 1.0);
        fit.columns.push(vec![ColumnPart::Number(1.0)]);
        fit.scales = vec![0.5, 0.5, 0.5];
        fit.order = vec![0, 1, 2];
        fit.triangle = vec![1.0, 0.0, 0.0, 0.0, 1.0, 1.0];
        // The rows determine the sum of the two coefficients of the columns that are one
        // column, and nothing else about them: a function with the same weight on both is
        // determined.
        let (estimate, _, estimable) = linear(&fit, &[1.0, 1.0, 1.0]);
        assert!(estimable);
        assert_eq!(estimate, 1.5);
        // A weight on the column left out alone: not determined.
        let (_, _, estimable) = linear(&fit, &[1.0, 1.0, 0.0]);
        assert!(!estimable);
        let (_, _, estimable) = linear(&fit, &[1.0, 0.0, 1.0]);
        assert!(!estimable);
        // Within the tolerance of rounding, still determined.
        let (_, _, estimable) = linear(&fit, &[1.0, 1.0, 1.0 + 1e-12]);
        assert!(estimable);
    }
}
