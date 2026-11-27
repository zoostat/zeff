//! The drawing of a diagram (release 0.10.0, step 22a; open decisions #121 and #191). What
//! every drawing of a diagram must be: each node drawn once as an outline with its label,
//! each edge once as a line, each under its element name, an arrow head where the kind of
//! the edge has one, a dashed line for a latent variable and for a common cause, a second
//! outline for a selected node, the exposure and the outcome filled; nothing off the page.
//! A diagram too large for the page is drawn in smaller type, one that leaves room at the
//! size of the theme's labels; where the script places the nodes they stand on its grid.
//! The three writers give the same figure every time; and with `ZEFF_WRITE_FIGURES` set to
//! a folder, the figures are written there to be read by eye.

use zeff_core::value::PlotTheme;
use zeff_plot::drawing::{Drawing, LineType, Place, Primitive, Step};
use zeff_plot::graph::{
    Direction, Edge, EdgeKind, End, GraphForm, Node, NodeKind, NodeShape, Placing, Role,
};
use zeff_plot::layout::{JOURNAL, LayoutError, Size, Theme, layout, layout_with_notes};
use zeff_plot::raster::{DOTS, render};
use zeff_plot::scene::Scene;
use zeff_plot::{diagram, pdf, png, svg};

/// A form of nodes called by `names`, those in `latent` latent, with arrows and ties.
fn form(
    names: &[&str],
    latent: &[usize],
    arrows: &[(usize, usize)],
    ties: &[(usize, usize, EdgeKind)],
) -> GraphForm {
    let end = |node: usize| End { node, port: None };
    GraphForm {
        nodes: names
            .iter()
            .enumerate()
            .map(|(at, name)| Node {
                id: (*name).to_string(),
                kind: if latent.contains(&at) {
                    NodeKind::Latent
                } else {
                    NodeKind::Observed
                },
                shape: if latent.contains(&at) {
                    NodeShape::Ellipse
                } else {
                    NodeShape::Box
                },
                label: vec![(*name).to_string()],
                selected: false,
                role: None,
                ports: Vec::new(),
            })
            .collect(),
        edges: arrows
            .iter()
            .map(|&(from, to)| Edge {
                from: end(from),
                to: end(to),
                kind: EdgeKind::Path,
                label: Vec::new(),
            })
            .chain(ties.iter().map(|&(from, to, kind)| Edge {
                from: end(from),
                to: end(to),
                kind,
                label: Vec::new(),
            }))
            .collect(),
        groups: Vec::new(),
        placing: Placing::default(),
    }
}

/// The graph of section 7.3, with training the exposure and productivity the outcome.
fn training() -> GraphForm {
    let mut form = form(
        &[
            "ability",
            "training",
            "productivity",
            "tenure",
            "age",
            "region",
        ],
        &[0],
        &[(0, 1), (1, 2), (0, 2), (1, 3), (3, 2), (4, 1), (5, 1)],
        &[],
    );
    form.nodes[1].role = Some(Role::Exposure);
    form.nodes[2].role = Some(Role::Outcome);
    form
}

/// A graph with a common cause over a layer and one between layers, a link, a selected
/// node, two latent ones and a label of two lines.
fn mixed() -> GraphForm {
    let mut form = form(
        &["price", "demand", "season", "taste", "listed", "x̄ and β̂"],
        &[3, 5],
        &[(2, 1), (3, 1), (1, 4), (5, 0)],
        &[
            (0, 1, EdgeKind::CommonCause),
            (0, 2, EdgeKind::CommonCause),
            (2, 4, EdgeKind::Link),
        ],
    );
    form.nodes[4].selected = true;
    form.nodes[1].label = vec!["demand".to_string(), "per week".to_string()];
    form
}

fn scene_of(form: GraphForm) -> Scene {
    Scene::of_diagram(form, PlotTheme::default(), None, None, None)
}

fn drawn(form: GraphForm, size: Size) -> Drawing {
    layout(&scene_of(form), size, &JOURNAL).unwrap()
}

/// The places a primitive is drawn at or towards, and how far beyond them its ink may
/// reach.
fn reach(primitive: &Primitive) -> Vec<Place> {
    match primitive {
        Primitive::Rectangle {
            at, width, height, ..
        } => vec![*at, (at.0 + width, at.1 + height)],
        Primitive::Path { steps, .. } => steps
            .iter()
            .flat_map(|step| match *step {
                Step::Move(p) | Step::Line(p) => vec![p],
                Step::Curve(c1, c2, p) => vec![c1, c2, p],
                Step::Arc { to, .. } => vec![to],
                Step::Close => Vec::new(),
            })
            .collect(),
        Primitive::Polygon { rings, .. } => rings.iter().flatten().copied().collect(),
        Primitive::Text { at, .. } => vec![*at],
        _ => Vec::new(),
    }
}

