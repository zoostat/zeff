//! The stochastic class of the testing handbook, run nightly (release 0.3.0, step 14): each of the
//! fourteen distributions drawn 100,000 times under each of 50 seeds and compared with its exact
//! mean, variance and distribution function. R's draws cannot be matched value for value (another
//! generator), so the comparison is the one the handbook allows: agreement within Monte Carlo
//! error. Run with `cargo test -p zeff-dist --release --test nightly_stochastic -- --ignored`.

use zeff_dist::families;
use zeff_numeric::random::Stream;

mod common;

const SEEDS: u64 = 50;
const N: usize = 100_000;

#[test]
#[ignore = "nightly: 50 seeds of 100,000 draws for each of 14 distributions"]
fn fifty_seeds_recover_every_distribution_within_monte_carlo_error() {
    let mut failures = Vec::new();
    for (ordinal, (name, d, mean, variance)) in common::recovery_cases().into_iter().enumerate() {
        let se = (variance / N as f64).sqrt();
        let mut z_sum = 0.0;
        let mut worst_ks = 0.0f64;
        for seed in 0..SEEDS {
            let mut stream = Stream::new(9_000 + seed, ordinal as u32 + 1, 0);
            let mut draws: Vec<f64> = (0..N).map(|_| families::draw(&d, &mut stream)).collect();
            let sample_mean = draws.iter().sum::<f64>() / N as f64;
            let z = (sample_mean - mean) / se;
            z_sum += z;
            if z.abs() > 5.0 {
                failures.push(format!("{name}, seed {seed}: mean z = {z:.2}"));
            }
            let sample_variance =
                draws.iter().map(|x| (x - sample_mean).powi(2)).sum::<f64>() / (N as f64 - 1.0);
            if (sample_variance - variance).abs() > 0.05 * variance {
                failures.push(format!(
                    "{name}, seed {seed}: variance {sample_variance} against {variance}"
                ));
            }
            draws.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let mut ks = 0.0f64;
            if families::is_discrete(&d) {
                let mut i = 0;
                while i < N {
                    let x = draws[i];
                    let mut j = i;
                    while j < N && draws[j] == x {
                        j += 1;
                    }
                    ks = ks.max((families::cdf(&d, x).lower - j as f64 / N as f64).abs());
                    i = j;
                }
            } else {
                for (i, x) in draws.iter().enumerate() {
                    let f = families::cdf(&d, *x).lower;
                    ks = ks
                        .max((f - i as f64 / N as f64).abs())
                        .max(((i + 1) as f64 / N as f64 - f).abs());
                }
            }
            worst_ks = worst_ks.max(ks);
            // 2.6 / sqrt(n) is a tail probability of about 3e-6 for one seed.
            if ks > 2.6 / (N as f64).sqrt() {
                failures.push(format!("{name}, seed {seed}: Kolmogorov–Smirnov {ks:.5}"));
            }
        }
        // Over the seeds the mean's errors must not lean one way: z-scores sum to a standard
        // normal times the square root of their number.
        let lean = z_sum / (SEEDS as f64).sqrt();
        if lean.abs() > 4.5 {
            failures.push(format!("{name}: the mean leans, summed z = {lean:.2}"));
        }
        println!(
            "{name}: summed z {lean:+.2}, worst Kolmogorov–Smirnov {:.5} (limit {:.5})",
            worst_ks,
            2.6 / (N as f64).sqrt()
        );
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
