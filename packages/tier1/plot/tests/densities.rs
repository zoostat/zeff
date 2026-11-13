//! The density layer (Language Reference, section 7.6, "Statistical layers": `density`;
//! release 0.7.0, step 28b): the kernel estimate of the density of `x`, which `stats`
//! computes, drawn as a line for each group. The reference values are R's (4.6.0).

mod common;

use common::*;
use proptest::prelude::*;
use zeff_columns::Column;
use zeff_core::value::{Aesthetic, Frame, Kernel, Mark, Plot, PlotFacet, PlotLayer, Stat};
use zeff_plot::scene::{Scene, start, with_layer};

/// A layer that draws the density of its rows.
fn smooth(bandwidth: Option<f64>, adjust: f64, kernel: Kernel) -> PlotLayer {
    PlotLayer {
        mark: Mark::Lines,
        stat: Some(Stat::Density {
            bandwidth,
            adjust,
            kernel,
        }),
        ..points(1.0)
    }
}

/// The density of a variable of a frame.
fn density(frame: Frame, x: &str, layer: PlotLayer) -> Scene {
    let plot: Plot = start(frame, vec![mapped(Aesthetic::X, x)], Vec::new());
    with_layer(&plot, layer).1
}

/// The 57 values the reference numbers were computed on in R:
/// `sapply(1:57, function(i) ((i * i) %% 23) / 4 + i / 100)`.
fn series() -> Vec<f64> {
    (1..=57)
        .map(|i| f64::from((i * i) % 23) / 4.0 + f64::from(i) / 100.0)
        .collect()
}

fn of(values: &[f64]) -> Frame {
    frame("d", vec![("v", numbers(values))])
}

/// The area under a line through places a step apart, by the trapezium rule.
fn area(x: &[f64], y: &[f64]) -> f64 {
    x.windows(2)
        .zip(y.windows(2))
        .map(|(x, y)| (y[0] + y[1]) / 2.0 * (x[1] - x[0]))
        .sum()
}

#[test]
fn the_line_is_the_estimate_of_density_at_its_512_places() {
    // `density(b)`: Silverman's bandwidth, 0.572892458379114, and 512 places from
    // -1.48867737513734 to 6.75867737513734. At the 1st, 100th, 256th, 400th and 512th the
    // mean of the Gaussian kernel over the values, as R computes it directly.
    let scene = density(of(&series()), "v", smooth(None, 1.0, Kernel::Gaussian));
    let layer = &scene.panels[0].layers[0];
    assert_eq!(layer.mark, Mark::Lines);
    assert_eq!((layer.x.len(), layer.y.len()), (512, 512));
    assert!(layer.line.iter().all(|line| *line == 1));
    assert!((layer.x[0] - -1.48867737513734).abs() < 1e-12);
    assert!((layer.x[511] - 6.75867737513734).abs() < 1e-12);
    for (place, wanted) in [
        (0, 0.000417997998549039),
        (99, 0.125692404138292),
        (255, 0.188389106490863),
        (399, 0.0928170532473728),
        (511, 0.000262313211759516),
    ] {
        assert!(
            (layer.y[place] - wanted).abs() < 1e-12,
            "at {place}: {}",
            layer.y[place]
        );
    }
    // The axis of `y` is named `density`, and holds the whole line.
    let y = scene.scale("y").unwrap();
    assert_eq!(y.title, "density");
    assert!(y.limits.0 <= 0.0 && y.limits.1 > 0.22);
    // Another kernel, at the 200th place: the Epanechnikov kernel, and a bandwidth doubled.
    let scene = density(of(&series()), "v", smooth(None, 1.0, Kernel::Epanechnikov));
    assert!((scene.panels[0].layers[0].y[199] - 0.232986017426868).abs() < 1e-12);
    let scene = density(of(&series()), "v", smooth(None, 2.0, Kernel::Gaussian));
    assert!((scene.panels[0].layers[0].x[0] - -3.20735475027468).abs() < 1e-12);
    insta::assert_snapshot!(
        "a_density",
        density(
            of(&[1.0, 2.0, 2.5, 4.0]),
            "v",
            smooth(Some(0.5), 1.0, Kernel::Triangular)
        )
        .json()
        .write()
        .lines()
        .filter(|line| !line.contains("\"x\": [") && !line.contains("\"y\": ["))
        .collect::<Vec<&str>>()
        .join("\n")
    );
}

