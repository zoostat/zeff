//! The least-squares line of one variable on another, with its classical confidence band
//! (Language Reference, section 7.6, "Statistical layers": `scatter(fit: linear)` is fitted
//! by the kernel's least squares, the QR `reg` also uses, with a classical band; release
//! 0.7.0, step 29b), which the shorthand `scatter` of the `plot` package draws. The line is
//! that of R's `lm(y ~ x)`, and the band that of `predict(interval = "confidence")`: at each
//! place the fitted value, and to either side the quantile of Student's t on `n - 2` degrees
//! of freedom times the standard error of the fitted value there. The band is taken at 80
//! places from the smallest value of `x` to the largest, equally apart, as ggplot2 takes it.

use zeff_core::value::Distribution;
use zeff_kernels::matrix::Matrix;
use zeff_kernels::qr::least_squares;
use zeff_kernels::triangular::solve_upper_transposed;
use zeff_numeric::maths::sqrt;

/// At how many places the line and its band are taken.
pub const PLACES: usize = 80;

/// The level of the band where none is given.
pub const LEVEL: f64 = 0.95;

/// The pivot tolerance of section 7.2 of the Language Reference, which `reg` uses: a column
/// whose remaining norm is below this share of its norm is collinear with those before it.
const PIVOT_TOLERANCE: f64 = 1e-7;

/// The rows the tall QR takes at a time, as elsewhere in the engine: the order of the
/// reduction is fixed (Engine and Project Specification, section 8.3, point 3).
const BLOCK_ROWS: usize = 256;

/// A fitted line with its band.
#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    pub intercept: f64,
    pub slope: f64,
    /// The residual standard deviation, on `df` degrees of freedom.
    pub sigma: f64,
    /// The residual degrees of freedom, the number of rows less two.
    pub df: usize,
    /// The level of the band.
    pub level: f64,
    /// The places, from the smallest value of `x` to the largest.
    pub x: Vec<f64>,
    /// The fitted value at each place.
    pub y: Vec<f64>,
    /// The lower and the upper limit of the confidence interval of the fitted value at each
    /// place.
    pub lower: Vec<f64>,
    pub upper: Vec<f64>,
}

