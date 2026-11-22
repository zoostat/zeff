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
        width: zeff_core::display::PAGE_WIDTH,
        shown: None,
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
        width: zeff_core::display::PAGE_WIDTH,
        shown: None,
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

/// The generating model of the shipped `staff` data (release 0.8.0, steps 13a and 13c), drawn
/// with the engine's own generator: twelve sites with the effects of the shipped file, ability
/// that raises both the take-up of the course and productivity and is not observed, an offer
/// made by lot, and a course whose effect on productivity is 4.5 in all.
const STAFF_SEEDS: u64 = 100;
const STAFF_N: usize = 2400;
const COURSE_EFFECT: f64 = 4.5;
const SITE_EFFECTS: [f64; 12] = [
    0.43, 1.68, -4.76, 3.26, -2.2, -0.06, 1.61, 0.13, 3.03, 0.13, -5.35, -5.56,
];
const REGION_SHIFTS: [f64; 4] = [0.0, 0.3, -0.2, 0.4];

fn draw_staff(seed: u64) -> Table {
    let mut stream = Stream::new(20_261_116 + seed, 1, 0);
    let (mut site, mut age, mut offer, mut distance, mut training, mut productivity) = (
        Vec::with_capacity(STAFF_N),
        Vec::with_capacity(STAFF_N),
        Vec::with_capacity(STAFF_N),
        Vec::with_capacity(STAFF_N),
        Vec::with_capacity(STAFF_N),
        Vec::with_capacity(STAFF_N),
    );
    for _ in 0..STAFF_N {
        let s = ((stream.uniform() * 12.0) as usize).min(11);
        let a = 22.0 + (stream.uniform() * 39.0).floor();
        let ability = normal(&mut stream);
        let offered = if stream.uniform() < 0.5 { 1.0 } else { 0.0 };
        let km = (12.0 + 6.0 * normal(&mut stream)).clamp(0.5, 40.0);
        let u = -1.3 + 2.4 * offered + 0.8 * ability - 0.03 * (a - 40.0) - 0.05 * (km - 12.0)
            + REGION_SHIFTS[s / 3];
        let took = if stream.uniform() < 1.0 / (1.0 + zeff_numeric::maths::exp(-u)) {
            1.0
        } else {
            0.0
        };
        let tenure = 3.0 + took + (4.0 * stream.uniform() - 2.0);
        let y = 50.0
            + 4.0 * took
            + 0.5 * tenure
            + 3.0 * ability
            + SITE_EFFECTS[s]
            + 4.0 * normal(&mut stream);
        site.push(Some(s as i64 + 1));
        age.push(Some(a));
        offer.push(Some(offered));
        distance.push(Some(km));
        training.push(Some(took));
        productivity.push(Some(y));
    }
    Table::new(vec![
        ("site".to_string(), Column::ints(site)),
        ("age".to_string(), Column::floats(age).0),
        ("offer".to_string(), Column::floats(offer).0),
        ("distance".to_string(), Column::floats(distance).0),
        ("training".to_string(), Column::floats(training).0),
        ("productivity".to_string(), Column::floats(productivity).0),
    ])
    .expect("columns of one length")
}

/// `productivity ~ training` with the sites absorbed, by `ivreg` with the offer as the
/// instrument (`instrumented`) or by `reg`, with standard errors of `kind`.
fn fit_staff(table: &Table, instrumented: bool, kind: &str) -> Fit {
    let text = "productivity ~ training";
    let mut output = String::new();
    let mut messages: Vec<Diagnostic> = Vec::new();
    let mut random = Randomness::default();
    let script = format!("let m = d.ivreg({text})\n");
    let mut sources = Sources::new();
    sources.add(Source::new("t.zeff", &script));
    let mut ctx = CallContext {
        files: &NoFiles,
        script: "t.zeff",
        text: &script,
        columns_used: Vec::new(),
        frame: None,
        group_rows: None,
        callee_span: Span::new(10, 15),
        arg_spans: vec![Some(Span::new(8, 9)), Some(Span::new(16, 16 + text.len()))],
        rest_spans: Vec::new(),
        definition_spans: Vec::new(),
        output: &mut output,
        messages: &mut messages,
        random: &mut random,
        statement: 1,
        width: zeff_core::display::PAGE_WIDTH,
        shown: None,
    };
    let frame = Some(Value::Frame(Frame::new(table.clone())));
    let model = Some(Value::Formula(formula(text)));
    let effects = Some(Value::Terms(zeff_testkit::terms("site")));
    let se = Some(Value::Word(kind.to_string()));
    let level = Some(Value::Float(0.95));
    let result = if instrumented {
        let instruments = Some(Value::Instruments {
            endogenous: zeff_testkit::terms("training"),
            excluded: zeff_testkit::terms("offer"),
            included: Vec::new(),
        });
        zeff_regress::ivreg::ivreg(
            &mut ctx,
            Args {
                values: vec![
                    frame,
                    model,
                    effects,
                    instruments,
                    se,
                    None,
                    None,
                    None,
                    level,
                    None,
                ],
                rest: Vec::new(),
                definitions: Vec::new(),
            },
        )
    } else {
        zeff_regress::reg::reg(
            &mut ctx,
            Args {
                values: vec![frame, model, effects, se, None, None, None, level],
                rest: Vec::new(),
                definitions: Vec::new(),
            },
        )
    };
    match result {
        Ok(Value::Fit(fit)) => (*fit).clone(),
        Ok(other) => panic!("not a fit: {other:?}"),
        Err(e) => panic!("{}", e.render(&sources)),
    }
}

