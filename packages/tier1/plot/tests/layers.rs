//! A layer's own mappings (Language Reference, section 7.6, "Mappings and constants"; release
//! 0.7.0, step 23a; open decision #106): what a layer names of its own takes the place, for
//! that layer, of what the plot maps. A layer that repeats the mappings of its plot, or names
//! them where the plot names none, draws the scene the plot alone would give; a layer that
//! names another variable draws that one on the scales the layers share.

mod common;

use common::*;
use proptest::prelude::*;
use zeff_core::value::{Aesthetic, PlotConstant, PlotSetting};
use zeff_plot::scene::{Paint, Shaping, Sizing, start, with_layer};

proptest! {
    #![proptest_config(ProptestConfig::with_cases(property_cases(200)))]

    #[test]
    fn a_layer_that_names_what_the_plot_maps_draws_what_the_plot_draws(
        rows in rows(),
        mappings in showing(),
        alpha in prop_oneof![Just(1.0), 0.05..1.0f64],
    ) {
        let frame = tried(&rows);
        let on_the_plot = drawn_with(frame.clone(), &mappings, Vec::new(), alpha);
        let named: Vec<_> = mappings.iter().map(|(a, n)| mapped(*a, n)).collect();
        let mut layer = points(alpha);
        layer.mappings = named.clone();
        // Named by the layer alone, on a plot that maps nothing.
        let bare = start(frame.clone(), Vec::new(), Vec::new());
        prop_assert_eq!(&with_layer(&bare, layer.clone()).1, &on_the_plot);
        // Named by the plot and by the layer both.
        let both = start(frame, named, Vec::new());
        prop_assert_eq!(&with_layer(&both, layer).1, &on_the_plot);
    }
}

#[test]
fn a_layer_draws_its_own_variable_on_the_scale_the_layers_share() {
    let plot = start(
        small("d"),
        vec![
            mapped(Aesthetic::X, "dose"),
            mapped(Aesthetic::Y, "response"),
        ],
        Vec::new(),
    );
    let (plot, _) = with_layer(&plot, points(1.0));
    let mut own = points(1.0);
    own.mappings = vec![mapped(Aesthetic::Y, "weight")];
    let (_, scene) = with_layer(&plot, own);
    let layers = &scene.panels[0].layers;
    // The first layer draws the response, the second the weight, against the same doses.
    assert_eq!(layers[0].y, [2.1, 3.9, 6.2, 7.8, 9.0, 11.5]);
    assert_eq!(layers[1].y, [60.0, 72.5, 81.0, 55.0, 90.0, 68.0]);
    assert_eq!(layers[1].x, layers[0].x);
    assert_eq!(layers[1].rows, [1, 2, 3, 4, 5, 6]);
    // The axis runs over both, 5% of the range beyond each end, under the title of the plot.
    let y = scene.scale("y").unwrap();
    assert_eq!(y.title, "response");
    assert!((y.limits.0 - (2.1 - 0.05 * 87.9)).abs() < 1e-9);
    assert!((y.limits.1 - (90.0 + 0.05 * 87.9)).abs() < 1e-9);
}

#[test]
fn what_a_layer_sets_or_names_comes_before_what_the_plot_says() {
    let colour = |hex: &str| PlotConstant {
        aesthetic: Aesthetic::Color,
        value: PlotSetting::Colour {
            hex: hex.to_string(),
            written: hex.to_string(),
        },
    };
    // The plot colours by the arm; one layer sets a colour, one names the arm again, one
    // says nothing.
    let plot = start(
        small("d"),
        vec![
            mapped(Aesthetic::X, "dose"),
            mapped(Aesthetic::Y, "response"),
            mapped(Aesthetic::Color, "arm"),
        ],
        Vec::new(),
    );
    let mut set = points(1.0);
    set.constants = vec![colour("#B22222")];
    let (plot, _) = with_layer(&plot, set);
    let mut named = points(1.0);
    named.mappings = vec![mapped(Aesthetic::Color, "arm")];
    let (plot, _) = with_layer(&plot, named);
    let (_, scene) = with_layer(&plot, points(1.0));
    let layers = &scene.panels[0].layers;
    assert_eq!(layers[0].color, Paint::Constant("#B22222".to_string()));
    assert!(matches!(&layers[1].color, Paint::Scaled(values) if values.len() == 6));
    assert_eq!(layers[1].color, layers[2].color);
    // One legend, for the scale the two layers read.
    assert_eq!(scene.legends.len(), 1);

    // The plot sets a colour, a size and a shape; a layer that names a variable for each
    // reads the variable, and one that says nothing takes what the plot sets.
    let plot = start(
        small("d"),
        vec![
            mapped(Aesthetic::X, "dose"),
            mapped(Aesthetic::Y, "response"),
        ],
        vec![
            colour("#4682B4"),
            PlotConstant {
                aesthetic: Aesthetic::Size,
                value: PlotSetting::Size(7.0),
            },
            PlotConstant {
                aesthetic: Aesthetic::Shape,
                value: PlotSetting::Shape("square"),
            },
        ],
    );
    let mut named = points(1.0);
    named.mappings = vec![
        mapped(Aesthetic::Color, "arm"),
        mapped(Aesthetic::Size, "weight"),
        mapped(Aesthetic::Shape, "arm"),
    ];
    let (plot, _) = with_layer(&plot, named);
    let (_, scene) = with_layer(&plot, points(1.0));
    let layers = &scene.panels[0].layers;
    assert!(matches!(&layers[0].color, Paint::Scaled(_)));
    assert!(matches!(&layers[0].size, Sizing::Scaled(_)));
    assert!(matches!(&layers[0].shape, Shaping::Scaled(_)));
    assert_eq!(layers[1].color, Paint::Constant("#4682B4".to_string()));
    assert_eq!(layers[1].size, Sizing::Constant(7.0));
    assert_eq!(layers[1].shape, Shaping::Constant("square"));
    // The scales are named by the first layer that names a variable for them.
    assert_eq!(scene.scale("color").unwrap().title, "arm");
    assert_eq!(scene.scale("size").unwrap().title, "weight");
    assert_eq!(scene.scale("shape").unwrap().title, "arm");
}
