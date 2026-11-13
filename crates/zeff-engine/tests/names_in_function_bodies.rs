//! Inside the body of a function, the names the function binds come first (Language Reference,
//! section 3.7, "Name lookup", and section 6, "Names inside a frame's methods"; open decision
//! #140, option A; release 0.7.0, step 38c). A parameter, a `let`, a loop variable or the
//! parameter of a function written inside the body is that binding inside a frame's methods
//! too, whatever the variables of the frame are called, so that a function gives the same
//! result on every frame (section 5.3, "Scope"). A name the function does not bind is the
//! frame's variable, as in a script, and a variable that shares a bound name is reached as
//! `data["name"]`. A script is unchanged: there the frame's variable comes before a `let`
//! (W0102) and a clash with a loop variable or a parameter is an error (Z0106).

use std::path::Path;
use zeff_engine::VERSION;
use zeff_engine::fs::MemoryFileSystem;
use zeff_engine::project::Project;
use zeff_engine::run::{Outcome, run_script};

fn run_with(modules: &[(&str, &str)], script: &str) -> Outcome {
    let fs = MemoryFileSystem::new();
    fs.add_file(
        "/p/zeff.toml",
        format!("[project]\nname = \"p\"\nzeff = \"{VERSION}\"\nedition = \"preview\"\n"),
    );
    for (name, text) in modules {
        fs.add_file(format!("/p/functions/{name}.zeff"), *text);
    }
    let project = Project::open(&fs, Path::new("/p")).unwrap();
    run_script(&fs, &project, "scripts/main.zeff", script)
}

fn run(script: &str) -> Outcome {
    run_with(&[], script)
}

/// The functions: each binds names and uses them inside a method of the frame it is given.
const FUNCTIONS: &str = "\
/// The median distance from the median: a `let` and a `Var` parameter.
fn spread(data: Frame, var: Var) -> Float {
    let centre = data.median(var)
    data.add(distance = abs(var - centre)).median(distance)
}

/// Rows above a limit: a parameter that holds a number.
fn above(data: Frame, var: Var, limit: Float) -> Int {
    data.where(var > limit).n_rows()
}

/// The mean of two variables, each plus a shift: a loop variable of a list.
fn means(data: Frame, shift: Float) -> List<Float> {
    [for v in vars(x, y) { data.mean(v) + shift }]
}

/// The largest value of two variables, printed: a loop variable of a `for`.
fn largest(data: Frame) -> Int {
    for v in vars(x, y) {
        print(data.max(v))
    }
    data.n_rows()
}

/// The mean of `x` times each of two numbers and `k`: the parameter of a function written
/// inside the body.
fn scaled(data: Frame, k: Float) -> List<Float> {
    map([1.0, 2], fn(t) data.add(z = x * t * k).mean(z))
}

/// Rows of a year: `year` is not bound here, so it is the frame's variable.
fn in_year(data: Frame, wanted: Int) -> Int {
    data.where(year == wanted).n_rows()
}
";

/// The frame's own variables, and the same with a variable for every name the functions bind.
const PLAIN: &str =
    "let d = frame(x = [1.0, 2, 3, 9], y = [2.0, 4, 7, 8], year = [2014, 2015, 2015, 2016])\n";
const NAMED: &str = "let d = frame(x = [1.0, 2, 3, 9], y = [2.0, 4, 7, 8], year = [2014, 2015, 2015, 2016], \
centre = [5.0, 5, 5, 5], var = [0.0, 0, 0, 0], limit = [0.0, 10, 0, 10], data = [1.0, 1, 1, 1], \
shift = [9.0, 9, 9, 9], v = [7.0, 7, 7, 7], wanted = [2016, 2016, 2016, 2016], \
t = [100.0, 100, 100, 100], k = [50.0, 50, 50, 50])\n";

const CALLS: &str =
    "d.spread(x)\nd.above(x, 2.5)\nd.means(1)\nd.largest()\nd.scaled(2)\nd.in_year(2015)\n";

#[test]
fn a_function_gives_the_same_result_whatever_the_variables_are_called() {
    let plain = run(&format!("{FUNCTIONS}{PLAIN}{CALLS}"));
    assert_eq!(plain.exit_code, 0, "{}", plain.output.stderr);
    assert_eq!(plain.output.stderr, "");
    // R (4.6.0): median(abs(x - median(x))) 1; sum(x > 2.5) 2; mean(x) + 1 and mean(y) + 1,
    // 4.75 and 6.25; max 9 and 8; mean(x * 2) and mean(x * 4), 7.5 and 15; two rows of 2015.
    assert_eq!(
        plain.output.stdout,
        "1\n2\n4.75\n6.25\n9\n8\n4\n7.5\n15\n2\n"
    );
    let named = run(&format!("{FUNCTIONS}{NAMED}{CALLS}"));
    assert_eq!(named.exit_code, 0, "{}", named.output.stderr);
    assert_eq!(named.output.stderr, "", "no warning and no error");
    assert_eq!(named.output.stdout, plain.output.stdout);
}

