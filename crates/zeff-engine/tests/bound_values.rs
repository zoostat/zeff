//! A bare name given where an option takes a variable or a value (Language Reference, section
//! 3.7, "Resolution by parameter type"; open decision #127, option B; release 0.7.0, step
//! 23f): `at:` of `hline` and `vline` (`Float | Var`), `size:` of `plot` and `points`
//! (`Var | Float`), and the options that take a variable or a string (`color:`, `fill:`,
//! `shape:`, `linetype:`). The name is the variable of the frame the call draws where the
//! frame has one; where it has none, a `let` binding that holds a number or text; both at
//! once is the variable, with W0102. A loop variable or a parameter holding such a value is
//! that value, and a clash with a variable is Z0106, as everywhere in a frame method.

use std::path::Path;
use zeff_engine::VERSION;
use zeff_engine::fs::MemoryFileSystem;
use zeff_engine::project::Project;
use zeff_engine::run::{Outcome, run_script};

fn run(script: &str) -> Outcome {
    let fs = MemoryFileSystem::new();
    fs.add_file(
        "/p/zeff.toml",
        format!("[project]\nname = \"p\"\nzeff = \"{VERSION}\"\nedition = \"preview\"\n"),
    );
    let project = Project::open(&fs, Path::new("/p")).unwrap();
    run_script(&fs, &project, "scripts/main.zeff", script)
}

const FRAME: &str = "let d = frame(dose = [1, 2, 3, 4], change = [-1.2, 0.4, 1.1, 2.3], arm = [\"a\", \"a\", \"b\", \"b\"])\n";

/// What a script that runs prints, and its messages.
fn ran(script: &str) -> (String, String) {
    let outcome = run(&format!("{FRAME}{script}"));
    assert_eq!(outcome.exit_code, 0, "{script}\n{}", outcome.output.stderr);
    (outcome.output.stdout, outcome.output.stderr)
}

#[test]
fn a_bound_number_is_the_place_of_a_reference_line_and_the_size_of_a_point() {
    let (printed, said) = ran(
        "let centre = 0.5\nlet cut = 3\nlet big = 9\nd.plot(x: dose, y: change).points(size: big).hline(at: centre).vline(at: cut)\nd.plot(x: dose, y: change, size: big).points()\n",
    );
    for line in [
        "      1   points   d          4   size 9, alpha 1\n",
        "      2   hline    d          1   at 0.5, width 0.5, style dashed\n",
        "      3   vline    d          1   at 3, width 0.5, style dashed\n",
        "  size                constant   every mark   9\n",
    ] {
        assert!(printed.contains(line), "{line}\n{printed}");
    }
    assert_eq!(said, "");
}

#[test]
fn bound_text_is_a_colour_a_shape_and_a_type_of_line() {
    let (printed, said) = ran(
        "let red = \"firebrick\"\nlet mark = \"square\"\nlet dashes = \"dotted\"\nd.plot(x: dose, y: change).points(color: red, shape: mark).lines(color: red, linetype: dashes).area(fill: red).hline(at: 0, color: red)\nd.plot(x: dose, y: change, color: red, fill: red, shape: mark).points()\n",
    );
    for line in [
        "      1   points   d          4   color firebrick, size 4.5, shape square, alpha 1\n",
        "      2   lines    d          4   color firebrick, width 1, linetype dotted, 1 line\n",
        "      3   area     d          4   fill firebrick, alpha 0.6, 1 area\n",
        "      4   hline    d          1   at 0, color firebrick, width 0.5, style dashed\n",
        "  color               constant   every mark   firebrick\n",
        "  shape               constant   every mark   square\n",
    ] {
        assert!(printed.contains(line), "{line}\n{printed}");
    }
    assert_eq!(said, "");
}

