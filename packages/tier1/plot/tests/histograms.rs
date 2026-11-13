//! The histogram layer (Language Reference, section 7.6, "Statistical layers": `histogram`;
//! release 0.7.0, step 28a): the rows of `x` counted in bins of one width, a bar for each
//! bin. The numbers of bins are those of R's `nclass.Sturges`, `nclass.scott` and
//! `nclass.FD`, and the counts and densities those of `hist` on the same breaks (R 4.6.0).

mod common;

use common::*;
use proptest::prelude::*;
use zeff_columns::Column;
use zeff_core::value::{
    Aesthetic, Arrangement, BinRule, BinScale, Frame, Mark, Plot, PlotFacet, PlotLayer, Stat,
};
use zeff_plot::scene::{Scene, start, with_layer};

/// A layer that counts its rows in bins.
fn bins(rule: BinRule, binwidth: Option<f64>, from: Option<f64>, scale: BinScale) -> PlotLayer {
    PlotLayer {
        mark: Mark::Bars(Arrangement::Stack),
        width: Some(1.0),
        stat: Some(Stat::Bins {
            rule,
            binwidth,
            start: from,
            scale,
        }),
        ..points(1.0)
    }
}

/// The histogram of a variable of a frame.
fn histogram(frame: Frame, x: &str, layer: PlotLayer) -> Scene {
    let plot: Plot = start(frame, vec![mapped(Aesthetic::X, x)], Vec::new());
    with_layer(&plot, layer).1
}

/// The 200 values the reference numbers were computed on in R:
/// `sapply(1:200, function(i) (i * 37) %% 101 + (i %% 7) * 0.25)`.
fn series() -> Vec<f64> {
    (1..=200)
        .map(|i| f64::from((i * 37) % 101) + f64::from(i % 7) * 0.25)
        .collect()
}

fn of(values: &[f64]) -> Frame {
    frame("d", vec![("v", numbers(values))])
}

#[test]
fn the_counts_and_the_densities_are_those_of_hist() {
    // Freedman and Diaconis give six bins, as `nclass.FD`: from 0.75 to 101.25, each 16.75
    // wide. `hist(a, breaks = seq(0.75, 101.25, length.out = 7), right = FALSE,
    // include.lowest = TRUE)` counts 34, 33, 33, 32, 35 and 33.
    let scene = histogram(
        of(&series()),
        "v",
        bins(BinRule::Fd, None, None, BinScale::Count),
    );
    let layer = &scene.panels[0].layers[0];
    assert_eq!(layer.mark, Mark::Bars(Arrangement::Stack));
    assert_eq!(layer.y, [34.0, 33.0, 33.0, 32.0, 35.0, 33.0]);
    assert_eq!(layer.low, [0.0; 6]);
    for (i, (x, span)) in layer.x.iter().zip(&layer.span).enumerate() {
        assert!((span - 16.75).abs() < 1e-12);
        assert!((x - (0.75 + 16.75 * (i as f64 + 0.5))).abs() < 1e-9);
    }
    // A bar that counts rows is drawn from no one row.
    assert!(layer.rows.is_empty());
    // The axis is named by what the bars show, and starts at zero.
    let y = scene.scale("y").unwrap();
    assert_eq!(y.title, "count");
    assert!(y.limits.0 < 0.0 && y.limits.1 > 35.0);
    // The densities of `hist`.
    let scene = histogram(
        of(&series()),
        "v",
        bins(BinRule::Fd, None, None, BinScale::Density),
    );
    let wanted = [
        0.01014925373134,
        0.00985074626866,
        0.00985074626866,
        0.00955223880597,
        0.01044776119403,
        0.00985074626866,
    ];
    for (found, wanted) in scene.panels[0].layers[0].y.iter().zip(wanted) {
        assert!((found - wanted).abs() < 1e-13, "{found} against {wanted}");
    }
    assert_eq!(scene.scale("y").unwrap().title, "density");
    // Sturges gives nine bins, as `nclass.Sturges`, and `hist` on those breaks counts 22,
    // 22, 23, 22, 23, 20, 24, 22 and 22; Scott gives six, as `nclass.scott`.
    let scene = histogram(
        of(&series()),
        "v",
        bins(BinRule::Sturges, None, None, BinScale::Count),
    );
    assert_eq!(
        scene.panels[0].layers[0].y,
        [22.0, 22.0, 23.0, 22.0, 23.0, 20.0, 24.0, 22.0, 22.0]
    );
    let scene = histogram(
        of(&series()),
        "v",
        bins(BinRule::Scott, None, None, BinScale::Count),
    );
    assert_eq!(scene.panels[0].layers[0].y.len(), 6);
    let scene = histogram(
        of(&series()),
        "v",
        bins(BinRule::Count(9), None, None, BinScale::Fraction),
    );
    assert!((scene.panels[0].layers[0].y[0] - 0.11).abs() < 1e-12);
    assert_eq!(scene.scale("y").unwrap().title, "fraction");
    insta::assert_snapshot!(
        "six_bins",
        histogram(
            of(&series()),
            "v",
            bins(BinRule::Fd, None, None, BinScale::Count)
        )
        .json()
        .write()
    );
}

