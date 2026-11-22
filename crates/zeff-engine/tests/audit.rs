//! `zeff audit` (Language Reference, section 11.1; open decision #206; release 0.9.0, step
//! 22c): fixtures of run records read against a list with an issue corrected in a later patch,
//! and what is said of them, the example of section 11.1 among it.

use zeff_engine::audit::{audit, call_said, day_said};
use zeff_engine::known_issues::{Issue, read};
use zeff_engine::runs::{CallMade, RunRecord};

/// The list of an engine after two patches: the issue of section 11.1's example, corrected in
/// 0.9.2; one about every call of a function, corrected in 0.9.1; one that nothing corrects
/// yet; and one of long ago.
const LIST: &str = r#"format = 1

[[issue]]
number = 123
summary = "`reg` with `weights:` gave standard errors that left the weights out of the residual variance"
functions = ["reg", "ivreg"]
options = ["weights"]
from = "0.9.0"
fixed = "0.9.2"
affected_when = "the weights were not all equal"
document = "d"

[[issue]]
number = 131
summary = "`confusion` counted a row at the cutoff as predicted 0"
functions = ["confusion"]
options = []
from = "0.9.0"
fixed = "0.9.1"
affected_when = "a fitted probability was equal to the cutoff"
document = "d"

[[issue]]
number = 140
summary = "`margins` after `glm` with `at:` and `type: mem` held a category at its first level"
functions = ["margins"]
options = ["at", "type"]
from = "0.9.0"
affected_when = "the model had a category among its regressors"
document = "d"

[[issue]]
number = 2
summary = "`correlation` printed 0.000 for a variable with a value above 1e150"
functions = ["correlation"]
options = []
from = "0.3.0"
fixed = "0.7.0"
affected_when = "a variable held such a value"
document = "d"
"#;

fn issues() -> Vec<Issue> {
    read(LIST).expect("the fixture list reads")
}

fn call(function: &str, options: &[&str]) -> CallMade {
    CallMade {
        function: function.to_string(),
        package: "regress".to_string(),
        options: options.iter().map(|option| (*option).to_string()).collect(),
    }
}

fn run(engine: &str, on: Option<&str>, script: &str, calls: &[CallMade]) -> RunRecord {
    RunRecord {
        engine: engine.to_string(),
        build: "official".to_string(),
        on: on.map(str::to_string),
        at: on.map(|_| "09:30:00".to_string()),
        script: script.to_string(),
        exit_code: 0,
        calls: calls.to_vec(),
    }
}

/// The example of section 11.1 of the Language Reference, in its form.
#[test]
fn the_example_of_the_reference_prints_in_its_form() {
    let records = [run(
        "0.9.0",
        Some("2026-10-12"),
        "scripts/main.zeff",
        &[call("dataset", &[]), call("reg", &["se", "weights"])],
    )];
    let report = audit(&records, &issues(), "0.9.0");
    assert_eq!(report.affected.len(), 1);
    assert_eq!(
        report.affected[0].sentence(Some("2026")),
        "your run of 12 October used `reg` with `weights:`, affected by issue 123, fixed in 0.9.2"
    );
    insta::assert_snapshot!(report.text("0.9.3", Some("2026")));
}

#[test]
fn a_day_and_a_call_as_they_are_said() {
    assert_eq!(day_said("2026-10-12", Some("2026")), "12 October");
    assert_eq!(day_said("2025-01-03", Some("2026")), "3 January 2025");
    assert_eq!(day_said("2026-10-12", None), "12 October 2026");
    assert_eq!(day_said("2026-13-01", Some("2026")), "2026-13-01");
    assert_eq!(day_said("some day", None), "some day");
    let options =
        |names: &[&str]| -> Vec<String> { names.iter().map(|name| (*name).to_string()).collect() };
    assert_eq!(call_said("mad", &options(&[])), "`mad`");
    assert_eq!(
        call_said("reg", &options(&["weights"])),
        "`reg` with `weights:`"
    );
    assert_eq!(
        call_said("margins", &options(&["at", "type"])),
        "`margins` with `at:` and `type:`"
    );
    assert_eq!(
        call_said("f", &options(&["a", "b", "c"])),
        "`f` with `a:`, `b:` and `c:`"
    );
}

#[test]
fn a_run_is_affected_by_its_engine_and_its_calls_together() {
    let found = |engine: &str, calls: &[CallMade]| {
        let report = audit(
            &[run(engine, Some("2026-11-02"), "scripts/main.zeff", calls)],
            &issues(),
            engine,
        );
        report
            .affected
            .iter()
            .map(|found| found.issue)
            .collect::<Vec<u64>>()
    };
    // The engine and the call, both: 0.9.0 with `weights:` written.
    assert_eq!(found("0.9.0", &[call("reg", &["weights"])]), [123]);
    assert_eq!(
        found("0.9.1", &[call("ivreg", &["cluster", "weights"])]),
        [123]
    );
    // The call without the option; the option on another function; the engine that corrects it.
    assert_eq!(found("0.9.0", &[call("reg", &["se"])]), Vec::<u64>::new());
    assert_eq!(
        found("0.9.0", &[call("glm", &["weights"])]),
        Vec::<u64>::new()
    );
    assert_eq!(
        found("0.9.2", &[call("reg", &["weights"])]),
        Vec::<u64>::new()
    );
    assert_eq!(
        found("0.8.0", &[call("reg", &["weights"])]),
        Vec::<u64>::new()
    );
    // Two issues on one run, in the order of the list; one that nothing corrects yet stays.
    assert_eq!(
        found(
            "0.9.0",
            &[
                call("confusion", &[]),
                call("margins", &["type"]),
                call("reg", &["weights"])
            ]
        ),
        [123, 131, 140]
    );
    assert_eq!(found("0.9.1", &[call("confusion", &[])]), Vec::<u64>::new());
    assert_eq!(found("1.4.0", &[call("margins", &["at"])]), [140]);
    // An issue of long ago and a run of long ago.
    assert_eq!(found("0.6.0", &[call("correlation", &[])]), [2]);
    assert_eq!(
        found("0.9.0", &[call("correlation", &[])]),
        Vec::<u64>::new()
    );
}

