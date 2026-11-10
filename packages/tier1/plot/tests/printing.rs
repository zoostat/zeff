//! `plot` and `points` as a script writes them (Language Reference, section 7.6; release 0.7.0,
//! step 20): the printed form of a plot, which is its scene read as text, in every shape; the
//! notes of the rows not drawn and of the seed; jitter repeated under a seed; and a snapshot of
//! every message.

use zeff_testkit::run;

/// Exit code, standard output and messages of a script.
fn go(script: &str) -> (i32, String, String) {
    let outcome = run(script);
    (
        outcome.exit_code,
        outcome.output.stdout,
        outcome.output.stderr,
    )
}

/// What a script that runs prints, with its messages after a rule.
fn printed(script: &str) -> String {
    let (code, stdout, stderr) = go(script);
    assert_eq!(code, 0, "{script}\n{stderr}");
    format!("{stdout}---\n{stderr}")
}

const DOSES: &str =
    "let d = frame(dose = [1, 2, 3, 4, NA], response = [2.1, 3.9, 6.2, 7.8, 9.0])\n";

#[test]
fn the_wages_scatter_reads_sensibly_as_text() {
    let text = printed(
        "seed(20261111)\nlet w = dataset(wages)\nw.plot(x: educ, y: wage)\nw.plot(x: educ, y: wage).points()\nw.plot(x: educ, y: wage).points(alpha: 0.3, jitter: 0.4)\nlet top = w.where(wage > 100)\nw.plot(x: educ, y: wage).points(alpha: 0.3).points(data: top, size: 8)\n",
    );
    // The reference, from R on the shipped file: 18,412 rows, 18,201 of them complete in educ
    // and wage; educ from 8 to 20 and wage from 3.53 to 152.39, so the limits, 5% of the range
    // beyond each end, are 7.4 and 20.6, and -3.913 and 159.833; 26 wages above 100. The breaks
    // are the multiples of 2 and of 50 within the limits.
    for line in [
        "  x      Years of schooling   linear   7.4 to 20.6         8, 10, 12, 14, 16, 18, 20\n",
        "  y      Hourly wage          linear   -3.913 to 159.833   0, 50, 100, 150\n",
        "      1   points   w      18,201   size 4.5, alpha 1\n",
        "      2   points   top        26   size 8, alpha 1\n",
        "note: 211 rows with NA in educ or wage dropped (line 4)\n",
    ] {
        assert!(text.contains(line), "{line}\n{text}");
    }
    insta::assert_snapshot!("wages", text);
}

#[test]
fn every_shape_of_a_printed_plot() {
    let script = format!(
        "{DOSES}let p = d.plot(x: dose, y: response)\np\np.points()\np.points(size: 2, alpha: 0.5).points(data: frame(dose = [2.5], response = [5.0]), size: 9)\nd.plot(x: dose)\nd.plot(y: response)\nd.plot()\nplot(d, x: response, y: dose).points()\nframe(a = [1, 2, 3], b = [true, false, true]).plot(x: a, y: b).points()\nframe(a = [1000000, 3000000], b = [0.00001, 0.00003]).plot(x: a, y: b).points()\nframe(a = [1.5, 2.0], b = [NA, 1.0]).where(a < 2).plot(x: a, y: b).points()\nframe(a = [1, 2, 3], b = [1.0, 2.0, 1 / 0]).plot(x: a, y: b).points()\n"
    );
    insta::assert_snapshot!("shapes", printed(&script));
}

