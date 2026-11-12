//! `plot` and `points` as a script writes them (Language Reference, section 7.6; release 0.7.0,
//! step 20): the printed form of a plot, which is its scene read as text, in every shape; the
//! notes of the rows not drawn and of the seed; jitter repeated under a seed; and a snapshot of
//! every message. From step 20b, a position mapped from an expression (`y: log(wage)`): what it
//! prints against R's values, the names it reads, the layers that compute it on their own
//! frame, its edge cases and its messages. From step 20c, colour and fill; from step 20d, size,
//! shape, group and label: what a sized and shaped plot prints against R's counts, the edge
//! cases, and a snapshot of every message. From step 20e, an expression given to an aesthetic
//! that takes a variable by its name: the message says so, for each of the six, and a name the
//! frame lacks keeps its own message.

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

// ----- a position mapped from an expression (step 20b) -----

#[test]
fn a_position_mapped_from_an_expression_reads_sensibly_as_text() {
    let text = printed(
        "let w = dataset(wages)\nw.plot(x: educ, y: log(wage)).points()\nw.plot(x: educ, y: wage / hours).points()\nw.plot(x: educ, y: exper^2).points()\nlet top = w.where(wage > 100)\nw.plot(x: educ, y: log( wage )).points(alpha: 0.3).points(data: top, size: 8)\nw.plot(x: log(wage), y: log(wage, base: 10)).points()\nplot(w, x: -educ, y: wage > 20)\n",
    );
    // The reference, from R 4.6.0 on the shipped file (`log`, `/` and `^` of its columns, the
    // range of the finite values and 5% of it beyond each end): log(wage) runs from 1.26130 to
    // 5.02644 over the 18,201 rows complete in educ and wage; wage / hours has 18,167 complete
    // rows and exper^2 18,390; the 26 wages above 100 are all complete.
    for line in [
        "plot · w · log(wage) against educ\n",
        "  y      log(wage)            linear   1.07304 to 5.2147   2, 3, 4, 5\n",
        "      1   points   w      18,201   size 4.5, alpha 1\n",
        "note: 211 rows with NA in educ or log(wage) dropped (line 2)\n",
        "  y      wage / hours         linear   -0.169223 to 5.72676   0, 1, 2, 3, 4, 5\n",
        "      1   points   w      18,167   size 4.5, alpha 1\n",
        "note: 245 rows with NA in educ or wage / hours dropped (line 3)\n",
        "  y      exper^2              linear   -64.8 to 1360.8   0, 200, 400, 600, 800, 1000, 1200\n",
        "      1   points   w      18,390   size 4.5, alpha 1\n",
        "note: 22 rows with NA in educ or exper^2 dropped (line 4)\n",
        "      2   points   top        26   size 8, alpha 1\n",
        "plot · w · log(wage, base: 10) against log(wage)\n",
        "note: 211 rows with NA in log(wage) or log(wage, base: 10) dropped (line 7)\n",
        "plot · w · wage > 20 against -educ\n",
    ] {
        assert!(text.contains(line), "{line}\n{text}");
    }
    insta::assert_snapshot!("expressions", text);
}

/// The printed plot as its words, with `written` read as `name`: what two plots that differ
/// only in how a mapping is written have in common.
fn words(printed: &str, written: &str, name: &str) -> Vec<String> {
    printed
        .replace(written, name)
        .split_whitespace()
        .map(str::to_string)
        .collect()
}

#[test]
fn an_expression_that_changes_no_value_draws_what_its_variable_draws() {
    let frame = "let d = frame(a = [1, 2, 3, 4, 5, 6], b = [0.5, NA, 2.5, 1 / 0, -3.0, 4.25], c = [3, 1, NA, 4, 1, 5])\n";
    let plain = printed(&format!(
        "{frame}d.plot(x: a, y: b)\nd.plot(x: a, y: b).points()\nd.plot(x: c, y: b).points(size: 2)\n"
    ));
    for written in ["b * 1", "b + 0", "b / 1", "-(-b)", "b^1"] {
        let mapped = printed(&format!(
            "{frame}d.plot(x: a, y: {written})\nd.plot(x: a, y: {written}).points()\nd.plot(x: c, y: {written}).points(size: 2)\n"
        ));
        assert_eq!(
            words(&mapped, written, "b"),
            words(&plain, "b", "b"),
            "{written}"
        );
    }
    // On the other axis, and with whole numbers, which stay whole.
    let plain = printed(&format!("{frame}d.plot(x: c, y: a).points()\n"));
    for written in ["c * 1", "c + 0", "c - 0"] {
        let mapped = printed(&format!("{frame}d.plot(x: {written}, y: a).points()\n"));
        assert_eq!(
            words(&mapped, written, "c"),
            words(&plain, "c", "c"),
            "{written}"
        );
    }
    // A name in brackets is still the variable, with its label.
    let (_, stdout, _) = go(&format!(
        "{frame}let l = d.label(b, \"Response\")\nl.plot(x: a, y: (b))\nl.plot(x: a, y: b * 1)\n"
    ));
    assert!(stdout.contains("  y      Response   linear"), "{stdout}");
    assert!(stdout.contains("  y      b * 1   linear"), "{stdout}");
}

#[test]
fn a_constant_of_core_in_an_expression_is_its_value() {
    // `PI` and `E` are read as anywhere else; a variable of the frame with the name hides them.
    let (code, stdout, stderr) = go(
        "let d = frame(r = [1.0, 2.0, 3.0])\nd.plot(x: r, y: r * PI).points()\nlet h = frame(r = [1.0, 2.0, 3.0], PI = [10, 10, 10])\nh.plot(x: r, y: r * PI).points()\n",
    );
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stderr, "");
    assert!(
        stdout.contains("  y      r * PI   linear   2.82743 to 9.73894   3, 4, 5, 6, 7, 8, 9\n"),
        "{stdout}"
    );
    assert!(
        stdout.contains("  y      r * PI   linear   9 to 31"),
        "{stdout}"
    );
}

#[test]
fn a_name_in_an_expression_is_read_where_plot_stands() {
    // A constant of the script is taken with the value it has at the call; a loop variable that
    // holds a variable stands for it, and the plot names the variable.
    let (code, stdout, stderr) = go(
        "let d = frame(a = [1, 2, 3], b = [2.0, 4.0, 8.0], c = [1.0, 10.0, 100.0])\nlet rate = 10\nlet p = d.plot(x: a, y: b * rate)\nlet rate = 1000\np.points()\nfor v in vars(b, c) {\n    print(d.plot(x: a, y: log(v, base: 2) + rate).points())\n}\n",
    );
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(stderr, "");
    assert!(
        stdout.contains("  y      b * rate   linear   17 to 83"),
        "{stdout}"
    );
    assert!(
        stdout.contains("plot · d · log(b, base: 2) + rate against a\n"),
        "{stdout}"
    );
    assert!(
        stdout.contains("  y      log(c, base: 2) + rate   linear"),
        "{stdout}"
    );
    // The variable is named as a script writes it, in backticks where it needs them; a text
    // and the name of an argument are left as they stand.
    let (code, stdout, stderr) = go(
        "let d = import(\"data/awkward_names.csv\")\nfor digits in vars(`Gross Sales (total)`, `日本語`) {\n    print(d.plot(x: round(digits / 2, digits: 1), y: \"digits\" == \"digits\"))\n}\nfor `my var` in vars(`日本語`) {\n    print(d.plot(x: `my var` / 2, y: sqrt(`my var`)))\n}\n",
    );
    assert_eq!(code, 0, "{stderr}");
    for title in [
        "plot · d · \"digits\" == \"digits\" against round(`Gross Sales (total)` / 2, digits: 1)\n",
        "plot · d · \"digits\" == \"digits\" against round(`日本語` / 2, digits: 1)\n",
        "plot · d · sqrt(`日本語`) against `日本語` / 2\n",
    ] {
        assert!(stdout.contains(title), "{title}\n{stdout}");
    }
    // A variable of the frame comes before a binding of the same name, and the script is told.
    let (code, stdout, stderr) = go(
        "let d = frame(a = [1, 2, 3], b = [2.0, 4.0, 8.0])\nlet b = 100\nd.plot(x: a, y: b * 2).points()\n",
    );
    assert_eq!(code, 0, "{stderr}");
    assert!(
        stdout.contains("  y      b * 2   linear   3.4 to 16.6"),
        "{stdout}"
    );
    assert!(
        stderr.starts_with(
            "warning[W0102]: `b` is both a variable of d and a binding; the variable was used (line 3)\n"
        ),
        "{stderr}"
    );
}

