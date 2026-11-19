//! `cluster:` takes the variables `fe:` takes (release 0.8.0, step 20h): text, a category, a
//! whole number, a date and a date with a time each serve as clusters by the identity of their
//! values, in `reg` and in `ivreg`. Before, a text variable was refused as text in a model,
//! and a date with a message that named release 0.6.0 as one to come.

use zeff_testkit::run;

/// Eight rows in four clusters, the same four whatever the variable that names them.
const DATA: &str = "let d = frame(y = [1.0, 2.5, 2.0, 4.0, 3.5, 5.0, 4.5, 6.5], x = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.5], z = [2.0, 1.0, 4.0, 3.0, 6.0, 5.0, 8.0, 7.0], g = [1, 1, 2, 2, 3, 3, 4, 4], f = [1.5, 1.5, 2.5, 2.5, 3.5, 3.5, 4.5, 4.5], s = [\"a\", \"a\", \"b\", \"b\", \"c\", \"c\", \"d\", \"d\"], days = [\"2024-01-01\", \"2024-01-01\", \"2024-01-02\", \"2024-01-02\", \"2024-01-03\", \"2024-01-03\", \"2024-01-04\", \"2024-01-04\"], gaps = [\"a\", \"a\", \"b\", \"b\", \"c\", \"c\", \"d\", NA]).add(day = to(days, Date), moment = datetime(days, format: \"%Y-%m-%d\"), c = category(s))\n";

fn go(statement: &str) -> (i32, String, String) {
    let outcome = run(&format!("{DATA}{statement}\n"));
    (
        outcome.exit_code,
        outcome.output.stdout,
        outcome.output.stderr,
    )
}

#[test]
fn every_kind_of_variable_names_the_same_clusters() {
    // The errors of the slope under each, as the engine holds them: one figure.
    let mut errors: Vec<(String, String)> = Vec::new();
    for function in ["reg(y ~ x", "ivreg(y ~ x, instruments: x ~ z"] {
        for cluster in ["g", "f", "s", "c", "day", "moment"] {
            let (exit, stdout, stderr) = go(&format!(
                "print(d.{function}, cluster: {cluster}).std_errors(x))"
            ));
            assert_eq!(exit, 0, "{function}, cluster: {cluster}: {stderr}");
            errors.push((format!("{function}, cluster: {cluster})"), stdout));
        }
    }
    for pair in errors[..6].windows(2) {
        assert_eq!(pair[0].1, pair[1].1, "{} and {}", pair[0].0, pair[1].0);
    }
    for pair in errors[6..].windows(2) {
        assert_eq!(pair[0].1, pair[1].1, "{} and {}", pair[0].0, pair[1].0);
    }
    // Printed: a cluster of text, one of dates beside an absorbed effect, a row dropped for a
    // missing cluster, and the instrumented fit.
    let mut all = String::new();
    for statement in [
        "d.reg(y ~ x, cluster: s)",
        "d.reg(y ~ x, fe: g, cluster: day)",
        "d.reg(y ~ x, cluster: gaps)",
        "d.ivreg(y ~ x, instruments: x ~ z, cluster: moment)",
    ] {
        let (exit, stdout, stderr) = go(statement);
        assert_eq!(exit, 0, "{statement}: {stderr}");
        all.push_str(&format!("==== {statement}\n{stdout}{stderr}"));
    }
    insta::assert_snapshot!("clusters_of_every_type", all);
}

/// The refusals that are left where a cluster is named: each with its caret on the argument,
/// not on the name of the method; and a text variable or a date in the formula itself, which a
/// model still cannot make a column of.
#[test]
fn the_refusals_left_point_at_the_argument() {
    let mut all = String::new();
    for statement in [
        "d.reg(y ~ x, cluster: nowhere)",
        "d.add(one = 1).reg(y ~ x, cluster: one)",
        "d.reg(y ~ x, se: hc3, cluster: s)",
        "d.ivreg(y ~ x, instruments: x ~ z, cluster: nowhere)",
        "d.reg(y ~ x + s)",
        "d.reg(y ~ x + day)",
        "d.reg(y ~ x, weights: s)",
    ] {
        let (exit, stdout, stderr) = go(statement);
        assert_eq!(exit, 1, "{statement}: {stdout}");
        assert!(!stderr.contains("0.6.0"), "{statement}: {stderr}");
        all.push_str(&format!("==== {statement}\n{stderr}"));
    }
    insta::assert_snapshot!("cluster_refusals", all);
    // The caret of a refusal about the cluster is under the cluster's argument.
    for statement in [
        "d.reg(y ~ x, cluster: nowhere)",
        "d.add(one = 1).reg(y ~ x, cluster: one)",
    ] {
        let (_, _, stderr) = go(statement);
        let line = stderr
            .lines()
            .find(|line| line.contains('^'))
            .unwrap_or_else(|| panic!("{statement}: no caret:\n{stderr}"));
        let caret = line.find('^').unwrap();
        let written = stderr
            .lines()
            .find(|line| line.contains("cluster:"))
            .unwrap();
        let argument = written.find("cluster: ").unwrap() + "cluster: ".len();
        assert_eq!(caret, argument, "{statement}:\n{stderr}");
    }
}
