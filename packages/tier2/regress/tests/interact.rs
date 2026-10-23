//! `interact` (Language Reference, section 7.2; release 0.4.0, step 8): admitted inside a formula,
//! refused outside one.

use zeff_testkit::run;

#[test]
fn interact_is_a_term_and_nothing_else() {
    let outcome = run(
        "let f = y ~ interact([a, b, c], order: 2) + d\nf\nlet a = 1\nlet b = 2\ninteract([a, b], order: 2)\n",
    );
    insta::assert_snapshot!(
        "interact",
        format!(
            "{}--- messages ---\n{}[exit {}]\n",
            outcome.output.stdout, outcome.output.stderr, outcome.exit_code
        )
    );
}
