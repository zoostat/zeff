//! The messages an R user meets first (release 0.9.0, step 35; open decision #196, option A).
//! Three things were said before that helped nobody: `I(exper^2)` in a formula was offered
//! `PI`; `poly(exper, 2)` was sent to the list of functions; and a function the catalogue has
//! for a later release was treated as a misspelling. Each case names what was said before; the
//! snapshots hold what is said now.

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

const WAGES: &str = "let w = dataset(wages).where(year == 2019)\n";

/// `I(...)` inside a formula. Before: "did you mean `PI`?". Now the term as Zeff writes it,
/// made from what the script wrote: as it stands for a power or a call, in brackets for the
/// operators that are the formula's own outside them.
#[test]
fn i_in_a_formula_is_answered_with_the_term_without_it() {
    let text = all(&[
        &format!("{WAGES}w.reg(log(wage) ~ educ + exper + I(exper^2))\n"),
        &format!("{WAGES}w.reg(log(wage) ~ educ + I(exper * hours))\n"),
        &format!("{WAGES}w.reg(log(wage) ~ educ + I(log(hours)))\n"),
        &format!("{WAGES}w.reg(log(wage) ~ educ + I((exper - 10)^2))\n"),
        &format!("{WAGES}w.reg(log(wage) ~ educ + I(exper / 10))\n"),
        &format!("{WAGES}w.reg(log(wage) ~ educ + I())\n"),
    ]);
    insta::assert_snapshot!("i_in_a_formula", text);
    assert!(!text.contains("PI"), "{text}");
    assert!(
        text.contains("so `I()` is not needed: write `exper^2`\n"),
        "{text}"
    );
    assert!(
        text.contains("write `(exper * hours)`, since operators inside brackets are arithmetic\n"),
        "{text}"
    );
    // What each help line offers is a formula Zeff fits, with the term named as written.
    let fits = all(&[
        &format!("{WAGES}w.reg(log(wage) ~ educ + exper + exper^2).coef(`exper^2`)\n"),
        &format!("{WAGES}w.reg(log(wage) ~ educ + (exper * hours)).coef(`(exper * hours)`)\n"),
        &format!("{WAGES}w.reg(log(wage) ~ educ + (exper / 10)).coef(`(exper / 10)`)\n"),
    ]);
    insta::assert_snapshot!("i_in_a_formula_rewritten", fits);
    assert!(!fits.contains("error"), "{fits}");
}

/// `poly(x, k)` inside a formula. Before: "`zeff help` lists the functions of this release".
/// Now the powers written out to the degree the script gave, and the contrasts of an ordered
/// category, the other use of R's `poly`.
#[test]
fn poly_in_a_formula_is_answered_with_its_powers() {
    let text = all(&[
        &format!("{WAGES}w.reg(log(wage) ~ educ + poly(exper, 2))\n"),
        &format!("{WAGES}w.reg(log(wage) ~ educ + poly(exper, 3))\n"),
        &format!("{WAGES}w.reg(log(wage) ~ educ + poly(exper))\n"),
        &format!("{WAGES}w.reg(log(wage) ~ educ + poly(log(hours), degree: 7))\n"),
        &format!("{WAGES}w.reg(log(wage) ~ educ + poly(exper - 10, 2))\n"),
        &format!("{WAGES}w.reg(log(wage) ~ educ + poly(exper, 1))\n"),
    ]);
    insta::assert_snapshot!("poly_in_a_formula", text);
    assert!(
        text.contains("write the powers out: `exper + exper^2`;"),
        "{text}"
    );
    assert!(
        text.contains("write the powers out: `exper + exper^2 + exper^3`;"),
        "{text}"
    );
    assert!(
        text.contains("`category(x, contrast: poly)` gives the polynomial contrasts"),
        "{text}"
    );
    assert!(!text.contains("lists the functions"), "{text}");
    // Both forms the help names are fitted.
    let fits = all(&[
        &format!("{WAGES}w.reg(log(wage) ~ educ + exper + exper^2 + exper^3).n_obs()\n"),
        &format!("{WAGES}w.reg(log(wage) ~ educ + category(industry, contrast: poly)).n_obs()\n"),
    ]);
    assert!(!fits.contains("error"), "{fits}");
}