#[test]
fn a_layer_with_its_own_frame_computes_the_expression_there() {
    let (code, stdout, stderr) = go(
        "let d = frame(a = [1, 2, 3], b = [1.0, 10.0, 100.0])\nlet e = frame(b = [0.0, -1.0, 1000.0, NA], a = [1, 2, 3, 4])\nlet p = d.plot(x: a, y: log10(b))\np.points(data: e)\np.points().points(data: e)\n",
    );
    assert_eq!(code, 0, "{stderr}");
    // log10 of 1000 is 3: the one row of `e` that is drawn, alone on its axes in the first plot
    // and beside the three of `d` in the second.
    assert!(
        stdout.contains("  y      log10(b)   linear   2.5 to 3.5 "),
        "{stdout}"
    );
    assert!(
        stdout.contains("      1   points   e          1   "),
        "{stdout}"
    );
    assert!(
        stdout.contains("  y      log10(b)   linear   -0.15 to 3.15 "),
        "{stdout}"
    );
    assert!(
        stdout.contains("      2   points   e          1   "),
        "{stdout}"
    );
    // The warnings of computing the expression are raised where it is computed: none on `d`,
    // and on `e` once for each layer that reads it, with the rows it leaves out.
    insta::assert_snapshot!("expression_own_frame", format!("{stdout}---\n{stderr}"));
}

#[test]
fn the_edge_cases_of_an_expression() {
    let mut all = String::new();
    for script in [
        // A constant, alone and from numbers; one that is `NA` on every row.
        "let d = frame(a = [1, 2, 3], b = [0.5, NA, 2.5])\nd.plot(x: a, y: 0).points()\nd.plot(x: a, y: 2 + 3).points()\nd.plot(x: a * 0, y: b).points()\nd.plot(x: a, y: len(b)).points()\n",
        "let d = frame(a = [1, 2, 3], b = [0.5, NA, 2.5])\nd.plot(x: a, y: NA + 1).points()\nd.plot(x: a, y: b * NA).points()\nd.plot(x: a, y: sqrt(-a)).points()\n",
        // The rows that give no value or an infinite one, each warned of once and counted.
        "let d = frame(a = [1, 2, 3, 4], b = [0.0, -1.0, 5.0, NA])\nlet p = d.plot(x: a, y: log(b))\np\np.points()\nd.plot(x: a / (a - 2), y: b).points()\n",
        // A frame with no rows: nothing to compute, and the axes of nothing.
        "let d = frame(a = [1, 2, 3], b = [0.5, NA, 2.5]).where(a > 5)\nd.plot(x: a * 2, y: log(b)).points()\n",
        // Truth values and whole numbers.
        "let d = frame(a = [1, 2, 3], b = [0.5, NA, 2.5])\nd.plot(x: a, y: b > 1).points()\nd.plot(x: a, y: is_missing(b)).points()\nd.plot(x: div(a, 2), y: mod(a, 2)).points()\n",
        // The edge-case datasets of the testing handbook, section 1.2.
        "let d = import(\"data/one_row.csv\")\nd.plot(x: x * 2, y: -y).points()\n",
        "seed(5)\nlet d = import(\"data/zero_variance.csv\")\nd.plot(x: constant - constant, y: big_constant * 2).points()\nd.plot(x: constant_with_na / constant, y: constant).points(jitter: 0.3)\n",
        "let d = import(\"data/infinities.csv\")\nd.plot(x: finite, y: plus - plus).points()\nd.plot(x: abs(both), y: nan_mixed * 0).points()\n",
        "let d = import(\"data/magnitudes.csv\")\nd.plot(x: offset - 1e9, y: log10(wide)).points()\nd.plot(x: tiny * tiny, y: wide * wide).points()\n",
        "let d = import(\"data/bools.csv\")\nd.plot(x: treated && with_na, y: !treated).points()\n",
        "let d = import(\"data/empty.csv\")\nd.plot(x: x + 1, y: log(y)).points()\n",
        "let d = import(\"data/all_na.csv\")\nd.plot(x: x + 1, y: log(y)).points()\n",
        "let d = import(\"data/awkward_names.csv\")\nd.plot(x: `Gross Sales (total)` / 1000, y: sqrt(`日本語`)).points()\n",
        "let d = import(\"data/wages_large.csv\")\nd.plot(x: educ + hours / 10, y: log(wage)).points()\n",
    ] {
        let (code, stdout, stderr) = go(script);
        all.push_str(&format!(
            "{script}exit code {code}\n{stdout}---\n{stderr}\n"
        ));
    }
    insta::assert_snapshot!("expression_edges", all);
}

#[test]
fn every_message_of_an_expression() {
    let mut all = String::new();
    for script in [
        // What a mapping cannot hold.
        "let w = dataset(wages)\nw.plot(x: educ, y: mean(wage))\n",
        "let w = dataset(wages)\nw.plot(x: educ, y: wage - w.mean(wage))\n",
        "let w = dataset(wages)\nw.plot(x: upper(industry), y: wage)\n",
        "let w = dataset(wages)\nw.plot(x: educ, y: case { wage > 10 => 1, _ => 0 })\n",
        "let w = dataset(wages)\nw.plot(x: educ, y: if true { wage } else { educ })\n",
        "let w = dataset(wages)\nw.plot(x: educ, y: w[\"wage\"])\n",
        "let w = dataset(wages)\nw.plot(x: educ, y: wage..hours)\n",
        "let w = dataset(wages)\nw.plot(x: educ, y: seed(3))\n",
        "let w = dataset(wages)\nw.plot(x: educ, y: logit(wage / 200))\n",
        "let w = dataset(wages)\n/// Twice a number.\nfn twice(x: Float) -> Float { x * 2 }\nw.plot(x: educ, y: twice(wage))\n",
        // What it does not come to.
        "let w = dataset(wages)\nw.plot(x: educ, y: \"a\")\n",
        "let w = dataset(wages)\nw.plot(x: educ, y: [1, 2])\n",
        "let w = dataset(wages)\nw.plot(x: category(educ), y: wage)\n",
        // The operators' own refusals, underlined at the mapping.
        "let w = dataset(wages)\nw.plot(x: educ, y: industry + 1)\n",
        "let w = dataset(wages)\nw.plot(x: educ, y: !wage)\n",
        "let w = dataset(wages)\nw.plot(x: educ, y: wage == \"a\")\n",
        // The names.
        "let w = dataset(wages)\nw.plot(x: educ, y: log(wages))\n",
        "let w = dataset(wages)\nw.plot(x: educ, y: log(Wage))\n",
        "let w = dataset(wages)\nfor wage in [1, 2] {\n    w.plot(x: educ, y: wage * 2)\n}\n",
        "let w = dataset(wages)\nlet other = frame(a = [1, 2])\nw.plot(x: educ, y: wage * other)\n",
        "let w = dataset(wages)\nfor v in vars(pay) {\n    w.plot(x: educ, y: log(v))\n}\n",
        // A layer whose frame cannot give the expression.
        "let w = dataset(wages)\nlet d = frame(educ = [1, 2], pay = [3.0, 4.0])\nw.plot(x: educ, y: log(wage)).points(data: d)\n",
        "let w = dataset(wages)\nlet d = frame(educ = [1, 2], Wage = [3.0, 4.0])\nw.plot(x: educ, y: log(wage)).points(data: d)\n",
        "let w = dataset(wages)\nlet d = frame(educ = [1, 2], wage = [\"a\", \"b\"])\nw.plot(x: educ, y: abs(wage)).points(data: d)\n",
        "let w = dataset(wages)\nlet d = frame(educ = [1, 2], wage = [\"a\", \"b\"])\nw.plot(x: educ, y: wage * 2).points(data: d)\n",
        "let w = dataset(wages)\nw.plot(x: educ, y: wage * 2).points(data: frame(educ = [\"a\"], wage = [1.0]))\n",
    ] {
        let (code, stdout, stderr) = go(script);
        assert_eq!(code, 1, "{script}\n{stdout}\n{stderr}");
        assert_eq!(stdout, "", "{script}");
        all.push_str(script);
        all.push_str(&stderr);
        all.push('\n');
    }
    insta::assert_snapshot!("expression_messages", all);
}

// ----- colour and fill (step 20c) -----

#[test]
fn a_coloured_plot_reads_sensibly_as_text() {
    let text = printed(
        "let w = dataset(wages)\nw.plot(x: educ, y: wage, color: female)\nw.plot(x: educ, y: wage, color: female).points()\nw.plot(x: educ, y: wage, color: \"steelblue\").points()\nw.plot(x: educ, y: wage, color: exper, fill: \"#48b\").points(alpha: 0.3)\nw.plot(x: educ, y: wage, color: industry).points()\nlet top = w.where(wage > 100)\nw.plot(x: educ, y: wage, color: industry).points(data: top)\nw.plot(x: educ, y: wage, color: exper).points(data: top)\nw.plot(x: educ, y: wage, color: union, fill: female).points().points(data: top, size: 8)\n",
    );
    // The reference, from R on the shipped file: of the 18,412 rows, 18,201 are complete in
    // educ and wage, 9,375 of them men and 8,826 women, and 18,179 in exper as well, where
    // exper runs from 0 to 36; they hold all 12 industries. The 26 rows with a wage above 100
    // hold 8 industries (Construction, Manufacturing, Retail, Finance, Services and three more,
    // in the order of the levels) and exper from 8 to 30. A table wider than the page is
    // continued below itself, as every table is.
    for line in [
        "  color       Sex     category   male, female   orange, sky blue\n",
        "  color               constant   every mark   steelblue\n",
        "  color       Potential labour-market experience: age less schooling less six   linear     0 to 36\n",
        "  fill                                                                          constant   every mark\n",
        "  shown as\n  viridis, dark purple to yellow\n  #48b\n",
        "no layer draws `fill`: `points` takes its colour from `color`\n",
        "      1   points   w      18,179   size 4.5, alpha 0.3\n",
        "note: 233 rows with NA in educ, wage or exper dropped (line 5)\n",
        "Agriculture, Mining, Utilities, Construction, Manufacturing and 7 more",
        "orange, sky blue, bluish green, yellow, blue and 7 more",
        "note: `industry` has 12 categories and the palette 8 colours, so the colours repeat from the ninth category (line 6)\n",
        "Construction, Manufacturing, Retail, Finance, Services and 3 more",
        "  color       Potential labour-market experience: age less schooling less six   linear   8 to 30\n",
        "      2   points   top        26   size 8, alpha 1\n",
    ] {
        assert!(text.contains(line), "{line}\n{text}");
    }
    insta::assert_snapshot!("colour", text);
}