#[test]
fn the_variable_of_the_frame_comes_first_and_a_warning_says_so() {
    // `dose` and `arm` are variables of `d` and bindings too.
    let (printed, said) = ran(
        "let dose = 2.5\nlet arm = \"steelblue\"\nd.plot(x: dose, y: change).points(color: arm).vline(at: dose)\n",
    );
    for line in [
        "  color       arm     category   a, b     orange, sky blue\n",
        "      1   points   d          4   color by arm, size 4.5, alpha 1\n",
        "      2   vline    d          4   at dose, width 0.5, style dashed\n",
    ] {
        assert!(printed.contains(line), "{line}\n{printed}");
    }
    assert!(
        said.contains("warning[W0102]: `arm` is both a variable of d and a binding; the variable was used (line 4)"),
        "{said}"
    );
    assert!(
        said.contains("warning[W0102]: `dose` is both a variable of d and a binding; the variable was used (line 4)"),
        "{said}"
    );
    // The frame the layer draws decides: `level` is a variable of `limits` and not of `d`,
    // and `centre` is a variable of neither.
    let (printed, said) = ran(
        "let limits = frame(level = [-1, 1])\nlet level = 0\nlet centre = 0.5\nd.plot(x: dose, y: change).points().hline(data: limits, at: level).hline(data: limits, at: centre).hline(at: level)\n",
    );
    for line in [
        "      2   hline    limits       2   at level, width 0.5, style dashed\n",
        "      3   hline    limits       1   at 0.5, width 0.5, style dashed\n",
        "      4   hline    d            1   at 0, width 0.5, style dashed\n",
    ] {
        assert!(printed.contains(line), "{line}\n{printed}");
    }
    assert_eq!(said.matches("W0102").count(), 1, "{said}");
    assert!(
        said.contains("`level` is both a variable of limits and a binding"),
        "{said}"
    );
}

#[test]
fn a_loop_variable_and_a_parameter_hold_the_value() {
    let (printed, said) = ran(
        "for level in [0, 1] { print(d.plot(x: dose, y: change).hline(at: level)) }\nfor shade in [\"navy\", \"gold\"] { print(d.plot(x: dose, y: change).points(color: shade)) }\n",
    );
    for line in [
        "      1   hline   d          1   at 0, width 0.5, style dashed\n",
        "      1   hline   d          1   at 1, width 0.5, style dashed\n",
        "      1   points   d          4   color navy, size 4.5, alpha 1\n",
        "      1   points   d          4   color gold, size 4.5, alpha 1\n",
    ] {
        assert!(printed.contains(line), "{line}\n{printed}");
    }
    assert_eq!(said, "");
    // A function's parameter.
    let outcome = run(&format!(
        "/// A plot with a line at a level.\nfn marked(p: Plot, level: Float) -> Plot {{ p.hline(at: level) }}\n{FRAME}marked(d.plot(x: dose, y: change).points(), 1.5)\n"
    ));
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    assert!(
        outcome
            .output
            .stdout
            .contains("      2   hline    d          1   at 1.5, width 0.5, style dashed\n"),
        "{}",
        outcome.output.stdout
    );
}

#[test]
fn every_message_of_a_name_that_is_a_variable_or_a_value() {
    let mut all = String::new();
    for (name, script) in [
        // A loop variable of the name of a variable of the frame is never resolved silently.
        (
            "a_loop_variable_that_is_a_variable_too",
            "for dose in [1, 2] { print(d.plot(x: change, y: change).vline(at: dose)) }\n",
        ),
        // A binding that holds what the option does not take is not looked at.
        (
            "a_binding_of_another_type",
            "let places = [0, 1]\nd.plot(x: dose, y: change).hline(at: places)\n",
        ),
        (
            "text_where_a_number_is_taken",
            "let centre = \"0.5\"\nd.plot(x: dose, y: change).hline(at: centre)\n",
        ),
        (
            "a_number_where_text_is_taken",
            "let shade = 3\nd.plot(x: dose, y: change).points(color: shade)\n",
        ),
        (
            "neither_a_variable_nor_a_binding",
            "let centre = 0.5\nd.plot(x: dose, y: change).hline(at: center)\n",
        ),
        (
            "bound_text_that_is_no_colour",
            "let shade = \"blu\"\nd.plot(x: dose, y: change).points(color: shade)\n",
        ),
        (
            "a_bound_number_that_is_no_size",
            "let big = -2\nd.plot(x: dose, y: change).points(size: big)\n",
        ),
    ] {
        let outcome = run(&format!("{FRAME}{script}"));
        assert_eq!(outcome.exit_code, 1, "{name}\n{}", outcome.output.stdout);
        assert_eq!(outcome.output.stdout, "", "{name}");
        all.push_str(&format!("== {name}\n{}\n", outcome.output.stderr));
    }
    insta::assert_snapshot!("every_message", all);
}
