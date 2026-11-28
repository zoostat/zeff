//! `g.plot(exposure:, outcome:, direction:, layout:, at:, same_rank:, spacing:)` (release
//! 0.10.0, step 22b; catalogue row 501; open decision #121, D2 and D4), through scripts: what
//! the plot of a graph prints; the three formats `export` writes, and the statement of the
//! specification's sample script; that `layout:` and `at:` giving the same places give the
//! same figure to the byte, and that each option changes the figure; what `labels` and
//! `theme` do and that the methods of a plot of data are refused; every form and every
//! message. The layout and the drawing themselves are held in the `plot` package.

use zeff_testkit::{FileSystem, project, run, run_script};

fn go(script: &str) -> (i32, String, String) {
    let outcome = run(script);
    (
        outcome.exit_code,
        outcome.output.stdout.clone(),
        outcome.output.stderr.clone(),
    )
}

fn printed(script: &str) -> String {
    let (exit, stdout, stderr) = go(script);
    assert_eq!(exit, 0, "{script}\n{stderr}");
    stdout
}

/// The files a script wrote under `output/`, by name.
fn written(script: &str, names: &[&str]) -> Vec<Vec<u8>> {
    let (fs, project) = project();
    let outcome = run_script(&fs, &project, "scripts/main.zeff", script);
    assert_eq!(outcome.exit_code, 0, "{script}\n{}", outcome.output.stderr);
    names
        .iter()
        .map(|name| {
            FileSystem::read(&fs, std::path::Path::new(&format!("/t/output/{name}")))
                .unwrap_or_else(|_| panic!("{name} was not written"))
        })
        .collect()
}

const TRAINING: &str = "let g = dag {\n    ability -> training -> productivity\n    ability -> productivity\n    training -> tenure -> productivity\n    [age, region] -> training\n    latent ability\n}\n";

const FRONT: &str = "let f = dag {\n    x -> m -> y\n    u -> [x, y]\n    latent u\n}\n";

#[test]
fn the_graph_of_section_7_3_prints_and_is_written_in_three_formats() {
    assert_eq!(
        printed(&format!(
            "{TRAINING}g.plot(exposure: training, outcome: productivity)\n"
        )),
        "plot · causal graph · 6 nodes, 7 edges\n\n  node           kind       part\n  ability        latent\n  training       observed   exposure\n  productivity   observed   outcome\n  tenure         observed\n  age            observed\n  region         observed\n\nplaced in layers, down the page\n"
    );
    // Without a part marked, the table has no column for one; the value is a plot.
    assert_eq!(
        printed(&format!(
            "{TRAINING}let p = g.plot()\np\nprint(type_of(p))\n"
        )),
        "plot · causal graph · 6 nodes, 7 edges\n\n  node           kind\n  ability        latent\n  training       observed\n  productivity   observed\n  tenure         observed\n  age            observed\n  region         observed\n\nplaced in layers, down the page\nPlot\n"
    );
    // The statement of the specification's sample script, and the other two formats.
    let files = written(
        &format!(
            "{TRAINING}g.plot().export(\"output/fig_dag.pdf\", size: one_column)\nlet p = g.plot(exposure: training, outcome: productivity)\np.export(\"output/fig_dag.svg\")\np.export(\"output/fig_dag.png\", size: two_column)\n"
        ),
        &["fig_dag.pdf", "fig_dag.svg", "fig_dag.png"],
    );
    assert!(files[0].starts_with(b"%PDF-"));
    let svg = String::from_utf8(files[1].clone()).unwrap();
    for id in [
        "id=\"node-training\"",
        "id=\"node-ability\"",
        "id=\"edge-training--tenure\"",
        "id=\"key\"",
    ] {
        assert!(svg.contains(id), "the SVG lacks {id}");
    }
    assert_eq!(&files[2][..8], b"\x89PNG\r\n\x1a\n");
    // The same script writes the same bytes again.
    let again = written(
        &format!(
            "{TRAINING}g.plot().export(\"output/fig_dag.pdf\", size: one_column)\nlet p = g.plot(exposure: training, outcome: productivity)\np.export(\"output/fig_dag.svg\")\np.export(\"output/fig_dag.png\", size: two_column)\n"
        ),
        &["fig_dag.pdf", "fig_dag.svg", "fig_dag.png"],
    );
    assert_eq!(files, again);
    // The graph written with its lines in another order is the same figure.
    let turned = "let g = dag {\n    latent ability\n    [region, age] -> training\n    tenure -> productivity\n    training -> [tenure, productivity]\n    ability -> [productivity, training]\n}\n";
    let other = written(
        &format!(
            "{turned}g.plot(exposure: training, outcome: productivity).export(\"output/fig_dag.svg\")\n"
        ),
        &["fig_dag.svg"],
    );
    assert_eq!(other[0], files[1]);
}

