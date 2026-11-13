//! The shorthands (Language Reference, section 7.6, "Shorthands"; release 0.7.0, step 29):
//! each gives the `Plot` of its grammar form, value for value, and so the same scene byte
//! for byte, with the same notes; and where the grammar refuses, the shorthand refuses with
//! the same words.

mod common;

use common::*;
use proptest::prelude::*;
use zeff_core::runtime::{Args, Body, CallContext, Fail, Files, Randomness};
use zeff_core::value::{Frame, Value};
use zeff_diagnostics::{Diagnostic, Span};
use zeff_plot::{grammar, marks, scales, scene, shorthands};

struct NoFiles;

impl Files for NoFiles {
    fn read(&self, _: &str, _: Span, _: &str) -> Result<Vec<u8>, Fail> {
        unreachable!("a plot reads no file")
    }
}

/// What a call gave: the value, or the words of its refusal; and the notes it raised.
type Gave = (Result<Value, String>, Vec<String>);

/// A chain of calls, each given the value of the one before as its first argument, as a
/// script's statement runs them: `steps` are the bodies with the arguments after the first.
fn chain(first: Value, steps: Vec<(Body, Vec<Option<Value>>)>) -> Gave {
    let mut output = String::new();
    let mut messages: Vec<Diagnostic> = Vec::new();
    let mut random = Randomness::default();
    let mut value = first;
    let mut refusal = None;
    for (body, more) in steps {
        let mut values = vec![Some(value.clone())];
        values.extend(more);
        let mut ctx = CallContext {
            files: &NoFiles,
            script: "t.zeff",
            text: "",
            columns_used: Vec::new(),
            frame: None,
            group_rows: None,
            callee_span: Span::new(0, 1),
            arg_spans: (0..values.len())
                .map(|at| Some(Span::new(at, at + 1)))
                .collect(),
            rest_spans: Vec::new(),
            definition_spans: Vec::new(),
            output: &mut output,
            messages: &mut messages,
            random: &mut random,
            statement: 1,
        };
        match body(
            &mut ctx,
            Args {
                values,
                rest: Vec::new(),
                definitions: Vec::new(),
            },
        ) {
            Ok(next) => value = next,
            Err(refused) => {
                refusal = Some(format!("{}: {}", refused.code, refused.message));
                break;
            }
        }
    }
    let notes = messages.iter().map(|m| m.message.clone()).collect();
    (refusal.map_or(Ok(value), Err), notes)
}

fn var(name: &str) -> Option<Value> {
    Some(Value::Var(name.to_string()))
}

/// The scene of what a call gave, as it is written.
fn written(gave: &Gave) -> Result<String, String> {
    match &gave.0 {
        Ok(Value::Plot(plot)) => Ok(scene(plot).json().write()),
        Ok(_) => Err("no plot".to_string()),
        Err(words) => Err(words.clone()),
    }
}

/// `data.line(x, y, color:, by:)` and its grammar form.
fn lines_both(frame: &Frame, color: Option<&str>, by: Option<&str>) -> (Gave, Gave) {
    let data = Value::Frame(frame.clone());
    let short = chain(
        data.clone(),
        vec![(
            shorthands::line,
            vec![var("x"), var("y"), color.and_then(var), by.and_then(var)],
        )],
    );
    let mut steps: Vec<(Body, Vec<Option<Value>>)> = vec![
        (grammar::plot, vec![var("x"), var("y"), color.and_then(var)]),
        (marks::lines, Vec::new()),
    ];
    if let Some(by) = by {
        steps.push((scales::facet, vec![var(by)]));
    }
    (short, chain(data, steps))
}

/// `data.bar(var, color:, by:)` and its grammar form.
fn bars_both(frame: &Frame, of: &str, color: Option<&str>, by: Option<&str>) -> (Gave, Gave) {
    let data = Value::Frame(frame.clone());
    let short = chain(
        data.clone(),
        vec![(
            shorthands::bar,
            vec![var(of), color.and_then(var), by.and_then(var)],
        )],
    );
    let mut steps: Vec<(Body, Vec<Option<Value>>)> = vec![
        (grammar::plot, vec![var(of), None, color.and_then(var)]),
        (marks::bars, Vec::new()),
    ];
    if let Some(by) = by {
        steps.push((scales::facet, vec![var(by)]));
    }
    (short, chain(data, steps))
}