/// The edge-case datasets of the testing handbook (section 1.2), each drawn as points: what is
/// printed, noted or refused is fixed for every one.
#[test]
fn the_edge_case_datasets() {
    let mut all = String::new();
    for script in [
        "let d = import(\"data/one_row.csv\")\nd.plot(x: x, y: y).points()\n",
        "seed(5)\nlet d = import(\"data/zero_variance.csv\")\nd.plot(x: constant, y: big_constant).points()\nd.plot(x: constant_with_na, y: constant).points(jitter: 0.3)\n",
        "let d = import(\"data/infinities.csv\")\nd.plot(x: finite, y: plus).points()\nd.plot(x: both, y: nan_mixed).points()\n",
        "let d = import(\"data/magnitudes.csv\")\nd.plot(x: offset, y: wide).points()\nd.plot(x: tiny, y: wide).points()\n",
        "let d = import(\"data/bools.csv\")\nd.plot(x: treated, y: with_na).points()\n",
        "let d = import(\"data/empty.csv\")\nd.plot(x: x, y: y).points()\n",
        "let d = import(\"data/all_na.csv\")\nd.plot(x: x, y: y).points()\n",
        "let d = import(\"data/awkward_names.csv\")\nd.plot(x: `Gross Sales (total)`, y: `日本語`).points()\n",
        "let d = import(\"data/unicode_labels.csv\")\nd.plot(x: n, y: n).points()\nd.plot(x: n, y: city).points()\n",
        "let d = import(\"data/guess_boundary.csv\")\nd.plot(x: id, y: hours).points()\n",
        "let d = import(\"data/level_symbols.csv\")\nd.plot(x: n, y: y).points()\nd.plot(x: grp, y: y).points()\n",
        "let d = import(\"data/wages.csv\")\nd.plot(x: hours, y: wage).points()\n",
        "seed(5)\nlet d = import(\"data/wages_large.csv\")\nd.plot(x: educ, y: wage).points()\nd.plot(x: female, y: hours).points(jitter: 0.2)\n",
    ] {
        let (code, stdout, stderr) = go(script);
        all.push_str(&format!(
            "{script}exit code {code}\n{stdout}---\n{stderr}\n"
        ));
    }
    insta::assert_snapshot!("edge_case_datasets", all);
}

#[test]
fn a_variable_label_is_the_title_of_its_axis() {
    let (code, stdout, stderr) = go(&format!(
        "{DOSES}let l = d.label(dose, \"Dose (mg)\")\nl.plot(x: dose, y: response)\n"
    ));
    assert_eq!(code, 0, "{stderr}");
    assert!(
        stdout.contains("  x      Dose (mg)   linear   0.85 to 4.15"),
        "{stdout}"
    );
    assert!(stdout.contains("  y      response    linear"), "{stdout}");
}

#[test]
fn a_plot_is_a_value_that_adding_a_layer_does_not_change() {
    let (code, stdout, stderr) = go(&format!(
        "{DOSES}let p = d.plot(x: dose, y: response)\nlet q = p.points()\nlet r = q.points(size: 2)\np\nq\nr\n"
    ));
    assert_eq!(code, 0, "{stderr}");
    let plots: Vec<&str> = stdout.split("plot · d · response against dose\n").collect();
    assert_eq!(plots.len(), 4, "{stdout}");
    assert!(plots[1].contains("no layer yet"), "{stdout}");
    assert!(
        plots[2].contains("      1   points") && !plots[2].contains("      2   points"),
        "{stdout}"
    );
    assert!(plots[3].contains("      2   points"), "{stdout}");
    // The note is raised where the layer is added, once: the second layer leaves out the same
    // row and does not repeat it, and printing a plot raises nothing.
    assert_eq!(
        stderr,
        "note: 1 row with NA in dose or response dropped (line 3)\n"
    );
}

#[test]
fn a_layer_of_another_frame_raises_its_own_note() {
    let (code, _, stderr) = go(&format!(
        "{DOSES}let e = frame(dose = [1, NA, NA], response = [1.0, 2.0, 3.0])\nd.plot(x: dose, y: response).points().points(data: e)\n"
    ));
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(
        stderr,
        "note: 1 row with NA in dose or response dropped (line 3)\nnote: 2 rows with NA in dose or response dropped (line 3)\n"
    );
}

