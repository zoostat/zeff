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

// ----- histogram, density and boxplot of a frame (step 29c2) -----

/// A shorthand of one variable and its grammar form: the plot's mappings, the layer with
/// its options, and the panels.
fn one_variable_both(
    frame: &Frame,
    short: (Body, Vec<Option<Value>>),
    mapped: Vec<Option<Value>>,
    layer: (Body, Vec<Option<Value>>),
    by: Option<&str>,
) -> (Gave, Gave) {
    let data = Value::Frame(frame.clone());
    let short = chain(data.clone(), vec![short]);
    let mut steps: Vec<(Body, Vec<Option<Value>>)> = vec![(grammar::plot, mapped), layer];
    if let Some(by) = by {
        steps.push((scales::facet, vec![var(by)]));
    }
    (short, chain(data, steps))
}

fn int(value: i64) -> Option<Value> {
    Some(Value::Int(value))
}

fn float(value: f64) -> Option<Value> {
    Some(Value::Float(value))
}

/// The three shorthands on a frame with `y`, the number, `g`, the categories, and `z`.
fn three_both(
    frame: &Frame,
    color: Option<&str>,
    by: Option<&str>,
    options: bool,
) -> Vec<(Gave, Gave)> {
    let (bins, scale) = if options {
        (int(4), word("density"))
    } else {
        (None, None)
    };
    let (adjust, kernel) = if options {
        (float(0.5), word("epanechnikov"))
    } else {
        (None, None)
    };
    vec![
        // `histogram(y, bins:, scale:, by:)` is `plot(x: y).histogram(bins:, scale:)`.
        one_variable_both(
            frame,
            (
                shorthands::histogram,
                vec![
                    var("y"),
                    bins.clone(),
                    None,
                    None,
                    scale.clone(),
                    by.and_then(var),
                ],
            ),
            vec![var("y")],
            (marks::histogram, vec![None, bins, None, None, scale]),
            by,
        ),
        // `density(y, color:, adjust:, kernel:, by:)` is `plot(x: y, color:).density(...)`.
        one_variable_both(
            frame,
            (
                shorthands::density,
                vec![
                    var("y"),
                    color.and_then(var),
                    None,
                    adjust.clone(),
                    kernel.clone(),
                    by.and_then(var),
                ],
            ),
            vec![var("y"), None, color.and_then(var)],
            (marks::density, vec![None, None, adjust, kernel]),
            by,
        ),
        // `boxplot(y, group:, by:)` is `plot(x: group, y: y).boxplot()`.
        one_variable_both(
            frame,
            (
                shorthands::boxplot,
                vec![var("y"), color.and_then(var), by.and_then(var)],
            ),
            vec![color.and_then(var), var("y")],
            (marks::boxplot, Vec::new()),
            by,
        ),
    ]
}

#[test]
fn histogram_density_and_boxplot_of_a_frame_are_their_grammar_forms() {
    let frame = doses();
    for (color, by, options) in [
        (None, None, false),
        (Some("g"), None, false),
        (None, Some("g"), true),
        (Some("g"), Some("g"), true),
    ] {
        for (short, long) in three_both(&frame, color, by, options) {
            assert!(short.0.is_ok(), "{:?}", short.0);
            same(&short, &long).unwrap();
        }
    }
    // What the grammar refuses the shorthand refuses, in the same words: a variable the
    // frame lacks for the colour, the group or the panels.
    for (color, by) in [(Some("w"), None), (None, Some("w"))] {
        for (at, (short, long)) in three_both(&frame, color, by, false).into_iter().enumerate() {
            // A histogram takes no colour: it is refused for its panels alone.
            if at == 0 && by.is_none() {
                continue;
            }
            assert!(short.0.is_err(), "{at} {color:?} {by:?}");
            same(&short, &long).unwrap();
        }
    }
    // A scene of each, kept.
    let all = three_both(&frame, None, None, false);
    insta::assert_snapshot!("a_histogram_of_a_frame", written(&all[0].0).unwrap());
    insta::assert_snapshot!("a_boxplot_of_a_frame", written(&all[2].0).unwrap());
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(64)))]

    /// Whatever the rows, with a colour or a group or without, in panels or not, with
    /// options or without: each of the three gives what its grammar form gives, the plot,
    /// the scene byte for byte and the notes, or the same refusal.
    #[test]
    fn the_three_of_a_frame_give_the_scenes_of_their_grammar_forms(
        rows in rows(),
        color in prop::sample::select(vec![None, Some("g")]),
        by in prop::sample::select(vec![None, Some("g")]),
        options in any::<bool>(),
    ) {
        let frame = tried(&rows);
        for (short, long) in three_both(&frame, color, by, options) {
            prop_assert_eq!(same(&short, &long), Ok(()));
        }
    }
}

