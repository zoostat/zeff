//! The scales that are not positions, as a script sets them (Language Reference, section 7.6,
//! "Scales": `scale`; release 0.7.0, step 25d): the colours of categories and of a continuous
//! scale, the sizes read by area from zero or from the smallest value to the largest, and the
//! shapes; what does not fit a scale is found and named.

mod common;

use common::*;
use proptest::prelude::*;
use zeff_columns::Column;
use zeff_core::value::{Aesthetic, Arrangement, Frame, Mark, Palette, Plot, PlotLayer, PlotScale};
use zeff_plot::colour::{self, NA_COLOUR, OKABE_ITO, RAMP_STEPS, VIRIDIS, ramp_through};
use zeff_plot::layout::{JOURNAL, Size, layout};
use zeff_plot::scene::{
    PaletteFault, Range, ScaleKind, Scene, diameter, palette_fault, start, with_layer,
};

/// Six rows: a place, a level, a group of three with one row that has none, and an amount
/// that goes below zero.
fn rows() -> Frame {
    frame(
        "d",
        vec![
            ("x", numbers(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0])),
            ("y", numbers(&[2.0, 1.0, 4.0, 3.0, 6.0, 5.0])),
            (
                "g",
                Column::strs([Some("a"), Some("b"), Some("c"), Some("a"), None, Some("b")]),
            ),
            ("z", numbers(&[-2.0, 0.0, 2.0, 4.0, 6.0, 8.0])),
        ],
    )
}

/// The rows with `g` or `z` mapped to an aesthetic and a palette set for it.
fn set(aesthetic: Aesthetic, variable: &str, palette: Option<Palette>) -> Plot {
    let mut plot = start(
        rows(),
        vec![
            mapped(Aesthetic::X, "x"),
            mapped(Aesthetic::Y, "y"),
            mapped(aesthetic, variable),
        ],
        Vec::new(),
    );
    if let Some(palette) = palette {
        plot.scales.push(PlotScale {
            aesthetic,
            palette,
            limits: None,
        });
    }
    plot
}

fn drawn(plot: &Plot) -> Scene {
    with_layer(plot, points(1.0)).1
}

fn given(colours: &[(&str, &str)]) -> Palette {
    Palette::Colours(
        colours
            .iter()
            .map(|(hex, written)| (hex.to_string(), written.to_string()))
            .collect(),
    )
}

fn colours_of(scene: &Scene, id: &str) -> Vec<String> {
    match &scene.scale(id).unwrap().range {
        Range::Colours(colours) => colours.clone(),
        other => panic!("`{id}` is not a scale of colours: {other:?}"),
    }
}

#[test]
fn the_categories_take_the_colours_of_the_script_in_their_order() {
    // Without a palette: the first three of Okabe and Ito, and the grey of `NA`.
    let usual = drawn(&set(Aesthetic::Color, "g", None));
    assert_eq!(
        colours_of(&usual, "color"),
        [OKABE_ITO[0].0, OKABE_ITO[1].0, OKABE_ITO[2].0, NA_COLOUR.0]
    );
    // `okabe_ito` says the same outright.
    let said = drawn(&set(Aesthetic::Color, "g", Some(Palette::OkabeIto)));
    assert_eq!(colours_of(&said, "color"), colours_of(&usual, "color"));
    // A colour for each category, as given; `NA` keeps its grey.
    let palette = given(&[
        ("#808080", "grey"),
        ("#B22222", "firebrick"),
        ("#000080", "navy"),
    ]);
    let own = drawn(&set(Aesthetic::Color, "g", Some(palette)));
    assert_eq!(
        colours_of(&own, "color"),
        ["#808080", "#B22222", "#000080", NA_COLOUR.0]
    );
    assert!(own.notes.is_empty(), "{:?}", own.notes);
    // What each mark is drawn as does not change: its place among the categories.
    assert_eq!(
        own.panels[0].layers[0].color,
        usual.panels[0].layers[0].color
    );
    // A longer list is used as far as it is needed.
    let palette = given(&[
        ("#111111", "a"),
        ("#222222", "b"),
        ("#333333", "c"),
        ("#444444", "d"),
    ]);
    let longer = drawn(&set(Aesthetic::Color, "g", Some(palette)));
    assert_eq!(
        colours_of(&longer, "color"),
        ["#111111", "#222222", "#333333", NA_COLOUR.0]
    );
}

