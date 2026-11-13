//! The kernel estimate of a density (Language Reference, section 7.6, "Statistical layers":
//! `density`; release 0.7.0, step 28b), which the layer `density` of the `plot` package
//! draws. The estimate at a place is the mean, over the values, of a kernel centred on each:
//! it is computed exactly, value by value, where R's `density` bins the values and
//! convolves, so the two agree to about three places. The bandwidth is Silverman's rule of
//! thumb, R's `bw.nrd0`, unless it is given; each kernel is scaled so that the bandwidth is
//! its standard deviation, as R's are; and the estimate is taken at 512 places from three
//! bandwidths below the smallest value to three above the largest, as R takes it.

use zeff_core::value::Kernel;
use zeff_numeric::maths::{cos, exp, sqrt};

/// At how many places an estimate is taken.
pub const POINTS: usize = 512;

/// How many bandwidths beyond the smallest and the largest value the places reach.
pub const CUT: f64 = 3.0;

/// The quantile of values in order, between the two nearest: R's default.
fn quantile(sorted: &[f64], p: f64) -> f64 {
    let at = (sorted.len() - 1) as f64 * p;
    let lo = at.floor() as usize;
    let hi = (lo + 1).min(sorted.len() - 1);
    let share = at - lo as f64;
    (1.0 - share) * sorted[lo] + share * sorted[hi]
}

/// Silverman's rule of thumb for the bandwidth of a Gaussian kernel: nine tenths of the
/// smaller of the standard deviation and the interquartile range over 1.34, over the fifth
/// root of the number of values. Where that spread is zero it is the standard deviation;
/// where that is zero too, the size of the first value, and where that is zero, one: as R's
/// `bw.nrd0`. `None` for fewer than two values, which have no spread.
pub fn silverman(values: &[f64]) -> Option<f64> {
    let n = values.len();
    if n < 2 {
        return None;
    }
    let mean = values.iter().sum::<f64>() / n as f64;
    let deviation =
        sqrt(values.iter().map(|v| (v - mean) * (v - mean)).sum::<f64>() / (n - 1) as f64);
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let quartiles = (quantile(&sorted, 0.75) - quantile(&sorted, 0.25)) / 1.34;
    let mut spread = deviation.min(quartiles);
    if spread == 0.0 {
        spread = deviation;
    }
    if spread == 0.0 {
        spread = values[0].abs();
    }
    if spread == 0.0 {
        spread = 1.0;
    }
    Some(0.9 * spread * zeff_numeric::pow(n as f64, -0.2))
}

/// A kernel at `u` from its centre, with `bandwidth` as its standard deviation.
pub fn kernel(kernel: Kernel, u: f64, bandwidth: f64) -> f64 {
    let within = |reach: f64, shape: &dyn Fn(f64) -> f64| {
        let at = u.abs();
        if at < reach {
            shape(at / reach) / reach
        } else {
            0.0
        }
    };
    match kernel {
        Kernel::Gaussian => {
            let z = u / bandwidth;
            exp(-0.5 * z * z) / (bandwidth * sqrt(2.0 * std::f64::consts::PI))
        }
        Kernel::Rectangular => within(bandwidth * sqrt(3.0), &|_| 0.5),
        Kernel::Triangular => within(bandwidth * sqrt(6.0), &|t| 1.0 - t),
        Kernel::Epanechnikov => within(bandwidth * sqrt(5.0), &|t| 0.75 * (1.0 - t * t)),
        Kernel::Biweight => within(bandwidth * sqrt(7.0), &|t| {
            0.9375 * (1.0 - t * t) * (1.0 - t * t)
        }),
        Kernel::Cosine => {
            let pi = std::f64::consts::PI;
            within(bandwidth / sqrt(1.0 / 3.0 - 2.0 / (pi * pi)), &|t| {
                0.5 * (1.0 + cos(pi * t))
            })
        }
    }
}

/// The estimate of a density: its bandwidth, and its height at each of its places.
#[derive(Debug, Clone, PartialEq)]
pub struct Estimate {
    pub bandwidth: f64,
    pub x: Vec<f64>,
    pub y: Vec<f64>,
}