/// Outside a formula `I` and `poly` are names like any other: no advice about formulas, and
/// no near name for a name of one letter.
#[test]
fn outside_a_formula_the_two_names_get_no_advice_about_formulas() {
    let text = all(&["let x = I(3)\n", "let p = poly(3, 2)\n"]);
    insta::assert_snapshot!("outside_a_formula", text);
    assert!(!text.contains("PI"), "{text}");
    assert!(!text.contains("formula"), "{text}");
}

/// A name of one letter is offered no neighbour, as a function, a binding, a variable or an
/// option's word; the same letter in the other case is still offered; and names of two
/// letters and more are offered what they were.
#[test]
fn a_name_of_one_letter_is_offered_no_neighbour() {
    let text = all(&[
        "let t = I(3)\n",
        "let t = N(3)\n",
        "let pi = 3\nlet y = p + 1\n",
        "let d = frame(x = [1, 2], yy = [3, 4])\nd.mean(y)\n",
        "let d = frame(x = [1, 2], yy = [3, 4])\nd.mean(X)\n",
        // Longer names keep their suggestions.
        "let q = dataset(wages).maen(wage)\n",
        "let r = Pi * 2\n",
    ]);
    insta::assert_snapshot!("one_letter_names", text);
    assert!(text.contains("did you mean `mean`?"), "{text}");
    assert!(text.contains("did you mean `n`?"), "{text}");
    assert!(
        !text.contains("did you mean `PI`?\n\n====\nlet t = N"),
        "{text}"
    );
}

/// A function the catalogue has for a later release. Before: `did` was offered `div`, and a
/// name with no neighbour was sent to the list of this release's functions. Now the release and
/// the package, as `zeff help <name>` gives them; a name that will be an option of a function
/// is said to be one; a name a release took out still gets its rewrite first.
#[test]
fn a_function_of_a_later_release_is_named_with_its_release() {
    let text = all(&[
        &format!("{WAGES}w.did(wage, treat: union)\n"),
        &format!("{WAGES}w.poisson(union ~ educ)\n"),
        &format!("{WAGES}let c = w.reg(wage ~ educ).contrasts(\"educ = 0\")\n"),
        "let f = map([1, 2], did)\n",
        &format!("{WAGES}w.logistic(union ~ educ)\n"),
        // A name the catalogue does not have is a misspelling or nothing, as before.
        &format!("{WAGES}w.nosuchthing(wage)\n"),
    ]);
    insta::assert_snapshot!("later_release", text);
    assert!(
        text.contains("`did` arrives with a release after 1.0, in the `quasi` package; `zeff help did` says what it will do"),
        "{text}"
    );
    assert!(!text.contains("did you mean `div`"), "{text}");
    assert!(
        text.contains("as an option and not a function of its own"),
        "{text}"
    );
    assert!(text.contains("family: binomial"), "{text}");
}

/// Every name the catalogue has for a later release, as a function or a method, is answered
/// with the words of `zeff help`: none falls back to a near name or to the list.
#[test]
fn every_later_function_of_the_catalogue_is_answered() {
    let catalogue = zeff_registry::Catalogue::compiled().unwrap();
    let engine = zeff_engine::registry::registry().unwrap();
    let mut later: Vec<&str> = catalogue
        .rows()
        .iter()
        .filter(|row| {
            matches!(
                row.kind.as_str(),
                "function" | "frame method" | "result method"
            )
        })
        .map(|row| row.name.as_str())
        .filter(|name| engine.named(name).is_empty() && !name.contains('.'))
        .filter(|name| zeff_registry::replaced(name).is_none())
        .collect();
    later.sort_unstable();
    later.dedup();
    assert!(later.len() > 100, "{}", later.len());
    for name in later {
        let outcome = run(&format!("let z = {name}(1)\n"));
        assert_eq!(outcome.exit_code, 1, "{name}");
        let stderr = &outcome.output.stderr;
        assert!(
            stderr.contains(&format!("= help: `{name}` arrives with ")),
            "{name}: {stderr}"
        );
        assert!(!stderr.contains("did you mean"), "{name}: {stderr}");
    }
}
