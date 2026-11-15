//! A range of variables in every parameter that takes variables (release 0.8.0, step 8m;
//! Language Reference, section 6, "Names inside a frame's methods"): `[educ..age]` names the
//! variables from `educ` to `age` by their position in the frame, each once, wherever a list
//! of variables is taken, and gives what the names written out give. The table below has a
//! call for every function of the registry with such a parameter, written in Rust or in Zeff,
//! and the test fails for a function the table lacks, so that a new one cannot be left out.

use zeff_engine::registry::registry;
use zeff_testkit::run;

fn go(script: &str) -> (i32, String, String) {
    let outcome = run(script);
    (
        outcome.exit_code,
        outcome.output.stdout,
        outcome.output.stderr,
    )
}

/// The wages of 2019: worker, year, wage, educ, exper, age, hours, union, female, industry,
/// firm, in that order; and a small frame to reshape.
const FRAMES: &str = "let w = dataset(wages).where(year == 2019)\nlet wide = frame(firm = [1, 2], region = [\"North\", \"South\"], sales_2010 = [120, 80], sales_2011 = [130, NA])\nlet long = frame(firm = [1, 1, 2, 2], region = [\"North\", \"North\", \"South\", \"South\"], year = [2010, 2011, 2010, 2011], sales = [120, 130, 80, NA])\nlet other = w.select([worker, year, hours]).rename(h = hours)\n";

/// A function, its parameter, the call with a range and the call with the names written out.
const CASES: [(&str, &str, &str, &str, &str); 37] = [
    (
        "core",
        "vars",
        "name",
        "w.select(vars(educ..age)).head(3)",
        "w.select(vars(educ, exper, age)).head(3)",
    ),
    (
        "plot",
        "line",
        "by",
        "w.line(educ, wage, by: [union..female])",
        "w.line(educ, wage, by: [union, female])",
    ),
    (
        "plot",
        "bar",
        "by",
        "w.bar(industry, by: [union..female])",
        "w.bar(industry, by: [union, female])",
    ),
    (
        "plot",
        "scatter",
        "by",
        "w.scatter(educ, wage, by: [union..female])",
        "w.scatter(educ, wage, by: [union, female])",
    ),
    (
        "plot",
        "histogram",
        "by",
        "w.histogram(wage, by: [union..female])",
        "w.histogram(wage, by: [union, female])",
    ),
    (
        "plot",
        "density",
        "by",
        "w.density(wage, by: [union..female])",
        "w.density(wage, by: [union, female])",
    ),
    (
        "plot",
        "boxplot",
        "by",
        "w.boxplot(wage, by: [union..female])",
        "w.boxplot(wage, by: [union, female])",
    ),
    (
        "plot",
        "qq_plot",
        "by",
        "w.qq_plot(wage, by: [union..female])",
        "w.qq_plot(wage, by: [union, female])",
    ),
    (
        "plot",
        "heatmap",
        "vars",
        "w.heatmap([wage..age])",
        "w.heatmap([wage, educ, exper, age])",
    ),
    (
        "plot",
        "facet",
        "by",
        "w.scatter(educ, wage).facet(by: [union..female])",
        "w.scatter(educ, wage).facet(by: [union, female])",
    ),
    (
        "data",
        "select",
        "vars",
        "w.select([educ..age]).head(3)",
        "w.select([educ, exper, age]).head(3)",
    ),
    (
        "data",
        "drop",
        "vars",
        "w.drop([educ..age]).head(3)",
        "w.drop([educ, exper, age]).head(3)",
    ),
    (
        "data",
        "add",
        "by",
        "w.add(m = mean(wage), by: [union..female]).head(3)",
        "w.add(m = mean(wage), by: [union, female]).head(3)",
    ),
    (
        "data",
        "update",
        "by",
        "w.update(wage = mean(wage), by: [union..female]).head(3)",
        "w.update(wage = mean(wage), by: [union, female]).head(3)",
    ),
    (
        "data",
        "sort",
        "vars",
        "w.sort([educ..age]).head(3)",
        "w.sort([educ, exper, age]).head(3)",
    ),
    (
        "data",
        "describe",
        "by",
        "w.describe(by: [union..female])",
        "w.describe(by: [union, female])",
    ),
    (
        "data",
        "mean",
        "by",
        "w.mean(wage, by: [union..female])",
        "w.mean(wage, by: [union, female])",
    ),
    (
        "data",
        "sd",
        "by",
        "w.sd(wage, by: [union..female])",
        "w.sd(wage, by: [union, female])",
    ),
    (
        "data",
        "variance",
        "by",
        "w.variance(wage, by: [union..female])",
        "w.variance(wage, by: [union, female])",
    ),
    (
        "data",
        "median",
        "by",
        "w.median(wage, by: [union..female])",
        "w.median(wage, by: [union, female])",
    ),
    (
        "data",
        "quantile",
        "by",
        "w.quantile(wage, p: 0.9, by: [union..female])",
        "w.quantile(wage, p: 0.9, by: [union, female])",
    ),
    (
        "data",
        "min",
        "by",
        "w.min(wage, by: [union..female])",
        "w.min(wage, by: [union, female])",
    ),
    (
        "data",
        "max",
        "by",
        "w.max(wage, by: [union..female])",
        "w.max(wage, by: [union, female])",
    ),
    (
        "data",
        "sum",
        "by",
        "w.sum(wage, by: [union..female])",
        "w.sum(wage, by: [union, female])",
    ),
    (
        "data",
        "summary",
        "vars",
        "w.summary([educ..age])",
        "w.summary([educ, exper, age])",
    ),
    (
        "data",
        "summary",
        "by",
        "w.summary([wage], by: [union..female])",
        "w.summary([wage], by: [union, female])",
    ),
    (
        "data",
        "tabulate",
        "vars",
        "w.tabulate([union..female])",
        "w.tabulate([union, female])",
    ),
    (
        "data",
        "distinct",
        "vars",
        "w.distinct([union..female])",
        "w.distinct([union, female])",
    ),
    (
        "data",
        "duplicates",
        "vars",
        "w.duplicates([union..female])",
        "w.duplicates([union, female])",
    ),
    (
        "data",
        "aggregate",
        "by",
        "w.aggregate(m = mean(wage), by: [union..female])",
        "w.aggregate(m = mean(wage), by: [union, female])",
    ),
    (
        "data",
        "merge",
        "on",
        "w.merge(other, on: [worker..year]).head(3)",
        "w.merge(other, on: [worker, year]).head(3)",
    ),
    (
        "data",
        "to_long",
        "id",
        "wide.to_long(\"sales_\", id: [firm..region], names_to: \"year\", values_to: \"sales\", names_type: Int)",
        "wide.to_long(\"sales_\", id: [firm, region], names_to: \"year\", values_to: \"sales\", names_type: Int)",
    ),
    (
        "data",
        "to_wide",
        "id",
        "long.to_wide(values_from: sales, id: [firm..region], names_from: year)",
        "long.to_wide(values_from: sales, id: [firm, region], names_from: year)",
    ),
    (
        "data",
        "drop_missing",
        "vars",
        "w.drop_missing([wage..exper]).n_rows()",
        "w.drop_missing([wage, educ, exper]).n_rows()",
    ),
    (
        "data",
        "row",
        "vars",
        "w.add(s = row([educ..age], stat: sum)).head(3)",
        "w.add(s = row([educ, exper, age], stat: sum)).head(3)",
    ),
    (
        "stats",
        "correlation",
        "vars",
        "w.correlation([educ..age])",
        "w.correlation([educ, exper, age])",
    ),
    (
        "stats",
        "covariance",
        "vars",
        "w.covariance([educ..age])",
        "w.covariance([educ, exper, age])",
    ),
];

