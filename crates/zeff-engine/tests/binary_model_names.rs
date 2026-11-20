//! The names of the binary models and of the ROC curve before the 1.0 freeze (open decision
//! #112; release 0.9.0, step 2). The catalogue named `logistic`, `probit` and `fit.roc()` for
//! this release; the owner's decision of 11 November 2026 gave them one function each with an
//! option, `glm(y ~ x, family: binomial, link: logit | probit = logit)` and
//! `fit.curve(type: roc)`. A script that writes an old name is refused with the call to write
//! instead, and so is `d.logit(y ~ x)`, since `logit` is the maths function of a number; `zeff
//! help` answers the old names the same way.

use zeff_testkit::run;

fn said(script: &str) -> String {
    let outcome = run(script);
    format!(
        "{script}---- exit code {}\n{}{}",
        outcome.exit_code, outcome.output.stdout, outcome.output.stderr
    )
}

#[test]
fn the_names_taken_out_say_what_to_write() {
    let all = [
        "dataset(wages).logistic(union ~ educ)\n",
        "dataset(wages).probit(union ~ educ)\n",
        "dataset(wages).logit(union ~ educ)\n",
        "dataset(wages).reg(log(wage) ~ educ).roc()\n",
        // The maths functions keep their names and their meaning.
        "logit(0.25)\n",
    ]
    .iter()
    .map(|script| said(script))
    .collect::<Vec<_>>()
    .join("\n====\n");
    insta::assert_snapshot!("binary_model_names", all);
}

#[test]
fn help_answers_the_old_names() {
    for (name, written) in [
        ("logistic", "data.glm(y ~ x, family: binomial)"),
        ("probit", "data.glm(y ~ x, family: binomial, link: probit)"),
        ("roc", "fit.curve(type: roc)"),
    ] {
        let registry = zeff_engine::registry::registry().expect("the registry builds");
        let page = zeff_engine::help::help_text(registry, name).expect("an answer");
        assert!(page.contains(written), "{name}: {page}");
    }
}