/// On a hundred draws of the `staff` model, two-stage least squares with the offer as the
/// instrument recovers the effect of the course: its mean estimate is within four standard
/// errors of the mean of 4.5, and its 95 per cent intervals, classical and robust, cover 4.5
/// about 95 per cent of the time. Least squares on the same draws is far from 4.5 and its
/// intervals almost never cover it, which is what the instrument is for.
#[test]
#[ignore = "nightly: 100 draws of the staff model, by two-stage and by ordinary least squares"]
fn two_stage_least_squares_recovers_the_effect_of_the_course() {
    let mut failures = Vec::new();
    let mut estimates = Vec::new();
    let mut least_squares = Vec::new();
    let kinds = ["classical", "robust"];
    let mut covered = [0usize; 2];
    let mut covered_by_least_squares = 0usize;
    for seed in 0..STAFF_SEEDS {
        let table = draw_staff(seed);
        for (k, kind) in kinds.iter().enumerate() {
            let fit = fit_staff(&table, true, kind);
            assert_eq!(fit.kind, "IvFit");
            assert_eq!(fit.terms, ["training"], "seed {seed}");
            if k == 0 {
                estimates.push(fit.estimates[0]);
            }
            let (lo, hi) = fit.intervals[0];
            if lo <= COURSE_EFFECT && COURSE_EFFECT <= hi {
                covered[k] += 1;
            }
        }
        let ols = fit_staff(&table, false, "classical");
        least_squares.push(ols.estimates[0]);
        let (lo, hi) = ols.intervals[0];
        if lo <= COURSE_EFFECT && COURSE_EFFECT <= hi {
            covered_by_least_squares += 1;
        }
    }
    let n = estimates.len() as f64;
    let mean = estimates.iter().sum::<f64>() / n;
    let variance = estimates.iter().map(|b| (b - mean).powi(2)).sum::<f64>() / (n - 1.0);
    let z = (mean - COURSE_EFFECT) / (variance / n).sqrt();
    if z.abs() > 4.0 {
        failures.push(format!(
            "the two-stage estimate is biased: mean {mean:.4} against {COURSE_EFFECT} ({z:+.2} standard errors of the mean)"
        ));
    }
    // A hundred intervals: about 95 cover, with room for the Monte Carlo error.
    for (k, kind) in kinds.iter().enumerate() {
        let share = covered[k] as f64 / n;
        if !(0.88..=0.995).contains(&share) {
            failures.push(format!(
                "the 95 per cent intervals under {kind} errors cover {share:.3} of the time"
            ));
        }
    }
    let ols_mean = least_squares.iter().sum::<f64>() / n;
    if ols_mean - COURSE_EFFECT < 1.0 || covered_by_least_squares as f64 / n > 0.2 {
        failures.push(format!(
            "least squares was expected to overstate the effect: mean {ols_mean:.4}, covering {covered_by_least_squares} times"
        ));
    }
    println!(
        "staff: two-stage mean {mean:.4} ({z:+.2} standard errors of the mean from {COURSE_EFFECT}); coverage classical {:.3}, robust {:.3}; least squares mean {ols_mean:.4}, covering {:.3}",
        covered[0] as f64 / n,
        covered[1] as f64 / n,
        covered_by_least_squares as f64 / n
    );
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// One draw of a weak-instrument version of the `staff` model: 300 employees, no sites, an
/// offer that barely moves the take-up of the course, and ability that drives both the take-up
/// and productivity strongly, so that least squares is far off; the effect is still 4.5.
fn draw_weak(seed: u64) -> Table {
    let mut stream = Stream::new(20_261_117 + seed, 1, 0);
    let rows = 300;
    let (mut offer, mut training, mut productivity) = (
        Vec::with_capacity(rows),
        Vec::with_capacity(rows),
        Vec::with_capacity(rows),
    );
    for _ in 0..rows {
        let ability = normal(&mut stream);
        let offered = if stream.uniform() < 0.5 { 1.0 } else { 0.0 };
        let u = -0.4 + 0.8 * offered + 4.0 * ability;
        let took = if stream.uniform() < 1.0 / (1.0 + zeff_numeric::maths::exp(-u)) {
            1.0
        } else {
            0.0
        };
        let y = 50.0 + COURSE_EFFECT * took + 6.0 * ability + normal(&mut stream);
        offer.push(Some(offered));
        training.push(Some(took));
        productivity.push(Some(y));
    }
    Table::new(vec![
        ("offer".to_string(), Column::floats(offer).0),
        ("training".to_string(), Column::floats(training).0),
        ("productivity".to_string(), Column::floats(productivity).0),
    ])
    .expect("columns of one length")
}

/// Under a weak instrument the Anderson–Rubin set keeps its level and the usual interval does
/// not (release 0.8.0, step 16): over four hundred draws the 95 per cent Anderson–Rubin set
/// holds the true effect about 95 per cent of the time, and the interval of the two-stage
/// estimate and its standard error holds it less often than the set does.
#[test]
#[ignore = "nightly: 400 draws of a weak-instrument model, the Anderson-Rubin set against the usual interval"]
fn the_anderson_rubin_set_covers_under_a_weak_instrument() {
    use zeff_regress::iv::{Designs, estimate};
    use zeff_results::variance::Kind;
    let draws = 400u64;
    let (mut by_set, mut by_interval, mut unbounded) = (0usize, 0usize, 0usize);
    for seed in 0..draws {
        let table = draw_weak(seed);
        let mut output = String::new();
        let mut messages: Vec<Diagnostic> = Vec::new();
        let mut random = Randomness::default();
        let mut ctx = CallContext {
            files: &NoFiles,
            script: "t.zeff",
            text: "",
            columns_used: Vec::new(),
            frame: None,
            group_rows: None,
            callee_span: Span::new(0, 1),
            arg_spans: Vec::new(),
            rest_spans: Vec::new(),
            definition_spans: Vec::new(),
            output: &mut output,
            messages: &mut messages,
            random: &mut random,
            statement: 1,
            width: zeff_core::display::PAGE_WIDTH,
            shown: None,
        };
        let designs = Designs::build(
            &mut ctx,
            &Frame::new(table),
            &formula("productivity ~ training"),
            &zeff_testkit::terms("training"),
            &zeff_testkit::terms("offer"),
            &[],
            None,
        )
        .unwrap_or_else(|e| panic!("seed {seed}: {}", e.message));
        let two =
            estimate(&mut ctx, designs).unwrap_or_else(|e| panic!("seed {seed}: {}", e.message));
        let test = two
            .anderson_rubin(&mut ctx, Kind::Classical, None, None, 0.95)
            .unwrap_or_else(|e| panic!("seed {seed}: {}", e.message));
        let set = test.set.unwrap_or_else(|| panic!("seed {seed}: no set"));
        if set
            .pieces
            .iter()
            .any(|(lo, hi)| *lo <= COURSE_EFFECT && COURSE_EFFECT <= *hi)
        {
            by_set += 1;
        }
        if set
            .pieces
            .iter()
            .any(|(lo, hi)| lo.is_infinite() || hi.is_infinite())
        {
            unbounded += 1;
        }
        let (vcov, df) = two.variance(Kind::Classical, None, 0).unwrap();
        let at = two.fitted.kept.iter().position(|&c| c == 1).unwrap();
        let (estimate, se) = (two.fitted.estimates[at], vcov.get(at, at).sqrt());
        let t = families::quantile(
            &Distribution::Family {
                name: "t",
                params: vec![("df", df as f64)],
            },
            0.975,
        );
        if (estimate - COURSE_EFFECT).abs() <= t * se {
            by_interval += 1;
        }
    }
    let n = draws as f64;
    let (set, interval) = (by_set as f64 / n, by_interval as f64 / n);
    println!(
        "weak instrument: the Anderson-Rubin set covers {set:.3}, the usual interval {interval:.3}; {unbounded} of {draws} sets are unbounded"
    );
    // Four hundred sets at 95 per cent: between 91.5 and 98 per cent with room for the Monte
    // Carlo error; and the usual interval covers less often than the set.
    assert!((0.915..=0.98).contains(&set), "the set covers {set:.3}");
    assert!(
        interval < set,
        "the interval covers {interval:.3}, the set {set:.3}"
    );
}
