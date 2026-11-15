//! A comparison with the literal `NA` (release 0.8.0, step 8l; Language Reference, section 4.1,
//! the row "Testing"; Z0226): `x == NA` and `x != NA` are refused by the checker wherever they
//! are written, on a scalar and on a column, with `NA` on either side, and the help names
//! `is_missing`; a value that is `NA` when the script runs still compares as section 4.1 says.

use zeff_testkit::run;

fn go(script: &str) -> (i32, String, String) {
    let outcome = run(script);
    (
        outcome.exit_code,
        outcome.output.stdout,
        outcome.output.stderr,
    )
}

const FRAME: &str = "let s = frame(score = [72.5, NA, 60], name = [\"Ann\", NA, \"Cy\"])\n";

#[test]
fn the_literal_is_refused_wherever_it_is_written() {
    let mut all = String::new();
    for (name, script) in [
        ("in_where", "s.where(score == NA)\n"),
        ("in_where_not_equal", "s.where(score != NA).n_rows()\n"),
        ("na_on_the_left", "s.where(NA == score)\n"),
        ("in_add", "s.add(missing = score == NA)\n"),
        ("in_update", "s.update(score = 0, where: score == NA)\n"),
        (
            "in_case",
            "s.add(band = case { score == NA => \"none\", _ => \"some\" })\n",
        ),
        (
            "in_if",
            "let x = to(NA, Int)\nif x == NA { print(\"missing\") }\n",
        ),
        ("in_a_bare_statement", "let x = 3\nx == NA\n"),
        ("in_a_binding", "let x = 3\nlet known = x != NA\n"),
        ("in_brackets", "let x = 3\nprint((x) == (NA))\n"),
        ("text", "s.where(name == NA)\n"),
        ("both_sides", "NA == NA\n"),
        (
            "inside_a_larger_condition",
            "s.where(score > 50 && score != NA)\n",
        ),
        (
            "in_a_function",
            "/// Whether a value is missing.\nfn none(x: Float) -> Bool { x == NA }\nprint(none(1.0))\n",
        ),
    ] {
        let (code, stdout, stderr) = go(&format!("{FRAME}{script}"));
        assert_eq!(code, 1, "{name}\n{stdout}{stderr}");
        assert_eq!(stdout, "", "{name}: nothing runs");
        assert_eq!(
            stderr.matches("error[Z0226]").count(),
            1,
            "{name}\n{stderr}"
        );
        all.push_str(&format!("== {name}\n{stderr}\n"));
    }
    insta::assert_snapshot!("na_comparison", all);
}

/// Only the literal is refused: a binding or a variable that holds `NA` compares as before, the
/// comparison is `NA`, and `is_missing` is the test.
#[test]
fn a_value_that_is_na_still_compares_as_na() {
    let (code, stdout, stderr) = go(&format!(
        "{FRAME}let none = to(NA, Float)\nprint(1.5 == none)\nprint(none != none)\ns.where(score == none).n_rows()\ns.where(is_missing(score)).n_rows()\ns.where(!is_missing(score)).n_rows()\ns.add(same = score == score)\nprint(NA < 1)\nprint(is_missing(NA))\n"
    ));
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(
        stdout,
        "NA\nNA\n0\n1\n2\nframe · 3 rows × 3 variables\n\n      score   name   same\n  1    72.5   Ann    true\n  2      NA   NA     NA\n  3      60   Cy     true\nNA\ntrue\n"
    );
}
