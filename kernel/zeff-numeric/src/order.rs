//! Order statistics and weighted statistics (Language Reference, section 4.1; the owner's
//! decision of 19 October 2026 on weighted statistics): the nine sample-quantile definitions of
//! Hyndman and Fan (1996) as R's `quantile(type=)` computes them, the median, and the weighted
//! mean, variance, sum and quantile under frequency, analytic and sampling weights.

/// R's `quantile(x, probs = p, type = kind)` on `sorted`, which must be sorted ascending and
/// hold no not-a-number; `None` for no values or a type outside 1..=9. The arithmetic is R's own
/// (`stats:::quantile.default`), including its fuzz against rounding in the index, so that the
/// results agree to the bit.
pub fn quantile(sorted: &[f64], p: f64, kind: u8) -> Option<f64> {
    let n = sorted.len();
    if n == 0 || !(1..=9).contains(&kind) || !(0.0..=1.0).contains(&p) {
        return None;
    }
    let fuzz = 4.0 * f64::EPSILON;
    let nf = n as f64;
    // x[j] with R's 1-based index, clamped to the ends.
    let x = |j: i64| -> f64 { sorted[j.clamp(1, n as i64) as usize - 1] };
    if kind <= 3 {
        let nppm = if kind == 3 { nf * p - 0.5 } else { nf * p };
        let j = (nppm + fuzz).floor();
        let h = match kind {
            1 => {
                if nppm > j {
                    1.0
                } else {
                    0.0
                }
            }
            2 => {
                if nppm > j {
                    1.0
                } else {
                    0.5
                }
            }
            _ => {
                if nppm != j || (j as i64).rem_euclid(2) == 1 {
                    1.0
                } else {
                    0.0
                }
            }
        };
        let j = j as i64;
        return Some(if h == 0.0 {
            x(j)
        } else if h == 1.0 {
            x(j + 1)
        } else {
            (1.0 - h) * x(j) + h * x(j + 1)
        });
    }
    let (a, b) = match kind {
        4 => (0.0, 1.0),
        5 => (0.5, 0.5),
        6 => (0.0, 0.0),
        7 => (1.0, 1.0),
        8 => (1.0 / 3.0, 1.0 / 3.0),
        _ => (3.0 / 8.0, 3.0 / 8.0),
    };
    let nppm = a + p * (nf + 1.0 - a - b);
    let j = (nppm + fuzz).floor();
    let mut h = nppm - j;
    if h.abs() < fuzz {
        h = 0.0;
    }
    let j = j as i64;
    Some(if h == 0.0 {
        x(j)
    } else if h == 1.0 {
        x(j + 1)
    } else {
        (1.0 - h) * x(j) + h * x(j + 1)
    })
}

/// The median: `quantile` of type 7 at one half, the mean of the two middle values.
pub fn median(sorted: &[f64]) -> Option<f64> {
    quantile(sorted, 0.5, 7)
}

/// How weights enter a statistic (Stata's fweight, aweight and pweight).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeightKind {
    /// A row counted `w` times: N is the sum of the weights.
    Frequency,
    /// Precision weights: the mean is Σwx / Σw and the variance uses the weights scaled to sum
    /// to n, with divisor n − 1, as Stata's `summarize [aweight]`.
    Analytic,
    /// Survey weights: the mean is Σwx / Σw, the sum the weighted total Σwx, the variance as for
    /// analytic weights.
    Sampling,
}

/// Σwx / Σw, or `None` when the weights sum to zero or there is no value.
pub fn weighted_mean(values: &[f64], weights: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let total = crate::sum(weights);
    if total <= 0.0 {
        return None;
    }
    let products: Vec<f64> = values.iter().zip(weights).map(|(x, w)| x * w).collect();
    Some(crate::sum(&products) / total)
}

/// The weighted sum: Σwx under frequency and sampling weights; under analytic weights the
/// weights are scaled to sum to n first, so the sum is n · Σwx / Σw.
pub fn weighted_sum(values: &[f64], weights: &[f64], kind: WeightKind) -> f64 {
    let products: Vec<f64> = values.iter().zip(weights).map(|(x, w)| x * w).collect();
    let raw = crate::sum(&products);
    match kind {
        WeightKind::Analytic => {
            let total = crate::sum(weights);
            if total > 0.0 {
                raw * (values.len() as f64) / total
            } else {
                f64::NAN
            }
        }
        _ => raw,
    }
}

/// The weighted sample variance. Frequency weights replicate rows: Σw(x − m)² / (Σw − 1).
/// Analytic and sampling weights are scaled to sum to n: Σw'(x − m)² / (n − 1) with
/// w' = w · n / Σw, as Stata's `summarize [aweight]`. `None` when the divisor is not positive.
pub fn weighted_variance(values: &[f64], weights: &[f64], kind: WeightKind) -> Option<f64> {
    let n = values.len();
    if n == 0 {
        return None;
    }
    let total = crate::sum(weights);
    if total <= 0.0 {
        return None;
    }
    let mean = weighted_mean(values, weights)?;
    let (scale, divisor) = match kind {
        WeightKind::Frequency => (1.0, total - 1.0),
        _ => ((n as f64) / total, n as f64 - 1.0),
    };
    if divisor <= 0.0 {
        return None;
    }
    let terms: Vec<f64> = values
        .iter()
        .zip(weights)
        .map(|(x, w)| {
            let d = x - mean;
            w * scale * d * d
        })
        .collect();
    Some(crate::sum(&terms) / divisor)
}

