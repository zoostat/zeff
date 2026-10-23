//! Results through the engine (Language Reference, sections 3.7 and 4; release 0.4.0, step 12):
//! `type_of` and `is` on a fit, lists of results and the refusal of a frame method on a fit, with
//! the engine's own functions; then, with fixture result methods registered beside them, dispatch
//! by the receiver's type, `Term` arguments checked against the fit's terms (Z0414) and the
//! printed `TestResult` and its accessors.

use std::path::Path;
use std::rc::Rc;
use zeff_core::display::fixed;
use zeff_core::runtime::{Args, Body, CallContext, Fail};
use zeff_core::value::{Align, Statistic, Table, TestResult, Value};
use zeff_engine::VERSION;
use zeff_engine::fs::MemoryFileSystem;
use zeff_engine::project::Project;
use zeff_engine::registry::{bodies, specs};
use zeff_engine::run::{Outcome, RunOptions, run_script, run_script_in};
use zeff_registry::{Catalogue, FunctionSpec, ParamSpec, Registry, Rest};

/// Twelve rows of `wage = 5 + 0.4 hours + 1.2 educ` with a noise of ±0.3 alternating, so that no
/// one row drives a coefficient.
const WAGES: &str = "id,wage,hours,educ\n1,28.1,30,9\n2,31.9,32,12\n3,30.9,34,10\n4,35.9,36,14\n5,33.7,38,11\n6,39.9,40,16\n7,37.7,42,13\n8,43.9,44,18\n9,41.7,46,15\n10,47.9,48,20\n11,45.7,50,17\n12,48.3,52,19\n";

fn project() -> (MemoryFileSystem, Project) {
    let fs = MemoryFileSystem::new();
    fs.add_file(
        "/p/zeff.toml",
        format!("[project]\nname = \"p\"\nzeff = \"{VERSION}\"\nedition = \"preview\"\n"),
    );
    fs.add_file("/p/data/wages.csv", WAGES);
    let project = Project::open(&fs, Path::new("/p")).unwrap();
    (fs, project)
}

fn run(script: &str) -> Outcome {
    let (fs, project) = project();
    run_script(&fs, &project, "scripts/main.zeff", script)
}

fn go(outcome: Outcome) -> (i32, String, String) {
    let stderr: String = outcome
        .output
        .stderr
        .lines()
        .filter(|l| !l.contains(" read as CSV: "))
        .map(|l| format!("{l}\n"))
        .collect();
    (outcome.exit_code, outcome.output.stdout, stderr)
}

const SETUP: &str =
    "let wages = import(\"data/wages.csv\")\nlet m = wages.reg(wage ~ hours + educ)\n";

#[test]
fn a_fit_has_its_type_and_lists_of_fits_join() {
    let script = format!(
        "{SETUP}type_of(m)\nm.is(Result)\nm.is(RegFit)\nm.is(TestResult)\nm.is(Frame)\n[m, m]\n\
         type_of([m, m])\nlet fits = [m, wages.reg(wage ~ hours)]\nfits[2]\nprint(m)\n"
    );
    let (code, stdout, stderr) = go(run(&script));
    assert_eq!(code, 0, "{stderr}");
    insta::assert_snapshot!("fits", format!("{stdout}--- messages ---\n{stderr}"));
}

#[test]
fn every_message_of_the_engine() {
    let cases = [
        format!("{SETUP}m.head(3)\n"),
        format!("{SETUP}m == m\n"),
        format!("{SETUP}[m, 1]\n"),
        format!("{SETUP}m.mean(wage)\n"),
    ];
    let mut text = String::new();
    for script in &cases {
        let (code, _, stderr) = go(run(script));
        assert_eq!(code, 1, "{script}");
        text.push_str(&stderr);
        text.push('\n');
    }
    insta::assert_snapshot!("every_message", text);
}

// ----- fixture result methods beside the engine's functions -----

/// A complete page with the given arguments, so that the registry accepts the fixture.
fn page(name: &str, args: &[&str], rest: bool) -> &'static str {
    let mut text = format!(
        "# {name}\n\n## Summary\nA fixture for {name}\n\n## Description\nTests only.\n\n## Arguments\n"
    );
    for a in args {
        text.push_str(&format!("- {a}: The {a}.\n"));
    }
    if rest {
        text.push_str("- ...: More.\n");
    }
    text.push_str(&format!(
        "\n## Value\nA value.\n\n## Details\nFixture.\n\n## References\nNone (1900).\n\n## See also\nprint\n\n## Examples\n> {name}\n1\n"
    ));
    Box::leak(text.into_boxed_str())
}

fn fixture(
    name: &'static str,
    package: &'static str,
    params: Vec<ParamSpec>,
    rest: Option<Rest>,
    returns: &'static str,
) -> FunctionSpec {
    let names: Vec<&str> = params.iter().map(|p| p.name).collect();
    let has_rest = rest.is_some();
    FunctionSpec {
        name,
        package,
        method: true,
        params,
        rest,
        returns,
        diagnostics: vec![],
        help: page(name, &names, has_rest),
        accuracy: None,
    }
}

fn fixture_r_squared(_ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    Ok(match args.get(0) {
        Some(Value::Fit(fit)) => Value::Float(fit.r_squared.unwrap_or(f64::NAN)),
        _ => Value::Na,
    })
}