#[test]
fn fewer_colours_than_categories_repeat_and_a_note_says_so() {
    let palette = given(&[("#808080", "grey"), ("#B22222", "firebrick")]);
    let scene = drawn(&set(Aesthetic::Color, "g", Some(palette)));
    assert_eq!(
        colours_of(&scene, "color"),
        ["#808080", "#B22222", "#808080", NA_COLOUR.0]
    );
    assert_eq!(
        scene.notes,
        ["`g` has 3 categories and the scale 2 colours, so the colours repeat from category 3"]
    );
    let one = drawn(&set(
        Aesthetic::Color,
        "g",
        Some(given(&[("#808080", "grey")])),
    ));
    assert_eq!(
        one.notes,
        ["`g` has 3 categories and the scale 1 colour, so the colours repeat from category 2"]
    );
}

#[test]
fn viridis_gives_categories_its_colours_at_equal_steps() {
    let scene = drawn(&set(Aesthetic::Color, "g", Some(Palette::Viridis)));
    assert_eq!(
        colours_of(&scene, "color"),
        [VIRIDIS[0], VIRIDIS[128], VIRIDIS[255], NA_COLOUR.0]
    );
    assert_eq!(scene.scale("color").unwrap().kind, ScaleKind::Category);
    assert!(scene.notes.is_empty());
}

#[test]
fn a_number_is_shown_on_a_ramp_through_the_colours_of_the_script() {
    let usual = drawn(&set(Aesthetic::Color, "z", None));
    assert_eq!(colours_of(&usual, "color"), VIRIDIS);
    assert_eq!(
        colours_of(
            &drawn(&set(Aesthetic::Color, "z", Some(Palette::Viridis))),
            "color"
        ),
        VIRIDIS
    );
    let palette = given(&[("#FFFFFF", "white"), ("#4682B4", "steelblue")]);
    let own = drawn(&set(Aesthetic::Color, "z", Some(palette)));
    let ramp = colours_of(&own, "color");
    assert_eq!(ramp.len(), RAMP_STEPS);
    assert_eq!(
        (ramp[0].as_str(), ramp[255].as_str()),
        ("#FFFFFF", "#4682B4")
    );
    // The scale itself is as it was: linear, from the lowest value to the highest.
    let (ours, theirs) = (own.scale("color").unwrap(), usual.scale("color").unwrap());
    assert_eq!((ours.kind, ours.limits), (ScaleKind::Linear, (-2.0, 8.0)));
    assert_eq!(
        (&ours.breaks, &ours.labels),
        (&theirs.breaks, &theirs.labels)
    );
    assert_eq!(
        own.panels[0].layers[0].color,
        usual.panels[0].layers[0].color
    );
}

#[test]
fn a_fill_borrowed_from_the_colour_takes_the_palette_of_the_colour() {
    let palette = given(&[
        ("#808080", "grey"),
        ("#000000", "black"),
        ("#B22222", "firebrick"),
    ]);
    let plot = set(Aesthetic::Color, "g", Some(palette));
    let bars = PlotLayer {
        mark: Mark::Bars(Arrangement::Dodge),
        ..points(1.0)
    };
    let (_, scene) = with_layer(&plot, bars);
    assert_eq!(
        colours_of(&scene, "fill"),
        ["#808080", "#000000", "#B22222", NA_COLOUR.0]
    );
    // A palette said of the fill itself comes first.
    let mut plot = plot;
    plot.scales.push(PlotScale {
        aesthetic: Aesthetic::Fill,
        palette: Palette::OkabeIto,
        limits: None,
    });
    let bars = PlotLayer {
        mark: Mark::Bars(Arrangement::Dodge),
        ..points(1.0)
    };
    let (_, scene) = with_layer(&plot, bars);
    assert_eq!(colours_of(&scene, "fill")[0], OKABE_ITO[0].0);
}

