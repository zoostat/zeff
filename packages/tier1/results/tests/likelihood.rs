//! The likelihood trait and the shared optimiser (release 0.9.0, step 4), with no `glm` yet: the
//! binomial family with the logit and probit links, fitted by `zeff_kernels::irls`, against R's
//! `glm(family = binomial)` iterated to the maximum (`tests/reference/r/binary.json`), on a
//! design made by whole-number arithmetic and on the shipped wages; with the properties that the
//! log-likelihood never falls, that the order of the rows changes nothing, and that Fisher
//! scoring and Newton's method reach one maximum.

use proptest::prelude::*;
use serde_json::Value as Json;
use zeff_kernels::Matrix;
use zeff_kernels::irls::{Control, Method, Solution, irls};
use zeff_kernels::triangular::inverse_upper;
use zeff_results::likelihood::{Binomial, Likelihood, Link};

fn reference() -> Json {
    let path = zeff_testkit::root().join("tests/reference/r/binary.json");
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn number(v: &Json) -> f64 {
    v.as_str().unwrap().parse().unwrap()
}

/// The generated design of `binary.R`, built by the same whole-number arithmetic.
fn generated() -> (Matrix, Vec<f64>) {
    let mut rows = Vec::new();
    let mut y = Vec::new();
    for i in 0..400_i64 {
        let x1 = (i % 17) as f64 / 4.0 - 2.0;
        let x2 = ((i * 37) % 23) as f64 / 5.0 - 2.2;
        let threshold = 4 + (i % 17) / 3 - ((i * 37) % 23) / 6;
        y.push(if (i * 7919) % 11 < threshold {
            1.0
        } else {
            0.0
        });
        rows.push(vec![1.0, x1, x2]);
    }
    (Matrix::from_rows(&rows), y)
}

/// `union ~ educ + exper + female` on the shipped wages, their complete rows.
fn wages() -> (Matrix, Vec<f64>) {
    let path = zeff_testkit::root().join("packages/tier1/datasets/data/wages.csv");
    let text = std::fs::read_to_string(path).unwrap();
    let mut lines = text.lines();
    let header: Vec<&str> = lines.next().unwrap().split(',').collect();
    let at = |name: &str| header.iter().position(|h| *h == name).unwrap();
    let (union, educ, exper, female) = (at("union"), at("educ"), at("exper"), at("female"));
    let mut rows = Vec::new();
    let mut y = Vec::new();
    for line in lines {
        let cells: Vec<&str> = line.split(',').collect();
        let value = |c: usize| cells[c].parse::<f64>().ok();
        if let (Some(u), Some(e), Some(x), Some(f)) =
            (value(union), value(educ), value(exper), value(female))
        {
            rows.push(vec![1.0, e, x, f]);
            y.push(u);
        }
    }
    (Matrix::from_rows(&rows), y)
}

fn fit(x: &Matrix, family: &Binomial, method: Method) -> Solution {
    let start: Vec<f64> = (0..family.y.len()).map(|r| family.start(r)).collect();
    irls(
        x,
        family,
        &start,
        &Control {
            method,
            ..Control::default()
        },
    )
    .expect("a maximum")
}

/// The inverse of the information, S R⁻¹ R⁻ᵀ S.
fn variance(solution: &Solution) -> Matrix {
    let inverse = inverse_upper(&solution.r).unwrap();
    let k = solution.r.rows();
    Matrix::from_fn(k, k, |i, j| {
        let total: f64 = (0..k).map(|l| inverse.get(i, l) * inverse.get(j, l)).sum();
        total * solution.scales[i] * solution.scales[j]
    })
}

fn close(ours: f64, theirs: f64, tolerance: f64, what: &str) {
    assert!(
        (ours - theirs).abs() <= tolerance * theirs.abs().max(1e-3),
        "{what}: {ours} against R's {theirs}"
    );
}

#[test]
fn the_fits_agree_with_r() {
    let json = reference();
    assert_eq!(json["generated_rows"], 400);
    let (gx, gy) = generated();
    assert_eq!(
        gy.iter().sum::<f64>() as i64,
        json["generated_ones"].as_i64().unwrap()
    );
    let (wx, wy) = wages();
    for case in json["cases"].as_array().unwrap() {
        let name = case["name"].as_str().unwrap();
        let link = match case["link"].as_str().unwrap() {
            "logit" => Link::Logit,
            _ => Link::Probit,
        };
        let (x, y) = if name.starts_with("generated") {
            (&gx, &gy)
        } else {
            (&wx, &wy)
        };
        assert_eq!(y.len() as i64, case["n"].as_i64().unwrap(), "{name}");
        let family = Binomial::new(y.clone(), link);
        let solution = fit(x, &family, Method::Scoring);
        assert!(solution.converged, "{name}");
        for (j, theirs) in case["coefficients"].as_array().unwrap().iter().enumerate() {
            close(
                solution.coefficients[j],
                number(theirs),
                1e-9,
                &format!("{name} coefficient {j}"),
            );
        }
        close(
            solution.log_likelihood,
            number(&case["log_likelihood"]),
            1e-12,
            &format!("{name} log-likelihood"),
        );
        let deviance: f64 = solution
            .eta
            .iter()
            .enumerate()
            .map(|(row, &eta)| family.deviance(row, eta))
            .sum();
        close(
            deviance,
            number(&case["deviance"]),
            1e-12,
            &format!("{name} deviance"),
        );
        let v = variance(&solution);
        for (i, row) in case["vcov"].as_array().unwrap().iter().enumerate() {
            for (j, theirs) in row.as_array().unwrap().iter().enumerate() {
                close(
                    v.get(i, j),
                    number(theirs),
                    1e-8,
                    &format!("{name} vcov {i},{j}"),
                );
            }
        }
        // The log-likelihood never fell from one iteration to the next.
        for pair in solution.history.windows(2).skip(1) {
            assert!(pair[1] >= pair[0] - 1e-12 * pair[0].abs(), "{name}");
        }
    }
}

#[test]
fn scoring_and_newton_reach_one_maximum() {
    let (x, y) = generated();
    for link in [Link::Logit, Link::Probit] {
        let family = Binomial::new(y.clone(), link);
        let scoring = fit(&x, &family, Method::Scoring);
        let newton = fit(&x, &family, Method::Newton);
        assert!(newton.converged);
        for (a, b) in scoring.coefficients.iter().zip(&newton.coefficients) {
            close(*a, *b, 1e-10, link.name());
        }
        if link == Link::Logit {
            // A canonical link: the two informations are one, and so are the iterations.
            assert_eq!(scoring.coefficients, newton.coefficients);
        }
    }
}

#[test]
fn the_fit_is_the_same_bits_twice() {
    let (x, y) = wages();
    let family = Binomial::new(y, Link::Probit);
    let first = fit(&x, &family, Method::Scoring);
    let second = fit(&x, &family, Method::Scoring);
    assert_eq!(first, second);
}

/// An outcome that is a share of its prior weight (release 0.9.0, step 42c4: the family's own
/// routines take one, though `glm` gives them 0 and 1 at this release, and the mutation run
/// showed no test did). Thirty per cent of four trials at a linear predictor of 0.7, against
/// R 4.6.0: `m * (y * log(p) + (1 - y) * log(1 - p))` with `p = plogis(0.7)` and `pnorm(0.7)`,
/// and the deviance as twice the saturated log-likelihood less it.
#[test]
fn a_share_of_its_trials_has_r_s_log_likelihood_and_deviance() {
    use zeff_kernels::irls::IndexModel;
    for (link, log_likelihood, deviance) in [
        (Link::Logit, -3.5727441955418318, 2.2585739746445155),
        (Link::Probit, -4.305538463020845, 3.724162509602542),
    ] {
        let family = Binomial {
            y: vec![0.3],
            prior: vec![4.0],
            link,
        };
        let terms = family.at(0, 0.7, false);
        close(
            terms.log_likelihood,
            log_likelihood,
            1e-14,
            "log-likelihood",
        );
        close(family.deviance(0, 0.7), deviance, 1e-13, "deviance");
    }
    // An outcome of 0 or 1 has a saturated log-likelihood of nought: the deviance is minus
    // twice the log-likelihood.
    for y in [0.0, 1.0] {
        let family = Binomial::new(vec![y], Link::Logit);
        let terms = family.at(0, 0.7, false);
        close(
            family.deviance(0, 0.7),
            -2.0 * terms.log_likelihood,
            1e-15,
            "deviance of a 0 or a 1",
        );
    }
}

/// The logarithm of a probit probability near one keeps its digits: log Φ(5) and log Φ(3)
/// against R's `pnorm(x, log.p = TRUE)`. Taken as the logarithm of the probability itself it
/// would have ten digits at 5, where the probability is one less three parts in ten million.
#[test]
fn the_log_of_a_probit_probability_near_one_keeps_its_digits() {
    use zeff_kernels::irls::IndexModel;
    let family = Binomial::new(vec![1.0], Link::Probit);
    for (eta, log_probability) in [
        (3.0, -0.0013508099647481938),
        (4.5, -3.3976788968344657e-06),
        (5.0, -2.866516129637636e-07),
        (6.0, -9.865876455243756e-10),
        (7.5, -3.1908916729109475e-14),
    ] {
        let terms = family.at(0, eta, false);
        // As a share of the value itself, which is small.
        let apart = ((terms.log_likelihood - log_probability) / log_probability).abs();
        assert!(
            apart < 1e-13,
            "log Φ({eta}): {:e} against R's {log_probability:e}",
            terms.log_likelihood
        );
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    /// The rows in another order give the same maximum, to rounding.
    #[test]
    fn the_order_of_the_rows_changes_nothing(seed in any::<u64>()) {
        let (x, y) = generated();
        let n = y.len();
        let mut order: Vec<usize> = (0..n).collect();
        let mut state = seed | 1;
        for i in (1..n).rev() {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            order.swap(i, (state % (i as u64 + 1)) as usize);
        }
        let shuffled = Matrix::from_fn(n, 3, |i, j| x.get(order[i], j));
        let shuffled_y: Vec<f64> = order.iter().map(|&i| y[i]).collect();
        for link in [Link::Logit, Link::Probit] {
            let a = fit(&x, &Binomial::new(y.clone(), link), Method::Scoring);
            let b = fit(&shuffled, &Binomial::new(shuffled_y.clone(), link), Method::Scoring);
            for (p, q) in a.coefficients.iter().zip(&b.coefficients) {
                prop_assert!((p - q).abs() <= 1e-11 * p.abs().max(1.0));
            }
            prop_assert!((a.log_likelihood - b.log_likelihood).abs() <= 1e-12 * a.log_likelihood.abs());
        }
    }
}
