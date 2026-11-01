//! A column that shadows a binding of the same name (Language Reference, section 3.7, "Name
//! lookup": inside a frame method the variable wins, with W0102). The checker, which does not see
//! a frame's columns before the schema pass, types the name as the binding, so a scalar function
//! given it is not marked for row-by-row application; the interpreter therefore lifts the call
//! when it runs (release 0.6.0, found at step 5 by the samples). Before the fix `sqrt(x)` received
//! the whole column and gave `NA`, and `year(d)` an internal error.

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
