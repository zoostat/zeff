//! The themes of a figure (Language Reference, section 7.6, "Guides and theme": `theme`;
//! release 0.7.0, step 26b): `journal`, as frozen at step 22; `minimal`, without the lines
//! of the axes and their ticks; `slide`, twice the size; and every size of a theme in
//! proportion to its base size.

mod common;

use common::*;
use zeff_core::value::{Aesthetic, Plot, PlotTheme, ThemeName};
use zeff_plot::drawing::{Drawing, Primitive};
use zeff_plot::layout::{BASE_SIZE, JOURNAL, SLIDE_BASE_SIZE, Size, Theme, layout};
use zeff_plot::scene::{Scene, start, with_layer};

fn themed(name: ThemeName, base_size: Option<f64>) -> PlotTheme {
    PlotTheme { name, base_size }
}

/// Six points, with a theme.
fn scene_of(theme: PlotTheme) -> Scene {
    let mut plot: Plot = start(
        small("d"),
        vec![
            mapped(Aesthetic::X, "dose"),
            mapped(Aesthetic::Y, "response"),
        ],
        Vec::new(),
    );
    plot.theme = theme;
    with_layer(&plot, points(1.0)).1
}

/// The items of a drawing called `name`: how many are lines and how many are text.
fn of_axis(drawing: &Drawing, name: &str) -> (usize, usize) {
    let mut found = (0, 0);
    for item in drawing
        .items
        .iter()
        .filter(|item| item.name.as_deref() == Some(name))
    {
        match item.primitive {
            Primitive::Segment { .. } => found.0 += 1,
            Primitive::Text { .. } => found.1 += 1,
            _ => {}
        }
    }
    found
}

/// The sizes of the type of the items called `name`.
fn type_of(drawing: &Drawing, name: &str) -> Vec<f64> {
    drawing
        .items
        .iter()
        .filter(|item| item.name.as_deref() == Some(name))
        .filter_map(|item| match &item.primitive {
            Primitive::Text { spans, .. } => spans.first().map(|span| span.size),
            _ => None,
        })
        .collect()
}

#[test]
fn journal_at_its_own_size_is_the_theme_that_was_frozen() {
    assert_eq!(Theme::of(&PlotTheme::default()), JOURNAL);
    assert_eq!(
        Theme::of(&themed(ThemeName::Journal, Some(BASE_SIZE))),
        JOURNAL
    );
    assert_eq!((BASE_SIZE, SLIDE_BASE_SIZE), (9.0, 18.0));
}

#[test]
fn every_size_of_a_theme_is_in_proportion_to_its_base_size() {
    let larger = Theme::of(&themed(ThemeName::Journal, Some(12.0)));
    let k = 12.0 / 9.0;
    assert_eq!(larger.name, "journal");
    assert_eq!(larger.title_size, 12.0);
    for (ours, theirs) in [
        (larger.label_size, JOURNAL.label_size),
        (larger.heading_size, JOURNAL.heading_size),
        (larger.axis_width, JOURNAL.axis_width),
        (larger.grid_width, JOURNAL.grid_width),
        (larger.tick_length, JOURNAL.tick_length),
        (larger.margin, JOURNAL.margin),
        (larger.label_gap, JOURNAL.label_gap),
        (larger.title_gap, JOURNAL.title_gap),
        (larger.legend_gap, JOURNAL.legend_gap),
        (larger.ramp_width, JOURNAL.ramp_width),
        (larger.ramp_length, JOURNAL.ramp_length),
    ] {
        assert!(
            (ours - theirs * k).abs() < 1e-12,
            "{ours} is not {theirs} times {k}"
        );
    }
    // The colours, the leading and the key of a mark of the default size are the theme's
    // own, whatever the size.
    assert_eq!(
        (
            larger.ink,
            larger.grid,
            larger.background,
            larger.leading,
            larger.key_size
        ),
        (
            JOURNAL.ink,
            JOURNAL.grid,
            JOURNAL.background,
            JOURNAL.leading,
            JOURNAL.key_size
        )
    );
    // The slide theme is the journal theme at 18 points; a base size of its own comes first.
    let slide = Theme::of(&themed(ThemeName::Slide, None));
    assert_eq!(
        (slide.title_size, slide.label_size, slide.heading_size),
        (18.0, 16.0, 20.0)
    );
    assert_eq!(
        (slide.axis_width, slide.margin, slide.axis_lines),
        (1.0, 8.0, true)
    );
    assert_eq!(
        Theme::of(&themed(ThemeName::Slide, Some(24.0))).title_size,
        24.0
    );
}

