//! `fit.coef_plot()` held to its fit (Language Reference, section 7.6, "Result plots";
//! release 0.7.0, step 30c): a point at each estimate and a bar between the limits of its
//! confidence interval, the numbers the fit itself holds and `estimates()` and `ci()` give;
//! a term on each line in the order of the table, the intercept left out; the intervals
//! recorded in the scene as confidence intervals at the level of the fit, and said in the
//! caption (decision #118, F11).

use zeff_columns::Table;
use zeff_core::runtime::{Args, CallContext, Fail, Files, Randomness};
use zeff_core::value::{Coord, Fit, Mark, Plot, Value};
use zeff_diagnostics::{Diagnostic, Source, Sources, Span};
use zeff_plot::scene::{Interval, Scene};
use zeff_testkit::{formula, run, terms};

struct NoFiles;
impl Files for NoFiles {
    fn read(&self, _: &str, _: Span, _: &str) -> Result<Vec<u8>, Fail> {
        unreachable!("the tests read no file through the engine")
    }
    fn write(&self, _: &str, _: &[u8], _: Span, _: &str) -> Result<(), Fail> {
        unreachable!("the tests write no file")
    }
}

/// A fit of `text` on `table` at `level`, and its coefficient plot with its scene.
fn plotted(table: &Table, text: &str, fe: Option<&str>, level: Option<f64>) -> (Fit, Plot, Scene) {
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
            Some(Value::Frame(zeff_core::value::Frame::new(table.clone()))),
            Some(Value::Formula(formula(text))),
            fe.map(|fe| Value::Terms(terms(fe))),
            None,
            None,
            None,
            None,
            level.map(Value::Float),
        ],
        rest: Vec::new(),
        definitions: Vec::new(),
    };
    let fit = match zeff_regress::reg::reg(&mut ctx, args) {
        Ok(Value::Fit(fit)) => fit,
        Ok(other) => panic!("not a fit: {other:?}"),
        Err(e) => panic!("{}", e.render(&sources)),
    };
    let plot = match zeff_plot::fits::coef_plot(
        &mut ctx,
        Args {
            values: vec![Some(Value::Fit(fit.clone()))],
            rest: Vec::new(),
            definitions: Vec::new(),
        },
    ) {
        Ok(Value::Plot(plot)) => (*plot).clone(),
        Ok(other) => panic!("not a plot: {other:?}"),
        Err(e) => panic!("{}", e.render(&sources)),
    };
    let scene = zeff_plot::scene(&plot);
    ((*fit).clone(), plot, scene)
}

fn wages() -> Table {
    zeff_datasets::table("wages").expect("the shipped wages")
}

/// The plot against its fit: a line at zero, a bar and a point for each term but the
/// intercept, each at the line of its term, the first term at the top, with the estimate
/// and the limits of the fit.
fn agrees(fit: &Fit, plot: &Plot, scene: &Scene) {
    let layers = &scene.panels[0].layers;
    assert_eq!(
        layers.iter().map(|l| l.mark).collect::<Vec<_>>(),
        [Mark::Hline, Mark::Errorbars, Mark::Points]
    );
    assert_eq!(plot.coord, Coord::Flip);
    let drawn: Vec<usize> = (0..fit.terms.len())
        .filter(|&at| fit.terms[at] != "(intercept)")
        .collect();
    let (bars, points) = (&layers[1], &layers[2]);
    assert_eq!((bars.x.len(), points.x.len()), (drawn.len(), drawn.len()));
    // The axis of the terms names them from the foot upwards: the last term first.
    let names = &scene.scale("x").unwrap().labels;
    let wanted: Vec<&String> = drawn.iter().rev().map(|&at| &fit.terms[at]).collect();
    assert_eq!(names.iter().collect::<Vec<_>>(), wanted);
    assert_eq!(scene.scale("x").unwrap().title, "");
    assert_eq!(scene.scale("y").unwrap().title, "estimate");
    for (k, &at) in drawn.iter().enumerate() {
        // Term `k` of those drawn stands at the place counted from the top.
        let place = (drawn.len() - k) as f64;
        let i = (0..points.x.len())
            .find(|&i| points.x[i] == place)
            .unwrap_or_else(|| panic!("no point for {}", fit.terms[at]));
        assert_eq!(points.y[i], fit.estimates[at], "{}", fit.terms[at]);
        let j = (0..bars.x.len()).find(|&j| bars.x[j] == place).unwrap();
        assert_eq!(
            (bars.low[j], bars.y[j]),
            fit.intervals[at],
            "{}",
            fit.terms[at]
        );
        // Without caps.
        assert_eq!(bars.span[j], 0.0);
    }
    // The line of no effect.
    assert_eq!(layers[0].y, [0.0]);
    // What the intervals are: recorded, and said.
    assert_eq!(bars.interval, Some(Interval::Confidence(fit.level)));
    assert_eq!((points.interval, layers[0].interval), (None, None));
}

