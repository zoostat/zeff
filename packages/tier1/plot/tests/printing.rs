//! `plot` and `points` as a script writes them (Language Reference, section 7.6; release 0.7.0,
//! step 20): the printed form of a plot, which is its scene read as text, in every shape; the
//! notes of the rows not drawn and of the seed; jitter repeated under a seed; and a snapshot of
//! every message. From step 20b, a position mapped from an expression (`y: log(wage)`): what it
//! prints against R's values, the names it reads, the layers that compute it on their own
//! frame, its edge cases and its messages.

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