#[test]
fn a_width_a_start_and_rows_that_are_not_counted() {
    let frame = frame(
        "d",
        vec![(
            "v",
            floats(&[
                Some(1.0),
                Some(2.0),
                Some(2.5),
                None,
                Some(4.0),
                Some(10.0),
                Some(f64::INFINITY),
            ]),
        )],
    );
    // A width of 2 from the smallest value: five bins, from 1 to 11.
    let scene = histogram(
        frame.clone(),
        "v",
        bins(BinRule::Fd, Some(2.0), None, BinScale::Count),
    );
    let layer = &scene.panels[0].layers[0];
    assert_eq!(layer.x, [2.0, 4.0, 6.0, 8.0, 10.0]);
    assert_eq!(layer.y, [3.0, 1.0, 0.0, 0.0, 1.0]);
    assert_eq!(layer.span, [2.0; 5]);
    assert_eq!(
        scene.notes,
        [
            "1 row with NA in v dropped",
            "1 row with an infinite value in v dropped"
        ]
    );
    // A start below the smallest value: the bins begin there.
    let scene = histogram(
        frame.clone(),
        "v",
        bins(BinRule::Count(5), None, Some(0.0), BinScale::Count),
    );
    let layer = &scene.panels[0].layers[0];
    assert_eq!(layer.x, [1.0, 3.0, 5.0, 7.0, 9.0]);
    assert_eq!(layer.y, [1.0, 2.0, 1.0, 0.0, 1.0]);
    // A start above the smallest value, which the function refuses: the rows below it are
    // in no bin, and a note counts them.
    let scene = histogram(
        frame,
        "v",
        bins(BinRule::Fd, Some(4.0), Some(2.0), BinScale::Count),
    );
    let layer = &scene.panels[0].layers[0];
    assert_eq!((&layer.x, &layer.y), (&vec![4.0, 8.0], &vec![3.0, 1.0]));
    assert!(
        scene
            .notes
            .contains(&"1 row with v below `start:` is in no bin".to_string()),
        "{:?}",
        scene.notes
    );
}

#[test]
fn values_that_are_one_value_and_no_value() {
    // Zero variance, and one value: one bin, one unit wide, round the value.
    for values in [vec![3.0; 10], vec![3.0]] {
        let n = values.len() as f64;
        let scene = histogram(
            of(&values),
            "v",
            bins(BinRule::Fd, None, None, BinScale::Count),
        );
        let layer = &scene.panels[0].layers[0];
        assert_eq!(
            (&layer.x, &layer.y, &layer.span),
            (&vec![3.0], &vec![n], &vec![1.0])
        );
        let scene = histogram(
            of(&values),
            "v",
            bins(BinRule::Scott, None, None, BinScale::Density),
        );
        assert_eq!(scene.panels[0].layers[0].y, [1.0]);
    }
    // No value that can be counted: no bar, and the plot still has its axes.
    let none = frame("d", vec![("v", floats(&[None, None]))]);
    let scene = histogram(none, "v", bins(BinRule::Fd, None, None, BinScale::Count));
    assert!(scene.panels[0].layers[0].x.is_empty());
    assert_eq!(scene.notes, ["2 rows with NA in v dropped"]);
}

