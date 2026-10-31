//! The script tests of release 0.5.0 (testing handbook, section 1, "Script tests"; release
//! 0.5.0, step 23): the statements of sections 5.1 to 5.5 of the Language Reference run as one
//! script in a project holding the module of section 5.3, and the examples of the help
//! document's pages that this release can run (`reg`, `where`, `dist.normal`, `attempt` and the
//! project function), each against its stored output; and the specification's sample script
//! checked with the `robustness` module its section 6 shows, so that what remains refused is the
//! functions and constructs of later releases.
//!
//! The statements are adapted where the shipped data or the release differ from the text:
//! `fe:` (release 0.6.0) is left out, `logistic` (0.9.0) is `reg`, the subsamples are the
//! wages' years rather than a simulation's replications, and the default cluster is the
//! `worker` the wages have rather than `firm`.

use std::path::Path;
use zeff_diagnostics::{Source, Sources};
use zeff_engine::VERSION;
use zeff_engine::fs::MemoryFileSystem;
use zeff_engine::project::Project;
use zeff_engine::run::{Outcome, run_script};
use zeff_ir::ModuleSource;

/// The module of section 5.3, as the help document's last page writes it.
const SHARE_MISSING: &str = "/// Share of rows where a variable is missing.
///
/// Counts the rows where `var` is `NA` and divides by the row count.
///
/// data: The frame to check.
/// var: The variable whose missing values are counted.
/// returns: The share, a Float between 0 and 1.
/// see also: n_missing, is_missing
///
/// > dataset(wages).share_missing(wage)
/// 0.0114599
fn share_missing(data: Frame, var: Var) -> Float {
    data.where(is_missing(var)).n_rows() / data.n_rows()
}
";

/// The second function of section 5.3, without `fe:` and clustered by the worker by default.
const ROBUSTNESS: &str = "/// Three standard robustness checks for one specification.
///
/// Robust errors, errors clustered by `cluster`, and robust errors from 2015 on.
///
/// data: The frame to fit on.
/// f: The formula.
/// cluster: The variable to cluster by.
/// returns: The three fits.
///
/// > len(dataset(wages).robustness(log(wage) ~ educ))
/// 3
fn robustness(data: Frame, f: Formula, cluster: Var = worker) -> List<RegFit> {
    [
        data.reg(f, se: robust),
        data.reg(f, cluster: cluster),
        data.where(year >= 2015).reg(f, se: robust),
    ]
}
";

/// A project with the two modules of section 5.3 and the script given.
fn run(script: &str) -> Outcome {
    let fs = MemoryFileSystem::new();
    fs.add_file(
        "/p/zeff.toml",
        format!("[project]\nname = \"p\"\nzeff = \"{VERSION}\"\nedition = \"preview\"\n"),
    );
    fs.add_file("/p/functions/share_missing.zeff", SHARE_MISSING);
    fs.add_file("/p/functions/robustness.zeff", ROBUSTNESS);
    let project = Project::open(&fs, Path::new("/p")).unwrap();
    run_script(&fs, &project, "scripts/main.zeff", script)
}

/// What a script printed and its messages, one after the other, for a snapshot.
fn both(script: &str) -> String {
    let outcome = run(script);
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    format!("{}{}", outcome.output.stdout, outcome.output.stderr)
}

// ----- sections 5.1 to 5.5 -----

/// The statements of sections 5.1 to 5.5, in order, on the shipped wages.
const SECTIONS: &str = "use functions::share_missing
use functions::robustness
let wages = dataset(wages).add(lwage = log(wage))

