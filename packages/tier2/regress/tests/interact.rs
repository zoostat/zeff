//! `interact` (Language Reference, section 7.2; release 0.4.0, steps 8 and 9): admitted inside a
//! formula with its page's forms, refused outside one, and every message of the checker.

use zeff_testkit::run;

#[test]
fn interact_is_a_term_and_nothing_else() {
    let scripts = [
        "let f = y ~ interact([a, b, c], order: 2) + d\nf\nf + interact([category(e), g], order: 2)\nlet a = 1\nlet b = 2\ninteract([a, b], order: 2)\n",
        "let f = y ~ interact(a, order: 2)\n",
        "let f = y ~ interact([a, b], order: \"two\")\n",
        "let f = y ~ interact([a, b], order: 1)\nf\n",
    ];
    let mut all = String::new();
    for script in scripts {
        let outcome = run(script);
        all.push_str(&format!(
            "> {script}{}{}[exit {}]\n\n",
            outcome.output.stdout, outcome.output.stderr, outcome.exit_code
        ));
    }
    insta::assert_snapshot!("interact", all);
}