#[test]
fn the_edge_cases_of_a_colour() {
    const D: &str = "let d = frame(a = [1, 2, 3, 4], b = [1.0, 2.0, 3.0, 4.0], g = [\"b\", NA, \"a\", \"b\"], t = [true, false, NA, true], z = [0.5, NA, 1 / 0, 2.5], k = [3, 3, 3, 3])\n";
    let mut all = String::new();
    for script in [
        // Text and truth values with `NA`, before a layer and under one.
        format!("{D}d.plot(x: a, y: b, color: g)\nd.plot(x: a, y: b, color: g).points()\nd.plot(x: a, y: b, color: t).points()\n"),
        // A number: the rows without a finite value of it are dropped and counted; one value.
        format!("{D}d.plot(x: a, y: b, color: z)\nd.plot(x: a, y: b, color: z).points()\nd.plot(x: a, y: b, color: k).points()\nd.plot(x: a, y: b, color: a).points()\n"),
        // A frame with no rows, and a variable of nothing but `NA`.
        format!("{D}let e = d.where(a > 9)\ne.plot(x: a, y: b, color: g).points()\ne.plot(x: a, y: b, color: z).points()\nlet n = d.where(a == 2)\nn.plot(x: a, y: b, color: g).points()\nn.plot(x: a, y: b, color: z).points()\n"),
        // A category keeps the order of its levels; a level with no row is left out.
        format!("{D}let c = d.add(c = category(g))\nc.plot(x: a, y: b, color: c).points()\nc.where(g != \"a\").plot(x: a, y: b, color: c).points()\n"),
        // Six categories are listed whole, seven are shortened; nine repeat the palette.
        "let d = frame(a = [1, 2, 3, 4, 5, 6, 7, 8, 9], g = [\"a\", \"b\", \"c\", \"d\", \"e\", \"f\", \"g\", \"h\", \"i\"])\nd.where(a < 7).plot(x: a, y: a, color: g).points()\nd.where(a < 8).plot(x: a, y: a, color: g).points()\nd.where(a < 9).plot(x: a, y: a, color: g).points()\nd.plot(x: a, y: a, color: g)\nd.plot(x: a, y: a, color: g).points()\n".to_string(),
        // A layer with its own frame: its categories join the scale after the plot's own; a
        // constant colour asks nothing of that frame.
        format!("{D}let o = frame(a = [5, 6, 7], b = [0.5, 1.5, 2.5], g = [\"zz\", \"b\", NA])\nd.plot(x: a, y: b, color: g).points(data: o)\nd.plot(x: a, y: b, color: g).points().points(data: o)\nlet bare = frame(a = [5, 6], b = [0.5, 1.5])\nd.plot(x: a, y: b, color: \"tomato\", fill: \"#000\").points(data: bare)\nd.plot(x: a, y: b, fill: g).points(data: bare)\n"),
        // Colour and fill of one variable; fill alone; every form of a constant.
        format!("{D}d.plot(x: a, y: b, color: g, fill: g).points()\nd.plot(x: a, y: b, fill: z).points()\nd.plot(x: a, y: b, color: \"#4682B4\").points()\nd.plot(x: a, y: b, color: \"#abc\", fill: \"rebeccapurple\")\nd.plot(color: g)\nd.plot(x: a, color: \"grey\")\nd.plot(x: a, y: b, color: NA, fill: NA).points()\n"),
        // The edge-case datasets of the testing handbook, section 1.2.
        "let d = import(\"data/one_row.csv\")\nd.plot(x: x, y: y, color: x).points()\n".to_string(),
        "let d = import(\"data/zero_variance.csv\")\nd.plot(x: constant, y: big_constant, color: constant_with_na).points()\n".to_string(),
        "let d = import(\"data/infinities.csv\")\nd.plot(x: finite, y: finite, color: plus).points()\nd.plot(x: finite, y: finite, color: nan_mixed).points()\n".to_string(),
        "let d = import(\"data/magnitudes.csv\")\nd.plot(x: offset, y: tiny, color: wide).points()\n".to_string(),
        "let d = import(\"data/bools.csv\")\nd.plot(x: treated, y: treated, color: with_na).points()\n".to_string(),
        "let d = import(\"data/empty.csv\")\nd.plot(color: x)\nd.plot(fill: y)\n".to_string(),
        "let d = import(\"data/all_na.csv\")\nd.plot(color: x)\nd.plot(fill: y)\n".to_string(),
        "let d = import(\"data/awkward_names.csv\")\nd.plot(x: `Gross Sales (total)`, y: `日本語`, color: `Gross Sales (total)`).points()\n".to_string(),
        "let d = import(\"data/unicode_labels.csv\")\nd.plot(x: n, y: n, color: city).points()\n".to_string(),
        "let d = import(\"data/level_symbols.csv\")\nd.plot(x: n, y: y, color: grp).points()\n".to_string(),
        "let d = import(\"data/wages_large.csv\")\nd.plot(x: educ, y: wage, color: female).points()\nd.plot(x: educ, y: wage, color: region).points()\n".to_string(),
    ] {
        let (code, stdout, stderr) = go(&script);
        all.push_str(&format!(
            "{script}exit code {code}\n{stdout}---\n{stderr}\n"
        ));
    }
    insta::assert_snapshot!("colour_edges", all);
}

#[test]
fn every_message_of_a_colour() {
    const O: &str = "let w = dataset(wages)\nlet p = w.plot(x: educ, y: wage, color: exper)\nlet q = w.plot(x: educ, y: wage, fill: female)\n";
    let mut all = String::new();
    for script in [
        // A string that is not a colour.
        "let w = dataset(wages)\nw.plot(x: educ, y: wage, color: \"steel blue\")\n".to_string(),
        "let w = dataset(wages)\nw.plot(x: educ, y: wage, color: \"Red\")\n".to_string(),
        "let w = dataset(wages)\nw.plot(x: educ, y: wage, fill: \"female\")\n".to_string(),
        "let w = dataset(wages)\nw.plot(x: educ, y: wage, fill: \"a colour of my own\")\n".to_string(),
        "let w = dataset(wages)\nw.plot(x: educ, y: wage, color: \"\")\n".to_string(),
        "let w = dataset(wages)\nw.plot(x: educ, y: wage, color: \"#12G\")\n".to_string(),
        "let w = dataset(wages)\nw.plot(x: educ, y: wage, fill: \"#12345\")\n".to_string(),
        "let w = dataset(wages)\nw.plot(x: educ, y: wage, color: \"4682B4\")\n".to_string(),
        // What is neither a variable nor a string.
        "let w = dataset(wages)\nw.plot(x: educ, y: wage, color: 3)\n".to_string(),
        "let w = dataset(wages)\nw.plot(x: educ, y: wage, color: log(exper))\n".to_string(),
        "let w = dataset(wages)\nw.plot(x: educ, y: wage, fill: [\"red\", \"blue\"])\n".to_string(),
        // The names: a variable the frame lacks, and a binding, which a bare name never reads.
        "let w = dataset(wages)\nw.plot(x: educ, y: wage, color: sex)\n".to_string(),
        "let w = dataset(wages)\nw.plot(x: educ, y: wage, fill: Female)\n".to_string(),
        "let w = dataset(wages)\nlet shade = \"red\"\nw.plot(x: educ, y: wage, color: shade)\n".to_string(),
        // A type with no colour scale yet.
        "let d = frame(a = [1, 2], b = [1.0, 2.0], s = [\"2026-01-01\", \"2026-06-01\"]).add(when = date(s))\nd.plot(x: a, y: b, color: when)\n".to_string(),
        "let d = frame(a = [1, 2], b = [1.0, 2.0], s = [\"2026-01-01\", \"2026-06-01\"]).add(when = date(s))\nd.plot(x: a, y: b, fill: when)\n".to_string(),
        // A layer whose frame cannot give the colour.
        format!("{O}p.points(data: frame(educ = [1, 2], wage = [3.0, 4.0]))\n"),
        format!("{O}let o = frame(educ = [1, 2], wage = [3.0, 4.0], Exper = [1, 2])\np.points(data: o)\n"),
        format!("{O}let o = frame(educ = [1, 2], wage = [3.0, 4.0], exper = [\"a\", \"b\"])\np.points(data: o)\n"),
        format!("{O}let o = frame(educ = [1, 2], wage = [3.0, 4.0], exper = [true, false])\np.points(data: o)\n"),
        format!("{O}let o = w.where(wage > 100).select(vars(educ, wage, female)).rename(exper = female)\np.points(data: o)\n"),
        format!("{O}let o = frame(educ = [1, 2], wage = [3.0, 4.0], s = [\"2026-01-01\", \"2026-06-01\"]).add(exper = date(s))\np.points(data: o)\n"),
        "let w = dataset(wages)\nlet o = frame(educ = [1, 2], wage = [3.0, 4.0], female = [0, 1])\nw.plot(x: educ, y: wage, color: female).points(data: o)\n".to_string(),
        "let w = dataset(wages)\nlet o = frame(educ = [1, 2], wage = [3.0, 4.0], industry = [0.5, 1.5])\nw.plot(x: educ, y: wage, color: industry).points(data: o)\n".to_string(),
    ] {
        let (code, stdout, stderr) = go(&script);
        assert_eq!(code, 1, "{script}\n{stdout}\n{stderr}");
        assert_eq!(stdout, "", "{script}");
        all.push_str(&script);
        all.push_str(&stderr);
        all.push('\n');
    }
    insta::assert_snapshot!("colour_messages", all);
}