// ----- qq_plot (step 29d) -----

fn normal() -> zeff_core::value::Distribution {
    zeff_core::value::Distribution::Family {
        name: "normal",
        params: vec![("mean", 0.0), ("sd", 1.0)],
    }
}

/// `data.qq_plot(y, distribution, by:)`.
fn qq(
    frame: &Frame,
    distribution: Option<zeff_core::value::Distribution>,
    by: Option<&str>,
) -> Gave {
    chain(
        Value::Frame(frame.clone()),
        vec![(
            shorthands::qq_plot,
            vec![
                var("y"),
                distribution.map(Value::Distribution),
                by.and_then(var),
            ],
        )],
    )
}

/// Ten scores, out of order, of two arms.
fn scores() -> Frame {
    common::frame(
        "d",
        vec![
            (
                "y",
                numbers(&[3.0, 1.0, 9.0, 2.0, 4.0, 3.0, 2.0, 5.0, 4.0, 3.0]),
            ),
            (
                "g",
                zeff_columns::Column::strs(
                    ["a", "a", "a", "a", "a", "b", "b", "b", "b", "b"].map(Some),
                ),
            ),
        ],
    )
}

#[test]
fn the_pairs_and_the_line_of_a_qq_plot_are_those_of_qqnorm_and_qqline() {
    use zeff_core::value::Mark;
    let gave = qq(&scores(), None, None);
    let drawn = scene_of(&gave);
    let layers = &drawn.panels[0].layers;
    assert_eq!(
        layers.iter().map(|l| l.mark).collect::<Vec<_>>(),
        [Mark::Points, Mark::Lines]
    );
    // A point for each row, in the order of the rows, drawn from the frame given: the
    // value along `y`, and along `x` the quantile of the rank of the value. R 4.6.0:
    // `qqnorm(c(3, 1, 9, 2, 4, 3, 2, 5, 4, 3), plot.it = FALSE)$x`.
    let points = &layers[0];
    assert_eq!(points.y, [3.0, 1.0, 9.0, 2.0, 4.0, 3.0, 2.0, 5.0, 4.0, 3.0]);
    assert_eq!(points.rows, (1..=10).collect::<Vec<_>>());
    assert_eq!(points.data.as_deref(), Some("d"));
    for (found, wanted) in points.x.iter().zip([
        -0.375461770235518,
        -1.54663527139923,
        1.54663527139923,
        -1.000490545619315,
        0.375461770235518,
        -0.122580843888802,
        -0.655423505234427,
        1.000490545619315,
        0.655423505234427,
        0.122580843888803,
    ]) {
        assert!((found - wanted).abs() < 1e-11, "{found} against {wanted}");
    }
    // The reference line: through the quartiles, 3.125 + 1.2972769411924 x, from the
    // smallest quantile to the largest.
    let line = &layers[1];
    assert_eq!(line.x.len(), 2);
    for k in 0..2 {
        let at = line.x[k];
        assert!((at.abs() - 1.54663527139923).abs() < 1e-11);
        assert!((line.y[k] - (3.125 + 1.2972769411924 * at)).abs() < 1e-9);
    }
    // The axis of the quantiles names the distribution, and that of the values the
    // variable.
    assert_eq!(
        drawn.scale("x").unwrap().title,
        "Quantiles of normal(mean: 0, sd: 1)"
    );
    assert_eq!(drawn.scale("y").unwrap().title, "y");
    assert!(gave.1.is_empty(), "{:?}", gave.1);
    insta::assert_snapshot!("ten_scores_against_the_normal", drawn.json().write());

    // Against another distribution: the quantiles are its own.
    let t = zeff_core::value::Distribution::Family {
        name: "t",
        params: vec![("df", 3.0)],
    };
    let drawn = scene_of(&qq(&scores(), Some(t), None));
    assert_eq!(drawn.scale("x").unwrap().title, "Quantiles of t(df: 3)");
    let largest = drawn.panels[0].layers[0].x[2];
    // `qt(ppoints(10)[10], 3)` is 2.13914803216.
    assert!((largest - 2.13914803216).abs() < 1e-10, "{largest}");

    // In panels by the arm: each panel has the pairs of its five rows, ranked among them,
    // and its own line.
    let drawn = scene_of(&qq(&scores(), Some(normal()), Some("g")));
    assert_eq!(drawn.panels.len(), 2);
    for panel in &drawn.panels {
        assert_eq!(panel.layers[0].x.len(), 5);
        assert_eq!(panel.layers[1].x.len(), 2);
        let mut quantiles = panel.layers[0].x.clone();
        quantiles.sort_by(f64::total_cmp);
        // `qnorm(ppoints(5))`, to the twelve digits R prints.
        for (found, wanted) in quantiles.iter().zip([
            -1.179761117612,
            -0.497200570682,
            0.0,
            0.497200570682,
            1.179761117612,
        ]) {
            assert!((found - wanted).abs() < 1e-11, "{found}");
        }
    }
    // The first arm is 3, 1, 9, 2, 4: its 9 has the largest quantile.
    assert_eq!(drawn.panels[0].layers[0].y[2], 9.0);
    assert!((drawn.panels[0].layers[0].x[2] - 1.179761117612).abs() < 1e-11);
}

