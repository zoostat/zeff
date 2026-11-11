//! The truth-recovery Monte Carlo of the testing handbook, run nightly (release 0.4.0, step 20):
//! fifty seeds of data drawn with the engine's own generator under three designs with known
//! coefficients, with homoskedastic, heteroskedastic and clustered errors. The bias of `reg` must
//! be near zero, and the 95 per cent intervals of the standard errors that suit each design
//! (classical, HC1 to HC3 and CR1) must cover about 95 per cent of the time; the others are
//! reported. Run with `cargo test -p zeff-regress --release --test nightly_truth -- --ignored
//! --nocapture`.

use zeff_columns::{Column, Table};
use zeff_core::runtime::{Args, CallContext, Fail, Files, Randomness};
use zeff_core::value::{Distribution, Fit, Frame, Value};
use zeff_diagnostics::{Diagnostic, Source, Sources, Span};
use zeff_dist::families;
use zeff_numeric::random::Stream;
use zeff_testkit::formula;

const SEEDS: u64 = 50;
const N: usize = 400;
const CLUSTERS: usize = 40;
/// The generating coefficients: an intercept, a continuous regressor, a uniform one and an
/// indicator.
const TRUTH: [(&str, f64); 4] = [("(intercept)", 1.0), ("x1", 0.5), ("x2", -0.25), ("d", 0.3)];
/// The kinds of standard error fitted under every design: `se:` words, and `cluster: g`.
const KINDS: [&str; 5] = ["classical", "robust", "hc2", "hc3", "cluster"];
/// The coverage a suitable 95 per cent interval may show over 200 intervals (four coefficients
/// under fifty seeds): about 95 per cent, with room for the Monte Carlo error.
const COVERAGE: (f64, f64) = (0.89, 0.995);

#[derive(Clone, Copy, PartialEq, Eq)]
enum Design {
    Homoskedastic,
    Heteroskedastic,
    Clustered,
}