#[test]
fn a_variable_named_as_a_parameter_is_asked_for_by_that_name() {
    // The variable called `var` or `limit` is itself the one given: the parameter holds its
    // name, and stands for it.
    let out = run(&format!(
        "{FUNCTIONS}let d = frame(var = [1.0, 2, 3, 9], limit = [2.0, 4, 7, 8])\nd.spread(var)\nd.above(limit, 3)\n"
    ));
    assert_eq!(out.exit_code, 0, "{}", out.output.stderr);
    assert_eq!(out.output.stderr, "");
    assert_eq!(out.output.stdout, "1\n3\n");
}

#[test]
fn the_names_of_a_formula_are_the_function_s_too() {
    // `outcome ~ on` regresses the variables the two parameters name, on a frame that has
    // variables called `outcome` and `on` as on one that has not. R: coef(lm(b ~ a))[2]
    // 1.99761904761905.
    let function = "/// A regression's slope.\nfn slope(data: Frame, outcome: Var, on: Var) -> Float {\n    data.reg(outcome ~ on).coef(name_of(on))\n}\n";
    let columns =
        "a = [1.0, 2, 3, 4, 5, 6, 7, 8], b = [2.1, 3.9, 6.2, 7.8, 10.1, 12.2, 13.8, 16.1]";
    let plain = run(&format!(
        "{function}let d = frame({columns})\nd.slope(b, a)\n"
    ));
    assert_eq!(plain.exit_code, 0, "{}", plain.output.stderr);
    assert_eq!(plain.output.stderr, "");
    assert_eq!(plain.output.stdout, "1.99762\n");
    let named = run(&format!(
        "{function}let d = frame({columns}, outcome = [3.0, 1, 4, 1, 5, 9, 2, 6], on = [2.0, 7, 1, 8, 2, 8, 1, 8])\nd.slope(b, a)\n"
    ));
    assert_eq!(named.exit_code, 0, "{}", named.output.stderr);
    assert_eq!(named.output.stderr, "");
    assert_eq!(named.output.stdout, "1.99762\n");
}

#[test]
fn a_variable_that_shares_a_bound_name_is_reached_through_the_frame() {
    // `limit` the number against `limit` the variable, written `data[\"limit\"]`.
    let out = run(
        "/// Rows whose `limit` is above the number given.\nfn over(data: Frame, limit: Float) -> Int {\n    data.where(data[\"limit\"] > limit).n_rows()\n}\nlet d = frame(limit = [0.0, 10, 0, 10, 7])\nd.over(5)\nd.over(8)\n",
    );
    assert_eq!(out.exit_code, 0, "{}", out.output.stderr);
    assert_eq!(out.output.stderr, "");
    assert_eq!(out.output.stdout, "3\n2\n");
}

#[test]
fn an_option_that_takes_a_variable_or_a_value_takes_the_function_s_binding() {
    // `at:` of `vline` takes a variable or a number (open decision #127). In a script a `let`
    // called `dose` gives way to the variable `dose` under W0102; in a function it is the
    // function's number.
    let frame = "let d = frame(dose = [1, 2, 3, 4], change = [-1.2, 0.4, 1.1, 2.3])\n";
    let inside = run(&format!(
        "/// A plot with a line at a dose.\nfn marked(data: Frame, where_at: Float) -> Plot {{\n    let dose = where_at\n    data.plot(x: change, y: change).points().vline(at: dose)\n}}\n{frame}d.marked(2.5)\n"
    ));
    assert_eq!(inside.exit_code, 0, "{}", inside.output.stderr);
    assert_eq!(inside.output.stderr, "");
    assert!(
        inside
            .output
            .stdout
            .contains("vline    data       1   at 2.5,"),
        "{}",
        inside.output.stdout
    );
    let outside = run(&format!(
        "{frame}let dose = 2.5\nd.plot(x: change, y: change).points().vline(at: dose)\n"
    ));
    assert_eq!(outside.exit_code, 0, "{}", outside.output.stderr);
    assert!(
        outside.output.stderr.contains(
            "warning[W0102]: `dose` is both a variable of d and a binding; the variable was used"
        ),
        "{}",
        outside.output.stderr
    );
    assert!(
        outside
            .output
            .stdout
            .contains("vline    d          4   at dose,"),
        "{}",
        outside.output.stdout
    );
}

