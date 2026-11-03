//! The script tests (testing handbook, section 1, "Script tests"; release 0.5.0, step 23, and
//! release 0.6.0, step 23b): the statements of sections 5.1 to 5.5 of the Language Reference run
//! as one script in a project holding the module of section 5.3; the statements of sections 6
//! and 6.1 (every verb of the table, the shared arguments, panels, row order, the import options
//! and `save`) and of section 7.1 (every form of the formula table and the arguments `fe:` and
//! `cluster:`) as scripts of their own; the examples of the help document's pages that this
//! release can run (`reg` with and without `fe:`, `where`, `merge`, `dist.normal`, `attempt` and
//! the project function), each against its stored output; and the specification's sample script
//! checked with the `robustness` module its section 6 shows, so that what remains refused is the
//! functions and constructs of later releases.
//!
//! The statements are adapted where the shipped data or the release differ from the text:
//! `fe:` (release 0.6.0) is left out, `logistic` (0.9.0) is `reg`, the subsamples are the
//! wages' years rather than a simulation's replications, and the default cluster is the
//! `worker` the wages have rather than `firm`. The EM of section 5.5 runs on one year of the
//! wages (1,091 rows with a wage), so that even the 500 passes of its `max:`, which a mutated
//! build reaches when nothing converges, take seconds and not the minutes the weekly mutation
//! run allows a mutant for the whole suite.

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

/// A project with the two modules of section 5.3, the data files given (by their path under
/// `data/`) and the script given.
fn run_with_data(files: &[(&str, &[u8])], script: &str) -> Outcome {
    let fs = MemoryFileSystem::new();
    fs.add_file(
        "/p/zeff.toml",
        format!("[project]\nname = \"p\"\nzeff = \"{VERSION}\"\nedition = \"preview\"\n"),
    );
    fs.add_file("/p/functions/share_missing.zeff", SHARE_MISSING);
    fs.add_file("/p/functions/robustness.zeff", ROBUSTNESS);
    for (name, bytes) in files {
        fs.add_file(format!("/p/data/{name}"), bytes);
    }
    let project = Project::open(&fs, Path::new("/p")).unwrap();
    run_script(&fs, &project, "scripts/main.zeff", script)
}

/// A project with the two modules of section 5.3 and the script given.
fn run(script: &str) -> Outcome {
    run_with_data(&[], script)
}