/// A layer drawn from a frame that lacks a mapped fill is not refused: no point reads `fill`.
#[test]
fn a_layer_is_asked_only_for_what_its_mark_reads() {
    let (code, stdout, stderr) = go(
        "let w = dataset(wages)\nlet o = frame(educ = [10, 12], wage = [30.0, 40.0])\nw.plot(x: educ, y: wage, fill: female).points(data: o)\n",
    );
    assert_eq!(code, 0, "{stderr}");
    assert!(
        stdout.contains("      1   points   o          2   size 4.5, alpha 1\n"),
        "{stdout}"
    );
    assert_eq!(stderr, "");
}

// ----- size, shape, group and label (step 20d) -----

#[test]
fn a_sized_and_shaped_plot_reads_sensibly_as_text() {
    let text = printed(
        "let w = dataset(wages)\nw.plot(x: educ, y: wage, size: hours)\nw.plot(x: educ, y: wage, size: hours).points()\nw.plot(x: educ, y: wage, size: exper).points(alpha: 0.3)\nw.plot(x: educ, y: wage, size: 3, shape: \"cross\").points()\nw.plot(x: educ, y: wage, shape: union).points()\nw.plot(x: educ, y: wage, color: union, size: hours, shape: union).points(alpha: 0.5)\nw.plot(x: educ, y: wage, shape: industry).points()\nlet top = w.where(wage > 100)\nw.plot(x: educ, y: wage, size: hours, shape: union).points(alpha: 0.2).points(data: top, size: 8)\nw.plot(x: educ, y: wage, size: hours, shape: union).points(data: top)\nw.plot(x: educ, y: wage, group: worker, label: firm).points()\n",
    );
    // The reference, from R on the shipped file: of the 18,412 rows, 18,201 are complete in
    // educ and wage; 18,167 of them in hours as well (245 dropped), where hours runs from 7 to
    // 69.2 and is never 0; and 18,179 in exper (233 dropped), where exper runs from 0 to 36 and
    // is 0 in 340 rows. Union membership has two categories and industry twelve. The 26 rows
    // with a wage above 100 all have their hours, the largest 59.5.
    for line in [
        "  size        Hours worked   sqrt    0 to 69.2   by area, up to 18 across\n",
        "no layer yet: a mark such as `.points()` adds one\n",
        "      1   points   w      18,167   size by hours, alpha 1\n",
        "note: 245 rows with NA in educ, wage or hours dropped (line 3)\n",
        "  size        Potential labour-market experience: age less schooling less six   sqrt    0 to 36\n",
        "  shown as\n  by area, up to 18 across\n",
        "      1   points   w      18,179   size by exper, alpha 0.3\n",
        "note: 233 rows with NA in educ, wage or exper dropped (line 4)\n",
        "note: 340 rows with exper of 0 are drawn with no size (line 4)\n",
        "  size                constant   every mark   3\n",
        "  shape               constant   every mark   cross\n",
        "      1   points   w      18,201   size 3, shape cross, alpha 1\n",
        "  shape       Union member   category   no, yes   circle, triangle\n",
        "      1   points   w      18,201   size 4.5, shape by union, alpha 1\n",
        "  color       Union member   category   no, yes     orange, sky blue\n",
        "      1   points   w      18,167   size by hours, shape by union, alpha 0.5\n",
        "Agriculture, Mining, Utilities, Construction, Manufacturing and 7 more",
        "  circle, triangle, square, diamond, plus and 7 more\n",
        "note: `industry` has 12 categories and there are 8 shapes, so the shapes repeat from the ninth category (line 8)\n",
        "      2   points   top        26   size 8, shape by union, alpha 1\n",
        "  size        Hours worked   sqrt       0 to 59.5   by area, up to 18 across\n",
        "      1   points   top       26   size by hours, shape by union, alpha 1\n",
        "  group       Worker identifier           each value a group\n",
        "  label       Firm identifier             the text of a mark\n",
        "no layer draws `group`: `points` draws each row by itself\n",
        "no layer draws `label`: `points` writes no text\n",
        "note: 211 rows with NA in educ or wage dropped (line 12)\n",
    ] {
        assert!(text.contains(line), "{line}\n{text}");
    }
    // Sizing by a variable whose largest rows are dropped says so once for each statement.
    assert_eq!(
        text.matches("with NA in educ, wage or hours dropped")
            .count(),
        3
    );
    insta::assert_snapshot!("size_shape", text);
}

#[test]
fn the_edge_cases_of_a_size_and_a_shape() {
    const D: &str = "let d = frame(a = [1, 2, 3, 4], b = [1.0, 2.0, 3.0, 4.0], g = [\"b\", NA, \"a\", \"b\"], t = [true, false, NA, true], z = [0.5, NA, 1 / 0, 2.5], k = [3, 3, 3, 3], n = [0, -1, 2, 0])\n";
    let mut all = String::new();
    for script in [
        // A size: the rows without a finite value of it are dropped and counted, before a layer
        // nothing is; one value; whole numbers with zeros and one below 0.
        format!("{D}d.plot(x: a, y: b, size: z)\nd.plot(x: a, y: b, size: z).points()\nd.plot(x: a, y: b, size: k).points()\nd.plot(x: a, y: b, size: n)\nd.plot(x: a, y: b, size: n).points()\n"),
        // Nothing above 0 to size: every size 0, every size below 0, one row of 0.
        "let d = frame(a = [1, 2], s = [0, 0], m = [-1.5, -2.5])\nd.plot(x: a, y: a, size: s).points()\nd.plot(x: a, y: a, size: m).points()\nd.where(a == 1).plot(x: a, y: a, size: s).points()\n".to_string(),
        // A shape: text and truth values with `NA`, before a layer and under one.
        format!("{D}d.plot(x: a, y: b, shape: g)\nd.plot(x: a, y: b, shape: g).points()\nd.plot(x: a, y: b, shape: t).points()\n"),
        // A frame with no rows, and variables of nothing but `NA`.
        format!("{D}let e = d.where(a > 9)\ne.plot(x: a, y: b, size: z, shape: g).points()\nlet n = d.where(a == 2)\nn.plot(x: a, y: b, size: z, shape: g)\nn.plot(x: a, y: b, size: z, shape: g).points()\n"),
        // A category keeps the order of its levels; a level with no row is left out.
        format!("{D}let c = d.add(c = category(g))\nc.plot(x: a, y: b, shape: c).points()\nc.where(g != \"a\").plot(x: a, y: b, shape: c).points()\n"),
        // Eight categories have a shape each; nine repeat the shapes, and a layer says so.
        "let d = frame(a = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10], g = [\"a\", \"b\", \"c\", \"d\", \"e\", \"f\", \"g\", \"h\", \"i\", NA])\nd.where(a < 9).plot(x: a, y: a, shape: g).points()\nd.where(a < 10).plot(x: a, y: a, shape: g).points()\nd.where(a != 9).plot(x: a, y: a, shape: g).points()\nd.plot(x: a, y: a, shape: g)\nd.plot(x: a, y: a, shape: g).points()\n".to_string(),
        // A layer with its own frame: its sizes and its categories join the scales of the
        // plot; a constant asks nothing of that frame, and neither do `group` and `label`.
        format!("{D}let o = frame(a = [5, 6, 7], b = [0.5, 1.5, 2.5], g = [\"zz\", \"b\", NA], z = [10, 0, 2])\nd.plot(x: a, y: b, size: z, shape: g).points(data: o)\nd.plot(x: a, y: b, size: z, shape: g).points().points(data: o)\nlet bare = frame(a = [5, 6], b = [0.5, 1.5])\nd.plot(x: a, y: b, size: 7, shape: \"diamond_open\", group: g, label: t).points(data: bare)\nd.plot(x: a, y: b, size: z).points(data: bare, size: 2)\n"),
        // A constant beside a mapping, and every way a layer's own size meets the plot's.
        format!("{D}d.plot(x: a, y: b, size: 6, shape: g).points()\nd.plot(x: a, y: b, size: z, shape: \"square\", color: \"tomato\").points()\nd.plot(x: a, y: b, size: 6).points(size: 2)\nd.plot(x: a, y: b, size: z).points(size: 2)\nd.plot(x: a, y: b, size: z).points(size: 2).points()\nd.plot(x: a, y: b, size: 0.5, shape: \"circle\").points(alpha: 0.5, jitter: 0)\nd.plot(x: a, y: b, size: 2.25, shape: \"asterisk\")\n"),
        // `NA` for an aesthetic is the aesthetic left out.
        format!("{D}d.plot(x: a, y: b, size: NA, shape: NA, group: NA, label: NA).points()\n"),
        // One variable on two scales, which one legend shows; two variables.
        format!("{D}d.plot(x: a, y: b, color: g, shape: g).points()\nd.plot(x: a, y: b, color: k, size: k).points()\nd.plot(x: a, y: b, fill: g, shape: g).points()\nd.plot(x: a, y: b, color: t, fill: \"grey\", size: n, shape: g, group: g, label: a).points()\n"),
        // Group and label by themselves, and with one position or none.
        format!("{D}d.plot(group: g)\nd.plot(x: a, label: z)\nd.plot(x: a, y: b, group: z, label: g).points()\nd.plot(size: k, shape: t)\nd.plot(size: 3, shape: \"plus\")\n"),
        // A whole number is a size; with value labels it is a shape.
        "let d = frame(a = [1, 2, 3], b = [3, 0, 12]).label(a, [1 => \"low\", 2 => \"middle\", 3 => \"high\"])\nd.plot(x: a, y: b, size: b, shape: a).points()\n".to_string(),
        // The edge-case datasets of the testing handbook, section 1.2.
        "let d = import(\"data/one_row.csv\")\nd.plot(x: x, y: y, size: x).points()\n".to_string(),
        "let d = import(\"data/zero_variance.csv\")\nd.plot(x: constant, y: big_constant, size: constant_with_na).points()\nd.plot(x: constant, y: constant, size: big_constant).points()\n".to_string(),
        "let d = import(\"data/infinities.csv\")\nd.plot(x: finite, y: finite, size: plus).points()\nd.plot(x: finite, y: finite, size: both).points()\nd.plot(x: finite, y: finite, size: nan_mixed).points()\n".to_string(),
        "let d = import(\"data/magnitudes.csv\")\nd.plot(x: offset, y: tiny, size: wide).points()\nd.plot(x: offset, y: wide, size: tiny).points()\n".to_string(),
        "let d = import(\"data/bools.csv\")\nd.plot(x: treated, y: treated, shape: with_na).points()\n".to_string(),
        "let d = import(\"data/empty.csv\")\nd.plot(shape: x)\nd.plot(group: x, label: y)\nd.plot(size: x)\n".to_string(),
        "let d = import(\"data/all_na.csv\")\nd.plot(shape: x)\nd.plot(group: x, label: y)\nd.plot(size: x)\n".to_string(),
        "let d = import(\"data/awkward_names.csv\")\nd.plot(x: `Gross Sales (total)`, y: `日本語`, size: `Gross Sales (total)`, label: `日本語`).points()\n".to_string(),
        "let d = import(\"data/unicode_labels.csv\")\nd.plot(x: n, y: n, size: n, shape: city, label: city).points()\n".to_string(),
        "let d = import(\"data/level_symbols.csv\")\nd.plot(x: n, y: y, shape: grp, group: grp).points()\n".to_string(),
        "let d = import(\"data/wages_large.csv\")\nd.plot(x: educ, y: wage, size: hours, shape: region).points()\n".to_string(),
    ] {
        let (code, stdout, stderr) = go(&script);
        all.push_str(&format!(
            "{script}exit code {code}\n{stdout}---\n{stderr}\n"
        ));
    }
    insta::assert_snapshot!("size_shape_edges", all);
}