#[test]
fn the_plot_holds_the_estimates_and_the_intervals_of_its_fit() {
    let (fit, plot, scene) = plotted(&wages(), "log(wage) ~ educ + exper + union", None, None);
    agrees(&fit, &plot, &scene);
    assert_eq!(
        scene.caption.as_deref(),
        Some("Bars: 95% confidence intervals")
    );
    assert!(scene.json().write().contains("\"interval\": \"ci 0.95\""));
    insta::assert_snapshot!("three_coefficients", scene.json().write());
    // At another level the intervals are the fit's at that level, and the caption says it.
    let (fit, plot, scene) = plotted(
        &wages(),
        "log(wage) ~ educ + exper + exper^2 + female + industry",
        None,
        Some(0.9),
    );
    agrees(&fit, &plot, &scene);
    assert_eq!(fit.level, 0.9);
    assert_eq!(
        scene.caption.as_deref(),
        Some("Bars: 90% confidence intervals")
    );
    assert!(scene.json().write().contains("\"interval\": \"ci 0.9\""));
    // With fixed effects there is no intercept to leave out, and the terms are those
    // estimated.
    let (fit, plot, scene) = plotted(&wages(), "log(wage) ~ exper + hours", Some("worker"), None);
    assert!(!fit.terms.contains(&"(intercept)".to_string()));
    agrees(&fit, &plot, &scene);
    assert_eq!(scene.panels[0].layers[2].x.len(), fit.terms.len());
    // Without an intercept every term is drawn.
    let (fit, plot, scene) = plotted(&wages(), "log(wage) ~ educ + exper - 1", None, None);
    agrees(&fit, &plot, &scene);
    assert_eq!(scene.panels[0].layers[2].x.len(), 2);
}

#[test]
fn coef_plot_as_a_script_writes_it() {
    let outcome = run(
        "let w = dataset(wages)\nlet m = w.reg(log(wage) ~ educ + exper + union)\nm.coef_plot()\nm.coef_plot().labels(caption: \"Wages of 2010 to 2021\", title: \"Returns\")\nw.reg(log(wage) ~ educ + industry, se: robust, level: 0.99).coef_plot()\nexport(m.coef_plot(), \"coef.svg\")\nexport(m.coef_plot(), \"coef.pdf\")\nexport(m.coef_plot(), \"coef.png\")\n",
    );
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    let text = &outcome.output.stdout;
    assert!(text.contains("interval ci 0.95, width 0"), "{text}");
    assert!(
        text.contains("caption: Bars: 95% confidence intervals"),
        "{text}"
    );
    assert!(
        text.contains("caption: Bars: 99% confidence intervals"),
        "{text}"
    );
    // The script's own caption takes the place of the engine's.
    assert!(text.contains("caption: Wages of 2010 to 2021"), "{text}");
    insta::assert_snapshot!(
        "coef_plots_printed",
        format!("{text}---\n{}", outcome.output.stderr)
    );
    // What has no coefficients to draw.
    let mut all = String::new();
    for (name, script) in [
        (
            "of_a_test",
            "let d = frame(y = [1.0, 2.0, 4.0, 3.0])\nd.t_test(y).coef_plot()\n",
        ),
        (
            "of_a_frame",
            "let d = frame(y = [1, 2, 3])\nd.coef_plot()\n",
        ),
        (
            "of_the_intercept_alone",
            "let d = frame(y = [1.0, 3.0, 2.0, 5.0])\nd.reg(y ~ 1).coef_plot()\n",
        ),
        (
            "with_an_argument",
            "let d = frame(y = [1.0, 3.0, 2.0, 5.0], x = [1, 2, 3, 4])\nd.reg(y ~ x).coef_plot(level: 0.9)\n",
        ),
    ] {
        let outcome = run(script);
        assert_eq!(outcome.exit_code, 1, "{name}\n{}", outcome.output.stdout);
        all.push_str(&format!("== {name}\n{}\n", outcome.output.stderr));
    }
    insta::assert_snapshot!("coef_plot_messages", all);
}