#[test]
fn each_group_and_each_panel_has_its_own_line() {
    let values = series();
    let groups: Vec<&str> = (0..57).map(|i| ["a", "b", "c"][i % 3]).collect();
    let frame = frame(
        "d",
        vec![
            ("v", numbers(&values)),
            ("g", Column::strs(groups.iter().copied().map(Some))),
        ],
    );
    // Coloured by three groups: three lines of 512 places, each the density of its group
    // alone, with its own bandwidth and its own places.
    let mut plot: Plot = start(
        frame.clone(),
        vec![mapped(Aesthetic::X, "v"), mapped(Aesthetic::Color, "g")],
        Vec::new(),
    );
    let scene = with_layer(&plot, smooth(None, 1.0, Kernel::Gaussian)).1;
    let layer = &scene.panels[0].layers[0];
    assert_eq!(layer.x.len(), 3 * 512);
    for (i, name) in ["a", "b", "c"].iter().enumerate() {
        let own: Vec<f64> = values
            .iter()
            .zip(&groups)
            .filter(|(_, group)| *group == name)
            .map(|(value, _)| *value)
            .collect();
        let alone = density(of(&own), "v", smooth(None, 1.0, Kernel::Gaussian));
        let wanted = &alone.panels[0].layers[0];
        let (from, to) = (512 * i, 512 * (i + 1));
        assert_eq!(&layer.x[from..to], &wanted.x[..]);
        assert_eq!(&layer.y[from..to], &wanted.y[..]);
        assert!(layer.line[from..to].iter().all(|line| *line == i + 1));
        assert!((area(&layer.x[from..to], &layer.y[from..to]) - 1.0).abs() < 5e-3);
    }
    // Parted into panels: each panel has the line of its own rows.
    plot.mappings.pop();
    plot.facet = Some(PlotFacet {
        by: vec!["g".to_string()],
        ..PlotFacet::default()
    });
    let scene = with_layer(&plot, smooth(None, 1.0, Kernel::Gaussian)).1;
    assert_eq!(scene.panels.len(), 3);
    for (i, panel) in scene.panels.iter().enumerate() {
        assert_eq!(panel.layers[0].x.len(), 512);
        assert_eq!(&panel.layers[0].y[..], &layer.y[512 * i..512 * (i + 1)]);
    }
}

#[test]
fn too_few_values_no_spread_and_rows_that_are_not_used() {
    // One value has no density, and a note says so; a row without a value is left out.
    let one = frame("d", vec![("v", floats(&[Some(2.5), None]))]);
    let scene = density(one, "v", smooth(None, 1.0, Kernel::Gaussian));
    assert!(scene.panels[0].layers[0].x.is_empty());
    assert_eq!(
        scene.notes,
        [
            "1 row with NA in v dropped",
            "1 group with fewer than two values of v has no density"
        ]
    );
    // Two values, and ten that are one value, have a density: the bandwidth of `bw.nrd0`
    // where the values have no spread, 1.70358483009652 for ten threes.
    let scene = density(of(&[1.0, 4.0]), "v", smooth(None, 1.0, Kernel::Gaussian));
    assert_eq!(scene.panels[0].layers[0].x.len(), 512);
    let scene = density(of(&[3.0; 10]), "v", smooth(None, 1.0, Kernel::Gaussian));
    let layer = &scene.panels[0].layers[0];
    assert!((layer.x[0] - (3.0 - 3.0 * 1.70358483009652)).abs() < 1e-12);
    assert!((area(&layer.x, &layer.y) - 1.0).abs() < 5e-3);
    // An infinite value is left out and counted.
    let far = frame(
        "d",
        vec![("v", floats(&[Some(1.0), Some(2.0), Some(f64::INFINITY)]))],
    );
    let scene = density(far, "v", smooth(None, 1.0, Kernel::Gaussian));
    assert_eq!(scene.notes, ["1 row with an infinite value in v dropped"]);
    assert_eq!(scene.panels[0].layers[0].x.len(), 512);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(64)))]

    /// Whatever the rows, the kernel and the adjustment: the estimate is nowhere below
    /// zero, and the area under it is one wherever its 512 places are close enough to trace
    /// a kernel, less than half a bandwidth apart. One value far from all the others puts
    /// the places further apart than the peaks are wide, here as in R, and the line through
    /// them is then no measure of the area.
    #[test]
    fn a_density_is_never_below_zero_and_has_area_one(
        rows in rows(),
        kernel in prop::sample::select(vec![
            Kernel::Gaussian,
            Kernel::Epanechnikov,
            Kernel::Rectangular,
            Kernel::Triangular,
            Kernel::Biweight,
            Kernel::Cosine,
        ]),
        adjust in 0.5..3.0f64,
    ) {
        let scene = density(tried(&rows), "x", smooth(None, adjust, kernel));
        let layer = &scene.panels[0].layers[0];
        let finite = rows.iter().filter(|row| row.0.is_some()).count();
        if finite < 2 {
            prop_assert!(layer.x.is_empty());
        } else {
            prop_assert_eq!(layer.x.len(), 512);
            prop_assert!(layer.y.iter().all(|y| *y >= 0.0 && y.is_finite()));
            let values: Vec<f64> = rows.iter().filter_map(|row| row.0).collect();
            let bandwidth = zeff_stats::density::silverman(&values).unwrap() * adjust;
            if layer.x[1] - layer.x[0] < bandwidth / 2.0 {
                let found = area(&layer.x, &layer.y);
                prop_assert!((found - 1.0).abs() < 2e-2, "an area of {}", found);
            }
        }
    }
}