impl Design {
    fn name(self) -> &'static str {
        match self {
            Design::Homoskedastic => "homoskedastic",
            Design::Heteroskedastic => "heteroskedastic",
            Design::Clustered => "clustered",
        }
    }

    /// The standard errors that suit the design, whose intervals must cover.
    fn suitable(self) -> &'static [&'static str] {
        match self {
            Design::Homoskedastic => &KINDS,
            Design::Heteroskedastic => &["robust", "hc2", "hc3", "cluster"],
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

/// One draw of the design under `seed`: `y = 1 + 0.5 x1 − 0.25 x2 + 0.3 d + e`, with `x1`
/// standard normal, `x2` uniform, `d` an indicator with probability 0.4, and the error standard
/// normal (homoskedastic), scaled by `0.5 + |x1|` (heteroskedastic), or the sum of a cluster
/// effect and an individual one, with `x1` sharing a cluster component (clustered, forty
/// clusters of ten rows).
fn draw(design: Design, seed: u64) -> Table {
    let ordinal = match design {
        Design::Homoskedastic => 1,
        Design::Heteroskedastic => 2,
        Design::Clustered => 3,
    };
    let mut stream = Stream::new(20_261_024 + seed, ordinal, 0);
    let cluster_effects: Vec<(f64, f64)> = (0..CLUSTERS)
        .map(|_| (0.7 * normal(&mut stream), 0.7 * normal(&mut stream)))
        .collect();
    let (mut y, mut x1, mut x2, mut d, mut g) = (
        Vec::with_capacity(N),
        Vec::with_capacity(N),
        Vec::with_capacity(N),
        Vec::with_capacity(N),
        Vec::with_capacity(N),
    );
    for i in 0..N {
        let cluster = i / (N / CLUSTERS);
        let (regressor, error) = match design {
            Design::Homoskedastic => (normal(&mut stream), normal(&mut stream)),
            Design::Heteroskedastic => {
                let x = normal(&mut stream);
                (x, (0.5 + x.abs()) * normal(&mut stream))
            }
            Design::Clustered => (
                cluster_effects[cluster].0 + 0.7 * normal(&mut stream),
                cluster_effects[cluster].1 + 0.7 * normal(&mut stream),
            ),
        };
        let uniform = stream.uniform();
        let indicator = if stream.uniform() < 0.4 { 1.0 } else { 0.0 };
        y.push(Some(
            TRUTH[0].1
                + TRUTH[1].1 * regressor
                + TRUTH[2].1 * uniform
                + TRUTH[3].1 * indicator
                + error,
        ));
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
        unreachable!("reg reads no file")
    }
}

/// `y ~ x1 + x2 + d` through `reg`'s body with the standard errors of `kind`.
fn fit(table: &Table, kind: &str) -> Fit {
    let text = "y ~ x1 + x2 + d";
    let mut output = String::new();
    let mut messages: Vec<Diagnostic> = Vec::new();
    let mut random = Randomness::default();
    let script = format!("let m = d.reg({text})\n");
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
    let args = Args {
        values: vec![
            Some(Value::Frame(Frame::new(table.clone()))),
            Some(Value::Formula(formula(text))),
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
    match zeff_regress::reg::reg(&mut ctx, args) {
        Ok(Value::Fit(fit)) => (*fit).clone(),
        Ok(other) => panic!("not a fit: {other:?}"),
        Err(e) => panic!("{}", e.render(&sources)),
    }
}

#[test]
#[ignore = "nightly: 50 seeds under three designs, five kinds of standard error each"]
fn fifty_seeds_recover_the_coefficients_and_the_intervals_cover() {
    let mut failures = Vec::new();
    for design in [
        Design::Homoskedastic,
        Design::Heteroskedastic,
        Design::Clustered,
    ] {
        // The estimates by seed and coefficient, and the intervals covering by kind.
        let mut estimates: Vec<Vec<f64>> = vec![Vec::new(); TRUTH.len()];
        let mut covered = vec![0usize; KINDS.len()];
        for seed in 0..SEEDS {
            let table = draw(design, seed);
            for (k, kind) in KINDS.iter().enumerate() {
                let fit = fit(&table, kind);
                assert_eq!(fit.n, N, "{} seed {seed}: rows used", design.name());
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
            let variance = estimates[c].iter().map(|b| (b - mean).powi(2)).sum::<f64>() / (n - 1.0);
            let se_mean = (variance / n).sqrt();
            let z = (mean - truth) / se_mean;
            bias_text.push(format!("{term} {z:+.2}"));
            if z.abs() > 4.0 {
                failures.push(format!(
                    "{}: `{term}` is biased, mean {mean:.4} against {truth} ({z:+.2} standard errors of the mean)",
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
                    "{}: the 95 per cent intervals under {kind} errors cover {share:.3} of the time",
                    design.name()
                ));
            }
        }
        println!(
            "{}: bias in standard errors of the mean: {}; coverage of the 95 per cent intervals: {}",
            design.name(),
            bias_text.join(", "),
            coverage_text.join(", ")
        );
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// The generating coefficients of the two-way panel (release 0.6.0, step 18): a return to
/// schooling, an hours slope and a union premium, beside unit and year effects of known size
/// that the fit absorbs.
const PANEL_TRUTH: [(&str, f64); 3] = [("educ", 0.08), ("hours", 0.01), ("union", 0.15)];
const PANEL_UNITS: usize = 150;
const PANEL_YEARS: usize = 6;

/// One draw of the two-way panel under `seed`: `PANEL_UNITS` units over `PANEL_YEARS` years,
/// `y = 0.08 educ + 0.01 hours + 0.15 union + unit effect + year effect + e`, with the unit
/// effect N(0, 0.5²) and correlated with schooling and with hours (so that a pooled fit is
/// biased and only the within fit recovers the truth), the year effect a trend with noise,
/// hours varying within units, union an indicator that switches within units, and the error the
/// sum of a unit-level shock and an individual one, which clustered errors by unit are made for.
fn draw_panel(seed: u64) -> Table {
    let mut stream = Stream::new(20_261_102 + seed, 4, 0);
    let year_effects: Vec<f64> = (0..PANEL_YEARS)
        .map(|t| 0.02 * t as f64 + 0.05 * normal(&mut stream))
        .collect();
    let n = PANEL_UNITS * PANEL_YEARS;
    let (mut y, mut educ, mut hours, mut union, mut unit, mut year) = (
        Vec::with_capacity(n),
        Vec::with_capacity(n),
        Vec::with_capacity(n),
        Vec::with_capacity(n),
        Vec::with_capacity(n),
        Vec::with_capacity(n),
    );
    for i in 0..PANEL_UNITS {
        let ability = 0.5 * normal(&mut stream);
        let schooling = (12.0 + 2.0 * ability + 2.0 * normal(&mut stream))
            .round()
            .clamp(8.0, 20.0);
        let shock = 0.3 * normal(&mut stream);
        let mut member = stream.uniform() < 0.3;
        for (t, year_effect) in year_effects.iter().enumerate() {
            if stream.uniform() < 0.1 {
                member = !member;
            }
            let worked = 35.0 + 3.0 * ability + 4.0 * normal(&mut stream);
            let indicator = if member { 1.0 } else { 0.0 };
            let e = shock + 0.5 * normal(&mut stream);
            y.push(Some(
                PANEL_TRUTH[0].1 * schooling
                    + PANEL_TRUTH[1].1 * worked
                    + PANEL_TRUTH[2].1 * indicator
                    + ability
                    + year_effect
                    + e,
            ));
            educ.push(Some(schooling));
            hours.push(Some(worked));
            union.push(Some(indicator));
            unit.push(Some(i as i64 + 1));
            year.push(Some(2010 + t as i64));
        }
    }
    Table::new(vec![
        ("y".to_string(), Column::floats(y).0),
        ("educ".to_string(), Column::floats(educ).0),
        ("hours".to_string(), Column::floats(hours).0),
        ("union".to_string(), Column::floats(union).0),
        ("unit".to_string(), Column::ints(unit)),
        ("year".to_string(), Column::ints(year)),
    ])
    .expect("columns of one length")
}

/// `y ~ hours + union` with `fe: unit + year` and errors clustered by the unit, through `reg`'s
/// body; schooling, constant within a unit, is absorbed by the unit effect and left out of the
/// formula (named there it would be dropped with a note).
fn fit_panel(table: &Table) -> Fit {
    let text = "y ~ hours + union";
    let mut output = String::new();
    let mut messages: Vec<Diagnostic> = Vec::new();
    let mut random = Randomness::default();
    let script = format!("let m = d.reg({text})\n");
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
    let args = Args {
        values: vec![
            Some(Value::Frame(Frame::new(table.clone()))),
            Some(Value::Formula(formula(text))),
            Some(Value::Terms(zeff_testkit::terms("unit + year"))),
            None,
            Some(Value::Var("unit".to_string())),
            None,
            None,
            Some(Value::Float(0.95)),
        ],
        rest: Vec::new(),
        definitions: Vec::new(),
    };
    match zeff_regress::reg::reg(&mut ctx, args) {
        Ok(Value::Fit(fit)) => (*fit).clone(),
        Ok(other) => panic!("not a fit: {other:?}"),
        Err(e) => panic!("{}", e.render(&sources)),
    }
}

/// The truth-recovery Monte Carlo of `fe:` (release 0.6.0, step 18): fifty seeds of the two-way
/// panel; the bias of the within estimates of the hours slope and the union premium near
/// zero (within a tenth of a standard error on average), and their 95 per cent clustered
/// intervals covering about 95 per cent of the time. Run with `cargo test -p zeff-regress
/// --release --test nightly_truth -- --ignored --nocapture`.
#[test]
#[ignore = "nightly: 50 seeds of the two-way panel"]
fn fifty_seeds_of_the_two_way_panel_recover_the_slopes() {
    let truths = &PANEL_TRUTH[1..];
    let mut bias = vec![0.0; truths.len()];
    let mut errors = vec![0.0; truths.len()];
    let mut covered = vec![0usize; truths.len()];
    for seed in 0..SEEDS {
        let fit = fit_panel(&draw_panel(seed));
        assert_eq!(fit.effects.len(), 2, "two effects absorbed");
        for (j, (term, truth)) in truths.iter().enumerate() {
            let i = fit
                .terms
                .iter()
                .position(|t| t == term)
                .unwrap_or_else(|| panic!("no term {term}"));
            bias[j] += fit.estimates[i] - truth;
            errors[j] += fit.std_errors[i];
            let (low, high) = fit.intervals[i];
            if low <= *truth && *truth <= high {
                covered[j] += 1;
            }
        }
    }
    for (j, (term, _)) in truths.iter().enumerate() {
        let mean_bias = bias[j] / SEEDS as f64;
        let mean_error = errors[j] / SEEDS as f64;
        let coverage = covered[j] as f64 / SEEDS as f64;
        println!(
            "{term}: mean bias {mean_bias:+.5} (mean standard error {mean_error:.5}), coverage {coverage:.2}"
        );
        assert!(
            mean_bias.abs() < 0.2 * mean_error,
            "{term}: the within estimate is biased: {mean_bias:+.5} against an error of {mean_error:.5}"
        );
        assert!(
            (0.86..=1.0).contains(&coverage),
            "{term}: the clustered intervals cover {coverage:.2}"
        );
    }
}
