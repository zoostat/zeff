//! A distribution written inside a call (release 0.8.0, step 8n; Language Reference, section 4,
//! the row "Specification"): `quantile(dist.t(df: 24), 0.975)` is the quantile of the
//! distribution, as `quantile(d, 0.975)` is for a bound one and `dist.t(df: 24).quantile(0.975)`
//! in the method form. The checker chooses the home of a name by the type of the first
//! argument whatever its form; before this step it knew the type of a binding alone, took an
//! inline distribution for a frame and refused the call (Z0401). Every family, with `pdf`,
//! `cdf`, `quantile` and `draw`, in the call and the method form: the inline value and the
//! bound one give the same bits.

use zeff_testkit::run;

fn go(script: &str) -> (i32, String, String) {
    let outcome = run(script);
    (
        outcome.exit_code,
        outcome.output.stdout,
        outcome.output.stderr,
    )
}

/// Each family as it is written, with a point for `pdf` and `cdf`.
const FAMILIES: [(&str, &str); 16] = [
    ("dist.normal()", "0.3"),
    ("dist.normal(mean: 1, sd: 2)", "0.3"),
    ("dist.lognormal(meanlog: 0.5, sdlog: 0.4)", "1.7"),
    ("dist.t(df: 24)", "1.1"),
    ("dist.chi2(df: 5)", "3.2"),
    ("dist.f(df1: 3, df2: 20)", "2.4"),
    ("dist.binomial(n: 12, p: 0.3)", "4"),
    ("dist.poisson(lambda: 3.5)", "2"),
    ("dist.negbinomial(size: 4, p: 0.4)", "5"),
    ("dist.negbinomial(mu: 6, size: 4)", "5"),
    ("dist.uniform(min: -1, max: 3)", "0.5"),
    ("dist.exponential(rate: 2)", "0.4"),
    ("dist.gamma(shape: 2.5, rate: 1.5)", "1.2"),
    ("dist.beta(a: 2, b: 5)", "0.25"),
    (
        "dist.truncated(dist.normal(mean: 1, sd: 2), lower: 0)",
        "1.5",
    ),
    (
        "dist.mixture([dist.normal(), dist.normal(mean: 3, sd: 0.5)], probs: [0.3, 0.7])",
        "2.5",
    ),
];

#[test]
fn an_inline_distribution_and_a_bound_one_give_the_same_bits() {
    let mut families: Vec<(String, String)> = FAMILIES
        .iter()
        .map(|(d, x)| (d.to_string(), x.to_string()))
        .collect();
    // The studentized range has no density and no draws; its `cdf` and `quantile` are compared.
    families.push((
        "dist.tukey(groups: 4, df: 20)".to_string(),
        "3.1".to_string(),
    ));
    for (family, x) in &families {
        let tukey = family.starts_with("dist.tukey");
        let mut script = format!("let d = {family}\n");
        let mut expected = String::new();
        let mut same = |inline: String, bound: String| {
            script.push_str(&format!("print({inline} == {bound})\n"));
            expected.push_str("true\n");
        };
        // The call form and the method form, each against the bound distribution.
        if !tukey {
            same(format!("pdf({family}, {x})"), format!("pdf(d, {x})"));
            same(format!("{family}.pdf({x})"), format!("d.pdf({x})"));
        }
        same(format!("cdf({family}, {x})"), format!("cdf(d, {x})"));
        same(format!("{family}.cdf({x})"), format!("d.cdf({x})"));
        same(
            format!("cdf({family}, {x}, upper: true)"),
            format!("d.cdf({x}, upper: true)"),
        );
        same(
            format!("quantile({family}, 0.975)"),
            "quantile(d, 0.975)".to_string(),
        );
        same(
            format!("{family}.quantile(0.2)"),
            "d.quantile(0.2)".to_string(),
        );
        same(
            format!("quantile(({family}), 0.5)"),
            "d.quantile(0.5)".to_string(),
        );
        if !tukey {
            // Draws with one seed are the same values, row by row.
            script.push_str(&format!(
                "print(all(draw({family}, 7, seed: 11) == draw(d, 7, seed: 11)))\nprint(all({family}.draw(7, seed: 11) == d.draw(7, seed: 11)))\n"
            ));
            expected.push_str("true\ntrue\n");
        }
        let (code, stdout, stderr) = go(&script);
        assert_eq!(code, 0, "{family}\n{script}\n{stderr}");
        assert_eq!(stdout, expected, "{family}\n{script}\n{stderr}");
    }
}

/// What prints, stored: the four functions on an inline distribution in both forms, a
/// distribution inside another, and the names that have a home in `data` too still reaching a
/// frame written inline.
#[test]
fn the_function_is_chosen_by_the_first_argument_whatever_its_form() {
    let (code, stdout, stderr) = go(
        "quantile(dist.t(df: 24), 0.975)\ndist.t(df: 24).quantile(0.975)\nquantile(dist.normal(mean: 100, sd: 15), 0.025)\npdf(dist.normal(), 0)\ncdf(dist.chi2(df: 3), 7.81)\ndraw(dist.binomial(n: 10, p: 0.5), 4, seed: 2)\nquantile(dist.truncated(dist.normal(), lower: 0), 0.5)\nlet w = dataset(wages).where(year == 2019)\nquantile(w, wage, p: 0.5)\nquantile(dataset(wages).where(year == 2019), wage, p: 0.5)\nw.quantile(wage, p: 0.5)\nw.aggregate(q = quantile(wage, p: 0.9), cut = quantile(dist.normal(), 0.9))\n",
    );
    assert_eq!(code, 0, "{stderr}");
    insta::assert_snapshot!("inline_distribution", format!("{stdout}---\n{stderr}"));
}