fn word(name: &str) -> Option<Value> {
    Some(Value::Word(name.to_string()))
}

/// `data.scatter(x, y, color:, by:)` and its grammar form.
fn points_both(frame: &Frame, color: Option<&str>, by: Option<&str>) -> (Gave, Gave) {
    let data = Value::Frame(frame.clone());
    let short = chain(
        data.clone(),
        vec![(
            shorthands::scatter,
            vec![
                var("x"),
                var("y"),
                None,
                color.and_then(var),
                by.and_then(var),
            ],
        )],
    );
    let mut steps: Vec<(Body, Vec<Option<Value>>)> = vec![
        (grammar::plot, vec![var("x"), var("y"), color.and_then(var)]),
        (marks::points, Vec::new()),
    ];
    if let Some(by) = by {
        steps.push((scales::facet, vec![var(by)]));
    }
    (short, chain(data, steps))
}

/// `data.scatter(x, y, fit: linear, color:, by:)`.
fn fitted(frame: &Frame, color: Option<&str>, by: Option<&str>) -> Gave {
    chain(
        Value::Frame(frame.clone()),
        vec![(
            shorthands::scatter,
            vec![
                var("x"),
                var("y"),
                word("linear"),
                color.and_then(var),
                by.and_then(var),
            ],
        )],
    )
}

/// The scene of a plot a call gave.
fn scene_of(gave: &Gave) -> zeff_plot::Scene {
    match &gave.0 {
        Ok(Value::Plot(plot)) => scene(plot),
        other => panic!("no plot: {other:?}"),
    }
}

/// The two gave one thing: the same `Plot`, the same scene as written, the same notes; or
/// the same refusal.
fn same(short: &Gave, long: &Gave) -> Result<(), String> {
    if short.0 != long.0 {
        return Err(format!("{:?}\nagainst\n{:?}", short.0, long.0));
    }
    if written(short) != written(long) {
        return Err("two scenes".to_string());
    }
    if short.1 != long.1 {
        return Err(format!("{:?}\nagainst\n{:?}", short.1, long.1));
    }
    Ok(())
}

#[test]
fn line_and_bar_are_their_grammar_forms() {
    let frame = small("d");
    // `small` has `dose`, `response`, `arm` and `weight`; the property's frames have `x`,
    // `y`, `g` and `z`, so this one is renamed to them.
    let frame = common::frame(
        "d",
        vec![
            ("x", frame.table.column("dose").unwrap().clone()),
            ("y", frame.table.column("response").unwrap().clone()),
            ("g", frame.table.column("arm").unwrap().clone()),
            ("z", frame.table.column("weight").unwrap().clone()),
        ],
    );
    for (color, by) in [
        (None, None),
        (Some("g"), None),
        (None, Some("g")),
        (Some("g"), Some("g")),
    ] {
        let (short, long) = lines_both(&frame, color, by);
        assert!(short.0.is_ok(), "{:?}", short.0);
        same(&short, &long).unwrap();
        let (short, long) = bars_both(&frame, "g", color, by);
        assert!(short.0.is_ok(), "{:?}", short.0);
        same(&short, &long).unwrap();
    }
    // What the grammar refuses the shorthand refuses, in the same words: a line coloured by
    // a number, a variable the frame lacks, a panel by one it lacks.
    for (color, by) in [(Some("z"), None), (Some("w"), None), (None, Some("w"))] {
        let (short, long) = lines_both(&frame, color, by);
        assert!(short.0.is_err(), "{color:?} {by:?}");
        same(&short, &long).unwrap();
    }
    for (color, by) in [(Some("w"), None), (None, Some("w"))] {
        let (short, long) = bars_both(&frame, "g", color, by);
        assert!(short.0.is_err(), "{color:?} {by:?}");
        same(&short, &long).unwrap();
    }
    // A bar has one fill: coloured by a number its bars are whole, as the grammar's are.
    let (short, long) = bars_both(&frame, "g", Some("z"), None);
    assert!(short.0.is_ok());
    same(&short, &long).unwrap();
    // A scene of each, kept.
    let (short, _) = lines_both(&frame, Some("g"), None);
    insta::assert_snapshot!("a_line_by_arm", written(&short).unwrap());
    let (short, _) = bars_both(&frame, "g", None, None);
    insta::assert_snapshot!("a_bar_for_each_arm", written(&short).unwrap());
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(96)))]

    /// Whatever the rows, with a colour or without, in panels or not: the shorthand gives
    /// what its grammar form gives, the plot, the scene byte for byte and the notes, or the
    /// same refusal.
    #[test]
    fn a_shorthand_gives_the_scene_of_its_grammar_form(
        rows in rows(),
        color in prop::sample::select(vec![None, Some("g"), Some("z")]),
        by in prop::sample::select(vec![None, Some("g")]),
    ) {
        let frame = tried(&rows);
        let (short, long) = lines_both(&frame, color, by);
        prop_assert_eq!(same(&short, &long), Ok(()));
        for of in ["g", "x"] {
            let (short, long) = bars_both(&frame, of, color, by);
            prop_assert_eq!(same(&short, &long), Ok(()));
        }
    }
}