/// The least-squares line of `y` on `x`, with its confidence band at `level`. `None` where
/// there are fewer than three rows, which leave no degree of freedom for the band, or one
/// value of `x` alone, through which no line is determined. The values are finite, one of
/// `y` for each of `x`.
pub fn line(x: &[f64], y: &[f64], level: f64) -> Option<Line> {
    let n = x.len();
    if n < 3 || y.len() != n {
        return None;
    }
    let design = Matrix::from_columns(&[vec![1.0; n], x.to_vec()]);
    let fit = least_squares(&design, y, PIVOT_TOLERANCE, BLOCK_ROWS);
    if fit.rank < 2 {
        return None;
    }
    let (intercept, slope) = (fit.coefficients[0]?, fit.coefficients[1]?);
    let df = n - 2;
    let sigma = sqrt(fit.residual_sum_of_squares / df as f64);
    let t = zeff_dist::families::quantile(
        &Distribution::Family {
            name: "t",
            params: vec![("df", df as f64)],
        },
        1.0 - (1.0 - level) / 2.0,
    );
    let (lo, hi) = x
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), v| {
            (lo.min(*v), hi.max(*v))
        });
    let step = (hi - lo) / (PLACES - 1) as f64;
    let mut out = Line {
        intercept,
        slope,
        sigma,
        df,
        level,
        x: Vec::with_capacity(PLACES),
        y: Vec::with_capacity(PLACES),
        lower: Vec::with_capacity(PLACES),
        upper: Vec::with_capacity(PLACES),
    };
    for k in 0..PLACES {
        let at = if k == PLACES - 1 {
            hi
        } else {
            lo + k as f64 * step
        };
        // The leverage of the place: the squared length of R⁻ᵀ applied to its row of the
        // design, the columns in the order the QR accepted them.
        let row = [1.0, at];
        let pivoted: Vec<f64> = fit.order.iter().take(fit.rank).map(|&c| row[c]).collect();
        let z = solve_upper_transposed(&fit.r, &pivoted).ok()?;
        let leverage: f64 = z.iter().map(|v| v * v).sum();
        let fitted = intercept + slope * at;
        let reach = t * sigma * sqrt(leverage);
        out.x.push(at);
        out.y.push(fitted);
        out.lower.push(fitted - reach);
        out.upper.push(fitted + reach);
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(found: f64, wanted: f64) -> bool {
        (found - wanted).abs() <= 1e-9 * wanted.abs().max(1.0)
    }

    /// The fit, and the place, the fitted value and the two limits at the first, the
    /// fortieth and the last of the 80 places, against R 4.6.0: `lm(y ~ x)` and
    /// `predict(m, data.frame(x = seq(min(x), max(x), length.out = 80)), interval =
    /// "confidence")`.
    type Reference = ((f64, f64, f64, usize), [(usize, f64, f64, f64, f64); 3]);

    fn check(x: &[f64], y: &[f64], reference: Reference) {
        let found = line(x, y, LEVEL).expect("a line");
        let ((intercept, slope, sigma, df), places) = reference;
        assert!(close(found.intercept, intercept), "{}", found.intercept);
        assert!(close(found.slope, slope), "{}", found.slope);
        assert!(close(found.sigma, sigma), "{}", found.sigma);
        assert_eq!(found.df, df);
        assert_eq!(found.x.len(), PLACES);
        for (k, at, fitted, lower, upper) in places {
            assert!(close(found.x[k - 1], at), "{k}: {}", found.x[k - 1]);
            assert!(close(found.y[k - 1], fitted), "{k}: {}", found.y[k - 1]);
            assert!(
                close(found.lower[k - 1], lower),
                "{k}: {}",
                found.lower[k - 1]
            );
            assert!(
                close(found.upper[k - 1], upper),
                "{k}: {}",
                found.upper[k - 1]
            );
        }
    }

    #[test]
    fn the_line_and_its_band_are_those_of_lm_and_predict() {
        check(
            &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0],
            &[2.1, 3.9, 6.2, 7.8, 9.0, 11.5, 13.1, 16.4],
            (
                (0.00714285714285672, 1.94285714285714, 0.529600028772838, 6),
                [
                    (1, 1.0, 1.95, 1.11351009613819, 2.78648990386181),
                    (
                        40,
                        4.45569620253165,
                        8.66392405063291,
                        8.2056740218147,
                        9.12217407945112,
                    ),
                    (80, 8.0, 15.55, 14.7135100961382, 16.3864899038618),
                ],
            ),
        );
        // Years, far from zero: the intercept is large and the columns nearly collinear.
        let years: Vec<f64> = (0..12).map(|k| 2010.0 + f64::from(k)).collect();
        let found = line(
            &years,
            &[
                21.9, 22.6, 23.5, 23.9, 25.1, 26.0, 26.7, 27.7, 28.9, 30.0, 30.6, 30.7,
            ],
            LEVEL,
        )
        .expect("a line");
        assert!(
            (found.intercept - -1719.83018648033).abs() < 1e-7,
            "{}",
            found.intercept
        );
        assert!(close(found.slope, 0.866433566433636));
        assert!(close(found.sigma, 0.30258020185516));
        for (k, fitted, lower, upper) in [
            (1, 21.7012820512816, 21.3351823484064, 22.0673817541567),
            (40, 26.4063453424211, 26.2116836740908, 26.6010070107513),
            (80, 31.2320512820516, 30.8659515791764, 31.5981509849267),
        ] {
            assert!(
                (found.y[k - 1] - fitted).abs() < 1e-9,
                "{k}: {}",
                found.y[k - 1]
            );
            assert!((found.lower[k - 1] - lower).abs() < 1e-9, "{k}");
            assert!((found.upper[k - 1] - upper).abs() < 1e-9, "{k}");
        }
        // Three rows: one degree of freedom, and a wide band.
        check(
            &[1.0, 2.0, 3.0],
            &[1.0, 3.0, 2.0],
            (
                (1.0, 0.5, 1.22474487139159, 1),
                [
                    (1, 1.0, 1.5, -12.7059687630582, 15.7059687630582),
                    (
                        40,
                        1.9873417721519,
                        1.99367088607595,
                        -6.9920522931586,
                        10.9793940653105,
                    ),
                    (80, 3.0, 2.5, -11.7059687630582, 16.7059687630582),
                ],
            ),
        );
        // One value far from the others.
        check(
            &[0.001, 0.002, 0.004, 0.008, 1000.0],
            &[5.0, 4.0, 6.0, 3.0, 7.0],
            (
                (4.49999225001938, 0.00250000124988437, 1.29099864449306, 3),
                [
                    (
                        1,
                        0.001,
                        4.49999475002063,
                        2.44572216759508,
                        6.55426733244617,
                    ),
                    (
                        40,
                        493.671392405063,
                        5.73417134806419,
                        3.45475981607057,
                        8.01358288005781,
                    ),
                    (
                        80,
                        1000.0,
                        6.99999349990375,
                        2.8914596336696,
                        11.1085273661379,
                    ),
                ],
            ),
        );
    }

    #[test]
    fn a_line_through_every_row_has_a_band_of_no_width() {
        let found = line(&[1.0, 2.0, 3.0, 4.0], &[2.0, 4.0, 6.0, 8.0], LEVEL).expect("a line");
        assert!(found.intercept.abs() < 1e-12 && (found.slope - 2.0).abs() < 1e-12);
        for k in 0..PLACES {
            assert!((found.upper[k] - found.lower[k]).abs() < 1e-10);
            assert!((found.y[k] - 2.0 * found.x[k]).abs() < 1e-10);
        }
    }

    #[test]
    fn too_few_rows_and_one_value_of_x_have_no_line() {
        assert!(line(&[], &[], LEVEL).is_none());
        assert!(line(&[1.0, 2.0], &[1.0, 2.0], LEVEL).is_none());
        assert!(line(&[2.0, 2.0, 2.0, 2.0], &[1.0, 2.0, 3.0, 4.0], LEVEL).is_none());
        assert!(line(&[1.0, 2.0, 3.0], &[1.0, 2.0], LEVEL).is_none());
    }

    #[test]
    fn a_wider_level_has_a_wider_band_and_the_places_run_from_end_to_end() {
        let (x, y) = ([1.0, 2.0, 3.0, 4.0, 5.0], [1.0, 3.0, 2.0, 5.0, 4.0]);
        let (narrow, wide) = (line(&x, &y, 0.9).unwrap(), line(&x, &y, 0.99).unwrap());
        for k in 0..PLACES {
            assert!(wide.upper[k] - wide.lower[k] > narrow.upper[k] - narrow.lower[k]);
            assert!(narrow.lower[k] < narrow.y[k] && narrow.y[k] < narrow.upper[k]);
        }
        assert_eq!((narrow.x[0], narrow.x[PLACES - 1]), (1.0, 5.0));
        // The last place is the largest value itself, not the sum of the steps to it, which
        // from 0.07 to 0.93 falls short of it in the last digit.
        let odd = line(&[0.07, 0.5, 0.93], &[1.0, 3.0, 2.0], 0.9).unwrap();
        assert_eq!((odd.x[0], odd.x[PLACES - 1]), (0.07, 0.93));
        assert!(narrow.x.windows(2).all(|pair| pair[0] < pair[1]));
        // The band is narrowest at the mean of `x`, here 3.
        let widths: Vec<f64> = (0..PLACES)
            .map(|k| narrow.upper[k] - narrow.lower[k])
            .collect();
        let narrowest = widths
            .iter()
            .enumerate()
            .min_by(|a, b| a.1.total_cmp(b.1))
            .map(|(k, _)| narrow.x[k])
            .unwrap();
        assert!((narrowest - 3.0).abs() < 0.03, "{narrowest}");
    }
}
