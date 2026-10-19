//! The `Distribution` value and the `dist.` namespace through the engine (Language Reference,
//! sections 3.7 "Namespaces" and 4; release 0.3.0, step 3): a distribution prints as its family
//! and parameters with the defaults filled in; a parameter outside its domain is Z0413; `dist` is
//! a namespace and never a binding; an unknown family is Z0103 with a suggestion.

use zeff_testkit::run;

fn go(script: &str) -> (i32, String, String) {
    let outcome = run(script);
    (
        outcome.exit_code,
        outcome.output.stdout,
        outcome.output.stderr,
    )
}

#[test]
fn a_distribution_prints_as_its_call_with_the_defaults_filled_in() {
    let (code, stdout, stderr) = go(
        "dist.normal()\ndist.normal(mean: 100, sd: 15)\nlet d = dist.normal(sd: 2.5)\nd\nprint(d, [d])\n",
    );
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(
        stdout,
        "normal distribution · mean: 0, sd: 1\nnormal distribution · mean: 100, sd: 15\nnormal distribution · mean: 0, sd: 2.5\nnormal distribution · mean: 0, sd: 2.5\nnormal distribution · mean: 0, sd: 2.5\n"
    );
    assert_eq!(stderr, "");
}

#[test]
fn a_parameter_outside_its_domain_a_binding_called_dist_and_an_unknown_family_are_refused() {
    let mut all = String::new();
    for script in [
        "dist.normal(sd: 0)\n",
        "dist.normal(mean: 1, sd: -2)\n",
        "dist.normal(sd: sqrt(-1))\n",
        "let dist = 3\n",
        "dist.gaussian(sd: 2)\n",
        "dist.normal().mean(x)\n",
        "dist.normal(sd: \"wide\")\n",
    ] {
        let (code, _, stderr) = go(script);
        assert_eq!(code, 1, "{script}");
        all.push_str(&stderr);
        all.push('\n');
    }
    insta::assert_snapshot!("refusals", all);
}

#[test]
fn the_type_is_checked_before_the_script_runs() {
    // `zeff check` refuses a distribution where a frame is wanted, without running anything.
    let (code, _, stderr) = go("dist.normal().head(3)\n");
    assert_eq!(code, 1);
    assert!(stderr.contains("Z0401"), "{stderr}");
    assert!(stderr.contains("a Distribution"), "{stderr}");
}
