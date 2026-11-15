//! `print(x, ..., sep: " ")` (release 0.8.0, step 8p; open decision #95, option A): plain
//! values given side by side, an Int, a Float, a Str, a Bool, a Date, a DateTime or `NA`,
//! share one line with `sep:` between them; any other value ends the line and prints as its
//! own block; every line ends; and one value prints exactly as it did in release 0.7.0.

use zeff_testkit::run;

fn go(script: &str) -> (i32, String, String) {
    let outcome = run(script);
    (
        outcome.exit_code,
        outcome.output.stdout,
        outcome.output.stderr,
    )
}

fn printed(script: &str) -> String {
    let (code, stdout, stderr) = go(script);
    assert_eq!(code, 0, "{script}\n{stderr}");
    stdout
}

#[test]
fn plain_values_of_every_type_share_one_line() {
    assert_eq!(printed("print(1, 2)\n"), "1 2\n");
    assert_eq!(
        printed("print(\"Extra sleep, in hours:\", 1.58)\n"),
        "Extra sleep, in hours: 1.58\n"
    );
    assert_eq!(
        printed(
            "print(3, 0.1, \"text\", true, date(\"2026-11-16\"), datetime(\"2026-11-16T09:30:00\"), NA)\n"
        ),
        "3 0.1 text true 2026-11-16 2026-11-16T09:30:00 NA\n"
    );
    // Numbers keep the printing rules of section 3.8; `format` shapes them when asked.
    assert_eq!(
        printed("print(1 / 3, 1e15, 123456789, format(1 / 3, digits: 2))\n"),
        "0.333333 1e+15 123456789 0.33\n"
    );
    // A value from a binding or a call is plain when its type is.
    assert_eq!(
        printed(
            "let d = frame(x = [1, 2, 3])\nlet m = d.mean(x)\nprint(\"mean\", m, \"of\", d.n_rows(), \"rows\")\n"
        ),
        "mean 2 of 3 rows\n"
    );
    // An empty text is still a value of its line.
    assert_eq!(printed("print(\"a\", \"\", \"b\")\n"), "a  b\n");
}

#[test]
fn sep_stands_between_the_values_of_a_line() {
    assert_eq!(
        printed("print(\"n\", \"mean\", \"sd\", sep: \"\\t\")\n"),
        "n\tmean\tsd\n"
    );
    assert_eq!(printed("print(1, 2, 3, sep: \", \")\n"), "1, 2, 3\n");
    assert_eq!(printed("print(\"x is \", 3, sep: \"\")\n"), "x is 3\n");
    // One value has no neighbour, so `sep:` changes nothing; nor does it stand around a block.
    assert_eq!(printed("print(7, sep: \"--\")\n"), "7\n");
    assert_eq!(
        printed("print(1, [2, 3], 4, sep: \"--\")\n"),
        "1\n2\n3\n4\n"
    );
    // A text given by place is a value, never the separator.
    assert_eq!(printed("print(\"a\", \"b\", \"\\t\")\n"), "a b \t\n");
}

#[test]
fn any_other_value_ends_the_line_and_prints_as_its_own_block() {
    let frame = "let d = frame(x = [1, 2], y = [\"a\", \"b\"])\n";
    let alone = printed(&format!("{frame}print(d)\n"));
    // Before, between and after a block.
    assert_eq!(
        printed(&format!("{frame}print(\"Rows:\", d.n_rows(), d)\n")),
        format!("Rows: 2\n{alone}")
    );
    assert_eq!(
        printed(&format!("{frame}print(d, \"Rows:\", d.n_rows())\n")),
        format!("{alone}Rows: 2\n")
    );
    assert_eq!(
        printed(&format!("{frame}print(\"before\", 1, d, \"after\", 2)\n")),
        format!("before 1\n{alone}after 2\n")
    );
    // A list, a record and a column are blocks, a one-value column and a list of one included.
    assert_eq!(printed("print(\"a\", [1, 2.5], \"b\")\n"), "a\n1\n2.5\nb\n");
    assert_eq!(printed("print(\"a\", [1], \"b\")\n"), "a\n1\nb\n");
    let column = printed("print(frame(x = [5])[\"x\"])\n");
    assert_eq!(
        printed("print(\"a\", frame(x = [5])[\"x\"], \"b\")\n"),
        format!("a\n{column}b\n")
    );
    assert!(column.starts_with("Column<Int> · 1 value"), "{column}");
    let record = printed("print({name: \"Ann\", age: 30})\n");
    assert_eq!(
        printed("print(1, {name: \"Ann\", age: 30}, 2)\n"),
        format!("1\n{record}2\n")
    );
    // Two blocks side by side print one after the other, as they did.
    assert_eq!(
        printed(&format!("{frame}print(d, d)\n")),
        format!("{alone}{alone}")
    );
}

/// One argument prints exactly as a statement that is not a `let` prints it, for every type:
/// what release 0.7.0 printed.
#[test]
fn one_value_prints_as_before() {
    for value in [
        "42",
        "0.1",
        "1e15",
        "\"hello\"",
        "\"\"",
        "true",
        "NA",
        "date(\"2026-11-16\")",
        "datetime(\"2026-11-16T09:30:00.25\")",
        "[1, 2.5]",
        "[\"a\"]",
        "{name: \"Ann\", age: 30}",
        "frame(x = [1, 2], y = [\"a\", \"b\"])",
        "frame(x = [1, 2])[\"x\"]",
        "frame(x = [1.5, 2, 4]).summary([x])",
        "dist.normal()",
        "1..3",
    ] {
        let by_print = printed(&format!("print({value})\n"));
        let by_statement = printed(&format!("{value}\n"));
        assert_eq!(by_print, by_statement, "{value}");
        assert!(by_print.ends_with('\n'), "{value}: the line ends");
    }
    // In release 0.7.0 these printed as follows, and still do.
    assert_eq!(printed("print(42)\n"), "42\n");
    assert_eq!(printed("print(\"hello\")\n"), "hello\n");
    assert_eq!(printed("print(NA)\n"), "NA\n");
    assert_eq!(printed("print([1, 2.5])\n"), "1\n2.5\n");
}

#[test]
fn the_forms_and_the_messages() {
    let (code, stdout, stderr) = go(
        "let w = dataset(wages).where(year == 2019)\nprint(\"Rows:\", w.n_rows(), w.head(3))\nprint(\"n\", \"mean\", \"sd\", sep: \"\\t\")\nprint(w.n_rows(), w.mean(wage), w.sd(wage), sep: \"\\t\")\nfor i in 1..3 {\n    print(\"pass\", i, \"of\", 3)\n}\nprint(true, NA, date(\"2026-11-16\"))\nprint(\"Scores\", [1, 2.5], \"done\")\n",
    );
    assert_eq!(code, 0, "{stderr}");
    let mut all = format!("{stdout}---\n{stderr}");
    for script in [
        "print(\"n\", \"mean\", separator: \"\\t\")\n",
        "print(1, 2, sep: 3)\n",
        "print(sep: \"\\t\")\n",
        "print(1, sep: \"a\", 2)\n",
        "let x = print(1, 2)\n",
    ] {
        let (code, stdout, stderr) = go(script);
        assert_eq!(code, 1, "{script}");
        assert_eq!(stdout, "", "{script}");
        all.push_str(&format!("\n{script}{stderr}"));
    }
    insta::assert_snapshot!("print_line", all);
}