// ----- scatter (step 29b) -----

/// Eight doses and their responses, the first four of one arm and the last four of another.
fn doses() -> Frame {
    common::frame(
        "d",
        vec![
            ("x", numbers(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0])),
            ("y", numbers(&[2.1, 3.9, 6.2, 7.8, 9.0, 11.5, 13.1, 16.4])),
            (
                "g",
                zeff_columns::Column::strs(["a", "a", "a", "a", "b", "b", "b", "b"].map(Some)),
            ),
            (
                "z",
                numbers(&[60.0, 72.5, 81.0, 55.0, 90.0, 68.0, 77.0, 64.0]),
            ),
        ],
    )
}

#[test]
fn scatter_without_a_fit_is_its_grammar_form() {
    for (color, by) in [
        (None, None),
        (Some("g"), None),
        (Some("z"), None),
        (None, Some("g")),
        (Some("g"), Some("g")),
    ] {
        let (short, long) = points_both(&doses(), color, by);
        assert!(short.0.is_ok(), "{:?}", short.0);
        same(&short, &long).unwrap();
    }
    for (color, by) in [(Some("w"), None), (None, Some("w"))] {
        let (short, long) = points_both(&doses(), color, by);
        assert!(short.0.is_err());
        same(&short, &long).unwrap();
    }
}