// ----- `fit.plot()`, the default view of a fit (step 30d) -----

/// The default plot of a fit, with its scene.
fn default_plot(script: &str) -> String {
    let outcome = run(script);
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    format!("{}---\n{}", outcome.output.stdout, outcome.output.stderr)
}

/// Sixteen rows: four units over four years, three of them treated in different years, as
/// the help page of `event` has them.
const PANEL: &str = "let d = frame(unit = [1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4], year = [2010, 2011, 2012, 2013, 2010, 2011, 2012, 2013, 2010, 2011, 2012, 2013, 2010, 2011, 2012, 2013], treat = [2011, 2011, 2011, 2011, 2012, 2012, 2012, 2012, 2013, 2013, 2013, 2013, NA, NA, NA, NA], y = [1.0, 2.1, 2.4, 2.6, 1.5, 1.4, 2.6, 2.9, 0.8, 0.9, 1.1, 2.0, 1.2, 1.3, 1.1, 1.4])\nlet m = d.reg(y ~ event(year, treat), fe: unit + year)\n";

#[test]
fn the_default_plot_of_a_regression_is_its_coefficient_plot() {
    // `m.plot()` prints what `m.coef_plot()` prints, and exports the same figure.
    let both = default_plot(
        "let w = dataset(wages)\nlet m = w.reg(log(wage) ~ educ + exper + union)\nm.plot()\nm.coef_plot()\n",
    );
    let printed: Vec<&str> = both.split("plot · m · estimate against term").collect();
    assert_eq!(printed.len(), 3, "{both}");
    assert_eq!(
        printed[1].trim_end(),
        printed[2].split("---").next().unwrap().trim_end(),
        "{both}"
    );
}

#[test]
fn the_default_plot_of_an_event_study_is_the_estimates_by_period() {
    // The fit of the event study, by the script; and its numbers, by its table.
    let text = default_plot(&format!(
        "{PANEL}m.plot()\nm.estimates()\nm.plot().labels(title: \"Effect by year\")\nexport(m.plot(), \"event.svg\")\nexport(m.plot(), \"event.pdf\")\nexport(m.plot(), \"event.png\")\n"
    ));
    // The periods along the foot, each with its break, the base period among them; the
    // estimates up the side; the intervals said to be what they are.
    assert!(
        text.contains("  x      period relative to the event   linear   -3.25 to 2.25          -3, -2, -1, 0, 1, 2"),
        "{text}"
    );
    assert!(
        text.contains(
            "2   errorbars   m          5   low ci_lower, high ci_upper, interval ci 0.95, width 0"
        ),
        "{text}"
    );
    assert!(
        text.contains("3   points      m          5   size 4.5, alpha 1"),
        "{text}"
    );
    assert!(
        text.contains("caption: Bars: 95% confidence intervals"),
        "{text}"
    );
    assert!(!text.contains("coordinates: flipped"), "{text}");
    insta::assert_snapshot!("an_event_study_plotted", text);
}