#[test]
fn every_message_of_a_size_and_a_shape() {
    const W: &str = "let w = dataset(wages)\n";
    const O: &str = "let w = dataset(wages)\nlet p = w.plot(x: educ, y: wage, size: hours)\nlet q = w.plot(x: educ, y: wage, shape: union)\n";
    let mut all = String::new();
    for script in [
        // A string that is not the name of a shape.
        format!("{W}w.plot(x: educ, y: wage, shape: \"Circle\")\n"),
        format!("{W}w.plot(x: educ, y: wage, shape: \"triangel\")\n"),
        format!("{W}w.plot(x: educ, y: wage, shape: \"circle open\")\n"),
        format!("{W}w.plot(x: educ, y: wage, shape: \"union\")\n"),
        format!("{W}w.plot(x: educ, y: wage, shape: \"a shape of my own\")\n"),
        format!("{W}w.plot(x: educ, y: wage, shape: \"\")\n"),
        // What is neither a variable nor a string.
        format!("{W}w.plot(x: educ, y: wage, shape: 3)\n"),
        format!("{W}w.plot(x: educ, y: wage, shape: [\"circle\", \"cross\"])\n"),
        // A size that is not above 0, or is not a number.
        format!("{W}w.plot(x: educ, y: wage, size: 0)\n"),
        format!("{W}w.plot(x: educ, y: wage, size: -2.5)\n"),
        format!("{W}w.plot(x: educ, y: wage, size: 1 / 0)\n"),
        format!("{W}w.plot(x: educ, y: wage, size: \"large\")\n"),
        format!("{W}w.plot(x: educ, y: wage, size: [1, 2])\n"),
        format!("{W}w.plot(x: educ, y: wage, size: sqrt(hours))\n"),
        // A variable that is not an amount, for a size.
        format!("{W}w.plot(x: educ, y: wage, size: female)\n"),
        format!("{W}w.plot(x: educ, y: wage, size: industry)\n"),
        "let d = frame(a = [1, 2], b = [1.0, 2.0], s = [\"u\", \"v\"], t = [true, false])\nd.plot(x: a, y: b, size: s)\n".to_string(),
        "let d = frame(a = [1, 2], b = [1.0, 2.0], s = [\"u\", \"v\"], t = [true, false])\nd.plot(x: a, y: b, size: t)\n".to_string(),
        "let d = frame(a = [1, 2], b = [1.0, 2.0], s = [\"2026-01-01\", \"2026-06-01\"]).add(when = date(s))\nd.plot(x: a, y: b, size: when)\n".to_string(),
        // A variable that is not categories, for a shape.
        format!("{W}w.plot(x: educ, y: wage, shape: hours)\n"),
        format!("{W}w.plot(x: educ, y: wage, shape: educ)\n"),
        "let d = frame(a = [1, 2], b = [1.0, 2.0], s = [\"2026-01-01\", \"2026-06-01\"]).add(when = date(s))\nd.plot(x: a, y: b, shape: when)\n".to_string(),
        // The names: a variable the frame lacks, and a binding, which a bare name never reads.
        format!("{W}w.plot(x: educ, y: wage, size: Hours)\n"),
        format!("{W}w.plot(x: educ, y: wage, shape: sex)\n"),
        format!("{W}w.plot(x: educ, y: wage, group: person)\n"),
        format!("{W}w.plot(x: educ, y: wage, label: Firm)\n"),
        format!("{W}let s = 6\nw.plot(x: educ, y: wage, size: s)\n"),
        format!("{W}let mark = \"cross\"\nw.plot(x: educ, y: wage, shape: mark)\n"),
        // A group and a label are variables.
        format!("{W}w.plot(x: educ, y: wage, group: \"worker\")\n"),
        format!("{W}w.plot(x: educ, y: wage, label: 3)\n"),
        format!("{W}w.plot(x: educ, y: wage, group: floor(exper / 10))\n"),
        // A layer whose frame cannot give the size or the shape.
        format!("{O}p.points(data: frame(educ = [1, 2], wage = [3.0, 4.0]))\n"),
        format!("{O}let o = frame(educ = [1, 2], wage = [3.0, 4.0], Hours = [1, 2])\np.points(data: o)\n"),
        format!("{O}let o = frame(educ = [1, 2], wage = [3.0, 4.0], hours = [\"a\", \"b\"])\np.points(data: o)\n"),
        format!("{O}let o = w.where(wage > 100).select(vars(educ, wage, female)).rename(hours = female)\np.points(data: o)\n"),
        format!("{O}let o = frame(educ = [1, 2], wage = [3.0, 4.0], s = [\"2026-01-01\", \"2026-06-01\"]).add(hours = date(s))\np.points(data: o)\n"),
        format!("{O}q.points(data: frame(educ = [1, 2], wage = [3.0, 4.0]))\n"),
        format!("{O}let o = frame(educ = [1, 2], wage = [3.0, 4.0], union = [0, 1])\nq.points(data: o)\n"),
        format!("{O}let o = frame(educ = [1, 2], wage = [3.0, 4.0], union = [0.5, 1.5])\nq.points(data: o, size: 3)\n"),
        // The options of `points`: a size that is not above 0. A shape and a size by a
        // variable, which `points` refused as unknown until open decision #106 was taken, are
        // a layer's own now (step 23a), and the tests of that step draw them.
        format!("{O}p.points(size: 0)\n"),
    ] {
        let (code, stdout, stderr) = go(&script);
        assert_eq!(code, 1, "{script}\n{stdout}\n{stderr}");
        assert_eq!(stdout, "", "{script}");
        all.push_str(&script);
        all.push_str(&stderr);
        all.push('\n');
    }
    insta::assert_snapshot!("size_shape_messages", all);
}