#[test]
fn the_script_places_the_nodes_and_each_option_changes_the_figure() {
    let figure = |call: &str| -> Vec<u8> {
        written(
            &format!("{FRONT}f.{call}.export(\"output/f.svg\")\n"),
            &["f.svg"],
        )
        .remove(0)
    };
    let automatic = figure("plot()");
    // A grid row by row and the same places by name are one figure, to the byte.
    let grid = figure("plot(layout: [\". u .\", \"x m y\"])");
    let named = figure("plot(at: {u: [1, 0], x: [0, 1], m: [1, 1], y: [2, 1]})");
    assert_eq!(grid, named);
    assert_ne!(grid, automatic);
    // The fields of `at:` in another order, and whole numbers written as decimals.
    assert_eq!(
        figure("plot(at: {y: [2.0, 1.0], m: [1, 1], x: [0, 1], u: [1.0, 0]})"),
        named
    );
    // Half a step moves a node.
    assert_ne!(
        figure("plot(at: {u: [1, 0], x: [0, 1], m: [1, 1.5], y: [2, 1]})"),
        named
    );
    // Each option of the automatic layout changes the figure, and the same option twice
    // gives the same figure.
    for call in [
        "plot(direction: right)",
        "plot(spacing: [20, 10])",
        "plot(exposure: x)",
        "plot(exposure: x, outcome: y)",
    ] {
        assert_ne!(figure(call), automatic, "{call}");
        assert_eq!(figure(call), figure(call), "{call}");
    }
    // `direction: down` is the layout as it stands.
    assert_eq!(figure("plot(direction: down)"), automatic);
    // Two ends of a fork brought to one layer; one set may be written without the outer
    // list.
    let fork = |call: &str| -> Vec<u8> {
        written(
            &format!(
                "let k = dag {{\n    z -> a -> b -> x\n    z -> y\n}}\nk.{call}.export(\"output/k.svg\")\n"
            ),
            &["k.svg"],
        )
        .remove(0)
    };
    assert_ne!(fork("plot(same_rank: [[x, y]])"), fork("plot()"));
    assert_eq!(
        fork("plot(same_rank: [x, y])"),
        fork("plot(same_rank: [[x, y]])")
    );
    // What the placing prints.
    assert_eq!(
        printed(&format!("{FRONT}f.plot(layout: [\". u .\", \"x m y\"])\n")),
        "plot · causal graph · 4 nodes, 4 edges\n\n  node   kind\n  x      observed\n  m      observed\n  y      observed\n  u      latent\n\nplaced by the script, each node on its place of a grid\n"
    );
    let out = printed(
        "let k = dag {\n    z -> a -> b -> x\n    z -> y\n}\nk.plot(direction: right, spacing: [14, 5], same_rank: [[x, y]])\n",
    );
    assert!(
        out.ends_with("placed in layers, to the right\non one layer: x, y\n14 mm between layers, 5 mm between neighbours\n"),
        "{out}"
    );
}

#[test]
fn a_diagram_takes_titles_and_a_theme_and_no_marks() {
    // `labels` and `theme` change the figure and leave the printed form a diagram's.
    let files = written(
        &format!(
            "{TRAINING}let p = g.plot(exposure: training, outcome: productivity)\np.export(\"output/plain.svg\")\nlet q = p.labels(title: \"What is assumed of training\", caption: \"Ability is not observed.\")\nq.export(\"output/titled.svg\")\np.theme(slide).export(\"output/slide.svg\", size: slide)\n"
        ),
        &["plain.svg", "titled.svg", "slide.svg"],
    );
    let text = |bytes: &[u8]| String::from_utf8(bytes.to_vec()).unwrap();
    let (plain, titled, slide) = (text(&files[0]), text(&files[1]), text(&files[2]));
    assert!(!plain.contains("id=\"title\"") && !plain.contains("id=\"caption\""));
    assert!(titled.contains("id=\"title\"") && titled.contains("id=\"caption\""));
    assert!(titled.contains("What is assumed of training"));
    assert_ne!(slide, plain);
    assert!(
        printed(&format!("{TRAINING}g.plot().labels(title: \"A title\")\n"))
            .starts_with("plot · causal graph · 6 nodes, 7 edges\n")
    );
    // A graph with every kind of edge and a selected node is drawn too: undirected links
    // are for drawing.
    let out = printed(
        "dag {\n    price <-> demand\n    demand -- season\n    `ad spend` -> demand\n    demand -> listed\n    selected listed\n    latent taste\n}.plot(outcome: demand)\n",
    );
    assert_eq!(
        out,
        "plot · causal graph · 6 nodes, 4 edges\n\n  node       kind                    part\n  price      observed\n  demand     observed                outcome\n  season     observed\n  ad spend   observed\n  listed     observed, selected on\n  taste      latent\n\nplaced in layers, down the page\n"
    );
    // An empty graph is an empty figure.
    assert!(printed("dag { }.plot()\n").starts_with("plot · causal graph · 0 nodes, 0 edges\n"));
}