#[test]
fn the_event_study_holds_the_estimates_of_its_fit() {
    // Through the bodies: the point of each period is the estimate of its term, and its bar
    // the interval of the fit.
    let table = zeff_columns::Table::new(vec![
        (
            "unit".to_string(),
            zeff_columns::Column::ints([1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4].map(Some)),
        ),
        (
            "year".to_string(),
            zeff_columns::Column::ints(
                [
                    2010, 2011, 2012, 2013, 2010, 2011, 2012, 2013, 2010, 2011, 2012, 2013, 2010,
                    2011, 2012, 2013,
                ]
                .map(Some),
            ),
        ),
        (
            "treat".to_string(),
            zeff_columns::Column::ints([
                Some(2011),
                Some(2011),
                Some(2011),
                Some(2011),
                Some(2012),
                Some(2012),
                Some(2012),
                Some(2012),
                Some(2013),
                Some(2013),
                Some(2013),
                Some(2013),
                None,
                None,
                None,
                None,
            ]),
        ),
        (
            "y".to_string(),
            zeff_columns::Column::floats(
                [
                    1.0, 2.1, 2.4, 2.6, 1.5, 1.4, 2.6, 2.9, 0.8, 0.9, 1.1, 2.0, 1.2, 1.3, 1.1, 1.4,
                ]
                .map(Some),
            )
            .0,
        ),
    ])
    .unwrap();
    let (fit, _, _) = plotted(&table, "y ~ event(year, treat)", Some("unit + year"), None);
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
        arg_spans: vec![Some(Span::new(0, 1))],
        rest_spans: Vec::new(),
        definition_spans: Vec::new(),
        output: &mut output,
        messages: &mut messages,
        random: &mut random,
        statement: 1,
    };
    let Ok(Value::Plot(plot)) = zeff_plot::fits::plot_of_a_fit(
        &mut ctx,
        Args {
            values: vec![Some(Value::Fit(std::rc::Rc::new(fit.clone())))],
            rest: Vec::new(),
            definitions: Vec::new(),
        },
    ) else {
        panic!("no plot of the event study")
    };
    let scene = zeff_plot::scene(&plot);
    let layers = &scene.panels[0].layers;
    assert_eq!(
        layers.iter().map(|l| l.mark).collect::<Vec<_>>(),
        [Mark::Hline, Mark::Errorbars, Mark::Points]
    );
    assert_eq!(plot.coord, Coord::Cartesian);
    let (bars, points) = (&layers[1], &layers[2]);
    let mut seen = 0;
    for (at, term) in fit.terms.iter().enumerate() {
        let Some(period) = term
            .strip_prefix("event=")
            .and_then(|p| p.parse::<f64>().ok())
        else {
            continue;
        };
        seen += 1;
        let i = (0..points.x.len())
            .find(|&i| points.x[i] == period)
            .unwrap();
        assert_eq!(points.y[i], fit.estimates[at], "{term}");
        let j = (0..bars.x.len()).find(|&j| bars.x[j] == period).unwrap();
        assert_eq!((bars.low[j], bars.y[j]), fit.intervals[at], "{term}");
    }
    assert_eq!((seen, points.x.len()), (5, 5));
    // The periods in their order along the axis, the base period having none.
    assert_eq!(points.x, [-3.0, -2.0, 0.0, 1.0, 2.0]);
    assert_eq!(bars.interval, Some(Interval::Confidence(0.95)));
    assert_eq!(
        scene.scale("x").unwrap().breaks,
        [-3.0, -2.0, -1.0, 0.0, 1.0, 2.0]
    );
}

#[test]
fn every_message_of_the_plot_of_a_fit() {
    let mut all = String::new();
    for (name, script) in [
        (
            "of_a_test",
            "let d = frame(y = [1.0, 2.0, 4.0, 3.0])\nd.t_test(y).plot()\n",
        ),
        (
            "of_the_intercept_alone",
            "let d = frame(y = [1.0, 3.0, 2.0, 5.0, 4.0, 6.0])\nd.reg(y ~ 1).plot()\n",
        ),
        (
            "with_a_mapping_as_of_a_frame",
            "let d = frame(y = [1.0, 3.0, 2.0, 5.0], x = [1, 2, 3, 4])\nd.reg(y ~ x).plot(x: x)\n",
        ),
        ("of_a_number", "let k = 3\nk.plot()\n"),
    ] {
        let outcome = run(script);
        assert_eq!(outcome.exit_code, 1, "{name}\n{}", outcome.output.stdout);
        all.push_str(&format!("== {name}\n{}\n", outcome.output.stderr));
    }
    insta::assert_snapshot!("plot_of_a_fit_messages", all);
}