/// A layer is asked for a size only when it reads the plot's: one with a `size:` of its own is
/// drawn from a frame that lacks the variable, and no layer is asked for a group or a label.
#[test]
fn a_layer_with_its_own_size_is_not_asked_for_the_size_of_the_plot() {
    let (code, stdout, stderr) = go(
        "let w = dataset(wages)\nlet o = frame(educ = [10, 12], wage = [30.0, 40.0])\nw.plot(x: educ, y: wage, size: hours, group: worker, label: firm).points(data: o, size: 3)\n",
    );
    assert_eq!(code, 0, "{stderr}");
    assert!(
        stdout.contains("      1   points   o          2   size 3, alpha 1\n"),
        "{stdout}"
    );
    assert!(
        stdout.contains("no layer draws `size`: every layer sets a `size:` of its own\n"),
        "{stdout}"
    );
    assert_eq!(stderr, "");
}

/// A number bound to a name is not a size: a bare name is a variable of the frame, and a
/// constant is a constant only where it is written.
#[test]
fn a_bare_name_for_a_size_is_a_variable_whatever_is_bound_to_it() {
    let (code, stdout, stderr) = go(
        "let d = frame(a = [1, 2, 3], b = [1.0, 2.0, 3.0], s = [10, 20, 40])\nlet s = 6\nd.plot(x: a, y: b, size: s).points()\n",
    );
    assert_eq!(code, 0, "{stderr}");
    assert!(
        stdout.contains("  size        s       sqrt    0 to 40   by area, up to 18 across\n"),
        "{stdout}"
    );
    assert!(
        stdout.contains("      1   points   d          3   size by s, alpha 1\n"),
        "{stdout}"
    );
}

// ----- an expression where an aesthetic takes a variable (step 20e) -----

#[test]
fn no_aesthetic_answers_an_expression_with_a_message_about_bindings() {
    const W: &str = "let w = dataset(wages)\n";
    let mut all = String::new();
    for statement in [
        // The owner's case, and the same in each aesthetic: an operator, a sign, a function.
        "w.plot(x: educ, y: wage, color: wage > 20)",
        "w.plot(x: educ, y: wage, fill: wage > 20)",
        "w.plot(x: educ, y: wage, size: hours * 2)",
        "w.plot(x: educ, y: wage, shape: union == 1)",
        "w.plot(x: educ, y: wage, group: -educ)",
        "w.plot(x: educ, y: wage, label: upper(industry))",
        "w.plot(x: educ, y: wage, color: !female)",
        "w.plot(x: educ, y: wage, color: log(wage))",
        "w.plot(x: educ, y: wage, fill: industry == \"Retail\")",
        "w.plot(x: educ, y: wage, size: sqrt(hours))",
        "w.plot(x: educ, y: wage, shape: wage > 20 && union == 1)",
        "w.plot(x: educ, y: wage, group: floor(exper / 10))",
        "w.plot(x: educ, y: wage, label: industry + \"!\")",
        // In brackets, alone in the call, with a binding beside the variable, and with a name
        // the frame lacks: the mistake is still the expression.
        "w.plot(x: educ, y: wage, color: (wage > 20))",
        "w.plot(color: wage > 20)",
        "let cut = 20\nw.plot(x: educ, y: wage, color: wage > cut)",
        "w.plot(x: educ, y: wage, color: wagee > 20)",
        // A receiver that is no single name: the advice does not repeat it.
        "dataset(wages).where(year == 2010).plot(x: educ, y: wage, color: wage > 20)",
    ] {
        let script = format!("{W}{statement}\n");
        let (code, stdout, stderr) = go(&script);
        assert_eq!(code, 1, "{script}\n{stdout}\n{stderr}");
        assert_eq!(stdout, "", "{script}");
        assert!(stderr.starts_with("error[Z0401]: `"), "{script}\n{stderr}");
        assert!(
            stderr.contains("not an expression") && stderr.contains("compute it first"),
            "{script}\n{stderr}"
        );
        assert!(
            !stderr.contains("binding") && !stderr.contains("Z0101"),
            "{script}\n{stderr}"
        );
        all.push_str(statement);
        all.push('\n');
        all.push_str(&stderr);
        all.push('\n');
    }
    // A name the frame lacks, given alone, keeps its own message and its suggestion.
    for (statement, near) in [
        (
            "w.plot(x: educ, y: wage, color: wagee)",
            "did you mean `wage`?",
        ),
        (
            "w.plot(x: educ, y: wage, size: hour)",
            "did you mean `hours`?",
        ),
        (
            "w.plot(x: educ, y: wage, shape: unoin)",
            "did you mean `union`?",
        ),
        (
            "w.plot(x: educ, y: wage, group: industy)",
            "did you mean `industry`?",
        ),
        (
            "w.plot(x: educ, y: wage, label: industri)",
            "did you mean `industry`?",
        ),
        (
            "w.plot(x: educ, y: wage, fill: femal)",
            "did you mean `female`?",
        ),
    ] {
        let script = format!("{W}{statement}\n");
        let (code, _, stderr) = go(&script);
        assert_eq!(code, 1, "{script}");
        assert!(
            stderr.starts_with("error[Z0101]: no variable `"),
            "{stderr}"
        );
        assert!(stderr.contains(near), "{script}\n{stderr}");
        all.push_str(statement);
        all.push('\n');
        all.push_str(&stderr);
        all.push('\n');
    }
    insta::assert_snapshot!("aesthetic_expression_messages", all);
    // What is no such mistake still draws: a number worked out from numbers and bindings, a
    // variable in brackets, and an expression on an axis, which takes one.
    for statement in [
        "w.plot(x: educ, y: wage, size: 2 + 1)",
        "let k = 2\nw.plot(x: educ, y: wage, size: k * 1.5)",
        "w.plot(x: educ, y: wage, color: (union))",
        "w.plot(x: educ, y: log(wage), color: union)",
    ] {
        let (code, stdout, stderr) = go(&format!("{W}{statement}\n"));
        assert_eq!(code, 0, "{statement}\n{stderr}");
        assert!(stdout.starts_with("plot · w · "), "{statement}\n{stdout}");
    }
}

// ----- the seventh and eighth shapes (open decision #110, question 2) -----

#[test]
fn the_seventh_and_eighth_shapes_are_given_out_and_may_be_named() {
    // Eight categories take the eight shapes in their order, and nothing repeats.
    let (code, stdout, stderr) = go(
        "let e = frame(a = [1, 2, 3, 4, 5, 6, 7, 8], g = [\"a\", \"b\", \"c\", \"d\", \"e\", \"f\", \"g\", \"h\"])\ne.plot(x: a, y: a, shape: g).points()\ne.where(a > 6).plot(x: a, y: a, shape: g).points()\ne.where(a > 2).add(g = category(g, levels: [\"g\", \"h\", \"c\", \"d\", \"e\", \"f\"])).plot(x: a, y: a, shape: g).points()\n",
    );
    assert_eq!(code, 0, "{stderr}");
    assert!(
        !stderr.contains("repeat"),
        "eight categories repeat no shape: {stderr}"
    );
    assert!(stdout.contains("a, b, c, d, e and 3 more"), "{stdout}");
    // A shape goes with the place of a category, not with its name: the first two are a
    // circle and a triangle whichever they are.
    assert!(stdout.contains("g, h     circle, triangle\n"), "{stdout}");
    // Each of the two is a constant shape, solid or as an outline.
    for name in ["triangle_down", "star", "triangle_down_open", "star_open"] {
        let (code, stdout, stderr) = go(&format!(
            "let w = dataset(wages)\nw.plot(x: educ, y: wage, shape: \"{name}\").points()\n"
        ));
        assert_eq!(code, 0, "{name}: {stderr}");
        assert!(
            stdout.contains(&format!("constant   every mark   {name}\n")),
            "{name}: {stdout}"
        );
        assert!(
            stdout.contains(&format!("shape {name}, alpha 1")),
            "{name}: {stdout}"
        );
    }
    // A name written the way of another tool is answered with ours.
    let (code, _, stderr) =
        go("let w = dataset(wages)\nw.plot(x: educ, y: wage, shape: \"triangle down\")\n");
    assert_eq!(code, 1);
    assert!(
        stderr.contains("did you mean \"triangle_down\"?"),
        "{stderr}"
    );
}

// ----- a layer's own mappings (step 23a; open decision #106) -----

/// The wages of 2010, with a second variable on the scale of the wage.
const FITTED: &str =
    "let w = dataset(wages).where(year == 2010)\nlet d = w.add(fitted = 3 + 2 * educ)\n";