fn test_result(title: &str, table: Option<Table>, df: Vec<f64>, p: f64) -> Value {
    Value::TestResult(Rc::new(TestResult {
        method: "test".to_string(),
        title: title.to_string(),
        header: Vec::new(),
        table,
        statistic: Statistic {
            name: "F".to_string(),
            value: 4.213,
            df,
        },
        p_value: p,
        after: Vec::new(),
        notes: Vec::new(),
        n: 12,
    }))
}

fn fixture_test(_ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let df = match args.get(0) {
        Some(Value::Fit(fit)) => fit.df_residual as f64,
        _ => 0.0,
    };
    Ok(test_result(
        "test · m · hours == 0",
        None,
        vec![1.0, df],
        0.0402,
    ))
}

fn fixture_compare(_ctx: &mut CallContext<'_>, args: Args) -> Result<Value, Fail> {
    let row = |label: &str, value: Option<&Value>| -> Vec<String> {
        match value {
            Some(Value::Fit(fit)) => vec![
                label.to_string(),
                fit.n.to_string(),
                fixed(fit.residual_sum_of_squares, 2),
                fixed(fit.r_squared.unwrap_or(f64::NAN), 3),
            ],
            _ => vec![label.to_string(), "NA".into(), "NA".into(), "NA".into()],
        }
    };
    let table = Table {
        title: String::new(),
        columns: vec![
            ("model".to_string(), Align::Left),
            ("N".to_string(), Align::Right),
            ("RSS".to_string(), Align::Right),
            ("R²".to_string(), Align::Right),
        ],
        rows: vec![row("(1)", args.get(1)), row("(2)", args.get(0))],
        more: Vec::new(),
        before: Vec::new(),
        after: Vec::new(),
    };
    let df = match args.get(0) {
        Some(Value::Fit(fit)) => fit.df_residual as f64,
        _ => 0.0,
    };
    Ok(test_result(
        "compare · m against m",
        Some(table),
        vec![1.0, df],
        0.000_4,
    ))
}

/// The engine's registry and bodies with the fixture result methods added.
fn run_with_fixtures(script: &str) -> Outcome {
    use ParamSpec as P;
    let mut all = specs();
    all.extend([
        fixture(
            "r_squared",
            "regress",
            vec![P::required("fit", "RegFit")],
            None,
            "Float",
        ),
        fixture(
            "test",
            "results",
            vec![P::required("fit", "Result")],
            Some(Rest::Values("Bool")),
            "TestResult",
        ),
        fixture(
            "compare",
            "results",
            vec![P::required("fit", "Result"), P::required("other", "Result")],
            None,
            "TestResult",
        ),
    ]);
    let registry = Registry::build_with(all, Catalogue::compiled().unwrap())
        .unwrap_or_else(|e| panic!("{e:?}"));
    let mut bodies = bodies().clone();
    let fixtures: [(&str, &str, Body); 3] = [
        ("regress", "r_squared", fixture_r_squared),
        ("results", "test", fixture_test),
        ("results", "compare", fixture_compare),
    ];
    for (package, name, body) in fixtures {
        bodies.insert((package.to_string(), name.to_string()), body);
    }
    let (fs, project) = project();
    run_script_in(
        &fs,
        &project,
        "scripts/main.zeff",
        script,
        &RunOptions::default(),
        &registry,
        &bodies,
    )
}

#[test]
fn result_methods_dispatch_and_terms_resolve_against_the_fit() {
    let script = format!(
        "{SETUP}m.std_errors(hours)\nm.std_errors(\"(intercept)\")\nm.std_errors()\n\
         let educ = 5\nm.std_errors(educ)\nstd_errors(m, hours)\nm.r_squared()\n\
         let t = m.test(true)\nt\ntype_of(t)\nt.is(Result)\nt.is(TestResult)\nt.is(RegFit)\n\
         [m, t]\ntype_of([m, t])\nt.std_errors()\nt.p_values()\nt.n_obs()\nt.summary()\nt.estimates()\nt.ci()\n\
         let c = m.compare(wages.reg(wage ~ hours))\nc\nc.summary()\nc.p_values()\ntype_of(c.p_values())\n\
         let twice = wages.add(h2 = hours * 2).reg(wage ~ hours + h2)\ntwice.std_errors(h2)\n"
    );
    let (code, stdout, stderr) = go(run_with_fixtures(&script));
    assert_eq!(code, 0, "{stderr}");
    insta::assert_snapshot!(
        "fixture_methods",
        format!("{stdout}--- messages ---\n{stderr}")
    );
}

#[test]
fn every_message_of_the_fixture_methods() {
    let cases = [
        format!("{SETUP}m.std_errors(nope)\n"),
        format!("{SETUP}m.std_errors(hour)\n"),
        format!("{SETUP}m.std_errors(\"educ:hours\")\n"),
        format!("{SETUP}let t = m.test(true)\nt.std_errors(hours)\n"),
        format!("{SETUP}let t = m.test(true)\nt.r_squared()\n"),
        format!("{SETUP}let t = m.test(true)\n[m, t][1].r_squared()\n"),
        format!("{SETUP}m.std_errors(1)\n"),
        format!("{SETUP}wages.r_squared()\n"),
    ];
    let mut text = String::new();
    for script in &cases {
        let (code, _, stderr) = go(run_with_fixtures(script));
        assert_eq!(code, 1, "{script}");
        text.push_str(&stderr);
        text.push('\n');
    }
    insta::assert_snapshot!("fixture_messages", text);
}