#[test]
fn the_edge_cases_of_a_qq_plot() {
    // Rows without a value have no pair: they are left out and counted. One value has its
    // point and no line; no value, nothing.
    let frame = common::frame(
        "d",
        vec![
            (
                "y",
                floats(&[Some(2.0), None, Some(5.0), Some(f64::INFINITY), Some(1.0)]),
            ),
            (
                "g",
                zeff_columns::Column::strs(["a", "a", "a", "a", "b"].map(Some)),
            ),
        ],
    );
    let gave = qq(&frame, None, None);
    let drawn = scene_of(&gave);
    assert_eq!(drawn.panels[0].layers[0].y, [2.0, 5.0, 1.0]);
    assert_eq!(drawn.panels[0].layers[0].rows, [1, 3, 5]);
    assert_eq!(gave.1.len(), 1, "{:?}", gave.1);
    assert!(
        gave.1[0].starts_with("2 rows with NA in theoretical or y dropped"),
        "{:?}",
        gave.1
    );
    // By the group: the second has one value, a point and no line.
    let gave = qq(&frame, None, Some("g"));
    let drawn = scene_of(&gave);
    assert_eq!(drawn.panels[1].layers[0].x, [0.0]);
    assert!(drawn.panels[1].layers[1].x.is_empty());
    assert!(
        gave.1
            .contains(&"1 group with fewer than two values has no reference line".to_string()),
        "{:?}",
        gave.1
    );
    // Every value equal: the points lie level, and so does the line.
    let level = common::frame("d", vec![("y", numbers(&[4.0, 4.0, 4.0, 4.0]))]);
    let drawn = scene_of(&qq(&level, None, None));
    assert_eq!(drawn.panels[0].layers[1].y, [4.0, 4.0]);
    // A frame that has a variable called `theoretical` already is refused; so is a variable
    // that is not numbers, and a panel by a variable that is not there.
    let taken = common::frame(
        "d",
        vec![
            ("y", numbers(&[1.0, 2.0])),
            ("theoretical", numbers(&[1.0, 2.0])),
        ],
    );
    let refused = qq(&taken, None, None).0.unwrap_err();
    assert!(refused.contains("has one already"), "{refused}");
    let refused = chain(
        Value::Frame(scores()),
        vec![(shorthands::qq_plot, vec![var("g")])],
    )
    .0
    .unwrap_err();
    assert!(refused.contains("so it must be numbers"), "{refused}");
    let refused = qq(&scores(), None, Some("w")).0.unwrap_err();
    assert!(refused.contains("no variable `w`"), "{refused}");
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(96)))]

    /// Whatever the rows, in panels or not: a point for each row with a finite value and for
    /// no other, at its value; the quantiles of a panel rise with the values; and a
    /// reference line where a panel has two values or more.
    #[test]
    fn a_qq_plot_has_a_point_for_each_value_and_they_rise_together(
        rows in rows(),
        by in prop::sample::select(vec![None, Some("g")]),
    ) {
        let frame = tried(&rows);
        let gave = qq(&frame, None, by);
        prop_assert!(gave.0.is_ok(), "{:?}", gave.0);
        let drawn = scene_of(&gave);
        let finite = rows.iter().filter(|row| row.1.is_some_and(f64::is_finite)).count();
        let mut seen = 0;
        for panel in &drawn.panels {
            let points = &panel.layers[0];
            seen += points.x.len();
            let mut pairs: Vec<(f64, f64)> = points.y.iter().copied().zip(points.x.iter().copied()).collect();
            pairs.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.total_cmp(&b.1)));
            prop_assert!(pairs.windows(2).all(|pair| pair[0].1 < pair[1].1));
            for (at, row) in points.rows.iter().enumerate() {
                prop_assert_eq!(Some(points.y[at]), rows[row - 1].1);
            }
            let line = &panel.layers[1];
            prop_assert_eq!(line.x.len(), if points.x.len() >= 2 { 2 } else { 0 });
        }
        prop_assert_eq!(seen, finite);
    }
}

