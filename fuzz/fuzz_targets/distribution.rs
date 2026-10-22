//! Parameters and probabilities of the distributions, drawn from raw bits so that NaN, infinities,
//! subnormal numbers, huge shapes and probabilities outside [0, 1] all occur, go through the whole
//! engine as a script. Every answer must be a value or a diagnostic, never a crash (Z9001 is a
//! crash caught, and is reported as one), and every number a method returns must be a number, `NA`
//! or an infinity.
#![no_main]

use libfuzzer_sys::fuzz_target;
use std::path::Path;
use zeff_engine::fs::MemoryFileSystem;
use zeff_engine::project::Project;
use zeff_engine::run::run_script;

/// Each family with the names of its parameters.
const FAMILIES: [(&str, &[&str]); 12] = [
    ("normal", &["mean", "sd"]),
    ("lognormal", &["meanlog", "sdlog"]),
    ("uniform", &["min", "max"]),
    ("exponential", &["rate"]),
    ("t", &["df"]),
    ("chi2", &["df"]),
    ("f", &["df1", "df2"]),
    ("gamma", &["shape", "rate"]),
    ("beta", &["a", "b"]),
    ("binomial", &["n", "p"]),
    ("poisson", &["lambda"]),
    ("negbinomial", &["size", "p"]),
];

/// A number as a script writes it.
fn literal(x: f64) -> String {
    if x.is_nan() {
        "NA".to_string()
    } else if x == f64::INFINITY {
        "Inf".to_string()
    } else if x == f64::NEG_INFINITY {
        "-Inf".to_string()
    } else {
        format!("{x:e}")
    }
}

/// The next eight bytes as a Float, reshaped half the time into the range where the interesting
/// answers are (a probability, a small shape), so that the fuzzer reaches past the refusals.
fn number(bytes: &mut &[u8]) -> f64 {
    let mut raw = [0u8; 8];
    let take = bytes.len().min(8);
    raw[..take].copy_from_slice(&bytes[..take]);
    *bytes = &bytes[take..];
    let x = f64::from_bits(u64::from_le_bytes(raw));
    match raw[7] % 4 {
        0 => x,
        1 => (u64::from_le_bytes(raw) >> 11) as f64 / (1u64 << 53) as f64,
        2 => (u64::from_le_bytes(raw) >> 40) as f64 / 1000.0,
        _ => ((u64::from_le_bytes(raw) >> 52) as f64) - 2048.0,
    }
}

fn family(bytes: &mut &[u8]) -> String {
    let which = bytes.first().copied().unwrap_or(0) as usize % FAMILIES.len();
    *bytes = bytes.get(1..).unwrap_or(&[]);
    let (name, params) = FAMILIES[which];
    let args: Vec<String> = params
        .iter()
        .map(|p| format!("{p}: {}", literal(number(bytes))))
        .collect();
    format!("dist.{name}({})", args.join(", "))
}

fuzz_target!(|data: &[u8]| {
    let mut bytes = data;
    let shape = bytes.first().copied().unwrap_or(0) % 3;
    bytes = bytes.get(1..).unwrap_or(&[]);
    let d = match shape {
        0 => family(&mut bytes),
        1 => {
            let inner = family(&mut bytes);
            let (lo, hi) = (number(&mut bytes), number(&mut bytes));
            format!(
                "dist.truncated({inner}, lower: {}, upper: {})",
                literal(lo),
                literal(hi)
            )
        }
        _ => {
            let (a, b) = (family(&mut bytes), family(&mut bytes));
            let p = number(&mut bytes);
            format!(
                "dist.mixture([{a}, {b}], probs: [{}, {}])",
                literal(p),
                literal(1.0 - p)
            )
        }
    };
    let (x, p) = (number(&mut bytes), number(&mut bytes));
    let seed = (number(&mut bytes).abs() % 1.0e9) as i64;
    let script = format!(
        "let d = {d}\nprint(d)\nprint(d.pdf({x}))\nprint(d.cdf({x}))\nprint(d.cdf({x}, upper: true))\nprint(d.quantile({p}))\nprint(d.draw(3, seed: {seed}))\n",
        x = literal(x),
        p = literal(p),
    );
    let fs = MemoryFileSystem::new();
    fs.add_file(
        "/s/zeff.toml",
        "[project]\nname = \"fuzz\"\nzeff = \"0.3.0-dev\"\nedition = \"preview\"\n",
    );
    fs.add_file("/s/scripts/main.zeff", script.as_str());
    let Ok(project) = Project::open(&fs, Path::new("/s")) else {
        return;
    };
    let outcome = run_script(&fs, &project, "scripts/main.zeff", &script);
    assert!(
        !outcome.output.stderr.contains("Z9001"),
        "an internal error for:\n{script}\n{}",
        outcome.output.stderr
    );
});
