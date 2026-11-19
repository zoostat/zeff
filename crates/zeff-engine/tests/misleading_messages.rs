//! Messages the review against R of 19 November 2026 found misleading (release 0.8.0, steps
//! 20j to 20l): each said something true of the engine and nothing the writer could act on.
//! Every case names what was said before; the snapshot holds what is said now.

use zeff_testkit::run;

fn said(script: &str) -> String {
    let outcome = run(script);
    format!(
        "{script}---- exit code {}\n{}{}",
        outcome.exit_code, outcome.output.stdout, outcome.output.stderr
    )
}

fn all(scripts: &[&str]) -> String {
    scripts
        .iter()
        .map(|script| said(script))
        .collect::<Vec<_>>()
        .join("\n====\n")
}

/// Step 20j (b). `n_missing([x1, x2, x3])` inside `add` counted the three variables as the
/// items of a list, none of them `NA`, and gave 0 in every row with no word. A list of
/// variables is refused, and the help says how the count across each row is written; a list of
/// values and one variable count as before.
#[test]
fn n_missing_of_a_list_of_variables_is_refused() {
    let data = "let d = frame(x1 = [1.0, NA, 3.0], x2 = [NA, NA, 1.0], x3 = [1.0, 2.0, 3.0])\n";
    insta::assert_snapshot!(
        "n_missing_of_variables",
        all(&[
            &format!("{data}d.add(k = n_missing([x1, x2, x3]))\n"),
            &format!("{data}d.add(k = n_missing([x1]))\n"),
            &format!("{data}d.aggregate(k = n_missing([x1, x2]))\n"),
            // What the help line offers, and the two uses that stand.
            &format!(
                "{data}d.add(k = to(is_missing(x1), Int) + to(is_missing(x2), Int) + to(is_missing(x3), Int))\n"
            ),
            &format!(
                "{data}n_missing([1, NA, 3])\nn_missing(d[\"x1\"])\nd.aggregate(k = n_missing(x1))\n"
            ),
        ])
    );
}

/// Step 20j (d). A term of a fit written as text inside `test`, `m.test("rank=Associate" == 0)`,
/// was answered only that a Str is compared with an Int. The help now says that a term is
/// written in backticks, with the term as it should be written; so does the refusal of two terms
/// both written as text, which is met when the test runs. A comparison of text with a number
/// anywhere else keeps the general help.
#[test]
fn a_term_written_as_text_inside_test_is_pointed_to_backticks() {
    let data = "let d = frame(y = [1.0, 2.0, 4.0, 3.0, 6.0, 5.0, 8.0, 9.5], rank = [\"Prof\", \"Associate\", \"Prof\", \"Assistant\", \"Associate\", \"Assistant\", \"Prof\", \"Associate\"], x = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 9.0]).add(rank = category(rank))\nlet m = d.reg(y ~ x + rank, se: classical)\n";
    insta::assert_snapshot!(
        "term_as_text",
        all(&[
            &format!("{data}m.test(\"rank=Associate\" == 0)\n"),
            &format!("{data}m.test(0 == \"x\")\n"),
            &format!("{data}m.test(x == 0, \"rank=Prof\" == \"rank=Associate\")\n"),
            // As the help says to write it.
            &format!("{data}m.test(`rank=Associate` == 0)\n"),
            // Outside a restriction the general help stands.
            &format!("{data}\"a\" == 1\n"),
        ])
    );
}

/// Step 20j (e). A reserved word used as a name, `add(as = asin(x))`, was answered "expected an
/// expression, found `as`". The message now says that the word is reserved and how a variable
/// of that name is written. A reserved word at the start of a line is a statement out of place
/// and keeps the plain message.
#[test]
fn a_reserved_word_used_as_a_name_is_named_as_one() {
    let data = "let d = frame(x = [0.1, 0.2])\n";
    insta::assert_snapshot!(
        "reserved_word_as_a_name",
        all(&[
            &format!("{data}d.add(as = asin(x))\n"),
            &format!("{data}d.where(in > 0.1)\n"),
            &format!("{data}let y = 1 + use\n"),
            &format!("{data}d.add(model = x * 2, dag = x)\n"),
            &format!("{data}let as = 3\n"),
            &format!("{data}d.add(while = x)\n"),
            // In backticks, as the help says.
            &format!("{data}d.add(`as` = asin(x)).where(`as` > 0.15)\n"),
            // Not a name: a statement out of place.
            &format!("{data}else\n"),
            &format!("{data}d.add(y = )\n"),
        ])
    );
}