/// What every drawing of a diagram must be.
fn sound(form: &GraphForm, drawing: &Drawing) {
    let named = |name: &str| -> Vec<&Primitive> {
        drawing
            .items
            .iter()
            .filter(|item| item.name.as_deref() == Some(name))
            .map(|item| &item.primitive)
            .collect()
    };
    // The page first.
    assert_eq!(drawing.items[0].name.as_deref(), Some("background"));
    for (v, node) in form.nodes.iter().enumerate() {
        let parts = named(&node.name());
        // One outline, a second for a selected node, and a line of text for each line of
        // the label.
        let outlines: Vec<&&Primitive> = parts
            .iter()
            .filter(|p| matches!(p, Primitive::Rectangle { .. } | Primitive::Path { .. }))
            .collect();
        assert_eq!(
            outlines.len(),
            1 + usize::from(node.selected),
            "{}",
            node.id
        );
        let texts: Vec<String> = parts
            .iter()
            .filter_map(|p| match p {
                Primitive::Text { spans, .. } => Some(spans[0].text.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(texts, node.label, "node {v}");
        // A box for an observed node and an ellipse for a latent one, the latent dashed.
        let (stroke, fill) = match outlines[0] {
            Primitive::Rectangle { stroke, fill, .. } => {
                assert_eq!(node.shape, NodeShape::Box);
                (stroke.clone().unwrap(), fill.clone())
            }
            Primitive::Path { stroke, fill, .. } => {
                assert_eq!(node.shape, NodeShape::Ellipse);
                (stroke.clone().unwrap(), fill.clone())
            }
            _ => unreachable!(),
        };
        assert_eq!(
            stroke.dash == LineType::Dashed,
            node.kind == NodeKind::Latent,
            "{}",
            node.id
        );
        // The exposure and the outcome are filled with something other than the page, and
        // their labels are in the colour of the page.
        let text_colour = parts
            .iter()
            .find_map(|p| match p {
                Primitive::Text { spans, .. } => Some(spans[0].colour.clone()),
                _ => None,
            })
            .unwrap();
        let filled = fill.as_deref() != Some(JOURNAL.background);
        assert_eq!(filled, node.role.is_some(), "{}", node.id);
        assert_eq!(text_colour == JOURNAL.background, node.role.is_some());
    }
    for (e, edge) in form.edges.iter().enumerate() {
        let parts = named(&form.edge_name(e));
        let lines: Vec<&&Primitive> = parts
            .iter()
            .filter(|p| matches!(p, Primitive::Path { .. }))
            .collect();
        assert_eq!(lines.len(), 1, "{}", form.edge_name(e));
        let Primitive::Path { stroke, fill, .. } = lines[0] else {
            unreachable!()
        };
        assert!(fill.is_none());
        assert_eq!(
            stroke.as_ref().unwrap().dash == LineType::Dashed,
            edge.kind == EdgeKind::CommonCause
        );
        // As many heads as the kind of the edge has.
        let heads = parts
            .iter()
            .filter(|p| matches!(p, Primitive::Polygon { .. }))
            .count();
        let (first, second) = edge.kind.heads();
        assert_eq!(
            heads,
            usize::from(first) + usize::from(second),
            "{}",
            form.edge_name(e)
        );
    }
    // Every item is one of a node, an edge, the page or the text round the diagram.
    for item in &drawing.items {
        let name = item.name.as_deref().unwrap();
        assert!(
            name.starts_with("node:")
                || name.starts_with("edge:")
                || ["background", "title", "subtitle", "caption", "key", "notes"].contains(&name),
            "{name}"
        );
        // Nothing is drawn off the page.
        for (x, y) in reach(&item.primitive) {
            assert!(x.is_finite() && y.is_finite());
            assert!(x >= -1e-9 && x <= drawing.width + 1e-9, "{name}: {x}");
            assert!(y >= -1e-9 && y <= drawing.height + 1e-9, "{name}: {y}");
        }
    }
}

#[test]
fn the_graph_of_section_7_3_is_a_figure_in_the_three_formats() {
    let form = training();
    let drawing = drawn(form.clone(), Size::OneColumn);
    sound(&form, &drawing);
    assert_eq!((drawing.width, drawing.height), Size::OneColumn.points());
    assert!(drawing.notes.is_empty());
    // The line at the foot says what the marks mean, and names only what the diagram has.
    assert_eq!(
        diagram::key(&form).as_deref(),
        Some("exposure training · outcome productivity · dashed outline: latent")
    );
    let key: Vec<&Primitive> = drawing
        .items
        .iter()
        .filter(|item| item.name.as_deref() == Some("key"))
        .map(|item| &item.primitive)
        .collect();
    assert_eq!(key.len(), 1);
    // It fits the page of one column in the type of the theme's labels.
    assert_eq!(
        diagram::type_size(&scene_of(form.clone()), Size::OneColumn, &JOURNAL),
        Some(JOURNAL.label_size)
    );
    // The three writers, each the same bytes a second time.
    let svg_text = svg::write(&drawing);
    assert_eq!(svg_text, svg::write(&drawn(form.clone(), Size::OneColumn)));
    assert!(svg_text.starts_with("<svg") || svg_text.starts_with("<?xml"));
    // Each element is a group of the file under its name, written as an identifier: the
    // colon and the arrow of a name are not letters of one.
    for id in [
        "id=\"node-training\"",
        "id=\"edge-training--tenure\"",
        "id=\"node-ability\"",
        "id=\"key\"",
    ] {
        assert!(svg_text.contains(id), "the SVG lacks {id}");
    }
    // The words are in the file for search.
    for word in ["training", "productivity", "ability"] {
        assert!(svg_text.contains(word));
    }
    let pdf_bytes = pdf::write(&drawing);
    assert_eq!(pdf_bytes, pdf::write(&drawn(form.clone(), Size::OneColumn)));
    assert!(pdf_bytes.starts_with(b"%PDF-"));
    let picture = render(&drawing, DOTS).unwrap();
    let png_bytes = png::write(&picture, DOTS).unwrap();
    assert_eq!(&png_bytes[..8], b"\x89PNG\r\n\x1a\n");
    let again = render(&drawn(form, Size::OneColumn), DOTS).unwrap();
    assert_eq!(png_bytes, png::write(&again, DOTS).unwrap());
    insta::assert_snapshot!("diagram_training_svg", svg_text);
}

#[test]
fn every_mark_of_a_diagram() {
    let form = mixed();
    for size in [Size::OneColumn, Size::TwoColumn, Size::Slide] {
        let drawing = drawn(form.clone(), size);
        sound(&form, &drawing);
    }
    assert_eq!(
        diagram::key(&form).as_deref(),
        Some(
            "dashed outline: latent · double outline: selected on · dashed arrow: a common cause not observed"
        )
    );
    // A diagram with nothing to explain has no line at its foot.
    let plain = self::form(&["x", "y"], &[], &[(0, 1)], &[]);
    assert_eq!(diagram::key(&plain), None);
    let drawing = drawn(plain.clone(), Size::OneColumn);
    sound(&plain, &drawing);
    assert!(
        !drawing
            .items
            .iter()
            .any(|item| item.name.as_deref() == Some("key"))
    );
    // The titles and the caption are written, and the diagram is under and over them.
    let titled = Scene::of_diagram(
        training(),
        PlotTheme::default(),
        Some("What is assumed of training".to_string()),
        Some("The graph of section 7.3".to_string()),
        Some("Ability is not observed.".to_string()),
    );
    let drawing = layout(&titled, Size::OneColumn, &JOURNAL).unwrap();
    sound(&training(), &drawing);
    let baseline = |name: &str| -> f64 {
        drawing
            .items
            .iter()
            .find(|item| item.name.as_deref() == Some(name))
            .map(|item| match &item.primitive {
                Primitive::Text { at, .. } => at.1,
                _ => f64::NAN,
            })
            .unwrap()
    };
    let nodes: Vec<f64> = drawing
        .items
        .iter()
        .filter(|item| item.name.as_deref().is_some_and(|n| n.starts_with("node:")))
        .flat_map(|item| reach(&item.primitive))
        .map(|p| p.1)
        .collect();
    let (top, bottom) = (
        nodes.iter().copied().fold(f64::INFINITY, f64::min),
        nodes.iter().copied().fold(f64::NEG_INFINITY, f64::max),
    );
    assert!(baseline("title") < baseline("subtitle") && baseline("subtitle") < top);
    assert!(bottom < baseline("key") && baseline("key") < baseline("caption"));
    // An empty diagram is a page and nothing else.
    let empty = drawn(self::form(&[], &[], &[], &[]), Size::OneColumn);
    assert_eq!(empty.items.len(), 1);
    // A scene's notes are written at the foot when they are asked for.
    let mut noted = scene_of(training());
    noted.notes = vec!["a note about the graph".to_string()];
    let with = layout_with_notes(&noted, Size::OneColumn, &JOURNAL).unwrap();
    assert!(
        with.items
            .iter()
            .any(|item| item.name.as_deref() == Some("notes"))
    );
    let without = layout(&noted, Size::OneColumn, &JOURNAL).unwrap();
    assert!(
        !without
            .items
            .iter()
            .any(|item| item.name.as_deref() == Some("notes"))
    );
    assert_eq!(without.notes, noted.notes);
}

#[test]
fn a_diagram_too_large_for_the_page_is_drawn_smaller() {
    // Twelve nodes in a row do not fit one column in type of eight points.
    let names: Vec<String> = (0..12).map(|i| format!("variable {i}")).collect();
    let names: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
    let arrows: Vec<(usize, usize)> = (1..12).map(|v| (0, v)).collect();
    let wide = form(&names, &[], &arrows, &[]);
    let scene = scene_of(wide.clone());
    let small = diagram::type_size(&scene, Size::OneColumn, &JOURNAL).unwrap();
    assert!(small < JOURNAL.label_size && small > 0.5, "{small}");
    let drawing = layout(&scene, Size::OneColumn, &JOURNAL).unwrap();
    sound(&wide, &drawing);
    // The labels are in that type, and on a larger page in larger type.
    let label_size = |drawing: &Drawing| -> f64 {
        drawing
            .items
            .iter()
            .find_map(|item| match (&item.name, &item.primitive) {
                (Some(name), Primitive::Text { spans, .. }) if name.starts_with("node:") => {
                    Some(spans[0].size)
                }
                _ => None,
            })
            .unwrap()
    };
    assert_eq!(label_size(&drawing), small);
    let larger = layout(&scene, Size::Slide, &JOURNAL).unwrap();
    sound(&wide, &larger);
    assert!(label_size(&larger) > small);
    // The gaps a script gives in millimetres shrink with the type, so the shape is kept.
    let mut spaced = wide.clone();
    spaced.placing.spacing = Some((30.0, 8.0));
    let drawing = layout(&scene_of(spaced.clone()), Size::OneColumn, &JOURNAL).unwrap();
    sound(&spaced, &drawing);
    // A page with no room for a diagram is refused, as a page with no room for a panel is.
    assert!(matches!(
        layout(&scene, Size::Millimetres(10.0, 10.0), &JOURNAL),
        Err(LayoutError::TooSmall { .. })
    ));
    assert!(matches!(
        layout(&scene, Size::Millimetres(0.0, 50.0), &JOURNAL),
        Err(LayoutError::NotASize)
    ));
    // Arrows in a ring cannot be laid out, and the drawing says so.
    let ring = form(&["a", "b"], &[], &[(0, 1), (1, 0)], &[]);
    assert!(matches!(
        layout(&scene_of(ring), Size::OneColumn, &JOURNAL),
        Err(LayoutError::Undrawn(_))
    ));
}

#[test]
fn where_the_script_places_the_nodes_they_stand_on_its_grid() {
    // z over the first column, and x, m and y in a row under it: `layout: ["z . .", "x m y"]`.
    let mut placed = form(
        &["z", "x", "m", "y"],
        &[0],
        &[(0, 1), (1, 2), (2, 3), (0, 3)],
        &[],
    );
    placed.placing.at = Some(vec![(0.0, 0.0), (0.0, 1.0), (1.0, 1.0), (2.0, 1.0)]);
    let drawing = drawn(placed.clone(), Size::OneColumn);
    sound(&placed, &drawing);
    let middle = |name: &str| -> Place {
        drawing
            .items
            .iter()
            .find_map(|item| match (&item.name, &item.primitive) {
                (Some(n), Primitive::Text { at, .. }) if n == name => Some(*at),
                _ => None,
            })
            .unwrap()
    };
    let (z, x, m, y) = (
        middle("node:z"),
        middle("node:x"),
        middle("node:m"),
        middle("node:y"),
    );
    // One row, three columns a step apart, z over x.
    assert!((x.1 - m.1).abs() < 1e-9 && (m.1 - y.1).abs() < 1e-9);
    assert!((z.0 - x.0).abs() < 1e-9 && z.1 < x.1);
    assert!(((m.0 - x.0) - (y.0 - m.0)).abs() < 1e-9 && m.0 > x.0);
    // Every edge is one straight line.
    for (e, _) in placed.edges.iter().enumerate() {
        let name = placed.edge_name(e);
        let steps = drawing
            .items
            .iter()
            .find_map(|item| match (&item.name, &item.primitive) {
                (Some(n), Primitive::Path { steps, .. }) if *n == name => Some(steps.clone()),
                _ => None,
            })
            .unwrap();
        assert!(
            matches!(steps[..], [Step::Move(_), Step::Line(_)]),
            "{name}"
        );
    }
    // Half a step is allowed, and the layout to the right changes nothing where every
    // node is placed.
    let mut half = placed.clone();
    half.placing.at = Some(vec![(1.0, 0.0), (0.0, 1.0), (1.0, 1.5), (2.0, 1.0)]);
    half.placing.direction = Direction::Right;
    sound(&half, &drawn(half.clone(), Size::OneColumn));
    // To the right, the layers of the automatic layout follow from left to right.
    let mut right = training();
    right.placing.direction = Direction::Right;
    let drawing = drawn(right.clone(), Size::TwoColumn);
    sound(&right, &drawing);
    let x_of = |name: &str| -> f64 {
        drawing
            .items
            .iter()
            .find_map(|item| match (&item.name, &item.primitive) {
                (Some(n), Primitive::Text { at, .. }) if n == name => Some(at.0),
                _ => None,
            })
            .unwrap()
    };
    assert!(x_of("node:age") < x_of("node:training"));
    assert!(x_of("node:training") < x_of("node:tenure"));
    assert!(x_of("node:tenure") < x_of("node:productivity"));
    // Kept on one layer by the script, two nodes share it.
    let mut together = form(
        &["z", "a", "b", "x", "y"],
        &[],
        &[(0, 1), (1, 2), (2, 3), (0, 4)],
        &[],
    );
    together.placing.same_rank = vec![vec![3, 4]];
    let drawing = drawn(together.clone(), Size::OneColumn);
    sound(&together, &drawing);
}

/// With `ZEFF_WRITE_FIGURES` set to a folder, the figures of these tests are written there
/// as PNG, SVG and PDF, to be read by eye; otherwise nothing is written.
#[test]
fn the_figures_are_written_for_reading_by_eye_when_asked() {
    let Ok(folder) = std::env::var("ZEFF_WRITE_FIGURES") else {
        return;
    };
    let folder = std::path::PathBuf::from(folder);
    let mut right = training();
    right.placing.direction = Direction::Right;
    let mut placed = form(
        &["z", "x", "m", "y"],
        &[0],
        &[(0, 1), (1, 2), (2, 3), (0, 3)],
        &[],
    );
    placed.placing.at = Some(vec![(0.0, 0.0), (0.0, 1.0), (1.0, 1.0), (2.0, 1.0)]);
    let names: Vec<String> = (0..12).map(|i| format!("variable {i}")).collect();
    let names: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
    let arrows: Vec<(usize, usize)> = (1..12).map(|v| (0, v)).collect();
    let mut earnings = form(
        &[
            "female", "ability", "educ", "exper", "union", "lwage", "age",
        ],
        &[1],
        &[
            (0, 2),
            (1, 2),
            (0, 5),
            (2, 5),
            (3, 5),
            (4, 5),
            (1, 5),
            (0, 4),
            (6, 3),
            (2, 3),
        ],
        &[],
    );
    earnings.nodes[2].role = Some(Role::Exposure);
    earnings.nodes[5].role = Some(Role::Outcome);
    let figures: Vec<(&str, GraphForm, Size)> = vec![
        ("training", training(), Size::OneColumn),
        ("training_right", right, Size::TwoColumn),
        ("mixed", mixed(), Size::OneColumn),
        ("placed", placed, Size::OneColumn),
        ("wide", form(&names, &[], &arrows, &[]), Size::OneColumn),
        ("earnings", earnings, Size::OneColumn),
    ];
    for (name, form, size) in figures {
        let theme = Theme::of(&PlotTheme::default());
        let drawing = layout(&scene_of(form), size, &theme).unwrap();
        std::fs::write(folder.join(format!("{name}.svg")), svg::write(&drawing)).unwrap();
        std::fs::write(folder.join(format!("{name}.pdf")), pdf::write(&drawing)).unwrap();
        let picture = render(&drawing, DOTS).unwrap();
        std::fs::write(
            folder.join(format!("{name}.png")),
            png::write(&picture, DOTS).unwrap(),
        )
        .unwrap();
    }
}
