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

/// Step 20k (a). A warning raised inside one branch of `case` counted every row of the frame:
/// with `to(floor_txt, Int)` after the branches that take "Ground", "Upper" and "Lower", the
/// script was told that those three could not be converted, though no row reaches the
/// conversion with them. What a branch says is now said of the rows it decides: nothing when
/// they give no trouble, and the true count when they do. The values are the same as before,
/// reducers inside a branch still see every row, and a branch that decides every row says what
/// it always said.
#[test]
fn a_branch_of_case_warns_of_the_rows_it_decides() {
    let data = "let d = frame(floor_txt = [\"Ground\", \"3\", \"Upper\", \"x7\", \"Lower\", \"12\", \"none\"], g = [\"a\", \"a\", \"a\", \"b\", \"b\", \"b\", \"b\"], x = [1.0, 0.0, 3.0, 0.0, 5.0, 6.0, 0.0], y = [0.0, 0.0, 1.0, 0.0, 2.0, 1.0, 3.0])\n";
    let clean = "let c = d.where(floor_txt != \"x7\" && floor_txt != \"none\")\n";
    insta::assert_snapshot!(
        "branch_of_case",
        all(&[
            // The review's case: every text that is not a number is taken by an earlier branch.
            &format!(
                "{data}{clean}c.add(floor = case {{ floor_txt == \"Ground\" => 0, floor_txt == \"Upper\" => 99, floor_txt == \"Lower\" => -1, _ => to(floor_txt, Int) }})\n"
            ),
            // Two texts do reach the conversion: they are counted, and only they.
            &format!(
                "{data}d.add(floor = case {{ floor_txt == \"Ground\" => 0, floor_txt == \"Upper\" => 99, floor_txt == \"Lower\" => -1, _ => to(floor_txt, Int) }})\n"
            ),
            // A division guarded by its branch says nothing; unguarded, what it always said.
            &format!("{data}d.add(r = case {{ y > 0 => x / y, _ => 0 }})\n"),
            &format!("{data}d.add(r = case {{ y >= 0 => x / y, _ => 0 }})\n"),
            &format!("{data}d.add(r = x / y)\n"),
            // A reducer inside a branch still sees every row: the mean of x is 15 / 7.
            &format!("{data}d.add(r = case {{ y > 0 => x - mean(x), _ => 0 }})\n"),
            // In `where`, under `by:`, and a `case` inside a branch.
            &format!(
                "{data}d.where(case {{ floor_txt == \"Ground\" => true, floor_txt == \"Upper\" => false, floor_txt == \"Lower\" => false, _ => to(floor_txt, Int) > 5 }})\n"
            ),
            &format!(
                "{data}d.add(by: g, f = case {{ floor_txt == \"Ground\" => 0, _ => to(floor_txt, Int) }})\n"
            ),
            &format!(
                "{data}d.add(m = case {{ floor_txt == \"Ground\" => 0, _ => case {{ y > 0 => x / y, _ => to(floor_txt, Float) }} }})\n"
            ),
        ])
    );
}

/// A draw inside a branch is made once: the second evaluation that recounts a warning leaves
/// the run's randomness as it was, so the numbers are those of the branch alone.
#[test]
fn recounting_a_warning_moves_no_draw() {
    let data = "seed(7)\nlet d = frame(t = [\"1\", \"x\", \"3\", \"y\"], k = [1, 2, 3, 4])\n";
    let warned = run(&format!(
        "{data}let a = d.add(v = case {{ k == 2 => 0.0, _ => to(t, Float) + dist.normal().draw(n()) }})\nlet b = d.add(w = dist.normal().draw(n()))\nprint(a)\nprint(b)\n"
    ));
    let plain = run(&format!(
        "{data}let a = d.add(v = case {{ k == 2 => 0.0, k == 4 => 0.0, _ => to(t, Float) + dist.normal().draw(n()) }})\nlet b = d.add(w = dist.normal().draw(n()))\nprint(a)\nprint(b)\n"
    ));
    assert_eq!(warned.exit_code, 0, "{}", warned.output.stderr);
    assert_eq!(plain.exit_code, 0, "{}", plain.output.stderr);
    // The first warns of "y" and is evaluated twice; the second warns of nothing. Rows 1 and
    // 3 of `v` and every row of `w` are the same in both.
    assert!(
        warned
            .output
            .stderr
            .contains("1 value could not be converted to Float")
    );
    assert!(!plain.output.stderr.contains("could not be converted"));
    let rows = |stdout: &str, of: usize| -> Vec<String> {
        stdout
            .lines()
            .filter(|line| line.trim_start().starts_with(|c: char| c.is_ascii_digit()))
            .map(str::to_string)
            .collect::<Vec<_>>()
            .chunks(4)
            .nth(of)
            .unwrap()
            .to_vec()
    };
    let (first, second) = (
        rows(&warned.output.stdout, 0),
        rows(&plain.output.stdout, 0),
    );
    assert_eq!(first[0], second[0]);
    assert_eq!(first[2], second[2]);
    assert_eq!(
        rows(&warned.output.stdout, 1),
        rows(&plain.output.stdout, 1)
    );
}

/// Step 20k (c). In `aggregate`, a summary that takes the name of a variable is that variable
/// for the definitions after it: `spend = mean(spend)` followed by `sd_spend = sd(spend)` gave
/// the spread of a single value, 0, and `sum(spend) / sum(spend)` its ratio, with at most
/// "0 / 0 gave NA" to show for it. A note now names the definition that reads the name and
/// says how to summarise the variable instead. Definitions that read no such name, a summary
/// defined after its uses, and `add` (where the new variable has a value in every row and
/// says that it replaced the old) are as before.
#[test]
fn aggregate_notes_a_name_it_has_just_redefined() {
    let data = "let d = frame(g = [\"a\", \"a\", \"b\", \"b\"], spend = [10.0, 20.0, 30.0, 50.0], n = [1, 2, 3, 4])\n";
    insta::assert_snapshot!(
        "aggregate_redefined_name",
        all(&[
            &format!(
                "{data}d.aggregate(by: g, spend = mean(spend), share = sum(spend) / sum(spend), sd_spend = sd(spend))\n"
            ),
            &format!(
                "{data}d.aggregate(spend = mean(spend), n = sum(n), ratio = sum(spend) / sum(n))\n"
            ),
            &format!("{data}d.aggregate(by: g, spend = mean(spend), twice = spend * 2)\n"),
            // No note: the uses come first, or the summary has a name of its own.
            &format!("{data}d.aggregate(by: g, sd_spend = sd(spend), spend = mean(spend))\n"),
            &format!(
                "{data}d.aggregate(by: g, mean_spend = mean(spend), top = max(spend) - mean_spend)\n"
            ),
            &format!("{data}d.add(spend = spend * 2, more = spend + 1)\n"),
        ])
    );
}
