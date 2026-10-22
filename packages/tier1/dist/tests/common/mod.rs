//! The fourteen distributions of the stochastic class with their exact mean and variance, shared by
//! the quick recovery test and the nightly one.

use zeff_core::value::Distribution;
use zeff_dist::families;

/// Name, distribution, exact mean and exact variance.
pub fn recovery_cases() -> Vec<(String, Distribution, f64, f64)> {
    // A family, its parameters, its mean and its variance.
    type Case = (&'static str, Vec<(&'static str, f64)>, f64, f64);
    let cases: [Case; 12] = [
        ("normal", vec![("mean", 100.0), ("sd", 15.0)], 100.0, 225.0),
        (
            "lognormal",
            vec![("meanlog", 0.0), ("sdlog", 0.5)],
            (0.125f64).exp(),
            ((0.25f64).exp() - 1.0) * (0.25f64).exp(),
        ),
        ("uniform", vec![("min", 2.0), ("max", 5.0)], 3.5, 0.75),
        ("exponential", vec![("rate", 0.1)], 10.0, 100.0),
        ("t", vec![("df", 24.0)], 0.0, 24.0 / 22.0),
        ("chi2", vec![("df", 5.0)], 5.0, 10.0),
        (
            "f",
            vec![("df1", 10.0), ("df2", 20.0)],
            20.0 / 18.0,
            2.0 * 400.0 * 28.0 / (10.0 * 324.0 * 16.0),
        ),
        (
            "gamma",
            vec![("shape", 2.0), ("rate", 3.0)],
            2.0 / 3.0,
            2.0 / 9.0,
        ),
        ("beta", vec![("a", 2.0), ("b", 3.0)], 0.4, 6.0 / 150.0),
        ("binomial", vec![("n", 20.0), ("p", 0.3)], 6.0, 4.2),
        ("poisson", vec![("lambda", 4.0)], 4.0, 4.0),
        ("negbinomial", vec![("size", 3.0), ("p", 0.4)], 4.5, 11.25),
    ];
    // A truncated standard normal on [-1, 2] and a mixture of two normals, with their moments.
    let standard = || Distribution::Family {
        name: "normal",
        params: vec![("mean", 0.0), ("sd", 1.0)],
    };
    let (a, b) = (-1.0f64, 2.0f64);
    let z = families::truncated_mass(&standard(), a, b);
    let phi = |x: f64| (-0.5 * x * x).exp() / (2.0 * std::f64::consts::PI).sqrt();
    let truncated_mean = (phi(a) - phi(b)) / z;
    let truncated_variance = 1.0 + (a * phi(a) - b * phi(b)) / z - truncated_mean * truncated_mean;
    let mixture_mean = 0.7 * 3.0;
    let mixture_variance = 0.3 * 1.0 + 0.7 * (0.25 + 9.0) - mixture_mean * mixture_mean;
    let composites: Vec<(Distribution, f64, f64)> = vec![
        (
            Distribution::Truncated {
                inner: Box::new(standard()),
                lower: a,
                upper: b,
            },
            truncated_mean,
            truncated_variance,
        ),
        (
            Distribution::Mixture {
                parts: vec![
                    (standard(), 0.3),
                    (
                        Distribution::Family {
                            name: "normal",
                            params: vec![("mean", 3.0), ("sd", 0.5)],
                        },
                        0.7,
                    ),
                ],
            },
            mixture_mean,
            mixture_variance,
        ),
    ];
    cases
        .iter()
        .map(|(name, params, mean, variance)| {
            (
                (*name).to_string(),
                Distribution::Family {
                    name,
                    params: params.clone(),
                },
                *mean,
                *variance,
            )
        })
        .chain(
            composites
                .into_iter()
                .enumerate()
                .map(|(i, (d, m, v))| (format!("composite {i}"), d, m, v)),
        )
        .collect()
}
