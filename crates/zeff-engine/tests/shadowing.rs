//! A column that shadows a binding of the same name (Language Reference, section 3.7, "Name
//! lookup": inside a frame method the variable wins, with W0102). The checker, which does not see
//! a frame's columns before the schema pass, types the name as the binding, so a scalar function
//! given it is not marked for row-by-row application; the interpreter therefore lifts the call
//! when it runs (release 0.6.0, found at step 5 by the samples). Before the fix `sqrt(x)` received
//! the whole column and gave `NA`, and `year(d)` an internal error.

//!
//! A constant of `core` (`PI`, `E`) is the last thing a bare name inside a frame method can be:
//! the frame's variable first, then a binding, then the constant. Until step 20b of release 0.7.0
//! the constant was not looked for there at all, and `d.add(area = PI * r^2)` was refused for
//! want of a variable `PI`.

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

#[test]
fn a_scalar_function_lifts_over_a_column_that_shadows_a_binding() {
    let cases = [
        (
            "let x = 4\nframe(x = [9, 16]).add(y = sqrt(x))[\"y\"]\n",
            "Column<Float> · 2 values\n3\n4\n",
        ),
        (
            "let d = to(\"2026-10-16\", Date)\nframe(d = [to(\"2024-02-29\", Date), to(\"1999-12-31\", Date)]).add(y = year(d))[\"y\"]\n",
            "Column<Int> · 2 values\n2024\n1999\n",
        ),
        (
            "let n = 2.5\nframe(n = [1.5, 7.5]).add(m = min(n, 3), r = round(n))[\"m\"]\n",
            "Column<Float> · 2 values\n1.5\n3\n",
        ),
        (
            "let n = 2.5\nframe(n = [1.5, 7.5]).add(m = min(n, 3), r = round(n))[\"r\"]\n",
            "Column<Float> · 2 values\n2\n8\n",
        ),
        // A built-in written in Zeff chooses its form from the values it is given.
        (
            "let x = 4\nframe(x = [9, 16]).add(c = clamp(x, 10, 12))[\"c\"]\n",
            "Column<Int> · 2 values\n10\n12\n",
        ),
    ];
    for (script, want) in cases {
        let outcome = run(script);
        assert_eq!(outcome.exit_code, 0, "{script}\n{}", outcome.output.stderr);
        assert_eq!(outcome.output.stdout, want, "{script}");
        assert!(
            outcome
                .output
                .stderr
                .contains("is both a variable of the frame and a binding"),
            "{script}\n{}",
            outcome.output.stderr
        );
    }
}

#[test]
fn a_constant_of_core_is_read_inside_a_frame_method() {
    let cases = [
        (
            "frame(r = [1.0, 2.0]).add(area = PI * r^2)[\"area\"]\n",
            "Column<Float> · 2 values\n3.14159\n12.5664\n",
        ),
        // On its own, as an argument of a function and as a named argument.
        (
            "frame(r = [9, 16]).add(y = sqrt(E), z = r * E, w = round(PI, digits: 2))\n",
            "frame · 2 rows × 4 variables\n\n       r         y         z      w\n  1    9   1.64872   24.4645   3.14\n  2   16   1.64872   43.4925   3.14\n",
        ),
        (
            "frame(r = [1.0, 2.0]).add(l = log(r, base: E))[\"l\"]\n",
            "Column<Float> · 2 values\n0\n0.693147\n",
        ),
        // In a condition on the rows, in a summary, and in an arm of `case`.
        (
            "frame(r = [1.0, 4.0]).where(r > E)\n",
            "frame · 1 row × 1 variable\n\n      r\n  1   4\n",
        ),
        (
            "frame(r = [1.0, 4.0]).aggregate(m = max(r) / E)\n",
            "frame · 1 row × 1 variable\n\n            m\n  1   1.47152\n",
        ),
        (
            "frame(r = [1.0, 4.0]).add(a = case { r > E => PI, _ => 0.0 })[\"a\"]\n",
            "Column<Float> · 2 values\n0\n3.14159\n",
        ),
        // Beside a loop variable that stands for a variable.
        (
            "let d = frame(r = [1.0, 2.0])\nfor v in vars(r) { print(d.add(z = v * PI)[\"z\"]) }\n",
            "Column<Float> · 2 values\n3.14159\n6.28319\n",
        ),
    ];
    for (script, want) in cases {
        let outcome = run(script);
        assert_eq!(outcome.exit_code, 0, "{script}\n{}", outcome.output.stderr);
        assert_eq!(outcome.output.stdout, want, "{script}");
        assert_eq!(outcome.output.stderr, "", "{script}");
    }
}

#[test]
fn a_variable_named_as_a_constant_of_core_hides_it() {
    let cases = [
        (
            "frame(r = [1.0, 2.0], PI = [3, 3]).add(area = PI * r^2)[\"area\"]\n",
            "Column<Float> · 2 values\n3\n12\n",
        ),
        // A scalar function given the name applies row by row to the variable.
        (
            "frame(E = [9, 16]).add(y = sqrt(E), z = E * 2)\n",
            "frame · 2 rows × 3 variables\n\n       E   y    z\n  1    9   3   18\n  2   16   4   32\n",
        ),
        (
            "frame(r = [1.0, 2.0, 3.0], E = [10, 20, 30]).where(r < E / 10)\n",
            "frame · 0 rows × 2 variables\n\n     r   E\n",
        ),
    ];
    for (script, want) in cases {
        let outcome = run(script);
        assert_eq!(outcome.exit_code, 0, "{script}\n{}", outcome.output.stderr);
        assert_eq!(outcome.output.stdout, want, "{script}");
        // The variable is used without a warning: nothing of the script has the name.
        assert!(
            !outcome.output.stderr.contains("warning"),
            "{script}\n{}",
            outcome.output.stderr
        );
    }
}

#[test]
fn a_name_that_is_neither_variable_nor_binding_nor_constant_is_still_refused() {
    // `Pi` is not `PI`: the refusal is the one for a variable, as before.
    let outcome = run("let d = frame(r = [1.0, 2.0])\nd.add(a = r * Pi)\n");
    assert_eq!(outcome.exit_code, 1);
    assert!(
        outcome
            .output
            .stderr
            .starts_with("error[Z0101]: no variable `Pi` in `d`\n"),
        "{}",
        outcome.output.stderr
    );
    // A constant of `core` cannot be bound, so no binding ever stands between it and a variable.
    let outcome = run("let PI = 3\nframe(r = [1.0, 2.0]).add(a = r * PI)\n");
    assert_eq!(outcome.exit_code, 1);
    assert!(
        outcome
            .output
            .stderr
            .starts_with("error[Z0105]: `PI` is a name of `core` and cannot be rebound\n"),
        "{}",
        outcome.output.stderr
    );
}