// ----- heatmap (step 29e) -----

fn vars(names: &[&str]) -> Option<Value> {
    Some(Value::Vars(
        names.iter().map(|name| (*name).to_string()).collect(),
    ))
}

/// `data.heatmap(vars)`.
fn heat(frame: &Frame, names: &[&str]) -> Gave {
    chain(
        Value::Frame(frame.clone()),
        vec![(shorthands::heatmap, vec![vars(names)])],
    )
}

/// Three variables of five rows.
fn three() -> Frame {
    common::frame(
        "d",
        vec![
            ("a", numbers(&[1.0, 2.0, 3.0, 4.0, 5.0])),
            ("b", numbers(&[2.0, 4.0, 5.0, 4.0, 5.0])),
            ("c", numbers(&[5.0, 3.0, 4.0, 2.0, 1.0])),
        ],
    )
}

#[test]
fn a_heat_map_is_the_correlations_as_tiles_from_minus_one_to_one() {
    use zeff_core::value::Mark;
    use zeff_plot::scene::{Paint, Range};
    let gave = heat(&three(), &["a", "b", "c"]);
    let drawn = scene_of(&gave);
    let layer = &drawn.panels[0].layers[0];
    assert_eq!(layer.mark, Mark::Tile);
    assert_eq!(layer.x.len(), 9);
    // The variables along the foot in the order given, and down the side with the first at
    // the top; neither axis has a title.
    let (x, y) = (drawn.scale("x").unwrap(), drawn.scale("y").unwrap());
    assert_eq!(x.labels, ["a", "b", "c"]);
    assert_eq!(y.labels, ["c", "b", "a"]);
    assert_eq!((x.title.as_str(), y.title.as_str()), ("", ""));
    // The scale of the fill runs from -1 to 1 whatever the correlations are, through the
    // three colours.
    let fill = drawn.scale("fill").unwrap();
    assert_eq!(fill.limits, (-1.0, 1.0));
    assert_eq!(fill.title, "correlation");
    let Range::Colours(ramp) = &fill.range else {
        panic!("a ramp of colours")
    };
    assert_eq!(
        ramp.first().map(String::as_str),
        Some(shorthands::HEAT_COLOURS[0])
    );
    assert_eq!(
        ramp.last().map(String::as_str),
        Some(shorthands::HEAT_COLOURS[2])
    );
    // R 4.6.0: `cor(data.frame(a, b, c))` is 0.774597 for a and b, -0.9 for a and c and
    // -0.645497 for b and c. Each tile is at its column and at the row of its variable,
    // counted from the foot.
    let Paint::Scaled(values) = &layer.fill else {
        panic!("a fill on a scale")
    };
    let at = |column: f64, row: f64| -> f64 {
        let found = (0..9)
            .find(|k| layer.x[*k] == column && (layer.y[*k] + layer.low[*k]) / 2.0 == row)
            .unwrap_or_else(|| panic!("no tile at {column}, {row}"));
        values[found]
    };
    // `a` is the first column and the third row from the foot.
    for (column, row, wanted) in [
        (1.0, 3.0, 1.0),
        (2.0, 3.0, 0.774597),
        (3.0, 3.0, -0.9),
        (1.0, 2.0, 0.774597),
        (2.0, 2.0, 1.0),
        (3.0, 2.0, -0.645497),
        (1.0, 1.0, -0.9),
        (2.0, 1.0, -0.645497),
        (3.0, 1.0, 1.0),
    ] {
        assert!(
            (at(column, row) - wanted).abs() < 1e-6,
            "{column}, {row}: {}",
            at(column, row)
        );
    }
    assert!(gave.1.is_empty(), "{:?}", gave.1);
    insta::assert_snapshot!("three_variables_correlated", drawn.json().write());
    // Another order of the variables is another order of the tiles.
    let drawn = scene_of(&heat(&three(), &["c", "a"]));
    assert_eq!(drawn.scale("x").unwrap().labels, ["c", "a"]);
    assert_eq!(drawn.scale("y").unwrap().labels, ["a", "c"]);
    assert_eq!(drawn.panels[0].layers[0].x.len(), 4);
}