/// The weighted quantile by cumulative weight: the smallest value whose cumulative weight reaches
/// `p` of the total (with `p = 0.5`, the weighted median). `pairs` are (value, weight) sorted by
/// value. Under frequency weights this is the type-1 quantile of the replicated sample.
pub fn weighted_quantile(pairs: &[(f64, f64)], p: f64) -> Option<f64> {
    if pairs.is_empty() || !(0.0..=1.0).contains(&p) {
        return None;
    }
    let weights: Vec<f64> = pairs.iter().map(|(_, w)| *w).collect();
    let total = crate::sum(&weights);
    if total <= 0.0 {
        return None;
    }
    let target = p * total;
    let mut cumulative = 0.0;
    for (value, weight) in pairs {
        cumulative += weight;
        if cumulative >= target && *weight > 0.0 {
            return Some(*value);
        }
    }
    pairs.last().map(|(v, _)| *v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_nine_types_on_a_small_sample() {
        // R: quantile(c(1, 2, 3, 4, 10), 0.5, type = k) is 3 for every type but 3 (SAS's
        // nearest-even order statistic, 2) and 4 (the linear interpolation of the empirical
        // distribution function, 2.5); at p = 0.1 type 7 interpolates to 1.4.
        let x = [1.0, 2.0, 3.0, 4.0, 10.0];
        for kind in 1..=9 {
            let want = match kind {
                3 => 2.0,
                4 => 2.5,
                _ => 3.0,
            };
            assert_eq!(quantile(&x, 0.5, kind), Some(want), "type {kind}");
        }
        assert_eq!(quantile(&x, 0.1, 7), Some(1.4));
        assert_eq!(quantile(&x, 0.1, 1), Some(1.0));
        assert_eq!(quantile(&x, 0.0, 7), Some(1.0));
        assert_eq!(quantile(&x, 1.0, 7), Some(10.0));
        // R: quantile(1:4, 0.5, type = 1) is 2, type 2 is 2.5, type 6 is 2.5.
        let y = [1.0, 2.0, 3.0, 4.0];
        assert_eq!(quantile(&y, 0.5, 1), Some(2.0));
        assert_eq!(quantile(&y, 0.5, 2), Some(2.5));
        assert_eq!(quantile(&y, 0.5, 6), Some(2.5));
        assert_eq!(median(&y), Some(2.5));
        assert_eq!(quantile(&[], 0.5, 7), None);
        assert_eq!(quantile(&y, 0.5, 10), None);
    }

    #[test]
    fn weighted_statistics_follow_the_decided_formulas() {
        let x = [1.0, 2.0, 4.0];
        let w = [1.0, 1.0, 2.0];
        // Replicated: 1, 2, 4, 4: mean 2.75, variance (1.75² + 0.75² + 1.25² + 1.25²) / 3.
        assert_eq!(weighted_mean(&x, &w), Some(2.75));
        let replicated = [1.0, 2.0, 4.0, 4.0];
        assert!(
            (weighted_variance(&x, &w, WeightKind::Frequency).unwrap()
                - crate::variance(&replicated).unwrap())
            .abs()
                < 1e-15
        );
        assert_eq!(weighted_sum(&x, &w, WeightKind::Frequency), 11.0);
        assert_eq!(weighted_sum(&x, &w, WeightKind::Sampling), 11.0);
        // Analytic: weights scaled to sum to 3 (0.75, 0.75, 1.5), divisor 2.
        let scaled: f64 = [
            0.75 * 1.75f64.powi(2),
            0.75 * 0.75f64.powi(2),
            1.5 * 1.25f64.powi(2),
        ]
        .iter()
        .sum::<f64>()
            / 2.0;
        assert!((weighted_variance(&x, &w, WeightKind::Analytic).unwrap() - scaled).abs() < 1e-15);
        assert!((weighted_sum(&x, &w, WeightKind::Analytic) - 11.0 * 3.0 / 4.0).abs() < 1e-15);
        assert_eq!(
            weighted_quantile(&[(1.0, 1.0), (2.0, 1.0), (4.0, 2.0)], 0.5),
            Some(2.0)
        );
        assert_eq!(
            weighted_quantile(&[(1.0, 1.0), (2.0, 1.0), (4.0, 2.0)], 0.75),
            Some(4.0)
        );
        assert_eq!(weighted_mean(&[], &[]), None);
        assert_eq!(
            weighted_variance(&[1.0], &[1.0], WeightKind::Frequency),
            None
        );
    }
}
