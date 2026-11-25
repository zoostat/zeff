//! The traceback of an error raised inside a function (release 0.7.0, step 33d; open
//! decision #76): each function the error came through, the innermost first, with what the
//! function was given and where it was called, out to the line of the script. From release
//! 0.9.0, step 27 (C22 of open decision #175), the same chain is in the error's record as
//! fields, with each pass of a loop it went through, and the printed message is as it was.

use std::path::Path;
use zeff_engine::VERSION;
use zeff_engine::fs::MemoryFileSystem;
use zeff_engine::project::Project;
use zeff_engine::run::{Outcome, run_script};

const CLEAN: &str = "/// The mean of a variable, on a frame that has rows.
///
/// The mean after a check that the frame is not empty.
///
/// data: The frame.
/// var: The variable.
/// label: A name for the message.
/// returns: The mean, a Float.
/// > frame(x = [1.0, 3]).checked_mean(x, \"x\")
/// 2
fn checked_mean(data: Frame, var: Var, label: Str) -> Float {
    assert(data.n_rows() > 0, \"no rows to take the mean of\")
    data.mean(var)
}

/// The mean of the rows above a threshold.
///
/// The rows where the variable is above the threshold, then their mean.
///
/// data: The frame.
/// var: The variable.
/// above: The threshold.
/// returns: The mean, a Float.
/// > frame(x = [1.0, 3]).mean_above(x, 2)
/// 3
fn mean_above(data: Frame, var: Var, above: Float) -> Float {
    data.where(var > above).checked_mean(var, \"the rows above the threshold, whatever it was\")
}

/// The interquartile range by a definition that does not exist.
///
/// A call of a built-in written in Zeff that refuses its argument.
///
/// data: The frame.
/// var: The variable.
/// returns: The range, a Float.
/// > frame(x = [1.0, 3]).wrong_iqr(x)
/// 1
fn wrong_iqr(data: Frame, var: Var) -> Float {
    data.iqr(var, type: 12)
}

/// The mean above each of several thresholds.
///
/// A loop over the thresholds inside a function: the mean of the rows above each.
///
/// data: The frame.
/// var: The variable.
/// thresholds: The thresholds.
/// returns: The means, a list of numbers.
/// > frame(x = [1.0, 3]).means_above(x, [0.5, 2])
/// [2, 3]
fn means_above(data: Frame, var: Var, thresholds: List<Float>) -> List<Float> {
    [for above in thresholds { data.mean_above(var, above) }]
}
";

fn run(script: &str) -> Outcome {
    let fs = MemoryFileSystem::new();
    fs.add_file(
        "/p/zeff.toml",
        format!("[project]\nname = \"p\"\nzeff = \"{VERSION}\"\nedition = \"preview\"\n"),
    );
    fs.add_file("/p/functions/clean.zeff", CLEAN);
    let project = Project::open(&fs, Path::new("/p")).unwrap();
    run_script(&fs, &project, "scripts/main.zeff", script)
}

/// The record of the error that stopped a run: that of the last statement it reached.
fn record(out: &Outcome) -> serde_json::Value {
    let text = out
        .output
        .statements
        .last()
        .and_then(|statement| statement.error.as_deref())
        .expect("the run stopped at an error");
    serde_json::from_str(text).expect("the record is JSON")
}

/// The chain of a run's error, a link to a line, the innermost first.
fn chain(out: &Outcome) -> String {
    record(out)["chain"]
        .as_array()
        .map(|links| {
            links
                .iter()
                .map(|link| format!("{link}\n"))
                .collect::<String>()
        })
        .unwrap_or_default()
}

/// The lines of the traceback in what a run said.
fn trace(out: &Outcome) -> Vec<&str> {
    out.output
        .stderr
        .lines()
        .filter_map(|line| line.trim_start().strip_prefix("= in: "))
        .collect()
}

#[test]
fn an_error_two_functions_deep_names_both_calls_out_to_the_script() {
    let out = run(
        "use functions::clean\nlet d = frame(x = [1.0, 2, 3])\nprint(d.mean_above(x, 1.5))\nprint(d.mean_above(x, 10))\nprint(\"never\")\n",
    );
    assert_eq!(out.exit_code, 1);
    assert_eq!(out.output.stdout, "2.5\n");
    insta::assert_snapshot!("two_functions_deep", out.output.stderr);
    assert_eq!(
        trace(&out),
        [
            "`checked_mean(data: a frame of 0 rows × 1 variable, var: x, label: \"the rows above the thres…\")`, called at functions/clean.zeff:27:29",
            "`mean_above(data: a frame of 3 rows × 1 variable, var: x, above: 10)`, called at scripts/main.zeff:4:9",
        ]
    );
}

#[test]
fn an_error_one_function_deep_names_the_one_call() {
    // A function of the script itself.
    let out = run(
        "/// Half of a positive number.\nfn half(x: Float) -> Float {\n    assert(x > 0, \"a positive number is needed\")\n    x / 2\n}\nprint(half(4.0))\nprint(half(-1.0))\n",
    );
    assert_eq!(out.exit_code, 1);
    assert_eq!(out.output.stdout, "2\n");
    insta::assert_snapshot!("one_function_deep", out.output.stderr);
    assert_eq!(
        trace(&out),
        ["`half(x: -1)`, called at scripts/main.zeff:7:7"]
    );
}

#[test]
fn an_error_in_a_loop_names_the_call_of_the_pass_that_failed() {
    let out = run(
        "use functions::clean\nlet d = frame(x = [1.0, 2, 3])\nfor t in [0.5, 2.5, 10.0, 1.0] {\n    print(d.mean_above(x, t))\n}\n",
    );
    assert_eq!(out.exit_code, 1);
    // Two passes ran; the third failed, and the traceback has its threshold.
    assert_eq!(out.output.stdout, "2\n3\n");
    assert_eq!(trace(&out).len(), 2);
    assert!(
        trace(&out)[1].contains("above: 10)`, called at scripts/main.zeff:4:13"),
        "{:?}",
        trace(&out)
    );
}

#[test]
fn an_error_of_a_built_in_inside_a_function_is_placed_at_the_call_and_traced_from_there() {
    // `iqr` is written in Zeff; its refusal is placed at the call in the module, as before,
    // and the traceback starts at the module's function: a built-in's body is not a line of
    // the project.
    let out = run("use functions::clean\nlet d = frame(x = [1.0, 2, 3])\nprint(d.wrong_iqr(x))\n");
    assert_eq!(out.exit_code, 1);
    insta::assert_snapshot!("a_built_in_inside_a_function", out.output.stderr);
    assert_eq!(trace(&out).len(), 1);
    assert!(
        out.output.stderr.contains(" --> functions/clean.zeff:"),
        "{}",
        out.output.stderr
    );
}

#[test]
fn an_error_in_a_function_given_to_iterate_is_traced_through_it() {
    let out = run(
        "/// A step that fails once the state is small.\nfn shrink(state: Record) -> Record {\n    assert(state.x > 0.2, \"the state became too small\")\n    {x: state.x / 2}\n}\n/// Never.\nfn never(old: Record, new: Record) -> Bool {\n    false\n}\nlet end = iterate(start: {x: 1.0}, step: shrink, done: never, max: 10)\n",
    );
    assert_eq!(out.exit_code, 1);
    assert_eq!(trace(&out).len(), 1, "{}", out.output.stderr);
    assert!(
        // The state the step was given when it failed.
        trace(&out)[0].starts_with("`shrink(state: {x: 0.125})`"),
        "{:?}",
        trace(&out)
    );
    assert!(
        // And where the step was named.
        trace(&out)[0].ends_with("called at scripts/main.zeff:10:42"),
        "{:?}",
        trace(&out)
    );
}

#[test]
fn an_error_at_the_top_level_and_a_warning_inside_a_function_have_no_traceback() {
    let out = run("let d = frame(x = [1.0, 2, 3])\nerror(\"stopped at the top\")\n");
    assert_eq!(out.exit_code, 1);
    assert!(trace(&out).is_empty(), "{}", out.output.stderr);
    // A warning raised inside a function is not an error: it has its line and no traceback.
    let out = run(
        "use functions::clean\nlet d = frame(x = [1.0, NA, 3])\nprint(d.checked_mean(x, \"x\"))\n",
    );
    assert_eq!(out.exit_code, 0, "{}", out.output.stderr);
    assert!(
        out.output.stderr.contains("warning[W0201]"),
        "{}",
        out.output.stderr
    );
    assert!(trace(&out).is_empty(), "{}", out.output.stderr);
}

#[test]
fn an_error_three_calls_deep_has_each_link_as_fields_and_prints_as_it_did() {
    // The script's line calls `means_above`, whose loop calls `mean_above` at each
    // pass, which calls `checked_mean`, whose `assert` fails at the third threshold.
    let out = run(
        "use functions::clean\nlet d = frame(x = [1.0, 2, 3])\nprint(d.means_above(x, [0.5, 1.5, 10.0, 2.5]))\n",
    );
    assert_eq!(out.exit_code, 1);
    // What is printed: the three calls, the innermost first, and no line for the pass.
    insta::assert_snapshot!("three_calls_deep", out.output.stderr);
    assert_eq!(trace(&out).len(), 3);
    // The record: the same three calls and, between the second and the third, the pass.
    insta::assert_snapshot!("three_calls_deep_chain", chain(&out));
    let record = record(&out);
    // `trace` is the printed lines, as it was at 0.7.0.
    let printed: Vec<&str> = record["trace"]
        .as_array()
        .unwrap()
        .iter()
        .map(|line| line.as_str().unwrap())
        .collect();
    assert_eq!(printed, trace(&out));
    // Each call's fields make its printed line.
    let calls: Vec<String> = record["chain"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|link| link["kind"] == "call")
        .map(|link| {
            let given: Vec<String> = link["given"]
                .as_array()
                .unwrap()
                .iter()
                .map(|g| {
                    format!(
                        "{}: {}",
                        g["name"].as_str().unwrap(),
                        g["value"].as_str().unwrap()
                    )
                })
                .collect();
            format!(
                "`{}({})`, called at {}:{}:{}",
                link["function"].as_str().unwrap(),
                given.join(", "),
                link["file"].as_str().unwrap(),
                link["line"],
                link["column"]
            )
        })
        .collect();
    assert_eq!(calls, trace(&out));
    // The record is that of the error printed.
    assert_eq!(record["code"], "Z0001");
    assert_eq!(record["message"], "no rows to take the mean of");
    assert_eq!(record["file"], "functions/clean.zeff");
}

#[test]
fn an_error_inside_a_loop_has_the_pass_and_what_the_loop_variable_held() {
    // A loop of the script, a loop inside it, and a function called in the inner one.
    let out = run(
        "use functions::clean\nlet d = frame(x = [1.0, 2, 3], y = [4.0, 5, 6])\nfor v in vars(x, y) {\n    for t in [0.5, 2.5, 5.5, 1.0] {\n        print(d.mean_above(v, t))\n    }\n}\nprint(\"never\")\n",
    );
    assert_eq!(out.exit_code, 1);
    // Two passes of the inner loop ran on `x`; the third found no row above its threshold.
    assert_eq!(out.output.stdout, "2\n3\n", "{}", out.output.stderr);
    insta::assert_snapshot!("inside_a_loop", out.output.stderr);
    insta::assert_snapshot!("inside_a_loop_chain", chain(&out));
    // The message prints the calls alone.
    assert_eq!(trace(&out).len(), 2);
    let record = record(&out);
    let passes: Vec<(u64, Option<u64>, &str, &str)> = record["chain"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|link| link["kind"] == "pass")
        .map(|link| {
            (
                link["pass"].as_u64().unwrap(),
                link["of"].as_u64(),
                link["variable"].as_str().unwrap(),
                link["value"].as_str().unwrap(),
            )
        })
        .collect();
    // The inner loop first: its third pass of four, with the threshold; then the outer.
    assert_eq!(passes, [(3, Some(4), "t", "5.5"), (1, Some(2), "v", "x")]);
}

#[test]
fn an_error_in_a_loop_with_no_function_has_the_pass_alone_and_prints_no_traceback() {
    let out =
        run("let d = frame(x = [1.0, 2, 3])\nfor k in 1..5 {\n    assert(k < 4, \"too far\")\n}\n");
    assert_eq!(out.exit_code, 1);
    assert!(trace(&out).is_empty(), "{}", out.output.stderr);
    insta::assert_snapshot!("a_loop_alone", out.output.stderr);
    let record = record(&out);
    // No call, so no `trace`; the chain is the pass.
    assert!(record.get("trace").is_none(), "{record}");
    insta::assert_snapshot!("a_loop_alone_chain", chain(&out));
    // The whole record as the engine writes it, its fields in their order.
    insta::assert_snapshot!(
        "a_loop_alone_record",
        out.output
            .statements
            .last()
            .unwrap()
            .error
            .as_deref()
            .unwrap()
    );
}

#[test]
fn an_error_outside_a_function_and_a_loop_has_a_record_with_no_chain() {
    let out = run("let d = frame(x = [1.0, 2, 3])\nerror(\"stopped at the top\")\n");
    assert_eq!(out.exit_code, 1);
    let record = record(&out);
    assert_eq!(record["message"], "stopped at the top");
    assert_eq!(record["span"]["line"], 2);
    assert!(record.get("chain").is_none(), "{record}");
    assert!(record.get("trace").is_none(), "{record}");
    // A statement that ran has no error; a run that ends well has none at all.
    assert!(out.output.statements[0].error.is_none());
    let out = run("let d = frame(x = [1.0, 2, 3])\nprint(d.n_rows())\n");
    assert!(out.output.statements.iter().all(|s| s.error.is_none()));
}

#[test]
fn a_pass_of_a_loop_inside_a_built_in_is_not_in_the_chain() {
    // `iqr` is written in Zeff; whatever loops its body has, the chain of its refusal is the
    // project's own: the one call of the module's function.
    let out = run("use functions::clean\nlet d = frame(x = [1.0, 2, 3])\nprint(d.wrong_iqr(x))\n");
    assert_eq!(out.exit_code, 1);
    let record = record(&out);
    let links = record["chain"].as_array().unwrap();
    assert_eq!(links.len(), 1, "{record}");
    assert_eq!(links[0]["kind"], "call");
    assert_eq!(links[0]["function"], "wrong_iqr");
    assert!(
        links
            .iter()
            .all(|link| link["file"].as_str().unwrap().ends_with(".zeff")
                && !link["file"].as_str().unwrap().contains("builtin")),
        "{record}"
    );
}

/// What the loop variable held, where it is a record (release 0.9.0, step 42c3): a small
/// record of single values is shown as it is written, and one that holds a list by its type.
#[test]
fn a_loop_variable_that_is_a_record_is_shown_as_written_or_by_its_type() {
    let held = |script: &str| {
        let out = run(script);
        assert_eq!(out.exit_code, 1, "{}", out.output.stderr);
        let record = record(&out);
        let chain = record["chain"].as_array().unwrap();
        assert_eq!(chain.len(), 1, "{chain:?}");
        chain[0]["value"].as_str().unwrap().to_string()
    };
    assert_eq!(
        held("for r in [{a: 1, b: 2.5}] {\n    assert(r.a > 5, \"too small\")\n}\n"),
        "{a: 1, b: 2.5}"
    );
    assert_eq!(
        held("for r in [{a: 1, b: [1, 2]}] {\n    assert(r.a > 5, \"too small\")\n}\n"),
        "Record"
    );
}