#[test]
fn a_layer_that_names_its_own_variables_reads_sensibly_as_text() {
    let script = format!(
        "{FITTED}d.plot(x: educ, y: wage).points(alpha: 0.4).points(y: fitted, color: \"firebrick\", size: 3)\n\
         d.plot(x: educ, y: wage).points().points(y: wage * 2)\n\
         d.plot().points(x: educ, y: wage, color: union).points(x: educ, y: log(wage) * 20, shape: \"cross\")\n\
         d.plot(x: educ, y: wage, color: union).points().points(y: fitted, color: female)\n\
         d.plot(x: educ, y: wage, size: 6).points(size: hours).points(x: exper, shape: female)\n\
         let top = d.where(wage > 50)\n\
         d.plot(x: educ, y: wage).points(alpha: 0.2).points(data: top, y: hours, color: \"steelblue\")\n"
    );
    let text = printed(&script);
    // The reference, from R on the shipped file: 537 rows of 2010, all with educ, 531 of them
    // with a wage, 530 with a wage and hours, 530 with exper and a wage; 14 wages above 50.
    // The wage runs from 4.58 to 97.98, so with twice the wage on the same axis the limits, 5%
    // of the range beyond each end, are -4.989 and 205.529; educ and exper together run from
    // 0 to 25, so theirs are -1.25 and 26.25. `fitted` lies within the wages and moves nothing.
    for line in [
        // A second variable on the axis of the first, in a colour and a size of its own.
        "  y      Hourly wage          linear   -0.09 to 102.65   0, 20, 40, 60, 80, 100\n",
        "      1   points   d        531   size 4.5, alpha 0.4\n",
        "      2   points   d        537   y fitted, color firebrick, size 3, alpha 1\n",
        // An axis runs over the values of every layer, and keeps the title of the plot.
        "  y      Hourly wage          linear   -4.989 to 205.529   0, 50, 100, 150, 200\n",
        "      2   points   d        531   y wage * 2, size 4.5, alpha 1\n",
        // A plot that maps nothing: the layers name everything, and the first gives the titles.
        "plot · d · wage against educ\n",
        "      1   points   d        531   x educ, y wage, color by union, size 4.5, alpha 1\n",
        "      2   points   d        531   x educ, y log(wage) * 20, size 4.5, shape cross, alpha 1\n",
        // Two variables on one scale of colours: the categories of the second follow.
        "  color       Union member   category   no, yes, male, female   orange, sky blue, bluish green, yellow\n",
        "      2   points   d        537   y fitted, color by female, size 4.5, alpha 1\n",
        // A layer sized by a variable where the plot sets one size; another placed by exper.
        "  x      Years of schooling   linear   -1.25 to 26.25    0, 5, 10, 15, 20, 25\n",
        "      1   points   d        530   size by hours, alpha 1\n",
        "      2   points   d        530   x exper, size 6, shape by female, alpha 1\n",
        // A layer with a frame of its own and a variable of its own, read in that frame.
        "      2   points   top       14   y hours, color steelblue, size 4.5, alpha 1\n",
    ] {
        assert!(text.contains(line), "{line}\n{text}");
    }
    insta::assert_snapshot!("layer_mappings", text);
}

#[test]
fn the_edge_cases_of_a_layer_that_names_its_own_variables() {
    let mut all = String::new();
    for (name, script) in [
        // What the layer names is what the plot maps: the same marks, told in its settings.
        (
            "repeats_the_plot",
            "d.plot(x: educ, y: wage).points(x: educ, y: wage)\n",
        ),
        // The plot maps one position and each layer the other.
        (
            "each_its_own_y",
            "d.plot(x: educ).points(y: wage).points(y: hours)\n",
        ),
        // One colour for a layer where the plot colours by a variable: no mark reads the
        // scale, and no row is left out for a missing category.
        (
            "a_colour_of_its_own",
            "d.plot(x: educ, y: wage, color: union).points(color: \"black\")\n",
        ),
        // One size for a layer where the plot sizes by a variable, and the reverse.
        (
            "a_size_of_its_own",
            "d.plot(x: educ, y: wage, size: hours).points(size: 5)\n",
        ),
        (
            "a_size_by_a_variable",
            "d.plot(x: educ, y: wage, size: 5).points(size: hours)\n",
        ),
        // A shape by a variable where the plot sets one shape.
        (
            "a_shape_by_a_variable",
            "d.plot(x: educ, y: wage, shape: \"square\").points(shape: union)\n",
        ),
        // The rows a layer lacks are counted under its own variable.
        (
            "its_own_missing_rows",
            "d.plot(x: educ, y: wage).points(y: hours)\n",
        ),
        // A layer's own colour by a number, on a ramp, where the plot maps no colour.
        (
            "a_ramp_of_its_own",
            "d.plot(x: educ, y: wage).points(color: hours)\n",
        ),
        // A name that is a binding and a variable both: the variable, with a warning.
        (
            "a_name_twice",
            "let educ = 3\nd.plot(x: educ, y: wage).points(y: wage / educ)\n",
        ),
        // `data:` written after the variable it is read in.
        (
            "its_frame_written_last",
            "let top = d.where(wage > 50)\nd.plot(x: educ, y: wage).points(y: hours * 2, data: top)\n",
        ),
    ] {
        let (code, stdout, stderr) = go(&format!("{FITTED}{script}"));
        assert_eq!(code, 0, "{name}\n{stderr}");
        all.push_str(&format!("== {name}\n{stdout}---\n{stderr}\n"));
    }
    insta::assert_snapshot!("layer_mapping_edges", all);
}

#[test]
fn every_message_of_a_layer_that_names_its_own_variables() {
    let mut all = String::new();
    for (name, script) in [
        (
            "no_such_variable",
            "d.plot(x: educ, y: wage).points(y: fited)\n",
        ),
        (
            "another_case",
            "d.plot(x: educ, y: wage).points(y: Fitted)\n",
        ),
        (
            "not_a_number",
            "d.plot(x: educ, y: wage).points(y: industry)\n",
        ),
        (
            "no_such_variable_in_an_expression",
            "d.plot(x: educ, y: wage).points(y: log(wagee))\n",
        ),
        (
            "one_value_for_all_rows",
            "d.plot(x: educ, y: wage).points(y: mean(wage))\n",
        ),
        (
            "not_in_its_own_frame",
            "let top = d.where(wage > 50).drop(fitted)\nd.plot(x: educ, y: wage).points(data: top, y: fitted)\n",
        ),
        ("neither_maps_y", "d.plot(x: educ).points(x: exper)\n"),
        ("neither_maps_either", "d.plot().points(color: union)\n"),
        ("the_plot_maps_no_y", "d.plot(x: educ).points()\n"),
        (
            "not_a_colour",
            "d.plot(x: educ, y: wage).points(color: \"firebrik\")\n",
        ),
        (
            "a_variable_in_quotes",
            "d.plot(x: educ, y: wage).points(color: \"union\")\n",
        ),
        (
            "a_date_for_a_colour",
            "frame(a = [1, 2], b = [3, 4], day = [date(\"2026-01-01\"), date(\"2026-01-02\")]).plot(x: a, y: b).points(color: day)\n",
        ),
        (
            "a_number_where_categories_are",
            "d.plot(x: educ, y: wage, color: union).points(color: hours)\n",
        ),
        (
            "categories_where_a_number_is",
            "d.plot(x: educ, y: wage, color: hours).points(color: union)\n",
        ),
        (
            "another_kind_than_the_first_layer",
            "d.plot(x: educ, y: wage).points(color: union).points(color: hours)\n",
        ),
        (
            "categories_for_a_size",
            "d.plot(x: educ, y: wage).points(size: union)\n",
        ),
        ("no_size", "d.plot(x: educ, y: wage).points(size: 0)\n"),
        (
            "a_number_for_a_shape",
            "d.plot(x: educ, y: wage).points(shape: hours)\n",
        ),
        (
            "not_a_shape",
            "d.plot(x: educ, y: wage).points(shape: \"blob\")\n",
        ),
        (
            "an_expression_for_a_colour",
            "d.plot(x: educ, y: wage).points(color: wage > 20)\n",
        ),
        (
            "an_expression_for_a_size",
            "d.plot(x: educ, y: wage).points(size: hours / 2)\n",
        ),
        (
            "an_expression_for_a_shape",
            "d.plot(x: educ, y: wage).points(shape: union == 1)\n",
        ),
        (
            "text_for_a_position",
            "d.plot(x: educ, y: wage).points(y: \"fitted\")\n",
        ),
        (
            "no_group_on_a_mark_of_points",
            "d.plot(x: educ, y: wage).points(group: union)\n",
        ),
    ] {
        let (code, stdout, stderr) = go(&format!("{FITTED}{script}"));
        assert_eq!(code, 1, "{name}\n{stdout}");
        assert_eq!(stdout, "", "{name}");
        all.push_str(&format!("== {name}\n{stderr}\n"));
    }
    insta::assert_snapshot!("layer_mapping_messages", all);
}

// ----- lines (step 23c) -----

/// The shipped wages, their mean by year, and their mean by year and union membership.
const MEANS: &str = "let w = dataset(wages)\nlet m = w.aggregate(wage = mean(wage), by: year)\nlet u = w.aggregate(wage = mean(wage), by: [year, union])\n";