/// Whether a declared type takes a list of variables.
fn takes_variables(ty: &str) -> bool {
    ty.split('|').any(|part| part.trim() == "Vars")
}

#[test]
fn a_range_gives_what_its_names_give_in_every_parameter_that_takes_variables() {
    let mut problems = Vec::new();
    for (package, name, param, ranged, named) in CASES {
        let (code_r, out_r, err_r) = go(&format!("{FRAMES}{ranged}\n"));
        let (code_n, out_n, err_n) = go(&format!("{FRAMES}{named}\n"));
        if code_n != 0 {
            problems.push(format!(
                "{package}::{name}({param}:) the names written out fail:\n{err_n}"
            ));
        } else if (code_r, &out_r, &err_r) != (code_n, &out_n, &err_n) {
            problems.push(format!(
                "{package}::{name}({param}:) `{ranged}` and `{named}` differ:\n{out_r}{err_r}--- against ---\n{out_n}{err_n}"
            ));
        }
    }
    assert!(problems.is_empty(), "\n{}", problems.join("\n"));
}

#[test]
fn the_table_has_every_parameter_of_the_registry_that_takes_variables() {
    let registry = registry().unwrap_or_else(|e| panic!("{e:?}"));
    let mut missing = Vec::new();
    for function in registry.functions() {
        for param in &function.spec.params {
            if takes_variables(param.ty)
                && !CASES.iter().any(|(package, name, p, _, _)| {
                    *package == function.spec.package
                        && *name == function.spec.name
                        && *p == param.name
                })
            {
                missing.push(format!(
                    "{}::{}({}:)",
                    function.spec.package, function.spec.name, param.name
                ));
            }
        }
    }
    assert!(
        missing.is_empty(),
        "parameters that take variables without a case in this file: {missing:?}"
    );
}

/// A range out of order, a range to a variable the frame lacks, and a range repeated: refused
/// or resolved in the words `select` uses, whatever the function.
#[test]
fn a_range_out_of_order_is_refused_as_select_refuses_it() {
    let mut all = String::new();
    for script in [
        "w.select([age..educ])\n",
        "w.summary([age..educ])\n",
        "w.mean(wage, by: [female..union])\n",
        "w.aggregate(m = mean(wage), by: [female..union])\n",
        "w.tabulate([female..union])\n",
        "w.correlation([age..educ])\n",
        "w.histogram(wage, by: [female..union])\n",
        "w.summary([educ..nope])\n",
        "w.aggregate(m = mean(wage), by: [union..nope])\n",
        "w.correlation([nope..age])\n",
    ] {
        let (code, stdout, stderr) = go(&format!("{FRAMES}{script}"));
        assert_eq!(code, 1, "{script}\n{stdout}{stderr}");
        all.push_str(&format!("{script}{stderr}\n"));
    }
    // A name repeated by a range counts once, as in `select`.
    let (code, stdout, stderr) = go(&format!(
        "{FRAMES}w.summary([educ..age, exper, educ])\nw.select([educ..age, exper]).n_cols()\n"
    ));
    assert_eq!(code, 0, "{stderr}");
    all.push_str(&format!("{stdout}{stderr}"));
    insta::assert_snapshot!("ranges_out_of_order", all);
}