#[test]
fn the_scene_carries_the_theme_and_writes_it_when_it_is_not_the_usual() {
    let usual = scene_of(PlotTheme::default());
    assert_eq!(usual.theme, PlotTheme::default());
    assert!(!usual.json().write().contains("\"theme\""));
    let slide = scene_of(themed(ThemeName::Slide, Some(24.0)));
    let json = slide.json().write();
    assert!(json.contains("\"theme\""), "{json}");
    assert!(
        json.contains("\"name\": \"slide\"") && json.contains("\"base_size\": 24"),
        "{json}"
    );
    // The theme is all that differs: the scales and the marks are as they were.
    assert_eq!(
        (&slide.scales, &slide.panels),
        (&usual.scales, &usual.panels)
    );
}

#[test]
fn minimal_draws_no_line_of_an_axis_and_no_tick() {
    let scene = scene_of(themed(ThemeName::Minimal, None));
    let journal = layout(&scene, Size::OneColumn, &JOURNAL).unwrap();
    let minimal = layout(&scene, Size::OneColumn, &Theme::of(&scene.theme)).unwrap();
    for axis in ["axis:x", "axis:y"] {
        let (lines, texts) = of_axis(&journal, axis);
        assert!(
            lines >= 3 && texts >= 3,
            "{axis}: {lines} lines, {texts} texts"
        );
        let (bare_lines, bare_texts) = of_axis(&minimal, axis);
        // Not a line, and every label and the title still there.
        assert_eq!((bare_lines, bare_texts), (0, texts), "{axis}");
    }
    // The gridlines stay, and the panel gains the room of the ticks.
    assert_eq!(of_axis(&minimal, "grid").0, of_axis(&journal, "grid").0);
    let (_, width, height, _) = panel_of(&minimal);
    let (_, journal_width, journal_height, _) = panel_of(&journal);
    assert!((width - journal_width - JOURNAL.tick_length).abs() < 1e-9);
    assert!((height - journal_height - JOURNAL.tick_length).abs() < 1e-9);
    insta::assert_snapshot!("six_points_minimal", minimal.describe());
}

#[test]
fn a_larger_base_size_sets_larger_type_and_leaves_the_marks_their_size() {
    let scene = scene_of(themed(ThemeName::Slide, None));
    let drawing = layout(&scene, Size::Slide, &Theme::of(&scene.theme)).unwrap();
    // The labels at 16 points and the titles at 18.
    let sizes = type_of(&drawing, "axis:x");
    assert!(sizes.contains(&16.0) && sizes.contains(&18.0), "{sizes:?}");
    assert!(sizes.iter().all(|size| *size == 16.0 || *size == 18.0));
    // The points are 4.5 points across, as the plot has them: a theme does not size marks.
    let (_, _, _, marks) = panel_of(&drawing);
    let points: Vec<f64> = marks
        .iter()
        .filter_map(|item| match &item.primitive {
            Primitive::Point { size, .. } => Some(*size),
            _ => None,
        })
        .collect();
    assert!(
        !points.is_empty() && points.iter().all(|size| *size == 4.5),
        "{points:?}"
    );
    insta::assert_snapshot!("six_points_on_a_slide", drawing.describe());
    // At 12 points on a page of one column the type is a third larger than usual.
    let scene = scene_of(themed(ThemeName::Journal, Some(12.0)));
    let drawing = layout(&scene, Size::OneColumn, &Theme::of(&scene.theme)).unwrap();
    let sizes = type_of(&drawing, "axis:y");
    assert!(
        sizes
            .iter()
            .all(|size| (*size - 32.0 / 3.0).abs() < 1e-9 || *size == 12.0),
        "{sizes:?}"
    );
}
