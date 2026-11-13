//! The traceback of an error raised inside a function (release 0.7.0, step 33d; open
//! decision #76): each function the error came through, the innermost first, with what the
//! function was given and where it was called, out to the line of the script.

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