#[test]
fn a_size_shows_an_amount_by_area_or_a_difference_between_two_sizes() {
    // By area from zero, as without a palette, with the largest mark of the script.
    let usual = drawn(&set(Aesthetic::Size, "z", None));
    let wider = drawn(&set(
        Aesthetic::Size,
        "z",
        Some(Palette::Sizes {
            smallest: None,
            largest: 24.0,
        }),
    ));
    let (a, b) = (usual.scale("size").unwrap(), wider.scale("size").unwrap());
    assert_eq!(a.limits, (0.0, 8.0));
    assert_eq!(b.limits, (0.0, 8.0));
    assert_eq!(b.range, Range::Sizes(0.0, 24.0));
    assert_ne!(a.range, b.range);
    // The row below zero is not drawn, and the row of zero is drawn with no size.
    assert_eq!(wider.panels[0].layers[0].x, [2.0, 3.0, 4.0, 5.0, 6.0]);
    assert_eq!(wider.notes, usual.notes);
    assert!(
        wider
            .notes
            .iter()
            .any(|note| note.contains("below 0 dropped"))
    );

    // From the smallest value to the largest: every row is drawn, the one below zero too.
    let ranged = drawn(&set(
        Aesthetic::Size,
        "z",
        Some(Palette::Sizes {
            smallest: Some(2.0),
            largest: 12.0,
        }),
    ));
    let scale = ranged.scale("size").unwrap();
    assert_eq!((scale.kind, scale.limits), (ScaleKind::Sqrt, (-2.0, 8.0)));
    assert_eq!(scale.range, Range::Sizes(2.0, 12.0));
    assert_eq!(scale.breaks, [-2.0, 0.0, 2.0, 4.0, 6.0, 8.0]);
    assert_eq!(ranged.panels[0].layers[0].x, [1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    assert!(ranged.notes.is_empty(), "{:?}", ranged.notes);
    // The two ends have the two diameters, and the area between is in proportion.
    assert_eq!(diameter(-2.0, scale.limits, (2.0, 12.0)), 2.0);
    assert_eq!(diameter(8.0, scale.limits, (2.0, 12.0)), 12.0);
    let middle = diameter(0.5, scale.limits, (2.0, 12.0));
    assert!((middle - 7.0).abs() < 1e-12, "{middle}");
}

#[test]
fn the_categories_take_the_shapes_of_the_script() {
    let usual = drawn(&set(Aesthetic::Shape, "g", None));
    assert_eq!(
        usual.scale("shape").unwrap().range,
        Range::Shapes(vec!["circle", "triangle", "square", "asterisk"])
    );
    let own = drawn(&set(
        Aesthetic::Shape,
        "g",
        Some(Palette::Shapes(vec!["circle_open", "cross"])),
    ));
    assert_eq!(
        own.scale("shape").unwrap().range,
        Range::Shapes(vec!["circle_open", "cross", "circle_open", "asterisk"])
    );
    assert_eq!(
        own.notes,
        ["`g` has 3 categories and the scale 2 shapes, so the shapes repeat from category 3"]
    );
}

#[test]
fn what_a_continuous_scale_cannot_be_shown_by_is_found() {
    let fault = |palette: Palette| palette_fault(&set(Aesthetic::Color, "z", Some(palette)));
    assert_eq!(
        fault(Palette::OkabeIto),
        Some(PaletteFault {
            aesthetic: "color",
            variable: "z".to_string(),
            colours: None
        })
    );
    assert_eq!(
        fault(given(&[("#000080", "navy")])).unwrap().colours,
        Some(1)
    );
    assert_eq!(fault(Palette::Viridis), None);
    assert_eq!(
        fault(given(&[("#FFFFFF", "white"), ("#000080", "navy")])),
        None
    );
    // Categories take any of them.
    for palette in [
        Palette::OkabeIto,
        Palette::Viridis,
        given(&[("#000080", "navy")]),
    ] {
        assert_eq!(
            palette_fault(&set(Aesthetic::Color, "g", Some(palette))),
            None
        );
    }
    // A fill borrowed from the colour is found by the colour.
    assert_eq!(
        palette_fault(&set(Aesthetic::Fill, "z", Some(Palette::OkabeIto)))
            .unwrap()
            .aesthetic,
        "fill"
    );
}

#[test]
fn the_legend_is_drawn_in_the_colours_and_sizes_of_the_script() {
    let palette = given(&[
        ("#808080", "grey"),
        ("#B22222", "firebrick"),
        ("#000080", "navy"),
    ]);
    let scene = drawn(&set(Aesthetic::Color, "g", Some(palette)));
    let drawing = layout(&scene, Size::OneColumn, &JOURNAL).unwrap();
    let written = drawing.describe();
    for hex in ["#808080", "#B22222", "#000080"] {
        assert!(written.contains(hex), "{hex}\n{written}");
    }
    assert!(!written.contains(OKABE_ITO[0].0));
    insta::assert_snapshot!("three_colours_of_the_script", written);
    let ranged = drawn(&set(
        Aesthetic::Size,
        "z",
        Some(Palette::Sizes {
            smallest: Some(2.0),
            largest: 12.0,
        }),
    ));
    let drawing = layout(&ranged, Size::OneColumn, &JOURNAL).unwrap();
    insta::assert_snapshot!("sizes_from_two_to_twelve", drawing.describe());
}

/// A colour as `#` and six hexadecimal digits.
fn hex() -> impl Strategy<Value = String> {
    (0_u32..=0xFF_FFFF).prop_map(|value| format!("#{value:06X}"))
}

fn channels(hex: &str) -> [i64; 3] {
    let value = i64::from_str_radix(&hex[1..], 16).unwrap();
    [(value >> 16) & 255, (value >> 8) & 255, value & 255]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(256)))]

    /// Whatever the colours: a ramp of 256, the first and the last as given, and every colour
    /// between two given ones within the two, channel by channel.
    #[test]
    fn a_ramp_stays_between_the_colours_it_passes_through(
        given in prop::collection::vec(hex(), 2..6),
    ) {
        let ramp = ramp_through(&given);
        prop_assert_eq!(ramp.len(), RAMP_STEPS);
        prop_assert_eq!(&ramp[0], &given[0]);
        prop_assert_eq!(&ramp[RAMP_STEPS - 1], &given[given.len() - 1]);
        let (least, most) = given.iter().map(|c| channels(c)).fold(
            ([255_i64; 3], [0_i64; 3]),
            |(mut lo, mut hi), c| {
                for k in 0..3 {
                    lo[k] = lo[k].min(c[k]);
                    hi[k] = hi[k].max(c[k]);
                }
                (lo, hi)
            },
        );
        for entry in &ramp {
            prop_assert_eq!(entry.len(), 7);
            let c = channels(entry);
            for k in 0..3 {
                prop_assert!(c[k] >= least[k] && c[k] <= most[k], "{} in {:?}", entry, given);
            }
        }
        // Two colours: each channel moves one way only.
        if given.len() == 2 {
            let (from, to) = (channels(&given[0]), channels(&given[1]));
            for pair in ramp.windows(2) {
                let (a, b) = (channels(&pair[0]), channels(&pair[1]));
                for k in 0..3 {
                    let one_way = if from[k] <= to[k] {
                        a[k] <= b[k]
                    } else {
                        a[k] >= b[k]
                    };
                    prop_assert!(one_way, "a channel turns back between two colours");
                }
            }
        }
        // Every entry is a colour the engine reads.
        prop_assert!(ramp.iter().all(|entry| colour::parse(entry).is_ok()));
    }
}