#[test]
fn every_form_of_the_report() {
    let weighted = [call("reg", &["weights"])];
    let mut out = String::new();
    let mut shown = |title: &str, records: &[RunRecord], pinned: &str, year: Option<&str>| {
        let report = audit(records, &issues(), pinned);
        out.push_str(&format!("----- {title}\n{}\n", report.text("0.9.3", year)));
        report
    };
    let none = shown("a project with no run", &[], "0.9.3", Some("2026"));
    // The one issue nothing corrects yet is still the pin's.
    assert!(none.affected.is_empty());
    assert_eq!(none.of_the_pin.len(), 1);
    shown(
        "runs that no issue affects, on a pin that is past them",
        &[
            run("0.9.3", Some("2026-11-20"), "scripts/main.zeff", &weighted),
            run(
                "0.9.3",
                Some("2026-11-21"),
                "scripts/main.zeff",
                &[call("print", &[])],
            ),
        ],
        "0.9.3",
        Some("2026"),
    );
    let many = shown(
        "many runs of one script, two scripts, two issues, and a pin that is affected",
        &[
            run("0.9.0", Some("2026-10-03"), "scripts/main.zeff", &weighted),
            run("0.9.0", Some("2026-10-08"), "scripts/main.zeff", &weighted),
            run("0.9.0", Some("2026-10-12"), "scripts/main.zeff", &weighted),
            run(
                "0.9.0",
                Some("2026-10-12"),
                "scripts/tables.zeff",
                &[call("confusion", &["cutoff"])],
            ),
            run("0.9.2", Some("2026-10-20"), "scripts/main.zeff", &weighted),
            run(
                "0.9.0",
                Some("2026-10-21"),
                "scripts/main.zeff",
                &[call("reg", &[])],
            ),
        ],
        "0.9.0",
        Some("2026"),
    );
    assert_eq!(many.affected.len(), 2);
    assert_eq!(
        (
            many.affected[0].runs,
            many.affected[0].first.as_deref(),
            many.affected[0].last.as_deref()
        ),
        (3, Some("2026-10-03"), Some("2026-10-12"))
    );
    assert_eq!(many.of_the_pin.len(), 3);
    shown(
        "two runs, last year, read without a clock; a run with no day; an issue not corrected",
        &[
            run("0.9.1", Some("2025-12-30"), "scripts/main.zeff", &weighted),
            run("0.9.1", Some("2025-12-31"), "scripts/main.zeff", &weighted),
            run(
                "0.9.2",
                None,
                "scripts/effects.zeff",
                &[call("margins", &["at", "type"])],
            ),
            run(
                "0.9.2",
                None,
                "scripts/effects.zeff",
                &[call("margins", &["at", "type"])],
            ),
        ],
        "0.9.2",
        None,
    );
    insta::assert_snapshot!(out);
}

#[test]
fn the_report_as_a_record() {
    let records = [
        run(
            "0.9.0",
            Some("2026-10-12"),
            "scripts/main.zeff",
            &[call("reg", &["weights"])],
        ),
        run(
            "0.9.0",
            None,
            "scripts/a \"b\".zeff",
            &[call("margins", &["at"])],
        ),
    ];
    let json = audit(&records, &issues(), "0.9.0").json("0.9.3");
    assert!(
        json.starts_with("{\"engine\": \"0.9.3\", \"runs\": 2, \"issues\": 4, \"affected\": [{\"issue\": 123, \"fixed\": \"0.9.2\", \"script\": \"scripts/main.zeff\", \"engine\": \"0.9.0\", \"function\": \"reg\", \"options\": [\"weights\"], \"runs\": 1, \"first\": \"2026-10-12\", \"last\": \"2026-10-12\"}, "),
        "{json}"
    );
    assert!(
        json.contains("{\"issue\": 140, \"fixed\": null, \"script\": \"scripts/a \\\"b\\\".zeff\", \"engine\": \"0.9.0\", \"function\": \"margins\", \"options\": [\"at\"], \"runs\": 1, \"first\": null, \"last\": null}"),
        "{json}"
    );
    assert!(
        json.ends_with("\"pinned\": \"0.9.0\", \"pinned_issues\": [{\"issue\": 123, \"functions\": [\"reg\", \"ivreg\"], \"options\": [\"weights\"], \"fixed\": \"0.9.2\"}, {\"issue\": 131, \"functions\": [\"confusion\"], \"options\": [], \"fixed\": \"0.9.1\"}, {\"issue\": 140, \"functions\": [\"margins\"], \"options\": [\"at\", \"type\"], \"fixed\": null}]}"),
        "{json}"
    );
}