#[test]
fn a_frame_whose_variables_are_known_before_the_run_is_not_refused_at_the_check() {
    // `wages` has a variable `wage`; a parameter of that name is no clash inside the body,
    // with the schema in the checker's hands.
    let out = run(
        "/// The mean of a variable above a floor.\nfn mean_above(data: Frame, wage: Var, hours: Float) -> Float {\n    data.where(wage > hours).mean(wage)\n}\nlet w = dataset(wages)\nw.mean_above(educ, 12)\nw.where(educ > 12).mean(educ)\n",
    );
    assert_eq!(out.exit_code, 0, "{}", out.output.stderr);
    assert_eq!(out.output.stderr, "");
    let lines: Vec<&str> = out.output.stdout.lines().collect();
    assert_eq!(lines.len(), 2, "{}", out.output.stdout);
    assert_eq!(lines[0], lines[1]);
}

#[test]
fn a_function_of_a_module_follows_the_same_rule() {
    let module = "/// The median distance from the median.\n///\n/// The median of the distances of `var` from its median.\n///\n/// data: The frame.\n/// var: The variable.\n/// returns: The median distance, a Float.\n///\n/// > frame(x = [1.0, 2, 3, 9]).spread(x)\n/// 1\nfn spread(data: Frame, var: Var) -> Float {\n    let centre = data.median(var)\n    data.add(distance = abs(var - centre)).median(distance)\n}\n";
    let out = run_with(
        &[("robust", module)],
        "use functions::robust\nlet d = frame(x = [1.0, 2, 3, 9], centre = [5.0, 5, 5, 5], var = [0.0, 0, 0, 0])\nd.spread(x)\n",
    );
    assert_eq!(out.exit_code, 0, "{}", out.output.stderr);
    assert_eq!(out.output.stderr, "");
    assert_eq!(out.output.stdout, "1\n");
}

#[test]
fn a_script_is_unchanged() {
    // A `let` of the script gives way to the frame's variable, with the warning.
    let out = run("let d = frame(x = [1.0, 2, 3, 9])\nlet x = 100\nd.where(x > 2.5).n_rows()\n");
    assert_eq!(out.exit_code, 0, "{}", out.output.stderr);
    assert_eq!(out.output.stdout, "2\n");
    assert!(
        out.output.stderr.starts_with(
            "warning[W0102]: `x` is both a variable of d and a binding; the variable was used"
        ),
        "{}",
        out.output.stderr
    );
    // A loop variable of the script that is also a variable is refused.
    let out = run(
        "let d = frame(x = [1.0, 2, 3, 9], v = [1.0, 1, 1, 1])\nfor v in vars(x) {\n    print(d.mean(v))\n}\n",
    );
    assert_eq!(out.exit_code, 1);
    assert!(
        out.output.stderr.starts_with("error[Z0106]: `v` is both"),
        "{}",
        out.output.stderr
    );
    // So is the parameter of a function written in the script's own statements.
    let out = run(
        "let d = frame(x = [1.0, 2, 3, 9], t = [1.0, 1, 1, 1])\nmap([1.0, 2], fn(t) d.add(z = x * t).mean(z))\n",
    );
    assert_eq!(out.exit_code, 1);
    assert!(
        out.output.stderr.starts_with("error[Z0106]: `t` is both"),
        "{}",
        out.output.stderr
    );
    // And a function does not see the script's bindings, as before (section 5.3, "Scope").
    let out = run(
        "/// Rows above the script's limit.\nfn above(data: Frame) -> Int {\n    data.where(x > limit).n_rows()\n}\nlet limit = 2.5\nlet d = frame(x = [1.0, 2, 3, 9])\nd.above()\n",
    );
    assert_eq!(out.exit_code, 1);
    assert!(out.output.stderr.contains("limit"), "{}", out.output.stderr);
}

#[test]
fn every_message_of_names_in_function_bodies() {
    let cases: [(&str, String); 4] = [
        (
            "same_result_on_a_frame_with_the_function_s_names",
            format!("{FUNCTIONS}{NAMED}{CALLS}"),
        ),
        (
            "a_script_s_let_gives_way",
            "let d = frame(x = [1.0, 2, 3, 9])\nlet x = 100\nd.where(x > 2.5).n_rows()\n".to_string(),
        ),
        (
            "a_script_s_loop_variable_clashes",
            "let d = frame(x = [1.0, 2, 3, 9], v = [1.0, 1, 1, 1])\nfor v in vars(x) {\n    print(d.mean(v))\n}\n"
                .to_string(),
        ),
        (
            "a_function_does_not_see_the_script",
            "/// Rows above the script's limit.\nfn above(data: Frame) -> Int {\n    data.where(x > limit).n_rows()\n}\nlet limit = 2.5\nlet d = frame(x = [1.0, 2, 3, 9])\nd.above()\n"
                .to_string(),
        ),
    ];
    let mut all = String::new();
    for (name, script) in cases {
        let outcome = run(&script);
        all.push_str(&format!(
            "## {name} (exit {})\n{}{}\n",
            outcome.exit_code, outcome.output.stdout, outcome.output.stderr
        ));
    }
    insta::assert_snapshot!("every_message", all);
}
