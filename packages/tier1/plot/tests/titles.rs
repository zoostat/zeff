//! The words of a plot (Language Reference, section 7.6, "Guides and theme": `labels`;
//! release 0.7.0, step 26a): a title and a subtitle above the panel, a caption under it, and
//! the titles of the two axes and of the legend of colour in the words of the script. The
//! label of a variable stays where the script says nothing; an empty text takes a title away;
//! a text too long for its line is broken between words.

mod common;

use common::*;
use zeff_columns::{Column, MetadataKey};
use zeff_core::value::{Aesthetic, Arrangement, Frame, Mark, Plot, PlotLabels, PlotLayer};
use zeff_plot::drawing::{Drawing, Primitive};
use zeff_plot::font::FaceName;
use zeff_plot::layout::{JOURNAL, LayoutError, Size, layout};
use zeff_plot::scene::{Scene, start, with_layer};

/// Four rows: a dose with a label, a change, and an arm.
fn trial() -> Frame {
    frame(
        "d",
        vec![
            (
                "dose",
                numbers(&[1.0, 2.0, 3.0, 4.0]).with_entry(MetadataKey::Label, "Dose (mg)"),
            ),
            ("change", numbers(&[0.4, 1.1, 2.3, 2.9])),
            (
                "arm",
                Column::strs([Some("a"), Some("b"), Some("a"), Some("b")]),
            ),
        ],
    )
}

/// The trial coloured by its arm, with the words of the script.
fn worded(labels: PlotLabels) -> Scene {
    let mut plot: Plot = start(
        trial(),
        vec![
            mapped(Aesthetic::X, "dose"),
            mapped(Aesthetic::Y, "change"),
            mapped(Aesthetic::Color, "arm"),
        ],
        Vec::new(),
    );
    plot.labels = labels;
    with_layer(&plot, points(1.0)).1
}

fn words(text: &str) -> Option<String> {
    Some(text.to_string())
}

/// The texts of the items called `name`: each line, its face, its size and where it stands.
fn lines_of(drawing: &Drawing, name: &str) -> Vec<(String, FaceName, f64, (f64, f64))> {
    drawing
        .items
        .iter()
        .filter(|item| item.name.as_deref() == Some(name))
        .filter_map(|item| match &item.primitive {
            Primitive::Text { at, spans, .. } => spans
                .first()
                .map(|span| (span.text.clone(), span.face, span.size, *at)),
            _ => None,
        })
        .collect()
}

fn drawn(scene: &Scene) -> Drawing {
    layout(scene, Size::OneColumn, &JOURNAL).unwrap()
}

#[test]
fn the_label_of_a_variable_titles_its_axis_until_the_script_says_otherwise() {
    let plain = worded(PlotLabels::default());
    assert_eq!(plain.scale("x").unwrap().title, "Dose (mg)");
    assert_eq!(plain.scale("y").unwrap().title, "change");
    assert_eq!(plain.scale("color").unwrap().title, "arm");
    assert_eq!(plain.legends[0].title, "arm");
    assert_eq!(
        (&plain.title, &plain.subtitle, &plain.caption),
        (&None, &None, &None)
    );
    // A scene with no words of the script is written as it was before `labels` existed.
    assert!(!plain.json().write().contains("\"titles\""));

    let own = worded(PlotLabels {
        x: words("Daily dose"),
        y: words("Change in score"),
        color: words("Arm of the trial"),
        ..PlotLabels::default()
    });
    assert_eq!(own.scale("x").unwrap().title, "Daily dose");
    assert_eq!(own.scale("y").unwrap().title, "Change in score");
    assert_eq!(own.scale("color").unwrap().title, "Arm of the trial");
    assert_eq!(own.legends[0].title, "Arm of the trial");
    // The titles are all that differs: the limits, the breaks and the marks are as they were.
    assert_eq!(
        own.scale("x").unwrap().breaks,
        plain.scale("x").unwrap().breaks
    );
    assert_eq!(own.panels, plain.panels);
}