#[test]
fn groups_are_stacked_and_panels_share_their_bins() {
    let values = series();
    let groups: Vec<&str> = (0..200).map(|i| ["a", "b", "c"][i % 3]).collect();
    let frame = frame(
        "d",
        vec![
            ("v", numbers(&values)),
            ("g", Column::strs(groups.iter().copied().map(Some))),
        ],
    );
    let whole = histogram(
        of(&values),
        "v",
        bins(BinRule::Fd, None, None, BinScale::Count),
    );
    let counts = &whole.panels[0].layers[0].y;
    // Filled by three groups: each bin has three bars, one on another, the first group on
    // top, and the top of the stack is the count of the bin.
    let mut plot: Plot = start(
        frame.clone(),
        vec![mapped(Aesthetic::X, "v"), mapped(Aesthetic::Fill, "g")],
        Vec::new(),
    );
    let scene = with_layer(&plot, bins(BinRule::Fd, None, None, BinScale::Count)).1;
    let layer = &scene.panels[0].layers[0];
    assert_eq!(layer.x.len(), 18);
    assert_eq!(counts.len(), 6);
    for (bin, count) in counts.iter().enumerate() {
        let (first, second, third) = (3 * bin, 3 * bin + 1, 3 * bin + 2);
        assert_eq!(layer.low[third], 0.0);
        assert_eq!(layer.low[second], layer.y[third]);
        assert_eq!(layer.low[first], layer.y[second]);
        assert_eq!(layer.y[first], *count);
    }
    // Under `density` the whole stack has area one.
    let scene = with_layer(&plot, bins(BinRule::Fd, None, None, BinScale::Density)).1;
    let layer = &scene.panels[0].layers[0];
    let area: f64 = (0..18)
        .map(|i| (layer.y[i] - layer.low[i]) * layer.span[i])
        .sum();
    assert!((area - 1.0).abs() < 1e-12, "{area}");
    // Parted into panels: every panel has the six bins of all the rows, and under
    // `percent` the bars of each panel sum to a hundred.
    plot.mappings.pop();
    plot.facet = Some(PlotFacet {
        by: vec!["g".to_string()],
        ..PlotFacet::default()
    });
    let scene = with_layer(&plot, bins(BinRule::Fd, None, None, BinScale::Percent)).1;
    assert_eq!(scene.panels.len(), 3);
    for panel in &scene.panels {
        let layer = &panel.layers[0];
        assert_eq!(layer.x, whole.panels[0].layers[0].x);
        assert!((layer.y.iter().sum::<f64>() - 100.0).abs() < 1e-9);
    }
    let counted = with_layer(&plot, bins(BinRule::Fd, None, None, BinScale::Count)).1;
    for (bin, count) in counts.iter().enumerate() {
        let over: f64 = counted.panels.iter().map(|p| p.layers[0].y[bin]).sum();
        assert_eq!(over, *count);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(128)))]

    /// Whatever the rows and the bins: the counts sum to the rows with a finite value;
    /// under `density` the bars have area one; the fractions sum to one and the percentages
    /// to a hundred; and no bar is below zero.
    #[test]
    fn the_bars_of_a_histogram_sum_as_their_scale_says(
        rows in rows(),
        wanted in prop_oneof![
            (1usize..40).prop_map(BinRule::Count),
            Just(BinRule::Sturges),
            Just(BinRule::Scott),
            Just(BinRule::Fd),
        ],
    ) {
        let frame = tried(&rows);
        let finite = rows.iter().filter(|row| row.0.is_some()).count();
        let bars = |scale: BinScale| {
            let scene = histogram(frame.clone(), "x", bins(wanted, None, None, scale));
            let layer = &scene.panels[0].layers[0];
            (layer.y.clone(), layer.span.clone())
        };
        let (counts, spans) = bars(BinScale::Count);
        prop_assert!((counts.iter().sum::<f64>() - finite as f64).abs() < 1e-9);
        prop_assert!(counts.iter().all(|count| *count >= 0.0));
        if finite > 0 {
            let (densities, _) = bars(BinScale::Density);
            let area: f64 = densities.iter().zip(&spans).map(|(d, w)| d * w).sum();
            prop_assert!((area - 1.0).abs() < 1e-9, "an area of {}", area);
            let (fractions, _) = bars(BinScale::Fraction);
            prop_assert!((fractions.iter().sum::<f64>() - 1.0).abs() < 1e-9);
            let (percentages, _) = bars(BinScale::Percent);
            prop_assert!((percentages.iter().sum::<f64>() - 100.0).abs() < 1e-7);
            // The bins are of one width, side by side from the smallest value.
            prop_assert!(spans.windows(2).all(|pair| pair[0] == pair[1]));
        } else {
            prop_assert!(counts.is_empty());
        }
    }
}
