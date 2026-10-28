//! Random formulas and small frames go through the checker, the model matrix and `reg`, which
//! must answer with a result or a diagnostic, never an internal error (Z9001) or a crash. The
//! bytes choose the frame (one to twelve rows of five variables, with missing values, repeated
//! values and extremes, and a few category labels), the formula's terms and the fit's options;
//! the fit is then printed, used to predict on its own frame and asked for its influence
//! measures. A Z9001 in the messages is a failure of the target.
#![no_main]

use libfuzzer_sys::fuzz_target;
use std::path::Path;
use zeff_engine::fs::MemoryFileSystem;
use zeff_engine::project::Project;
use zeff_engine::run::run_script;

/// The terms a formula is built from: variables, transformations, interactions, categories with
/// each contrast, an arithmetic term, a duplicate and the intercept removed.
const TERMS: [&str; 16] = [
    "a",
    "b",
    "d",
    "c",
    "a^2",
    "log(a)",
    "a:b",
    "a*b",
    "category(c)",
    "category(c, contrast: sum)",
    "category(c, contrast: helmert)",
    "category(d)",
    "(a * 2)",
    "a + a",
    "-1",
    "d:category(c)",
];

/// The options of a fit, from one byte.
const OPTIONS: [&str; 8] = [
    "",
    ", se: robust",
    ", se: hc2",
    ", se: hc3",
    ", cluster: c",
    ", weights: a",
    ", weights: b, weight_type: frequency",
    ", cluster: d, se: robust",
];

/// A numeric cell from one byte: a missing value, zero, ones, extremes, a fraction, or a small
/// integer, so that collinearity, scaling and dropped rows all arise.
fn number(byte: u8) -> String {
    match byte % 16 {
        0 => String::new(),
        1 => "0".to_string(),
        2 => "1".to_string(),
        3 => "-1".to_string(),
        4 => "1e300".to_string(),
        5 => "-1e300".to_string(),
        6 => "1e-300".to_string(),
        7 => "0.5".to_string(),
        8 => "2".to_string(),
        9 => "3.25".to_string(),
        10 => "-0.0".to_string(),
        11 => "1e8".to_string(),
        _ => (i64::from(byte) - 128).to_string(),
    }
}

fn label(byte: u8) -> &'static str {
    match byte % 5 {
        0 => "x",
        1 => "y",
        2 => "z",
        3 => "",
        _ => "x",
    }
}

fuzz_target!(|data: &[u8]| {
    let Some((&shape, rest)) = data.split_first() else {
        return;
    };
    let rows = 1 + usize::from(shape % 12);
    let terms = 1 + usize::from((shape / 12) % 4);
    let mut bytes = rest.iter().copied();
    let mut next = || bytes.next().unwrap_or(0);
    // The frame.
    let mut csv = String::from("y,a,b,c,d\n");
    for _ in 0..rows {
        let y = number(next());
        let a = number(next());
        let b = number(next());
        let c = label(next());
        let d = next() % 2;
        csv.push_str(&format!("{y},{a},{b},{c},{d}\n"));
    }
    // The formula and the options.
    let mut formula = String::from("y ~ ");
    for i in 0..terms {
        if i > 0 {
            formula.push_str(" + ");
        }
        formula.push_str(TERMS[usize::from(next()) % TERMS.len()]);
    }
    let options = OPTIONS[usize::from(next()) % OPTIONS.len()];
    let script = format!(
        "let f = import(\"data/f.csv\")\nlet m = f.reg({formula}{options})\nm\nm.predict(newdata: f)\nm.influence()\nm.test(a == 0)\nm.derive(q = a / b)\n"
    );
    let fs = MemoryFileSystem::new();
    fs.add_file(
        "/p/zeff.toml",
        format!(
            "[project]\nname = \"p\"\nzeff = \"{}\"\nedition = \"preview\"\n",
            zeff_engine::VERSION
        ),
    );
    fs.add_file("/p/data/f.csv", csv);
    fs.add_file("/p/scripts/main.zeff", script.clone());
    let project = Project::open(&fs, Path::new("/p")).expect("the fuzz project opens");
    let outcome = run_script(&fs, &project, "scripts/main.zeff", &script);
    assert!(
        !outcome.output.stderr.contains("Z9001"),
        "an internal error on\n{script}\n{}",
        outcome.output.stderr
    );
});