#[test]
fn a_title_a_subtitle_and_a_caption_stand_above_and_under_the_figure() {
    let scene = worded(PlotLabels {
        title: words("Change by dose"),
        subtitle: words("Four patients"),
        caption: words("Source: the trial."),
        ..PlotLabels::default()
    });
    assert_eq!(scene.title.as_deref(), Some("Change by dose"));
    let json = scene.json().write();
    assert!(json.contains("\"titles\""), "{json}");
    assert!(json.contains("\"title\": \"Change by dose\""), "{json}");
    let drawing = drawn(&scene);
    let (panel, _, height, _) = panel_of(&drawing);
    let title = lines_of(&drawing, "title");
    let subtitle = lines_of(&drawing, "subtitle");
    let caption = lines_of(&drawing, "caption");
    // The title in the heavier face at 10 points, the subtitle under it at 9, both above
    // the panel and from the left margin of the page; the caption at 8 under everything.
    assert_eq!(title.len(), 1);
    assert_eq!(
        (title[0].0.as_str(), title[0].1, title[0].2),
        ("Change by dose", FaceName::SansSemiBold, 10.0)
    );
    assert_eq!(
        (subtitle[0].0.as_str(), subtitle[0].1, subtitle[0].2),
        ("Four patients", FaceName::Sans, 9.0)
    );
    assert_eq!(
        (caption[0].0.as_str(), caption[0].1, caption[0].2),
        ("Source: the trial.", FaceName::Sans, 8.0)
    );
    assert!(title[0].3.1 < subtitle[0].3.1 && subtitle[0].3.1 < panel.1);
    assert_eq!(
        (title[0].3.0, subtitle[0].3.0, caption[0].3.0),
        (JOURNAL.margin, JOURNAL.margin, JOURNAL.margin)
    );
    let (_, page_height) = Size::OneColumn.points();
    assert!(caption[0].3.1 > panel.1 + height + 20.0 && caption[0].3.1 < page_height);
    // The panel is lower than without them, by the room they take.
    let (bare, _, bare_height, _) = panel_of(&drawn(&worded(PlotLabels::default())));
    assert!(panel.1 > bare.1 + 20.0);
    assert!(height < bare_height - 30.0);
    insta::assert_snapshot!("a_titled_figure", drawing.describe());
}

#[test]
fn an_empty_text_takes_a_title_away() {
    let scene = worded(PlotLabels {
        title: words(""),
        subtitle: words("   "),
        x: words(""),
        ..PlotLabels::default()
    });
    assert_eq!((&scene.title, &scene.subtitle), (&None, &None));
    assert_eq!(scene.scale("x").unwrap().title, "");
    let drawing = drawn(&scene);
    assert!(lines_of(&drawing, "title").is_empty() && lines_of(&drawing, "subtitle").is_empty());
    // The axis keeps its labels and loses its title, and the panel gains the room.
    let written = drawing.describe();
    assert!(!written.contains("Dose (mg)"));
    let (_, _, height, _) = panel_of(&drawing);
    let (_, _, titled, _) = panel_of(&drawn(&worded(PlotLabels::default())));
    assert!(height > titled + 10.0);
}

#[test]
fn a_long_title_is_broken_between_words_and_a_long_word_is_not() {
    let long = "A title of many words that runs on and on, well past the right edge of a figure one column wide, and then some more";
    let scene = worded(PlotLabels {
        title: words(long),
        caption: words(long),
        ..PlotLabels::default()
    });
    let drawing = drawn(&scene);
    let title = lines_of(&drawing, "title");
    let caption = lines_of(&drawing, "caption");
    assert!(title.len() >= 3, "{} lines", title.len());
    // The lines together are the title, word for word; each line is under the one before.
    assert_eq!(
        title
            .iter()
            .map(|line| line.0.as_str())
            .collect::<Vec<&str>>()
            .join(" "),
        long
    );
    assert!(
        title
            .windows(2)
            .all(|pair| pair[1].3.1 - pair[0].3.1 == 10.0 * JOURNAL.leading)
    );
    // The caption is in the smaller type, so it takes fewer lines or as many.
    assert!(caption.len() <= title.len() && caption.len() >= 2);
    // One word wider than the page stands on a line of its own, unbroken.
    let word = "Pneumonoultramicroscopicsilicovolcanoconiosisandmoreandmoreandmoreandmore";
    let drawing = drawn(&worded(PlotLabels {
        title: Some(format!("About {word} today")),
        ..PlotLabels::default()
    }));
    let title = lines_of(&drawing, "title");
    assert!(title.iter().any(|line| line.0 == word), "{title:?}");
    // A title of very many lines leaves no room for the panel, and the layout says so.
    let endless = [long; 12].join(" ");
    let refused = layout(
        &worded(PlotLabels {
            title: Some(endless),
            ..PlotLabels::default()
        }),
        Size::OneColumn,
        &JOURNAL,
    );
    assert!(
        matches!(refused, Err(LayoutError::TooSmall { .. })),
        "{refused:?}"
    );
}

#[test]
fn the_legend_of_fills_is_titled_by_color_where_nothing_is_mapped_to_color() {
    // Bars filled by the arm, nothing mapped to `color`: the words for the colour title it.
    let mut plot: Plot = start(
        trial(),
        vec![mapped(Aesthetic::X, "dose"), mapped(Aesthetic::Fill, "arm")],
        Vec::new(),
    );
    plot.labels.color = words("Arm of the trial");
    let bars = PlotLayer {
        mark: Mark::Bars(Arrangement::Stack),
        ..points(1.0)
    };
    let (_, scene) = with_layer(&plot, bars);
    assert_eq!(scene.scale("fill").unwrap().title, "Arm of the trial");
    assert_eq!(scene.legends[0].title, "Arm of the trial");
    // The count axis takes the words for `y`.
    assert_eq!(scene.scale("y").unwrap().title, "count");
    let mut plot = plot;
    plot.labels.y = words("Patients");
    let bars = PlotLayer {
        mark: Mark::Bars(Arrangement::Stack),
        ..points(1.0)
    };
    let (_, scene) = with_layer(&plot, bars);
    assert_eq!(scene.scale("y").unwrap().title, "Patients");
}