// 5.1
if wages.n_rows() < 30 {
    warn(\"fewer than 30 rows: treat large-sample tests with care\")
}
let reps = if wages.n_rows() < 500 { 5000 } else { 1000 }
reps
let wages = wages.add(age_group = case {
    age < 25 => \"young\",
    age < 55 => \"prime\",
    _        => \"older\",
})
wages.summary([lwage], by: age_group)

// 5.2
let fits = [for y in vars(lwage, hours, union) {
    wages.reg(y ~ educ + exper, cluster: worker)
}]
report(fits)

// 5.3
wages.share_missing(educ)
report(wages.robustness(lwage ~ educ + exper))

// 5.4
let fits = [for s in 1..5 {
    attempt(wages.where(year == 2008 + 3 * s).reg(lwage ~ educ + exper))
}]
report(fits)

// 5.5
/// One E step and one M step for two normals with one sd.
fn em_step(s: Record, data: Frame) -> Record {
    let d = data.add(
        p1 = (1 - s.share) * dist.normal(s.mu1, s.sd).pdf(wage),
        p2 = s.share * dist.normal(s.mu2, s.sd).pdf(wage),
        r = p2 / (p1 + p2),
        ll = log(p1 + p2),
        q = 1 - r)
    let mu1 = d.mean(wage, weights: q)
    let mu2 = d.mean(wage, weights: r)
    let e = d.add(sq = q * (wage - mu1)^2 + r * (wage - mu2)^2)
    {mu1: mu1, mu2: mu2, sd: sqrt(e.mean(sq)), share: d.mean(r), loglik: d.sum(ll)}
}
/// True when the log-likelihood has stopped improving.
fn converged(old: Record, new: Record) -> Bool {
    abs(new.loglik - old.loglik) < 1e-8 * abs(new.loglik)
}
let s0 = {mu1: 10.0, mu2: 30.0, sd: 10.0, share: 0.5, loglik: -1e300}
let fit = iterate(start: s0, step: em_step, args: {data: wages.where(!is_missing(wage))}, done: converged, max: 500)
fit.loglik
";

#[test]
fn the_statements_of_sections_5_1_to_5_5() {
    insta::assert_snapshot!("sections_5_1_to_5_5", both(SECTIONS));
}

// ----- the help document's examples -----

/// The examples of the pages `zeff help reg`, `where`, `dist.normal` and `attempt` and of the
/// project function, as far as this release runs them: `fe:` left out of the fit, the export
/// of the table kept out of the comparison, and the `attempt` loop over the wages' years.
const HELP_EXAMPLES: &str = "use functions::share_missing
let wages = import(\"zeff:wages.csv\").add(lwage = log(wage))
// reg
let m = wages.reg(lwage ~ educ + exper + exper^2 + female, cluster: worker)
m.coef(educ)
m.std_errors(educ)
// where
wages.where(year >= 2010 && female == 1).n_rows()
wages.where(exper > 10).n_rows()
// dist.normal
dist.normal().cdf(1.96)
dist.normal().quantile(0.975)
dist.normal().cdf(30, upper: true)
dist.normal(mean: 100, sd: 15).draw(5, seed: 42)
// attempt
let fits = [for s in 1..5 {
    attempt(wages.where(year == 2008 + 3 * s).reg(lwage ~ educ + exper))
}]
report(fits)
// the project function
wages.share_missing(educ)
wages.share_missing(wage)
";

#[test]
fn the_examples_of_the_help_document_that_this_release_runs() {
    let text = both(HELP_EXAMPLES);
    // The far tail keeps its digits, as the page promises.
    assert!(text.contains("4.90671e-198"), "{text}");
    assert!(text.contains("0.975002\n1.95996\n"), "{text}");
    insta::assert_snapshot!("help_examples", text);
}

// ----- the specification's sample script with its module -----

/// The `robustness` module that section 6 of the sample script shows in a comment.
fn module_of_section_6(text: &str) -> String {
    let mut lines = text
        .lines()
        .skip_while(|l| !l.contains("functions/robustness.zeff contains"));
    lines.next();
    let mut module = String::new();
    for line in lines {
        if let Some(code) = line.strip_prefix("//     ") {
            module.push_str(code);
            module.push('\n');
        } else if line.trim() != "//" {
            break;
        }
    }
    assert!(module.contains("fn robustness("), "{module}");
    module
}

#[test]
fn the_sample_script_of_the_specification_checks_with_its_module() {
    let text = include_str!("../../../docs/spec/zeff_samples.zeff");
    let module = module_of_section_6(text);
    let registry = zeff_engine::registry::registry().expect("the registry builds");
    let parsed = zeff_syntax::parse("zeff_samples.zeff", text);
    let parsed_module = zeff_syntax::parse("functions/robustness.zeff", &module);
    let modules = [ModuleSource {
        name: "robustness".to_string(),
        file: "functions/robustness.zeff".to_string(),
        text: module.clone(),
        script: parsed_module.script,
    }];
    let checked = zeff_ir::check_project(
        "zeff_samples.zeff",
        &parsed.script,
        registry,
        Some(text),
        &modules,
    );
    let mut sources = Sources::new();
    sources.add(Source::new("zeff_samples.zeff", text));
    sources.add(Source::new("functions/robustness.zeff", module));
    let rendered: Vec<String> = checked
        .diagnostics
        .iter()
        .map(|d| {
            d.render(&sources)
                .lines()
                .take(2)
                .map(|l| l.trim_end().to_string())
                .collect::<Vec<_>>()
                .join("\n")
        })
        .collect();
    let all = rendered.join("\n");
    // The module is found and its function called; nothing of sections 5.1 to 5.5 is refused.
    for refused in [
        "no module is called",
        "`robustness`",
        "`fn`",
        "`for`",
        "`attempt`",
        "`iterate`",
        "`case`",
        "a label",
    ] {
        assert!(!all.contains(refused), "{refused}:\n{all}");
    }
    // The constructs still refused are the checker's remaining ones: `model { }` and `dag { }`
    // blocks, labels and third-party packages.
    for line in rendered.iter().filter(|l| l.starts_with("error[Z0304]")) {
        assert!(
            line.contains("`model { }`") || line.contains("`dag { }`") || line.contains("`use "),
            "{line}"
        );
    }
    insta::assert_snapshot!("sample_script_with_module", all);
}
