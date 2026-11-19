//! `NA` given a type (Language Reference, section 3.7: the help of Z0219 "gives the type at
//! the use, `to(NA, Int)`"; release 0.8.0, step 20h): `add(v = to(NA, Float))` makes a variable
//! of that type with no value in any row. The help of Z0219 had said since 0.4.0 that this
//! "arrives with release 0.5.0"; it had not arrived.

use zeff_testkit::run;

const DATA: &str = "let d = frame(x = [1.0, 2.5, 4.0], g = [1, 1, 2])\n";

#[test]
fn na_given_a_type_is_a_variable_of_that_type() {
    let script = format!(
        "{DATA}let e = d.add(f = to(NA, Float), i = to(NA, Int), s = to(NA, Str), b = to(NA, Bool), day = to(NA, Date), then = to(NA, DateTime))\ne.describe()\ne.add(sum = x + f, known = is_missing(i))\nd.add(f = to(NA, Float), by: g)[\"f\"]\ne.update(f = x * 2, where: g == 1)[\"f\"]\n"
    );
    let outcome = run(&script);
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    insta::assert_snapshot!(
        "na_given_a_type",
        format!("{}{}", outcome.output.stdout, outcome.output.stderr)
    );
}

#[test]
fn na_alone_still_has_no_type_and_the_help_says_how_to_give_one() {
    let outcome = run(&format!("{DATA}d.add(v = NA)\n"));
    assert_eq!(outcome.exit_code, 1);
    let said = outcome.output.stderr;
    assert!(
        said.contains("error[Z0219]: `v = NA` has no type"),
        "{said}"
    );
    assert!(
        said.contains("`to(NA, Float)` gives it one, as does a variable of the frame, `x * NA`"),
        "{said}"
    );
    assert!(!said.contains("arrives with"), "{said}");
}