#[test]
fn lines_read_sensibly_as_text() {
    let script = format!(
        "{MEANS}m.plot(x: year, y: wage).lines()\n\
         u.plot(x: year, y: wage, color: union).points().lines(linetype: union)\n\
         let few = w.where(worker <= 6)\n\
         few.plot(x: year, y: wage, group: worker).lines(width: 0.5)\n\
         few.plot(x: year, y: wage).lines(group: worker, color: female, linetype: \"dashed\")\n\
         m.plot(x: year, y: wage).points().lines(y: wage * 1.1, color: \"firebrick\", linetype: \"dotted\")\n"
    );
    let text = printed(&script);
    // The reference, from R on the shipped file: twelve years, 2010 to 2021, whose mean wages
    // run from 22.4203 to 32.4430, so the limits, 5% of the range beyond each end, are
    // 21.9191 and 32.9442, and 2009.45 and 2021.55 for the years; by union membership 24
    // means in two lines, with limits 21.131 and 37.1199 (ggplot2 gives 21.13096 and
    // 37.11989, and draws the second line dashed too); workers 1 to 6 have 30 rows, all with
    // a wage, in six lines, with limits 11.4635 and 65.5065.
    for line in [
        "  x      Calendar year   linear   2009.45 to 2021.55   2010, 2012, 2014, 2016, 2018, 2020\n",
        "  y      wage            linear   21.9191 to 32.9442   22, 24, 26, 28, 30, 32\n",
        "      1   lines   m         12   width 1, 1 line\n",
        "  y      wage            linear   21.131 to 37.1199    25, 30, 35\n",
        "  color       Union member   category   no, yes   orange, sky blue\n",
        "  linetype    Union member   category   no, yes   solid, dashed\n",
        "      2   lines    u         24   width 1, linetype by union, 2 lines\n",
        "  y      Hourly wage     linear   11.4635 to 65.5065   20, 30, 40, 50, 60\n",
        "  group       Worker identifier           each value a group\n",
        "      1   lines   few       30   width 0.5, 6 lines\n",
        "      1   lines   few       30   color by female, width 1, linetype dashed, group by worker, 6 lines\n",
        "      2   lines    m         12   y wage * 1.1, color firebrick, width 1, linetype dotted, 1 line\n",
    ] {
        assert!(text.contains(line), "{line}\n{text}");
    }
    insta::assert_snapshot!("lines", text);
}

#[test]
fn the_edge_cases_of_lines() {
    let gap = "let gap = frame(t = [7, 2, 3, 4, 5, 6, 1], v = [5.0, 2.0, NA, 4.0, 3.0, NA, 1.0], g = [\"b\", \"a\", \"a\", \"a\", \"b\", \"b\", \"a\"])\n";
    let mut all = String::new();
    for (name, script) in [
        // The rows in any order: joined in the order of `t`; a line broken where `v` is NA,
        // and the last row, left alone, drawing nothing.
        ("a_gap", "gap.plot(x: t, y: v).lines()\n"),
        (
            "a_gap_in_each_group",
            "gap.plot(x: t, y: v).lines(group: g)\n",
        ),
        // One point, and a group of one: nothing to join.
        (
            "one_point",
            "frame(a = [1], b = [2.0]).plot(x: a, y: b).lines()\n",
        ),
        (
            "a_group_of_one",
            "frame(a = [1, 2, 3], b = [2.0, 3.0, 1.0], g = [\"p\", \"p\", \"q\"]).plot(x: a, y: b, group: g).lines()\n",
        ),
        // No row at all, and no row with both positions.
        (
            "no_rows",
            "frame(a = [1], b = [2.0]).where(a > 1).plot(x: a, y: b).lines()\n",
        ),
        (
            "nothing_to_draw",
            "frame(a = [1, NA], b = [NA, 2.0]).plot(x: a, y: b).lines()\n",
        ),
        // Rows with one `x`: joined in the order of the frame.
        (
            "ties",
            "frame(a = [1, 1, 2, 2], b = [1.0, 3.0, 2.0, 0.0]).plot(x: a, y: b).lines()\n",
        ),
        // An infinite `y` breaks the line as `NA` does, and an infinite `x` has no place.
        (
            "infinite",
            "frame(a = [1, 2, 3, 4, 1 / 0], b = [1.0, 2.0, 1 / 0, 4.0, 5.0]).plot(x: a, y: b).lines()\n",
        ),
        // A category of the group that is NA is a line of its own.
        (
            "a_group_without_a_name",
            "frame(a = [1, 2, 3, 4], b = [1.0, 2.0, 3.0, 4.0], g = [\"p\", NA, \"p\", NA]).plot(x: a, y: b).lines(group: g)\n",
        ),
        // Seven categories and six types of line: the types repeat from the seventh.
        (
            "seven_types",
            "frame(a = [1, 2, 1, 2, 1, 2, 1, 2, 1, 2, 1, 2, 1, 2], b = [1.0, 2.0, 2.0, 3.0, 3.0, 4.0, 4.0, 5.0, 5.0, 6.0, 6.0, 7.0, 7.0, 8.0], g = [\"a\", \"a\", \"b\", \"b\", \"c\", \"c\", \"d\", \"d\", \"e\", \"e\", \"f\", \"f\", \"g\", \"g\"]).plot(x: a, y: b).lines(linetype: g)\n",
        ),
        // A layer of lines with a frame of its own, and one that takes the group of the plot.
        (
            "its_own_frame",
            "let late = gap.where(t > 3)\ngap.plot(x: t, y: v, group: g).points().lines(data: late)\n",
        ),
        // The plot sets one colour: the lines take it; the layer sets another.
        (
            "one_colour",
            "gap.plot(x: t, y: v, color: \"steelblue\").lines().lines(color: \"black\", width: 2)\n",
        ),
        // Size and shape of the plot are not read by a line.
        (
            "what_a_line_does_not_read",
            "gap.plot(x: t, y: v, size: 6, shape: \"square\").lines()\n",
        ),
    ] {
        let (code, stdout, stderr) = go(&format!("{gap}{script}"));
        assert_eq!(code, 0, "{name}\n{stderr}");
        all.push_str(&format!("== {name}\n{stdout}---\n{stderr}\n"));
    }
    insta::assert_snapshot!("lines_edges", all);
}

#[test]
fn every_message_of_lines() {
    let mut all = String::new();
    for (name, script) in [
        ("the_plot_maps_no_y", "m.plot(x: year).lines()\n"),
        ("neither_maps_x", "m.plot().lines(y: wage)\n"),
        (
            "no_such_variable",
            "m.plot(x: year, y: wage).lines(y: wag)\n",
        ),
        (
            "not_a_number",
            "w.plot(x: year, y: wage).lines(y: industry)\n",
        ),
        (
            "no_such_group",
            "u.plot(x: year, y: wage).lines(group: unionn)\n",
        ),
        (
            "a_group_in_quotes",
            "u.plot(x: year, y: wage).lines(group: \"union\")\n",
        ),
        (
            "a_group_computed",
            "u.plot(x: year, y: wage).lines(group: year > 2015)\n",
        ),
        (
            "a_colour_by_a_number",
            "u.plot(x: year, y: wage).lines(color: wage)\n",
        ),
        (
            "a_colour_by_a_number_of_the_plot",
            "u.plot(x: year, y: wage, color: wage).points().lines()\n",
        ),
        (
            "not_a_colour",
            "u.plot(x: year, y: wage).lines(color: \"blu\")\n",
        ),
        (
            "a_type_by_a_number",
            "u.plot(x: year, y: wage).lines(linetype: wage)\n",
        ),
        (
            "not_a_type",
            "u.plot(x: year, y: wage).lines(linetype: \"dash\")\n",
        ),
        (
            "a_type_without_quotes",
            "u.plot(x: year, y: wage).lines(linetype: dashed)\n",
        ),
        (
            "a_variable_in_quotes_for_a_type",
            "u.plot(x: year, y: wage).lines(linetype: \"union\")\n",
        ),
        (
            "a_type_computed",
            "u.plot(x: year, y: wage).lines(linetype: union == 1)\n",
        ),
        ("no_width", "u.plot(x: year, y: wage).lines(width: 0)\n"),
        (
            "a_negative_width",
            "u.plot(x: year, y: wage).lines(width: -1)\n",
        ),
        (
            "a_width_in_words",
            "u.plot(x: year, y: wage).lines(width: \"thin\")\n",
        ),
        (
            "no_size_on_lines",
            "u.plot(x: year, y: wage).lines(size: 2)\n",
        ),
        (
            "no_shape_on_lines",
            "u.plot(x: year, y: wage).lines(shape: union)\n",
        ),
        (
            "no_alpha_on_lines",
            "u.plot(x: year, y: wage).lines(alpha: 0.5)\n",
        ),
        ("lines_of_a_frame", "u.lines()\n"),
        (
            "its_frame_lacks_the_group",
            "u.plot(x: year, y: wage, group: union).lines(data: m)\n",
        ),
    ] {
        let (code, stdout, stderr) = go(&format!("{MEANS}{script}"));
        assert_eq!(code, 1, "{name}\n{stdout}");
        assert_eq!(stdout, "", "{name}");
        // The warning and the note of `aggregate` are not what is tested here.
        let said: String = stderr
            .lines()
            .filter(|l| {
                !l.starts_with("warning[W0201]") && !l.starts_with("note: the panel declaration")
            })
            .map(|l| format!("{l}\n"))
            .collect();
        all.push_str(&format!("== {name}\n{said}\n"));
    }
    insta::assert_snapshot!("lines_messages", all);
}
