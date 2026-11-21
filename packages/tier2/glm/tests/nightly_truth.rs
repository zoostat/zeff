//! The truth-recovery Monte Carlo of the testing handbook for `glm`, run nightly (release 0.9.0,
//! step 9): fifty seeds of binary outcomes drawn with the engine's own generator from a model
//! with known coefficients, under each link and two designs. The bias of the estimates must be
//! near zero, and the 95 per cent intervals of the standard errors that suit each design must
//! cover about 95 per cent of the time; the others are reported.
//!
//! In the independent design every kind of error suits. In the clustered design the rows of a
//! cluster share a part of the latent error and of a regressor, so only the clustered errors
//! suit; the latent error keeps its distribution row by row (a normal for the probit, and for
//! the logit the logistic quantile of a normal's probability), so that the model for one row is
//! still the logit or the probit with the generating coefficients, and the pooled fit estimates
//! them.
//!
//! Run with `cargo test -p zeff-glm --release --test nightly_truth -- --ignored --nocapture`.

use zeff_columns::{Column, Table};
use zeff_core::runtime::{Args, CallContext, Fail, Files, Randomness};
use zeff_core::value::{Distribution, Fit, Frame, Value};
use zeff_diagnostics::{Diagnostic, Source, Sources, Span};
use zeff_dist::families;
use zeff_numeric::random::Stream;
use zeff_numeric::special::normal_tails;
use zeff_testkit::formula;

const SEEDS: u64 = 50;
const N: usize = 1_000;
const CLUSTERS: usize = 100;
/// The share of the latent error's variance that the rows of a cluster have in common.
const SHARED: f64 = 0.5;
/// The generating coefficients: an intercept, a continuous regressor, a uniform one and an
/// indicator.
const TRUTH: [(&str, f64); 4] = [("(intercept)", -0.5), ("x1", 0.8), ("x2", -0.5), ("d", 0.6)];
/// The kinds of standard error fitted under every design: `se:` words, and `cluster: g`.
const KINDS: [&str; 5] = ["classical", "robust", "hc2", "hc3", "cluster"];
/// The coverage a suitable 95 per cent interval may show over 200 intervals (four coefficients
/// under fifty seeds): about 95 per cent, with room for the Monte Carlo error.
const COVERAGE: (f64, f64) = (0.89, 0.995);

#[derive(Clone, Copy, PartialEq, Eq)]
enum Design {
    Independent,
    Clustered,
}

impl Design {
    fn name(self) -> &'static str {
        match self {
            Design::Independent => "independent",
            Design::Clustered => "clustered",
        }
    }

    /// The standard errors that suit the design, whose intervals must cover.
    fn suitable(self) -> &'static [&'static str] {
        match self {
            Design::Independent => &KINDS,
            Design::Clustered => &["cluster"],
        }
    }
}

fn normal(stream: &mut Stream) -> f64 {
    let standard = Distribution::Family {
        name: "normal",
        params: vec![("mean", 0.0), ("sd", 1.0)],
    };
    families::draw(&standard, stream)
}

/// The latent error of a row from a standard normal value `z`: the value itself under the
/// probit; under the logit the logistic quantile of its probability, log(Φ(z)/(1 − Φ(z))),
/// which has the logistic distribution whatever the dependence between the `z` of two rows.
fn latent_error(z: f64, link: &str) -> f64 {
    if link == "probit" {
        z
    } else {
        let tails = normal_tails(z);
        (tails.lower / tails.upper).ln()
    }
}

/// One draw of the design under `seed` and `link`: the outcome is 1 when
/// `−0.5 + 0.8 x1 − 0.5 x2 + 0.6 d` plus the latent error is above zero, with `x1` standard
/// normal (in the clustered design, half of its variance shared within the cluster), `x2`
/// uniform and `d` an indicator with probability 0.4; a hundred clusters of ten rows.
fn draw(design: Design, link: &str, seed: u64) -> Table {
    let ordinal = match (design, link) {
        (Design::Independent, "logit") => 1,
        (Design::Independent, _) => 2,
        (Design::Clustered, "logit") => 3,
        (Design::Clustered, _) => 4,
    };
    let mut stream = Stream::new(20_261_121 + seed, ordinal, 0);
    let cluster_parts: Vec<(f64, f64)> = (0..CLUSTERS)
        .map(|_| (normal(&mut stream), normal(&mut stream)))
        .collect();
    let (mut y, mut x1, mut x2, mut d, mut g) = (
        Vec::with_capacity(N),
        Vec::with_capacity(N),
        Vec::with_capacity(N),
        Vec::with_capacity(N),
        Vec::with_capacity(N),
    );
    let (shared, own) = (SHARED.sqrt(), (1.0 - SHARED).sqrt());
    for i in 0..N {
        let cluster = i / (N / CLUSTERS);
        let (regressor, z) = match design {
            Design::Independent => (normal(&mut stream), normal(&mut stream)),
            Design::Clustered => (
                shared * cluster_parts[cluster].0 + own * normal(&mut stream),
                shared * cluster_parts[cluster].1 + own * normal(&mut stream),
            ),
        };
        let uniform = stream.uniform();
        let indicator = if stream.uniform() < 0.4 { 1.0 } else { 0.0 };
        let index =
            TRUTH[0].1 + TRUTH[1].1 * regressor + TRUTH[2].1 * uniform + TRUTH[3].1 * indicator;
        let outcome = if index + latent_error(z, link) > 0.0 {
            1.0
        } else {
            0.0
        };
        y.push(Some(outcome));
        x1.push(Some(regressor));
        x2.push(Some(uniform));
        d.push(Some(indicator));
        g.push(Some(cluster as i64 + 1));
    }
    Table::new(vec![
        ("y".to_string(), Column::floats(y).0),
        ("x1".to_string(), Column::floats(x1).0),
        ("x2".to_string(), Column::floats(x2).0),
        ("d".to_string(), Column::floats(d).0),
        ("g".to_string(), Column::ints(g)),
    ])
    .expect("columns of one length")
}

