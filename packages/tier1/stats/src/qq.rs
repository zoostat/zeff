//! The pairs of a quantile-quantile plot (Language Reference, section 7.6, "Shorthands":
//! `qq_plot` draws the quantile pairs with a reference line; release 0.7.0, step 29d), which
//! the shorthand `qq_plot` of the `plot` package draws. Each value is paired with the
//! quantile of a distribution at the plotting position of its rank, R's `ppoints`; and the
//! reference line passes through the first and the third quartiles of the two, as R's
//! `qqline` does (open decision #99: R's positions and R's line at this release).

use zeff_core::value::Distribution;

/// The plotting positions of `n` values in order, R's `ppoints`: `(i - a) / (n + 1 - 2a)`
/// for `i` from 1 to `n`, with `a` three eighths up to ten values and one half beyond.
pub fn positions(n: usize) -> Vec<f64> {
    let a = if n <= 10 { 0.375 } else { 0.5 };
    (1..=n)
        .map(|i| (i as f64 - a) / (n as f64 + 1.0 - 2.0 * a))
        .collect()
}

/// The quantile of values in order, between the two nearest: R's default, type 7.
fn quantile(sorted: &[f64], p: f64) -> f64 {
    let at = (sorted.len() - 1) as f64 * p;
    let lo = at.floor() as usize;
    let hi = (lo + 1).min(sorted.len() - 1);
    let share = at - lo as f64;
    (1.0 - share) * sorted[lo] + share * sorted[hi]
}

/// The pairs of a quantile-quantile plot.
#[derive(Debug, Clone, PartialEq)]
pub struct Pairs {
    /// For each value, in the order given, the quantile of the distribution at the plotting
    /// position of its rank; values that are equal take their ranks in the order given.
    pub theoretical: Vec<f64>,
    /// The reference line, its intercept and its slope: through the first and the third
    /// quartiles of the distribution and of the values. `None` for fewer than two values,
    /// and where the two quartiles of the distribution are one or are not finite.
    pub line: Option<(f64, f64)>,
}

/// The pairs of the values, which are finite, with the quantiles of `d`.
pub fn pairs(values: &[f64], d: &Distribution) -> Pairs {
    let n = values.len();
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|a, b| values[*a].total_cmp(&values[*b]).then(a.cmp(b)));
    let mut theoretical = vec![f64::NAN; n];
    for (rank, p) in positions(n).into_iter().enumerate() {
        theoretical[order[rank]] = zeff_dist::families::quantile(d, p);
    }
    let line = (n >= 2)
        .then(|| {
            let sorted: Vec<f64> = order.iter().map(|at| values[*at]).collect();
            let (y1, y2) = (quantile(&sorted, 0.25), quantile(&sorted, 0.75));
            let (x1, x2) = (
                zeff_dist::families::quantile(d, 0.25),
                zeff_dist::families::quantile(d, 0.75),
            );
            (x1.is_finite() && x2.is_finite() && x2 > x1).then(|| {
                let slope = (y2 - y1) / (x2 - x1);
                (y1 - slope * x1, slope)
            })
        })
        .flatten();
    Pairs { theoretical, line }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn normal() -> Distribution {
        Distribution::Family {
            name: "normal",
            params: vec![("mean", 0.0), ("sd", 1.0)],
        }
    }

    fn close(found: f64, wanted: f64) -> bool {
        (found - wanted).abs() < 1e-11
    }

    #[test]
    fn the_positions_are_those_of_ppoints() {
        // R 4.6.0: `ppoints(1)`, `ppoints(3)`, and the ends of `ppoints(10)` and
        // `ppoints(11)`.
        assert_eq!(positions(0), Vec::<f64>::new());
        assert_eq!(positions(1), [0.5]);
        let three = positions(3);
        for (found, wanted) in three
            .iter()
            .zip([0.192307692307692, 0.5, 0.807692307692308])
        {
            assert!(close(*found, wanted));
        }
        let ten = positions(10);
        assert!(close(ten[0], 0.625 / 10.25) && close(ten[9], 9.625 / 10.25));
        let eleven = positions(11);
        assert!(close(eleven[0], 0.5 / 11.0) && close(eleven[10], 10.5 / 11.0));
    }

    #[test]
    fn the_pairs_and_the_line_are_those_of_qqnorm_and_qqline() {
        // `qqnorm(y, plot.it = FALSE)$x`, and the line through the quartiles.
        let scores = [1.0, 2.0, 2.0, 3.0, 3.0, 3.0, 4.0, 4.0, 5.0, 9.0];
        let found = pairs(&scores, &normal());
        for (found, wanted) in found.theoretical.iter().zip([
            -1.54663527139923,
            -1.000490545619315,
            -0.655423505234427,
            -0.375461770235518,
            -0.122580843888802,
            0.122580843888803,
            0.375461770235518,
            0.655423505234427,
            1.000490545619315,
            1.54663527139923,
        ]) {
            assert!(close(*found, wanted), "{found}");
        }
        let (intercept, slope) = found.line.unwrap();
        assert!(close(intercept, 3.125) && close(slope, 1.2972769411924));
        // Twelve values out of order: each has the quantile of its own rank.
        let mixed = [2.5, -1.0, 4.0, 0.5, 3.0, 7.0, 1.5, 8.0, 2.0, 6.0, 5.0, 9.5];
        let found = pairs(&mixed, &normal());
        for (found, wanted) in found.theoretical.iter().zip([
            -0.318639363964375,
            -1.731664396122245,
            0.104633455614075,
            -1.150349380376008,
            -0.104633455614075,
            0.812217801499913,
            -0.812217801499913,
            1.150349380376008,
            -0.548522282698098,
            0.548522282698098,
            0.318639363964375,
            1.731664396122246,
        ]) {
            assert!(close(*found, wanted), "{found}");
        }
        let (intercept, slope) = found.line.unwrap();
        assert!(close(intercept, 4.0625) && close(slope, 3.243192352981));
        // Against Student's t on 3 degrees of freedom: `qt(ppoints(12), 3)`.
        let t = Distribution::Family {
            name: "t",
            params: vec![("df", 3.0)],
        };
        let found = pairs(&mixed, &t);
        let mut sorted = found.theoretical.clone();
        sorted.sort_by(f64::total_cmp);
        for (at, wanted) in [
            (0, -2.558356936064456),
            (5, -0.113687733237301),
            (11, 2.558356936064457),
        ] {
            assert!((sorted[at] - wanted).abs() < 1e-9, "{}", sorted[at]);
        }
        let (intercept, slope) = found.line.unwrap();
        assert!((intercept - 4.0625).abs() < 1e-9 && (slope - 2.85987964418911).abs() < 1e-9);
    }

    #[test]
    fn one_value_has_its_pair_and_no_line_and_none_has_nothing() {
        let one = pairs(&[4.0], &normal());
        assert_eq!(one.theoretical, [0.0]);
        assert_eq!(one.line, None);
        let none = pairs(&[], &normal());
        assert!(none.theoretical.is_empty() && none.line.is_none());
        // Equal values take their ranks in the order given, and a line of no slope.
        let equal = pairs(&[2.0, 2.0, 2.0], &normal());
        assert!(equal.theoretical[0] < equal.theoretical[1]);
        assert!(equal.theoretical[1] < equal.theoretical[2]);
        assert_eq!(equal.line, Some((2.0, 0.0)));
    }
}