/// What a script printed and its messages, run with the data files given, for a snapshot.
fn both_with_data(files: &[(&str, &[u8])], script: &str) -> String {
    let outcome = run_with_data(files, script);
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    format!("{}{}", outcome.output.stdout, outcome.output.stderr)
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
let fit = iterate(start: s0, step: em_step, args: {data: wages.where(year == 2020 && !is_missing(wage))}, done: converged, max: 500)
fit.loglik
";

#[test]
fn the_statements_of_sections_5_1_to_5_5() {
    insta::assert_snapshot!("sections_5_1_to_5_5", both(SECTIONS));
}

// ----- sections 6 and 6.1 -----

/// The wide sales file of the panel sample (`samples/panel/make_data.py`), the opening example
/// of section 6 reshaped, merged and aggregated.
const SALES_WIDE: &[u8] = include_bytes!("../../../samples/panel/data/sales_wide.csv");

/// The counties of the panel sample, for `treated * post` (section 7.1).
const COUNTIES: &[u8] = include_bytes!("../../../samples/panel/data/counties.csv");

/// A file as a German spreadsheet exports it, for the import options of section 6.1: a line to
/// skip, `;` between fields, headers with spaces, a decimal comma and a point between groups of
/// three digits, and -99 for a missing value.
const MESSY: &[u8] = "exported 2026-01-05
Staff Id;Monthly Wage;Weekly Hours;Region
1;2.345,50;40,0;K\u{f6}ln
2;-99;37,5;Z\u{fc}rich
3;1.980,25;-99;M\u{e1}laga
"
.as_bytes();

/// The statements of sections 6 and 6.1, in order: the opening chain, statistics as methods
/// and reducers, every verb of the table (`tabulate` arrives with 0.7.0), the shared arguments
/// `by:`, `cluster:` and `seed:`, the panel declaration kept and dropped, the row order of
/// `sort`, `merge` and `append`, and the import options and `save` of section 6.1.
const SECTIONS_6: &str = "// the opening example
let firms = dataset(firms)
let ind_year = import(\"data/sales_wide.csv\")
    .to_long(\"sales_\", id: firm, names_to: \"year\", values_to: \"sales\", names_type: Int)
    .merge(firms, on: firm, relation: many_to_one)
    .aggregate(avg_sales = mean(sales), firms = n(), by: [industry, year])
ind_year.head(4)

// statistics as methods and reducers
let wages = dataset(wages).add(lwage = log(wage))
wages.mean(lwage)
wages.add(dev = lwage - mean(lwage), by: firm).select([worker, year, firm, lwage, dev]).head(3)

// look
wages.n_rows()
wages.n_cols()
wages.variables()
wages.describe()
let few = wages.select([worker, year, wage, hours, female, industry, firm])
few.head(2)
few.tail(2)
wages.summary([lwage, hours], by: female)

// rows
few.where(year >= 2015).n_rows()
few.sort([year, worker]).head(3)
few.sort([industry], desc: true).head(2)
few.distinct([industry]).n_rows()
few.duplicates([worker, year])
few.sample(n: 3, seed: 7)
few.drop_missing([wage, hours]).n_rows()

// variables
let small = few.head(4)
small.drop([hours])
small.rename(pay = wage)
small.add(wage = wage * 2)
small.update(wage = 0, where: hours < 45)
small.label(female, [0 => \"male\", 1 => \"female\"])
small.recode(industry, [\"Retail\" => \"Trade\"])

// combine
attempt(small.merge(firms, on: firm))
small.merge(firms, on: firm, suffixes: [\"_w\", \"_f\"]).select([worker, firm, industry_w, industry_f, size])
small.append(small.head(1))

// reshape
let sales = frame(firm = [1, 2], region = [\"North\", \"South\"], sales_2010 = [120, 80], sales_2011 = [130, NA])
let long = sales.to_long(\"sales_\", id: firm, names_to: \"year\", values_to: \"sales\", names_type: Int)
long
long.to_wide(values_from: sales, id: firm, names_from: year)

// panels
let p = frame(firm = [2, 1, 2, 1, 2], year = [2021, 2020, 2020, 2021, 2023], revenue = [12.5, 8, 11, 9, NA]).panel(unit: firm, time: year)
p.add(prev = lag(revenue), next = lead(revenue), change = diff(revenue), filled = fill_forward(revenue))
p.where(year > 2020)
p.aggregate(total = sum(revenue), by: firm)

// row order
frame(k = [2, NA, 1, 2], v = [1, 2, 3, 4]).sort([k])
frame(k = [2, 1, 2], v = [\"a\", \"b\", \"c\"]).distinct([k])
frame(k = [2, 1]).merge(frame(k = [1, 1, 2], w = [10, 11, 12]), on: k, relation: one_to_many)

// 6.1: the import options, then save and import again
let messy = import(\"data/messy.csv\", delim: \";\", skip: 1, na_values: [-99], decimal: \",\", thousands: \".\", names: snake)
messy
messy.save(\"output/messy.csv\")
import(\"output/messy.csv\")
messy.save(\"output/messy.tsv\")
import(\"output/messy.tsv\").n_rows()
";

#[test]
fn the_statements_of_sections_6_and_6_1() {
    let text = both_with_data(
        &[("sales_wide.csv", SALES_WIDE), ("messy.csv", MESSY)],
        SECTIONS_6,
    );
    // The merge without suffixes is the error of section 6, kept as a value by `attempt`.
    assert!(text.contains("Z0604"), "{text}");
    // The panel declaration survives `where` and is dropped by `aggregate` with a note.
    assert!(text.contains("panel firm × year"), "{text}");
    assert!(text.contains("panel"), "{text}");
    insta::assert_snapshot!("sections_6_and_6_1", text);
}

// ----- section 7.1 -----

/// Every form of the formula table of section 7.1 on the shipped wages (the ranges on variables
/// made for them), the arguments `fe:` with `+` and `:` and `cluster:`, `treated * post` on the
/// counties, and the two fits of the section's code block in one table (`ivreg` and `mixed` are
/// later releases).
const SECTION_7_1: &str = "let wages = dataset(wages).add(lwage = log(wage), x1 = educ, x2 = exper, x3 = age, z1 = hours, z2 = union, z3 = female)
let f = lwage ~ educ + exper
wages.reg(f).coef(educ)
wages.reg(lwage ~ educ + exper^2 + log(hours)).estimates()
wages.reg(lwage ~ educ:female + female).estimates()
wages.reg(lwage ~ category(industry) + educ).estimates().head(4)
wages.reg(lwage ~ base(industry, \"Retail\") + educ).estimates().head(4)
wages.reg(lwage ~ x1..x3 + (z1..z3)).estimates()
wages.reg(f + educ:female).estimates()
wages.reg(lwage ~ educ - 1).estimates()
wages.reg(lwage ~ 1).estimates()
wages.reg(f - exper).estimates()
wages.reg(f, fe: firm + year).estimates()
wages.reg(f, fe: industry:year).estimates()
let counties = import(\"data/counties.csv\")
counties.reg(emp ~ treated * post).estimates()
let m1 = wages.reg(lwage ~ educ + exper + exper^2, se: robust)
let m2 = wages.reg(lwage ~ educ + exper + exper^2, fe: firm + year, cluster: firm)
report([m1, m2])
";

#[test]
fn the_statements_of_section_7_1() {
    let text = both_with_data(&[("counties.csv", COUNTIES)], SECTION_7_1);
    assert!(text.contains("treated:post"), "{text}");
    insta::assert_snapshot!("section_7_1", text);
    // `*` inside `fe:` is refused before the script runs, with help.
    let outcome = run("let wages = dataset(wages)\nwages.reg(log(wage) ~ educ, fe: firm * year)\n");
    assert_eq!(outcome.exit_code, 1);
    assert!(
        outcome.output.stderr.contains("fe:"),
        "{}",
        outcome.output.stderr
    );
    insta::assert_snapshot!("section_7_1_star_in_fe", outcome.output.stderr);
}

// ----- the help document's examples -----

/// The examples of the pages `zeff help reg`, `where`, `merge`, `dist.normal` and `attempt` and
/// of the project function, as far as this release runs them: the export of the table kept out
/// of the comparison, the merge's error kept as a value by `attempt`, and the `attempt` loop
/// over the wages' years.
const HELP_EXAMPLES: &str = "use functions::share_missing
let wages = import(\"zeff:wages.csv\").add(lwage = log(wage))
// reg
let m = wages.reg(lwage ~ educ + exper + exper^2 + female, cluster: worker)
m.coef(educ)
m.std_errors(educ)
let m = wages.reg(lwage ~ educ + exper + exper^2 + female, fe: firm, cluster: firm)
m.coef(educ)
m.std_errors(educ)
// merge
let firms = import(\"zeff:firms.csv\")
let merged = wages.merge(firms.select([firm, size, founded]), on: firm, relation: many_to_one)
merged.n_cols()
attempt(merged.merge(firms, on: firm))
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