struct NoFiles;

impl Files for NoFiles {
    fn read(&self, _: &str, _: Span, _: &str) -> Result<Vec<u8>, Fail> {
        unreachable!("glm reads no file")
    }
}

/// `y ~ x1 + x2 + d` through `glm`'s body with `link` and the standard errors of `kind`.
fn fit(table: &Table, link: &str, kind: &str) -> Fit {
    let text = "y ~ x1 + x2 + d";
    let mut output = String::new();
    let mut messages: Vec<Diagnostic> = Vec::new();
    let mut random = Randomness::default();
    let script = format!("let m = d.glm({text}, family: binomial)\n");
    let mut sources = Sources::new();
    sources.add(Source::new("t.zeff", &script));
    let mut ctx = CallContext {
        files: &NoFiles,
        script: "t.zeff",
        text: &script,
        columns_used: Vec::new(),
        frame: None,
        group_rows: None,
        callee_span: Span::new(10, 13),
        arg_spans: vec![Some(Span::new(8, 9)), Some(Span::new(14, 14 + text.len()))],
        rest_spans: Vec::new(),
        definition_spans: Vec::new(),
        output: &mut output,
        messages: &mut messages,
        random: &mut random,
        statement: 1,
    };
    let (se, cluster) = if kind == "cluster" {
        (None, Some(Value::Var("g".to_string())))
    } else {
        (Some(Value::Word(kind.to_string())), None)
    };
    // The arguments in the order `glm` declares them: data, formula, family, link, fe, se,
    // cluster, weights, weight_type, level.
    let args = Args {
        values: vec![
            Some(Value::Frame(Frame::new(table.clone()))),
            Some(Value::Formula(formula(text))),
            Some(Value::Word("binomial".to_string())),
            Some(Value::Word(link.to_string())),
            None,
            se,
            cluster,
            None,
            None,
            Some(Value::Float(0.95)),
        ],
        rest: Vec::new(),
        definitions: Vec::new(),
    };
    match zeff_glm::binary::glm(&mut ctx, args) {
        Ok(Value::Fit(fit)) => (*fit).clone(),
        Ok(other) => panic!("not a fit: {other:?}"),
        Err(e) => panic!("{}", e.render(&sources)),
    }
}

#[test]
#[ignore = "nightly: 50 seeds under two designs and two links, five kinds of standard error each"]
fn fifty_seeds_recover_the_coefficients_and_the_intervals_cover() {
    let mut failures = Vec::new();
    for link in ["logit", "probit"] {
        for design in [Design::Independent, Design::Clustered] {
            // The estimates by seed and coefficient, and the intervals covering by kind.
            let mut estimates: Vec<Vec<f64>> = vec![Vec::new(); TRUTH.len()];
            let mut covered = vec![0usize; KINDS.len()];
            for seed in 0..SEEDS {
                let table = draw(design, link, seed);
                for (k, kind) in KINDS.iter().enumerate() {
                    let fit = fit(&table, link, kind);
                    assert_eq!(fit.n, N, "{link}, {} seed {seed}: rows used", design.name());
                    for (c, (term, truth)) in TRUTH.iter().enumerate() {
                        let i = fit
                            .terms
                            .iter()
                            .position(|t| t == term)
                            .unwrap_or_else(|| panic!("no term `{term}`"));
                        if k == 0 {
                            estimates[c].push(fit.estimates[i]);
                        }
                        let (lo, hi) = fit.intervals[i];
                        if lo <= *truth && *truth <= hi {
                            covered[k] += 1;
                        }
                    }
                }
            }
            // The bias of each coefficient against the standard error of its Monte Carlo mean.
            let mut bias_text = Vec::new();
            for (c, (term, truth)) in TRUTH.iter().enumerate() {
                let n = estimates[c].len() as f64;
                let mean = estimates[c].iter().sum::<f64>() / n;
                let variance =
                    estimates[c].iter().map(|b| (b - mean).powi(2)).sum::<f64>() / (n - 1.0);
                let se_mean = (variance / n).sqrt();
                let z = (mean - truth) / se_mean;
                bias_text.push(format!("{term} {z:+.2}"));
                if z.abs() > 4.0 {
                    failures.push(format!(
                        "{link}, {}: `{term}` is biased, mean {mean:.4} against {truth} ({z:+.2} standard errors of the mean)",
                        design.name()
                    ));
                }
            }
            let total = (SEEDS as usize * TRUTH.len()) as f64;
            let mut coverage_text = Vec::new();
            for (k, kind) in KINDS.iter().enumerate() {
                let share = covered[k] as f64 / total;
                let suits = design.suitable().contains(kind);
                coverage_text.push(format!(
                    "{kind} {share:.3}{}",
                    if suits { "" } else { " (reported)" }
                ));
                if suits && !(COVERAGE.0..=COVERAGE.1).contains(&share) {
                    failures.push(format!(
                        "{link}, {}: the 95 per cent intervals under {kind} errors cover {share:.3} of the time",
                        design.name()
                    ));
                }
            }
            println!(
                "{link}, {}: bias in standard errors of the mean: {}; coverage of the 95 per cent intervals: {}",
                design.name(),
                bias_text.join(", "),
                coverage_text.join(", ")
            );
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