#[test]
fn the_fitted_line_and_its_band_are_those_of_lm_and_predict() {
    use zeff_core::value::Mark;
    use zeff_plot::scene::{Interval, Paint};
    let gave = fitted(&doses(), None, None);
    let drawn = scene_of(&gave);
    let layers = &drawn.panels[0].layers;
    // The band under the points, and the line over them.
    assert_eq!(
        layers.iter().map(|l| l.mark).collect::<Vec<_>>(),
        [Mark::Ribbon, Mark::Points, Mark::Lines]
    );
    let (band, line) = (&layers[0], &layers[2]);
    assert_eq!((band.x.len(), line.x.len()), (80, 80));
    // R 4.6.0: `m <- lm(y ~ x)` and `predict(m, data.frame(x = seq(1, 8, length.out =
    // 80)), interval = "confidence")` at the first, the fortieth and the last place.
    for (k, at, fit, lower, upper) in [
        (0, 1.0, 1.95, 1.11351009613819, 2.78648990386181),
        (
            39,
            4.45569620253165,
            8.66392405063291,
            8.2056740218147,
            9.12217407945112,
        ),
        (79, 8.0, 15.55, 14.7135100961382, 16.3864899038618),
    ] {
        let near = |a: f64, b: f64| (a - b).abs() < 1e-9;
        assert!(near(line.x[k], at) && near(band.x[k], at), "{k}");
        assert!(near(line.y[k], fit), "{k}: {}", line.y[k]);
        assert!(near(band.low[k], lower), "{k}: {}", band.low[k]);
        assert!(near(band.y[k], upper), "{k}: {}", band.y[k]);
    }
    // The band is a fifth opaque, and the scene says what it is; the figure says it too,
    // unless the script gives a caption of its own.
    assert_eq!(band.alpha, shorthands::BAND_ALPHA);
    assert_eq!(band.interval, Some(Interval::Confidence(0.95)));
    assert_eq!((line.interval, layers[1].interval), (None, None));
    assert!(drawn.json().write().contains("\"interval\": \"ci 0.95\""));
    assert_eq!(
        drawn.caption.as_deref(),
        Some("Band: 95% confidence interval of the fitted line")
    );
    // The axis of `y` reaches the band.
    let y = drawn.scale("y").unwrap();
    assert!(y.limits.0 < 1.11 && y.limits.1 > 16.4);
    insta::assert_snapshot!("eight_doses_fitted", drawn.json().write());

    // Coloured by the arm: a line and a band for each, of its own rows, in its colour.
    let drawn = scene_of(&fitted(&doses(), Some("g"), None));
    let (band, line) = (&drawn.panels[0].layers[0], &drawn.panels[0].layers[2]);
    assert_eq!((band.x.len(), line.x.len()), (160, 160));
    assert_eq!(
        (line.x[0], line.x[79], line.x[80], line.x[159]),
        (1.0, 4.0, 5.0, 8.0)
    );
    // `lm` of the first four rows: 0.15 + 1.94 x; of the last four: -2.97 + 2.38 x.
    assert!((line.y[0] - 2.09).abs() < 1e-9 && (line.y[79] - 7.91).abs() < 1e-9);
    assert!((line.y[80] - 8.93).abs() < 1e-9 && (line.y[159] - 16.07).abs() < 1e-9);
    assert!(matches!(&line.color, Paint::Scaled(values) if values[0] == 1.0 && values[159] == 2.0));
    assert!(matches!(&band.fill, Paint::Scaled(values) if values[0] == 1.0 && values[159] == 2.0));

    // Coloured by a number, on a ramp: the points are, and the line is one line of all the
    // rows in the colour of the theme.
    let drawn = scene_of(&fitted(&doses(), Some("z"), None));
    let layers = &drawn.panels[0].layers;
    assert!(matches!(&layers[1].color, Paint::Scaled(_)));
    assert!(matches!(&layers[2].color, Paint::Theme));
    assert_eq!(layers[2].x.len(), 80);
    assert!((layers[2].y[79] - 15.55).abs() < 1e-9);

    // In panels by the arm: each panel has the line of its rows.
    let drawn = scene_of(&fitted(&doses(), None, Some("g")));
    assert_eq!(drawn.panels.len(), 2);
    assert_eq!(drawn.panels[0].layers[2].x.len(), 80);
    assert!((drawn.panels[0].layers[2].y[79] - 7.91).abs() < 1e-9);
    assert!((drawn.panels[1].layers[2].y[0] - 8.93).abs() < 1e-9);
    assert!((drawn.panels[1].layers[0].low[0] - drawn.panels[1].layers[0].y[0]).abs() > 0.1);
}

#[test]
fn the_edge_cases_of_a_fitted_line() {
    // Two rows, and one value of `x`: no line, and a note; the points are drawn.
    for (xs, ys) in [
        (vec![1.0, 2.0], vec![3.0, 4.0]),
        (vec![2.0, 2.0, 2.0, 2.0], vec![1.0, 2.0, 3.0, 4.0]),
    ] {
        let frame = common::frame("d", vec![("x", numbers(&xs)), ("y", numbers(&ys))]);
        let gave = fitted(&frame, None, None);
        let drawn = scene_of(&gave);
        let layers = &drawn.panels[0].layers;
        assert_eq!(layers[1].x.len(), xs.len());
        assert!(layers[0].x.is_empty() && layers[2].x.is_empty());
        assert_eq!(
            gave.1,
            ["1 group with fewer than three rows, or with one value of x, has no fitted line"]
        );
    }
    // A group too small beside one that has its line; rows without a value are left out of
    // the points and of the fit, and counted once.
    let frame = common::frame(
        "d",
        vec![
            (
                "x",
                floats(&[Some(1.0), Some(2.0), Some(3.0), Some(4.0), Some(5.0), None]),
            ),
            (
                "y",
                floats(&[
                    Some(1.0),
                    Some(3.0),
                    Some(2.0),
                    Some(5.0),
                    Some(4.0),
                    Some(1.0),
                ]),
            ),
            (
                "g",
                zeff_columns::Column::strs(["a", "a", "a", "b", "b", "b"].map(Some)),
            ),
        ],
    );
    let gave = fitted(&frame, Some("g"), None);
    let drawn = scene_of(&gave);
    assert_eq!(drawn.panels[0].layers[2].x.len(), 80);
    assert_eq!(
        gave.1,
        [
            "1 row with NA in x or y dropped",
            "1 group with fewer than three rows, or with one value of x, has no fitted line",
        ]
    );
    // A line through every row: a band of no width, and a figure all the same.
    let frame = common::frame(
        "d",
        vec![
            ("x", numbers(&[1.0, 2.0, 3.0, 4.0])),
            ("y", numbers(&[2.0, 4.0, 6.0, 8.0])),
        ],
    );
    let drawn = scene_of(&fitted(&frame, None, None));
    let band = &drawn.panels[0].layers[0];
    assert!(
        band.low
            .iter()
            .zip(&band.y)
            .all(|(low, high)| (high - low).abs() < 1e-9)
    );
    assert!(
        zeff_plot::layout::layout(
            &drawn,
            zeff_plot::layout::Size::OneColumn,
            &zeff_plot::layout::JOURNAL
        )
        .is_ok()
    );
}