#[test]
fn jitter_is_repeated_by_the_seed_and_changed_by_another() {
    let script = |seed: &str| {
        format!(
            "{seed}let d = frame(a = [1, 1, 2, 2, 3, 3, 4, 4], b = [1, 2, 1, 2, 1, 2, 1, 2])\nd.plot(x: a, y: b).points(jitter: 0.4)\n"
        )
    };
    let (code, first, stderr) = go(&script("seed(7)\n"));
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stderr, "");
    let (_, again, _) = go(&script("seed(7)\n"));
    let (_, other, _) = go(&script("seed(8)\n"));
    assert_eq!(first, again);
    assert_ne!(first, other);
    assert!(first.contains("size 4.5, alpha 1, jitter 0.4"), "{first}");
    // Without a seed the engine chooses one at the layer that draws and says which.
    let (code, unseeded, stderr) = go(&script(""));
    assert_eq!(code, 0, "{stderr}");
    assert!(
        stderr.starts_with("note: no seed was set, so ")
            && stderr.ends_with("at the top of the script repeats it (line 2)\n"),
        "{stderr}"
    );
    // That seed, written at the top as the note advises, repeats the run (step 20a).
    let chosen: String = stderr["note: no seed was set, so ".len()..]
        .chars()
        .take_while(char::is_ascii_digit)
        .collect();
    let (code, repeated, stderr) = go(&script(&format!("seed({chosen})\n")));
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(
        unseeded, repeated,
        "seed({chosen}) did not repeat the jitter"
    );
    // A layer without jitter draws nothing from the stream, so no seed is chosen or named.
    let (_, _, stderr) = go(
        "let d = frame(a = [1, 2], b = [1, 2])\nd.plot(x: a, y: b).points()\nd.plot(x: a, y: b).points(jitter: 0)\n",
    );
    assert_eq!(stderr, "");
}

#[test]
fn every_message() {
    let mut all = String::new();
    for script in [
        "let w = dataset(wages)\nw.plot(x: industry, y: wage)\n",
        "let w = dataset(wages)\nw.plot(x: educ, y: region)\n",
        "let w = dataset(wages)\nw.plot(x: educ, y: wages)\n",
        "let w = dataset(wages)\nw.plot(x: Educ, y: wage)\n",
        "let w = dataset(wages)\nw.plot(x: educ).points()\n",
        "let w = dataset(wages)\nw.plot(y: wage).points()\n",
        "let w = dataset(wages)\nw.plot().points()\n",
        "let w = dataset(wages)\nw.plot(x: educ, y: wage).points(alpha: 1.5)\n",
        "let w = dataset(wages)\nw.plot(x: educ, y: wage).points(alpha: -0.1)\n",
        "let w = dataset(wages)\nw.plot(x: educ, y: wage).points(size: 0)\n",
        "let w = dataset(wages)\nw.plot(x: educ, y: wage).points(size: 1 / 0)\n",
        "let w = dataset(wages)\nw.plot(x: educ, y: wage).points(jitter: -1)\n",
        "let w = dataset(wages)\nw.plot(x: educ, y: wage).points(jitter: 1 / 0)\n",
        "let w = dataset(wages)\nlet d = frame(educ = [1, 2], pay = [3.0, 4.0])\nw.plot(x: educ, y: wage).points(data: d)\n",
        "let w = dataset(wages)\nlet d = frame(educ = [1, 2], Wage = [3.0, 4.0])\nw.plot(x: educ, y: wage).points(data: d)\n",
        "let w = dataset(wages)\nlet d = frame(educ = [1, 2], wage = [\"a\", \"b\"])\nw.plot(x: educ, y: wage).points(data: d)\n",
        "let w = dataset(wages)\nw.plot(x: educ, y: wage).points(data: frame(educ = [\"a\"], wage = [1.0]))\n",
        "let w = dataset(wages)\npoints(w)\n",
        "let w = dataset(wages)\nw.plot(x: educ, y: wage).points(data: 3)\n",
        "let w = dataset(wages)\nw.plot(x: educ, y: wage, colour: female)\n",
    ] {
        let (code, stdout, stderr) = go(script);
        assert_eq!(code, 1, "{script}\n{stdout}");
        assert_eq!(stdout, "", "{script}");
        all.push_str(&stderr);
        all.push('\n');
    }
    insta::assert_snapshot!("messages", all);
}
