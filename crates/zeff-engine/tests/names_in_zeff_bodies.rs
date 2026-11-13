//! The names inside the bodies of the built-ins written in Zeff (release 0.7.0, step 32d). A
//! built-in that takes a frame cannot know the names of its variables. Until this step `mad`,
//! `skewness` and `kurtosis` bound a name (`centre`) and used it inside a method of the frame
//! given, where a variable of that name comes first: on a frame that had a variable called
//! `centre` they gave a wrong result under a warning (W0102), and `skewness` and `kurtosis` of
//! a variable called `dev2` gave a wrong result under a note. These tests hold the four frame
//! functions to the same answer whatever the other variables of the frame are called, and
//! whatever the variable itself is called. The values are R's (4.6.0): `mad(x)`, `IQR(x)` and
//! the adjusted moments of Joanes and Gill computed from their definitions.

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

/// Each function with what it prints for 1, 2, 3, 9. R: `mad` 1.4826, `IQR` 2.75, G1
/// 1.69638679932913, G2 3.01436004162331.
const FUNCTIONS: [(&str, &str); 4] = [
    ("iqr", "2.75"),
    ("mad", "1.4826"),
    ("skewness", "1.69639"),
    ("kurtosis", "3.01436"),
];

/// Every name the four bodies bind or define, and two they do not.
const NAMES: [&str; 14] = [
    "centre",
    "clean",
    "count",
    "m",
    "g1",
    "g2",
    "value",
    "deviation",
    "dev2",
    "dev3",
    "dev4",
    "below",
    "f",
    "wage",
];

#[test]
fn the_other_variables_of_the_frame_do_not_change_the_answer() {
    for (function, printed) in FUNCTIONS {
        for name in NAMES {
            let script = format!(
                "let plain = frame(x = [1.0, 2, 3, 9])\nlet d = frame(x = [1.0, 2, 3, 9], {name} = [5.0, 7, 5, 6])\nd.{function}(x)\nd.{function}(x) == plain.{function}(x)\n"
            );
            let out = run(&script);
            assert_eq!(out.exit_code, 0, "{script}{}", out.output.stderr);
            assert_eq!(
                out.output.stdout,
                format!("{printed}\ntrue\n"),
                "{function} on a frame with a variable `{name}`"
            );
            // No warning that a name is both a variable and a binding, and no note.
            assert_eq!(out.output.stderr, "", "{function} with `{name}`");
        }
    }
}

#[test]
fn the_name_of_the_variable_itself_does_not_change_the_answer() {
    for (function, printed) in FUNCTIONS {
        for name in NAMES {
            let script = format!(
                "let plain = frame(x = [1.0, 2, 3, 9])\nlet d = frame({name} = [1.0, 2, 3, 9])\nd.{function}({name})\nd.{function}({name}) == plain.{function}(x)\n"
            );
            let out = run(&script);
            assert_eq!(out.exit_code, 0, "{script}{}", out.output.stderr);
            assert_eq!(
                out.output.stdout,
                format!("{printed}\ntrue\n"),
                "{function} of a variable called `{name}`"
            );
            assert_eq!(out.output.stderr, "", "{function} of `{name}`");
        }
    }
}

#[test]
fn the_rows_with_na_are_skipped_as_before() {
    // The warnings name the user's variable, then the deviations: as before the correction.
    let out = run(
        "let d = frame(x = [1.0, 2, NA, 3, 9], centre = [5.0, 5, 5, 5, 5])\nd.mad(x)\nd.skewness(x)\nd.kurtosis(x)\n",
    );
    assert_eq!(out.exit_code, 0, "{}", out.output.stderr);
    assert_eq!(out.output.stdout, "1.4826\n1.69639\n3.01436\n");
    assert_eq!(
        out.output.stderr,
        "warning[W0201]: 1 row with NA in x was skipped (line 2)\n\
         warning[W0201]: 1 row with NA in deviation was skipped (line 2)\n\
         warning[W0201]: 1 row with NA in x was skipped (line 3)\n\
         warning[W0201]: 1 row with NA in x was skipped (line 4)\n"
    );
}

/// A frame with a variable named as a parameter of the function is no obstacle (step 38c of
/// 0.7.0; open decision #140): inside the body the function's own names come first, so `var`
/// is the variable asked for and `type` the rule asked for, whatever the frame calls its
/// variables. Until that step such a frame was refused (Z0106). R: `IQR(x, type = 6)` 6.25,
/// `mad(x, constant = 1)` 1.
#[test]
fn a_variable_named_as_a_parameter_is_no_obstacle() {
    for (function, value) in FUNCTIONS {
        let out = run(&format!(
            "let d = frame(x = [1.0, 2, 3, 9], var = [5.0, 7, 5, 6])\nd.{function}(x)\n"
        ));
        assert_eq!(out.exit_code, 0, "{function}: {}", out.output.stderr);
        assert_eq!(out.output.stdout, format!("{value}\n"), "{function}");
        assert_eq!(out.output.stderr, "", "{function}");
        // The variable called `var` is itself asked for.
        let out = run(&format!(
            "let d = frame(var = [1.0, 2, 3, 9], x = [5.0, 7, 5, 6])\nd.{function}(var)\n"
        ));
        assert_eq!(out.exit_code, 0, "{function}: {}", out.output.stderr);
        assert_eq!(out.output.stdout, format!("{value}\n"), "{function}");
        assert_eq!(out.output.stderr, "", "{function}");
    }
    let out = run(
        "let d = frame(x = [1.0, 2, 3, 9], `type` = [5.0, 7, 5, 6], constant = [0.0, 0, 0, 0])\nd.iqr(x)\nd.iqr(x, type: 6)\nd.mad(x, constant: 1)\n",
    );
    assert_eq!(out.exit_code, 0, "{}", out.output.stderr);
    assert_eq!(out.output.stdout, "2.75\n6.25\n1\n");
    assert_eq!(out.output.stderr, "");
    // Nor are the other parameters.
    for name in ["data", "constant", "adjusted", "excess"] {
        let out = run(&format!(
            "let d = frame(x = [1.0, 2, 3, 9], {name} = [5.0, 7, 5, 6])\nd.mad(x)\nd.skewness(x)\nd.kurtosis(x)\nd.iqr(x)\n"
        ));
        assert_eq!(out.exit_code, 0, "{name}: {}", out.output.stderr);
        assert_eq!(
            out.output.stdout, "1.4826\n1.69639\n3.01436\n2.75\n",
            "{name}"
        );
    }
}