#[test]
fn on_a_log_axis_the_line_is_that_of_the_logarithms() {
    use zeff_core::value::{Aesthetic, AxisKind, PlotAxis};
    let gave = fitted(&doses(), None, None);
    let Ok(Value::Plot(plot)) = &gave.0 else {
        panic!("no plot")
    };
    let mut logged = (**plot).clone();
    logged.axes.push(PlotAxis::of(Aesthetic::Y, AxisKind::Log));
    let drawn = scene(&logged);
    let line = &drawn.panels[0].layers[2];
    // R: `lm(log10(y) ~ x)` has the fitted values 0.462000224445 at 1 and 1.274206197416
    // at 8.
    assert!(
        (line.y[0].log10() - 0.462000224445).abs() < 1e-9,
        "{}",
        line.y[0].log10()
    );
    assert!(
        (line.y[79].log10() - 1.274206197416).abs() < 1e-9,
        "{}",
        line.y[79].log10()
    );
    // Straight on the log axis: the logarithms of the fitted values rise by equal steps.
    let steps: Vec<f64> = line
        .y
        .windows(2)
        .map(|pair| pair[1].log10() - pair[0].log10())
        .collect();
    assert!(steps.iter().all(|step| (step - steps[0]).abs() < 1e-9));
    // The band holds the line, and is above zero.
    let band = &drawn.panels[0].layers[0];
    for k in 0..80 {
        assert!(0.0 < band.low[k] && band.low[k] < line.y[k] && line.y[k] < band.y[k]);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(96)))]

    /// Whatever the rows: without a fit `scatter` gives the scene of its grammar form; with
    /// one, the points are the same points, each band holds its line, the band and the line
    /// have the same places, and there is a line for each group of three rows or more with
    /// two values of `x`.
    #[test]
    fn a_scatter_is_its_points_and_a_fit_adds_to_them(
        rows in rows(),
        color in prop::sample::select(vec![None, Some("g"), Some("z")]),
        by in prop::sample::select(vec![None, Some("g")]),
    ) {
        let frame = tried(&rows);
        let (short, long) = points_both(&frame, color, by);
        prop_assert_eq!(same(&short, &long), Ok(()));
        let with_fit = fitted(&frame, color, by);
        prop_assert_eq!(with_fit.0.is_ok(), short.0.is_ok());
        if with_fit.0.is_ok() {
            let (plain, drawn) = (scene_of(&short), scene_of(&with_fit));
            prop_assert_eq!(plain.panels.len(), drawn.panels.len());
            for (before, after) in plain.panels.iter().zip(&drawn.panels) {
                let (band, points, line) = (&after.layers[0], &after.layers[1], &after.layers[2]);
                prop_assert_eq!(&before.layers[0].x, &points.x);
                prop_assert_eq!(&before.layers[0].y, &points.y);
                prop_assert_eq!(&band.x, &line.x);
                prop_assert_eq!(band.x.len() % 80, 0);
                for k in 0..band.x.len() {
                    prop_assert!(band.low[k] <= line.y[k] && line.y[k] <= band.y[k]);
                }
            }
        }
    }
}