#[test]
fn the_edge_cases_of_a_heat_map() {
    // Each pair on the rows that have both; a variable that does not vary has no
    // correlation, its tiles are left out, and a note names it.
    let frame = common::frame(
        "d",
        vec![
            (
                "a",
                floats(&[Some(1.0), Some(2.0), Some(3.0), Some(4.0), None]),
            ),
            (
                "b",
                floats(&[Some(2.0), None, Some(5.0), Some(4.0), Some(9.0)]),
            ),
            ("k", numbers(&[7.0, 7.0, 7.0, 7.0, 7.0])),
        ],
    );
    let gave = heat(&frame, &["a", "b", "k"]);
    let drawn = scene_of(&gave);
    let layer = &drawn.panels[0].layers[0];
    // The four tiles of `a` and `b`; the five of `k` are not drawn.
    assert_eq!(layer.x.len(), 4);
    let zeff_plot::scene::Paint::Scaled(values) = &layer.fill else {
        panic!("a fill on a scale")
    };
    // `cor(c(1, 3, 4), c(2, 5, 4))` is 0.785714.
    assert!(
        values.iter().any(|value| (value - 0.785714).abs() < 1e-6),
        "{values:?}"
    );
    assert!(
        gave.1
            .iter()
            .any(|note| note.starts_with("`k` has no correlation")),
        "{:?}",
        gave.1
    );
    // The scale keeps its two ends all the same.
    assert_eq!(drawn.scale("fill").unwrap().limits, (-1.0, 1.0));
    // One variable, one given twice, one that is not there and one of text are refused.
    for (names, words) in [
        (vec!["a"], "it needs two or more"),
        (vec!["a", "b", "a"], "is given `a` twice"),
        (vec!["a", "w"], "no variable `w`"),
    ] {
        let refused = heat(&frame, &names).0.unwrap_err();
        assert!(refused.contains(words), "{refused}");
    }
    let refused = heat(&doses(), &["x", "g"]).0.unwrap_err();
    assert!(refused.contains("which are of numbers"), "{refused}");
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(96)))]

    /// Whatever the rows: the tiles are the same above and below the diagonal, every
    /// correlation is between -1 and 1, a variable that has a tile has the tile of one on
    /// the diagonal, and the scale runs from -1 to 1.
    #[test]
    fn a_heat_map_is_the_same_above_and_below_its_diagonal(rows in rows()) {
        let frame = tried(&rows);
        let gave = heat(&frame, &["x", "y", "z"]);
        prop_assert!(gave.0.is_ok(), "{:?}", gave.0);
        let drawn = scene_of(&gave);
        let layer = &drawn.panels[0].layers[0];
        if let Some(fill) = drawn.scale("fill") {
            prop_assert_eq!(fill.limits, (-1.0, 1.0));
        }
        let values: Vec<f64> = match &layer.fill {
            zeff_plot::scene::Paint::Scaled(values) => values.clone(),
            _ => Vec::new(),
        };
        prop_assert!(values.len() == layer.x.len() || layer.x.is_empty());
        let tile = |column: usize, row: usize| -> Option<f64> {
            (0..layer.x.len())
                .find(|k| {
                    layer.x[*k] == column as f64
                        && (layer.y[*k] + layer.low[*k]) / 2.0 == row as f64
                })
                .map(|k| values[k])
        };
        for i in 1..=3usize {
            for j in 1..=3usize {
                // Variable `i` is column `i` and row `4 - i` from the foot.
                let (here, there) = (tile(j, 4 - i), tile(i, 4 - j));
                prop_assert_eq!(here.is_some(), there.is_some());
                if let (Some(here), Some(there)) = (here, there) {
                    prop_assert!((-1.0..=1.0).contains(&here));
                    prop_assert!((here - there).abs() < 1e-12);
                    if i == j {
                        prop_assert_eq!(here, 1.0);
                    }
                }
            }
        }
    }
}