#[test]
fn every_form_and_every_message() {
    let (exit, stdout, stderr) = go(&format!(
        "{TRAINING}g.plot()\ng.plot(exposure: training, outcome: productivity)\ng.plot(outcome: productivity, direction: right)\ng.plot(same_rank: [[ability, age, region], [tenure]])\ng.plot(spacing: [12, 6.5])\n{FRONT}f.plot(layout: [\". u .\", \"x m y\"], exposure: x, outcome: y)\nf.plot(at: {{u: [1, 0], x: [0, 1], m: [1, 1.5], y: [2, 1]}}, spacing: [10, 4])\ndag {{ `gross sales` -> profit }}.plot(layout: [\"`gross sales` profit\"])\ndag {{ a }}.plot()\ndag {{ }}.plot()\n"
    ));
    assert_eq!(exit, 0, "{stderr}");
    insta::assert_snapshot!("drawn_forms", format!("{stdout}---\n{stderr}"));
    let mut all = String::new();
    for call in [
        // A name that is no node, in each option that takes names.
        "f.plot(exposure: z)",
        "f.plot(outcome: mm)",
        "f.plot(layout: [\". u .\", \"x m z\"])",
        "f.plot(at: {u: [1, 0], x: [0, 1], m: [1, 1], w: [2, 1]})",
        "f.plot(same_rank: [[u, q]])",
        // A node left out, placed twice, or two on one place.
        "f.plot(layout: [\". u .\", \"x . y\"])",
        "f.plot(layout: [\"u u\", \"x m y\"])",
        "f.plot(at: {u: [1, 0], x: [0, 1]})",
        "f.plot(at: {u: [1, 0], x: [0, 1], m: [1, 1], y: [1, 1]})",
        // A place that is not two numbers; a row that is not a text; a backtick left open.
        "f.plot(at: {u: [1, 0, 2], x: [0, 1], m: [1, 1], y: [2, 1]})",
        "f.plot(at: {u: \"top\", x: [0, 1], m: [1, 1], y: [2, 1]})",
        "f.plot(layout: [1, 2])",
        "f.plot(layout: [\"`u x\", \"m y\"])",
        // Placed twice over; an option of the automatic layout beside places.
        "f.plot(layout: [\". u .\", \"x m y\"], at: {u: [1, 0], x: [0, 1], m: [1, 1], y: [2, 1]})",
        "f.plot(layout: [\". u .\", \"x m y\"], same_rank: [[x, y]])",
        // Nodes an arrow sets apart, and a node in two sets.
        "f.plot(same_rank: [[x, m]])",
        "f.plot(same_rank: [[x, y]])",
        "f.plot(same_rank: [[u, m], [m, y]])",
        // Gaps that are not two lengths above zero.
        "f.plot(spacing: [10])",
        "f.plot(spacing: [10, 0])",
        "f.plot(spacing: 10)",
        // The exposure again; a word that is no direction; an option the function lacks.
        "f.plot(exposure: x, outcome: x)",
        "f.plot(direction: up)",
        "f.plot(highlight: open_paths)",
        // The methods of a plot of data.
        "f.plot().points()",
        "f.plot().facet(by: x)",
        "f.plot().scale_x(log)",
        "f.plot().coord(flip)",
        "f.plot().legend(top)",
    ] {
        let (exit, _, stderr) = go(&format!("{FRONT}{call}\n"));
        assert_eq!(exit, 1, "{call}");
        all.push_str(&stderr);
        all.push('\n');
    }
    insta::assert_snapshot!("drawn_messages", all);
}