/// The kernel estimate of the density of `values`, which are finite: with `bandwidth`, or
/// Silverman's, times `adjust`, at [`POINTS`] places from [`CUT`] bandwidths below the
/// smallest value to as many above the largest. `None` for fewer than two values, and for a
/// bandwidth that is not above zero.
pub fn estimate(
    values: &[f64],
    bandwidth: Option<f64>,
    adjust: f64,
    of: Kernel,
) -> Option<Estimate> {
    if values.len() < 2 {
        return None;
    }
    let bandwidth = bandwidth.or_else(|| silverman(values))? * adjust;
    if !(bandwidth > 0.0 && bandwidth.is_finite()) {
        return None;
    }
    let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
    for &v in values {
        lo = lo.min(v);
        hi = hi.max(v);
    }
    let (from, to) = (lo - CUT * bandwidth, hi + CUT * bandwidth);
    let n = values.len() as f64;
    let x: Vec<f64> = (0..POINTS)
        .map(|i| from + (to - from) * i as f64 / (POINTS - 1) as f64)
        .collect();
    let y = x
        .iter()
        .map(|at| {
            values
                .iter()
                .map(|v| kernel(of, at - v, bandwidth))
                .sum::<f64>()
                / n
        })
        .collect();
    Some(Estimate { bandwidth, x, y })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The 200 values of `sapply(1:200, function(i) (i * 37) %% 101 + (i %% 7) * 0.25)`.
    fn a() -> Vec<f64> {
        (1..=200)
            .map(|i| f64::from((i * 37) % 101) + f64::from(i % 7) * 0.25)
            .collect()
    }

    /// The 57 values of `sapply(1:57, function(i) ((i * i) %% 23) / 4 + i / 100)`.
    fn b() -> Vec<f64> {
        (1..=57)
            .map(|i| f64::from((i * i) % 23) / 4.0 + f64::from(i) / 100.0)
            .collect()
    }

    #[test]
    fn the_bandwidth_is_that_of_bw_nrd0() {
        // R 4.6.0: `bw.nrd0` on each series.
        let mut c = vec![5.0; 40];
        c.extend([1.0, 9.0]);
        for (values, wanted) in [
            (a(), 9.0913003028051),
            (b(), 0.572892458379114),
            (c, 0.376509303608935),
            (vec![3.0; 10], 1.70358483009652),
            (vec![1.0, 4.0], 0.877047209290871),
            (vec![0.0, 0.0, 0.0], 0.722467405584208),
        ] {
            let found = silverman(&values).unwrap();
            assert!(
                (found / wanted - 1.0).abs() < 1e-13,
                "{found} against {wanted}"
            );
        }
        assert_eq!(silverman(&[2.5]), None);
        assert_eq!(silverman(&[]), None);
    }

    #[test]
    fn the_estimate_is_the_mean_of_the_kernels_at_each_place() {
        // The places are those of `density(b)`: 512 of them, from three bandwidths below
        // 0.23 to three above 5.04. At seven of them, for each kernel: the mean of the
        // kernel over the values as R computes it directly (`exact`), to twelve places,
        // and the value of `density()` itself, which bins the values, to three.
        let places = [0usize, 99, 199, 255, 299, 399, 511];
        let at = [
            -1.48867737513734,
            0.109146734994153,
            1.72311048260172,
            2.62693018126196,
            3.33707423020929,
            4.95103797781686,
            6.75867737513734,
        ];
        let reference: [(Kernel, [f64; 7], [f64; 7]); 6] = [
            (
                Kernel::Gaussian,
                [
                    0.000417997998549039,
                    0.125692404138292,
                    0.222427031448877,
                    0.188389106490863,
                    0.172354808533198,
                    0.0928170532473728,
                    0.000262313211759516,
                ],
                [
                    0.000419368432533764,
                    0.125699357300876,
                    0.222436353959559,
                    0.188386804431222,
                    0.172352225964787,
                    0.0928133126506322,
                    0.000263150972332575,
                ],
            ),
            (
                Kernel::Epanechnikov,
                [
                    0.0,
                    0.134138297957355,
                    0.232986017426868,
                    0.190829004229976,
                    0.169765282916529,
                    0.0878584183007365,
                    0.0,
                ],
                [
                    0.0,
                    0.134117404605999,
                    0.232949425289224,
                    0.190794055242535,
                    0.169719849678874,
                    0.0878441956533176,
                    0.0,
                ],
            ),
            (
                Kernel::Rectangular,
                [
                    0.0,
                    0.141442980341601,
                    0.238685029326452,
                    0.185643911698351,
                    0.167963539155651,
                    0.0884018627135007,
                    0.0,
                ],
                [
                    0.0,
                    0.141442980341601,
                    0.240869737875103,
                    0.186312256322745,
                    0.167797062716541,
                    0.0884018627135006,
                    0.0,
                ],
            ),
            (
                Kernel::Triangular,
                [
                    0.0,
                    0.12841128138122,
                    0.225786425798238,
                    0.188343554919123,
                    0.175846119877576,
                    0.0895477206642001,
                    0.0,
                ],
                [
                    0.0,
                    0.128412487056767,
                    0.225800459725206,
                    0.18836577224704,
                    0.175846119877576,
                    0.0896041620293002,
                    0.0,
                ],
            ),
            (
                Kernel::Biweight,
                [
                    0.0,
                    0.131009205337127,
                    0.229251593174503,
                    0.188932334783637,
                    0.170793390071037,
                    0.0894692969605809,
                    0.0,
                ],
                [
                    0.0,
                    0.131010273699434,
                    0.229248821275836,
                    0.188928305413915,
                    0.170795815384895,
                    0.0894664582330926,
                    0.0,
                ],
            ),
            (
                Kernel::Cosine,
                [
                    0.0,
                    0.130100402198413,
                    0.228012967933571,
                    0.188651900616908,
                    0.171112488667084,
                    0.0900100358164146,
                    0.0,
                ],
                [
                    0.0,
                    0.130101609787648,
                    0.228010976638862,
                    0.188649504186576,
                    0.171113005759033,
                    0.0900090263522034,
                    0.0,
                ],
            ),
        ];
        for (kernel, exact, binned) in reference {
            let found = estimate(&b(), None, 1.0, kernel).unwrap();
            assert_eq!((found.x.len(), found.y.len()), (POINTS, POINTS));
            assert!((found.bandwidth / 0.572892458379114 - 1.0).abs() < 1e-13);
            for (i, place) in places.iter().enumerate() {
                assert!((found.x[*place] - at[i]).abs() < 1e-12, "{kernel:?} at {i}");
                assert!(
                    (found.y[*place] - exact[i]).abs() < 1e-12,
                    "{kernel:?} at {i}: {} against {}",
                    found.y[*place],
                    exact[i]
                );
                assert!(
                    (found.y[*place] - binned[i]).abs() < 3e-3,
                    "{kernel:?} at {i}"
                );
            }
        }
    }

    #[test]
    fn the_estimate_is_never_below_zero_and_its_area_is_one() {
        for kernel in [
            Kernel::Gaussian,
            Kernel::Epanechnikov,
            Kernel::Rectangular,
            Kernel::Triangular,
            Kernel::Biweight,
            Kernel::Cosine,
        ] {
            for values in [a(), b(), vec![1.0, 4.0], vec![3.0; 10]] {
                let found = estimate(&values, None, 1.0, kernel).unwrap();
                assert!(found.y.iter().all(|y| *y >= 0.0));
                // The area under the line through the places, by the trapezium rule.
                let step = found.x[1] - found.x[0];
                let area: f64 = found
                    .y
                    .windows(2)
                    .map(|pair| (pair[0] + pair[1]) / 2.0 * step)
                    .sum();
                assert!((area - 1.0).abs() < 5e-3, "{kernel:?}: an area of {area}");
            }
        }
    }

    #[test]
    fn a_bandwidth_of_ones_own_an_adjustment_and_too_few_values() {
        // `density(b, adjust = 2)`: twice the bandwidth, and places three of those beyond.
        let wide = estimate(&b(), None, 2.0, Kernel::Gaussian).unwrap();
        assert!((wide.bandwidth - 1.14578491675823).abs() < 1e-13);
        assert!((wide.x[0] - -3.20735475027468).abs() < 1e-12);
        // A bandwidth that is given is that bandwidth, times the adjustment.
        let given = estimate(&b(), Some(0.5), 3.0, Kernel::Triangular).unwrap();
        assert_eq!(given.bandwidth, 1.5);
        assert!((given.x[0] - (0.23 - 4.5)).abs() < 1e-12);
        assert!((given.x[POINTS - 1] - (5.04 + 4.5)).abs() < 1e-12);
        // Each kernel has the bandwidth as its standard deviation: the second moment of the
        // estimate of two equal values is the square of the bandwidth.
        // Fewer than two values, and a bandwidth that is not above zero, have no estimate.
        assert_eq!(estimate(&[2.5], None, 1.0, Kernel::Gaussian), None);
        assert_eq!(estimate(&[], None, 1.0, Kernel::Gaussian), None);
        assert_eq!(estimate(&b(), Some(0.0), 1.0, Kernel::Gaussian), None);
        assert_eq!(estimate(&b(), None, 0.0, Kernel::Gaussian), None);
    }

    #[test]
    fn each_kernel_has_the_bandwidth_as_its_standard_deviation() {
        // The integral of u squared times the kernel, by the midpoint rule over ten
        // bandwidths: the square of the bandwidth, 1.7 squared.
        let bandwidth = 1.7;
        for kernel in [
            Kernel::Gaussian,
            Kernel::Epanechnikov,
            Kernel::Rectangular,
            Kernel::Triangular,
            Kernel::Biweight,
            Kernel::Cosine,
        ] {
            let steps = 200_000;
            let step = 20.0 * bandwidth / f64::from(steps);
            let (mut mass, mut second) = (0.0, 0.0);
            for i in 0..steps {
                let u = -10.0 * bandwidth + (f64::from(i) + 0.5) * step;
                let k = super::kernel(kernel, u, bandwidth);
                mass += k * step;
                second += u * u * k * step;
            }
            assert!((mass - 1.0).abs() < 1e-4, "{kernel:?}: a mass of {mass}");
            assert!(
                (second - bandwidth * bandwidth).abs() < 1e-3,
                "{kernel:?}: a second moment of {second}"
            );
        }
    }
}
