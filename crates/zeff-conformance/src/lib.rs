//! The conformance cases (Testing Handbook, section 1, "Cross-platform identity").
//!
//! Each case computes something with the engine's own code on data generated here, and reduces the
//! result to a 64-bit digest of its bits. Every target prints the same list; the continuous-
//! integration job compares the lists and fails on any difference. The expected list is committed
//! in `expected.txt`, so a change to any result is a deliberate edit of that file.
//!
//! The digest is FNV-1a over the little-endian bytes of the results' IEEE 754 bit patterns. It is
//! only for spotting a difference; the published checksums of `zeff selftest` (release 0.3.0) will
//! use the hash of section 8.7. A not-a-number is reduced to one pattern, because its payload is
//! not part of the contract (a column never stores one).

use zeff_numeric::{mean, sd, sum, variance};

#[cfg(target_arch = "wasm32")]
mod browser;

const UNIT: f64 = 1.0 / 9_007_199_254_740_992.0; // 2^-53, exact
const POWERS_OF_TEN: [f64; 17] = [
    1e-8, 1e-7, 1e-6, 1e-5, 1e-4, 1e-3, 1e-2, 1e-1, 1e0, 1e1, 1e2, 1e3, 1e4, 1e5, 1e6, 1e7, 1e8,
];

/// SplitMix64: integer arithmetic only, so the same stream everywhere.
struct SplitMix64(u64);

impl SplitMix64 {
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A uniform number in [0, 1) with 53 random bits.
    fn unit(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 * UNIT
    }
}

struct Digest(u64);

impl Digest {
    fn new() -> Self {
        Digest(0xcbf2_9ce4_8422_2325)
    }

    fn write(&mut self, bits: u64) {
        for byte in bits.to_le_bytes() {
            self.0 ^= u64::from(byte);
            self.0 = self.0.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
}

/// The bits of a result, with every not-a-number reduced to one pattern.
fn bits(x: f64) -> u64 {
    if x.is_nan() {
        0x7ff8_0000_0000_0000
    } else {
        x.to_bits()
    }
}

/// One line of the report: the case name, its digest, and (for a single result) the result's bits
/// as a comment, to see where two targets part.
struct Line {
    name: String,
    digest: u64,
    detail: Option<u64>,
}

fn scalar(name: String, result: Option<f64>) -> Line {
    let value = result.map_or(0x7ff0_0000_0000_0001, bits); // a marker for "no value"
    let mut digest = Digest::new();
    digest.write(value);
    Line {
        name,
        digest: digest.0,
        detail: Some(value),
    }
}

fn series(name: &str, values: &[f64]) -> Line {
    let mut digest = Digest::new();
    for &x in values {
        digest.write(bits(x));
    }
    Line {
        name: name.to_string(),
        digest: digest.0,
        detail: None,
    }
}

fn datasets() -> Vec<(&'static str, Vec<f64>)> {
    let mut rng = SplitMix64(20_261_018);
    let uniform = (0..10_000).map(|_| rng.unit()).collect();
    let offset = (0..10_000).map(|_| 1e9 + rng.unit()).collect();
    let wide = (0..5_000)
        .map(|_| {
            let scale = POWERS_OF_TEN[(rng.next_u64() % 17) as usize];
            (rng.unit() * 2.0 - 1.0) * scale
        })
        .collect();
    vec![
        ("uniform", uniform),
        ("offset", offset),
        ("wide", wide),
        ("constant", vec![3.25; 100]),
        ("pair", vec![-1.5, 2.5]),
        ("single", vec![0.1]),
        ("empty", Vec::new()),
        ("infinite", vec![1.0, f64::INFINITY, 2.0]),
        (
            "mixed_infinite",
            vec![f64::INFINITY, f64::NEG_INFINITY, 1.0],
        ),
    ]
}

fn reducer_cases() -> Vec<Line> {
    let mut lines = Vec::new();
    for (name, values) in datasets() {
        lines.push(scalar(format!("sum/{name}"), Some(sum(&values))));
        lines.push(scalar(format!("mean/{name}"), mean(&values)));
        lines.push(scalar(format!("variance/{name}"), variance(&values)));
        lines.push(scalar(format!("sd/{name}"), sd(&values)));
    }
    lines
}

/// Cases that would show a target fusing a multiplication with an addition, flushing subnormal
/// numbers to zero, or rounding a basic operation differently.
fn canary_cases() -> Vec<Line> {
    let mut x = 1.0_f64;
    let mut orbit = Vec::new();
    for _ in 0..10_000 {
        x = x * 1.000_000_1 + 0.1;
        orbit.push(x);
    }

    let mut roots = Vec::new();
    for i in 1..=10_000_u32 {
        roots.push(f64::from(i).sqrt());
    }

    let mut subnormal = 2.225_073_858_507_201_4e-308_f64; // the smallest normal number
    let mut descent = Vec::new();
    for _ in 0..60 {
        subnormal *= 0.5;
        descent.push(subnormal);
    }

    let quotients: Vec<f64> = (1..=1_000_u32).map(|i| 1.0 / f64::from(i)).collect();
    vec![
        series("canary/multiply_add_orbit", &orbit),
        series("canary/square_roots", &roots),
        series("canary/subnormal_descent", &descent),
        series("canary/quotients", &quotients),
    ]
}

/// A text as a line: the digest of its bytes.
fn text(name: &str, value: &str) -> Line {
    bytes(name, value.as_bytes())
}

/// A file's bytes as a line: their digest, eight at a time.
fn bytes(name: &str, value: &[u8]) -> Line {
    let mut digest = Digest::new();
    for chunk in value.chunks(8) {
        let mut word = [0u8; 8];
        word[..chunk.len()].copy_from_slice(chunk);
        digest.write(u64::from_le_bytes(word));
    }
    Line {
        name: name.to_string(),
        digest: digest.0,
        detail: None,
    }
}

/// The first milestone of release 0.1.0: a script that reads a CSV and prints the mean of a
/// column, run through the lexer, parser, checker and interpreter on an in-memory project. Its
/// printed output must be the same bytes on every target.
/// The milestone script run on its in-memory project: the whole engine from the lexer to the
/// printed output, on every construct the releases so far run.
fn milestone_run() -> Result<zeff_engine::run::Outcome, &'static str> {
    use std::path::Path;
    use zeff_engine::fs::MemoryFileSystem;
    use zeff_engine::project::Project;
    use zeff_engine::run::run_script;

    let fs = MemoryFileSystem::new();
    fs.add_file(
        "/p/zeff.toml",
        format!(
            "[project]\nname = \"p\"\nzeff = \"{}\"\nedition = \"preview\"\n",
            zeff_engine::VERSION
        ),
    );
    fs.add_file(
        "/p/data/wages.csv",
        "id,wage,hours\n1,23.4,40\n2,19.0,35\n3,31.2,\n4,27.75,38\n5,1000000.125,41\n6,0.000031,39\n",
    );
    let script = "let wages = import(\"data/wages.csv\")\nwages.mean(wage)\nwages.mean(hours)\nprint(wages, [1, 2.5], 1e-7)\nwages.mean(id)\nlet built = frame(x = [1, 2.5, NA], flag = [true, false, true], name = [\"a\", \"b\", \"c\"])\nbuilt\nbuilt.mean(x)\nbuilt.mean(flag)\nwages.n_rows()\nwages.n_cols()\nwages.variables()\nbuilt.variables()\nwages.describe()\nbuilt.describe()\nwages.sd(wage)\nwages.sd(hours)\nwages.sd(id)\nbuilt.sd(x)\nbuilt.sd(flag)\nwages.summary()\nbuilt.summary([x, flag], stats: [n, mean, max])\nprint(wages.summary([wage]), NA, -0.0, 1e15, [true, NA])\nlet a = 7\nlet b = 2.5\na / 3\na ^ 0.5\nb ^ a\n(a + b) * (a - b)\n1e300 * 1e10\n0.1 + 0.2 == 0.3\n2..6\n[1 => \"one\", 2 => \"two\"][2]\n{x: 1.5, y: [1, 2]}.y[2]\nprint(-a)\n!(a > b)\nwages[\"wage\"] * 2\nwages[\"hours\"] / wages[\"id\"]\nwages[\"wage\"] > 25 && wages[\"id\"] != 3\nframe(ratio = wages[\"wage\"] / 10, big = wages[\"wage\"] >= 27.75).describe()\nto(wages[\"wage\"], Int)\nto(wages[\"id\"], Str)\nto(\"42\", Int)\ntype_of(wages)\nwages[\"hours\"].is(Int)\nto(built[\"name\"], Category)\nlevels(category(built[\"name\"], levels: [\"c\", \"b\", \"a\"], ordered: true))\nto(base(category(built[\"name\"]), \"c\"), Int)\nwages.label(id, [1 => \"first\", 2 => \"second\"]).describe()\nwages.recode(hours, [NA => 0, 99 => 98])\nbuilt.add(name = category(name)).recode(name, [\"a\" => \"b\"]).describe()\nlet f = wage ~ hours + hours^2 + id:hours - 1\nf\nf + log(hours) - hours^2\ntype_of(f)\nwage ~ interact([hours, id], order: 2) + category(id, contrast: sum)\n";
    match Project::open(&fs, Path::new("/p")) {
        Ok(project) => Ok(run_script(&fs, &project, "scripts/main.zeff", script)),
        Err(_) => Err("could not open"),
    }
}

fn milestone_cases() -> Vec<Line> {
    let outcome = match milestone_run() {
        Ok(outcome) => outcome,
        Err(reason) => return vec![text("milestone/project", reason)],
    };
    vec![
        text("milestone/stdout", &outcome.output.stdout),
        text("milestone/stderr", &outcome.output.stderr),
        scalar(
            "milestone/exit_code".to_string(),
            Some(f64::from(outcome.exit_code)),
        ),
    ]
}

/// A sample project of a release (`samples/<name>`), compiled into the binary and run on an
/// in-memory copy: a script test on every target (testing handbook, section 1). The stored output
/// in `samples/<name>/expected/` is what every target must print, byte for byte.
pub struct Sample {
    /// The folder under `samples/`, and the prefix of its cases in the report.
    pub name: &'static str,
    pub manifest: &'static str,
    /// The data files, as `data/<file>`, in the bytes the project holds (one is Windows-1252).
    pub data: &'static [(&'static str, &'static [u8])],
    /// The modules of `functions/`, as (name, text).
    pub modules: &'static [(&'static str, &'static str)],
    pub script: &'static str,
    /// What the sample must print, as stored.
    pub stdout: &'static str,
    pub stderr: &'static str,
}

/// The sample of release 0.1.0: every function of the first engine on one data file.
pub const FIRST_ENGINE: Sample = Sample {
    name: "first_engine",
    manifest: include_str!("../../../samples/first_engine/zeff.toml"),
    data: &[(
        "wages.csv",
        include_bytes!("../../../samples/first_engine/data/wages.csv"),
    )],
    modules: &[],
    script: include_str!("../../../samples/first_engine/scripts/main.zeff"),
    stdout: include_str!("../../../samples/first_engine/expected/stdout.txt"),
    stderr: include_str!("../../../samples/first_engine/expected/stderr.txt"),
};

/// The sample of release 0.2.0: a data-cleaning script end to end, from a German spreadsheet
/// export to grouped summaries and a shipped dataset.
pub const DATA_CLEANING: Sample = Sample {
    name: "data_cleaning",
    manifest: include_str!("../../../samples/data_cleaning/zeff.toml"),
    data: &[(
        "staff_de.csv",
        include_bytes!("../../../samples/data_cleaning/data/staff_de.csv"),
    )],
    modules: &[],
    script: include_str!("../../../samples/data_cleaning/scripts/main.zeff"),
    stdout: include_str!("../../../samples/data_cleaning/expected/stdout.txt"),
    stderr: include_str!("../../../samples/data_cleaning/expected/stderr.txt"),
};

/// The sample of release 0.3.0: a simulation and its p-values end to end, with no data file.
pub const SIMULATION: Sample = Sample {
    name: "simulation",
    manifest: include_str!("../../../samples/simulation/zeff.toml"),
    data: &[],
    modules: &[],
    script: include_str!("../../../samples/simulation/scripts/main.zeff"),
    stdout: include_str!("../../../samples/simulation/expected/stdout.txt"),
    stderr: include_str!("../../../samples/simulation/expected/stderr.txt"),
};

/// The sample of release 0.4.0: a regression analysis end to end on the shipped wages, read
/// from the `zeff:` data root, to a publication table exported as Word and LaTeX.
pub const REGRESSION: Sample = Sample {
    name: "regression",
    manifest: include_str!("../../../samples/regression/zeff.toml"),
    data: &[],
    modules: &[],
    script: include_str!("../../../samples/regression/scripts/main.zeff"),
    stdout: include_str!("../../../samples/regression/expected/stdout.txt"),
    stderr: include_str!("../../../samples/regression/expected/stderr.txt"),
};

/// The sample of release 0.5.0: the project's own functions in two modules of `functions/`,
/// brought in with `use`, with loops over variables and subsamples, `attempt`, `iterate`, `if`
/// and `case`, on the shipped wages.
pub const USER_FUNCTIONS: Sample = Sample {
    name: "user_functions",
    manifest: include_str!("../../../samples/user_functions/zeff.toml"),
    data: &[],
    modules: &[
        (
            "share_missing",
            include_str!("../../../samples/user_functions/functions/share_missing.zeff"),
        ),
        (
            "robustness",
            include_str!("../../../samples/user_functions/functions/robustness.zeff"),
        ),
    ],
    script: include_str!("../../../samples/user_functions/scripts/main.zeff"),
    stdout: include_str!("../../../samples/user_functions/expected/stdout.txt"),
    stderr: include_str!("../../../samples/user_functions/expected/stderr.txt"),
};

/// The sample of release 0.6.0: panels end to end, on the shipped wages and firms and three
/// generated files: a merge from the `zeff:` root, a wide file reshaped, aggregated and saved,
/// the panel variables, dates, fixed-effects regressions in one table, an event study and a
/// two-period difference in differences.
pub const PANEL: Sample = Sample {
    name: "panel",
    manifest: include_str!("../../../samples/panel/zeff.toml"),
    data: &[
        (
            "sales_wide.csv",
            include_bytes!("../../../samples/panel/data/sales_wide.csv"),
        ),
        (
            "policy.csv",
            include_bytes!("../../../samples/panel/data/policy.csv"),
        ),
        (
            "counties.csv",
            include_bytes!("../../../samples/panel/data/counties.csv"),
        ),
    ],
    modules: &[],
    script: include_str!("../../../samples/panel/scripts/main.zeff"),
    stdout: include_str!("../../../samples/panel/expected/stdout.txt"),
    stderr: include_str!("../../../samples/panel/expected/stderr.txt"),
};

/// Every sample project, oldest first; each adds three cases to the report, and one more for
/// every file it writes under `output/`.
pub const SAMPLES: [&Sample; 6] = [
    &FIRST_ENGINE,
    &DATA_CLEANING,
    &SIMULATION,
    &REGRESSION,
    &USER_FUNCTIONS,
    &PANEL,
];

/// Run a sample project on an in-memory copy of its files.
pub fn sample_run(sample: &Sample) -> zeff_engine::run::Outcome {
    sample_run_with_outputs(sample).0
}

/// Run a sample project on an in-memory copy of its files: what it printed, and the files it
/// wrote under `output/`, by name in order, with their bytes.
pub fn sample_run_with_outputs(
    sample: &Sample,
) -> (zeff_engine::run::Outcome, Vec<(String, Vec<u8>)>) {
    use std::path::Path;
    use zeff_engine::fs::{FileSystem, MemoryFileSystem};
    use zeff_engine::interpreter::Output;
    use zeff_engine::project::Project;
    use zeff_engine::run::{Outcome, run_script};

    let fs = MemoryFileSystem::new();
    fs.add_file("/s/zeff.toml", sample.manifest);
    for (file, bytes) in sample.data {
        fs.add_file(format!("/s/data/{file}"), *bytes);
    }
    for (name, text) in sample.modules {
        fs.add_file(format!("/s/functions/{name}.zeff"), *text);
    }
    fs.add_file("/s/scripts/main.zeff", sample.script);
    let outcome = match Project::open(&fs, Path::new("/s")) {
        Ok(project) => run_script(&fs, &project, "scripts/main.zeff", sample.script),
        Err(_) => Outcome {
            output: Output::default(),
            exit_code: 4,
        },
    };
    let outputs = fs
        .list(Path::new("/s/output"))
        .unwrap_or_default()
        .into_iter()
        .filter_map(|name| {
            fs.read(Path::new(&format!("/s/output/{name}")))
                .ok()
                .map(|bytes| (name, bytes))
        })
        .collect();
    (outcome, outputs)
}

fn sample_cases() -> Vec<Line> {
    SAMPLES
        .iter()
        .flat_map(|sample| {
            let (outcome, outputs) = sample_run_with_outputs(sample);
            // The first sample keeps the case names of release 0.1.0.
            let prefix = if sample.name == "first_engine" {
                "sample".to_string()
            } else {
                sample.name.to_string()
            };
            let mut lines = vec![
                text(&format!("{prefix}/stdout"), &outcome.output.stdout),
                text(&format!("{prefix}/stderr"), &outcome.output.stderr),
                scalar(
                    format!("{prefix}/exit_code"),
                    Some(f64::from(outcome.exit_code)),
                ),
            ];
            // The files the sample exported (step 19's Word and LaTeX tables), so that the four
            // targets prove they write the same bytes.
            for (name, data) in &outputs {
                lines.push(bytes(&format!("{prefix}/output/{name}"), data));
            }
            lines
        })
        .collect()
}

/// The maths kernel on a fixed grid (release 0.2.0, step 5): every elementary function, the
/// roundings and the divisions, so that the four targets prove the pure-Rust implementation gives
/// the same bits everywhere.
fn maths_cases() -> Vec<Line> {
    use zeff_numeric::maths::{self, RoundingMode};
    let grid: [f64; 58] = [
        0.0,
        -0.0,
        1e-300,
        -1e-300,
        1e-10,
        0.001,
        0.1,
        0.25,
        0.5,
        0.75,
        0.9,
        0.999,
        1.0,
        1.001,
        1.5,
        2.0,
        std::f64::consts::E,
        3.0,
        std::f64::consts::PI,
        4.0,
        5.5,
        8.0,
        10.0,
        12.5,
        100.0,
        1000.0,
        123456.789,
        1e6,
        1e10,
        1e15,
        1e100,
        1e300,
        -1e-10,
        -0.001,
        -0.25,
        -0.5,
        -0.75,
        -1.0,
        -1.5,
        -2.0,
        -3.0,
        -std::f64::consts::PI,
        -10.0,
        -100.0,
        -1e6,
        -1e100,
        std::f64::consts::FRAC_PI_4,
        std::f64::consts::FRAC_PI_2,
        std::f64::consts::PI,
        std::f64::consts::TAU,
        100.5,
        -99.5,
        1e-5,
        2.5e-8,
        7.0,
        0.3,
        0.6,
        0.123456789,
    ];
    type Elementary = fn(f64) -> f64;
    let functions: [(&str, Elementary); 13] = [
        ("exp", maths::exp),
        ("log", maths::log),
        ("log2", maths::log2),
        ("log10", maths::log10),
        ("log1p", maths::log1p),
        ("expm1", maths::expm1),
        ("sin", maths::sin),
        ("cos", maths::cos),
        ("tan", maths::tan),
        ("asin", maths::asin),
        ("acos", maths::acos),
        ("atan", maths::atan),
        ("sqrt", maths::sqrt),
    ];
    let mut lines = Vec::new();
    for (name, f) in functions {
        let values: Vec<f64> = grid.iter().map(|&x| f(x)).collect();
        lines.push(series(&format!("maths/{name}"), &values));
    }
    let powers: Vec<f64> = grid
        .iter()
        .flat_map(|&x| [0.5, 2.0, -1.0, 3.0, 0.1].map(|y| zeff_numeric::pow(x, y)))
        .collect();
    lines.push(series("maths/pow", &powers));
    let angles: Vec<f64> = grid
        .iter()
        .flat_map(|&y| [1.0, -1.0, 0.0, 2.5].map(|x| maths::atan2(y, x)))
        .collect();
    lines.push(series("maths/atan2", &angles));
    let rounded: Vec<f64> = grid
        .iter()
        .flat_map(|&x| {
            [0, 1, 2, 3, -1].into_iter().flat_map(move |d| {
                [
                    maths::round(x, d, RoundingMode::Even),
                    maths::round(x, d, RoundingMode::Away),
                ]
            })
        })
        .chain(
            grid.iter()
                .map(|&x| maths::round_significant(x, 3, RoundingMode::Even)),
        )
        .collect();
    lines.push(series("maths/round", &rounded));
    let divisions: Vec<f64> = grid
        .iter()
        .flat_map(|&a| {
            [2.0, -3.0, 0.1, 7.5].map(|b| maths::div_float(a, b) + 1000.0 * maths::mod_float(a, b))
        })
        .collect();
    lines.push(series("maths/div_mod", &divisions));
    lines
}

/// The special functions on fixed grids (release 0.3.0, step 2): the gamma and beta functions,
/// the error functions and the normal tails and quantiles, the incomplete gamma and beta functions
/// and their inverses, so that the four targets prove the same bits everywhere, deep tails
/// included.
fn special_cases() -> Vec<Line> {
    use zeff_numeric::special::*;
    let positives: [f64; 16] = [
        1e-10, 0.001, 0.1, 0.5, 1.5, 2.5, 4.5, 7.25, 10.0, 15.5, 30.0, 100.0, 170.5, 1000.0,
        12345.678, 1e5,
    ];
    let signed: [f64; 8] = [-0.5, -1.5, -3.7, -10.5, -100.5, -0.001, 0.75, 2.0];
    let mut lines = Vec::new();
    let gammas: Vec<f64> = positives
        .iter()
        .chain(signed.iter())
        .flat_map(|&x| [log_gamma(x), gamma(x)])
        .collect();
    lines.push(series("special/gamma", &gammas));
    let betas: Vec<f64> = positives
        .iter()
        .flat_map(|&a| positives.iter().map(move |&b| log_beta(a, b)))
        .collect();
    lines.push(series("special/log_beta", &betas));
    let erfs: Vec<f64> = [
        0.0, 1e-10, 0.1, 0.5, 1.0, 2.0, 3.0, 5.0, 6.0, 10.0, 20.0, 27.2, -0.5, -3.0, -10.0,
    ]
    .iter()
    .flat_map(|&x| {
        [
            erf(x),
            erfc(x),
            normal_tails(x).lower,
            normal_tails(x).upper,
        ]
    })
    .collect();
    lines.push(series("special/erf", &erfs));
    let probabilities: [f64; 12] = [
        1e-300, 1e-100, 1e-20, 1e-10, 1e-6, 0.001, 0.025, 0.1, 0.5, 0.9, 0.975, 0.999_999,
    ];
    let quantiles: Vec<f64> = probabilities
        .iter()
        .flat_map(|&p| [normal_quantile(p, false), normal_quantile(p, true)])
        .collect();
    lines.push(series("special/normal_quantile", &quantiles));
    let shapes: [f64; 9] = [0.1, 0.5, 1.0, 2.5, 10.0, 100.0, 1000.0, 1e4, 1e5];
    let mut gammas = Vec::new();
    let mut gamma_inverses = Vec::new();
    for &a in &shapes {
        for factor in [0.01, 0.5, 0.9, 1.0, 1.1, 2.0, 3.0] {
            let tails = incomplete_gamma(a, a * factor);
            gammas.extend([tails.lower, tails.upper]);
        }
        for &p in &probabilities[2..] {
            gamma_inverses.extend([
                incomplete_gamma_inverse(a, p, false),
                incomplete_gamma_inverse(a, p, true),
            ]);
        }
    }
    lines.push(series("special/incomplete_gamma", &gammas));
    lines.push(series("special/incomplete_gamma_inverse", &gamma_inverses));
    let pairs: [(f64, f64); 8] = [
        (0.5, 0.5),
        (2.0, 3.0),
        (0.5, 10.0),
        (5.0, 5.0),
        (30.0, 70.0),
        (1000.0, 1000.0),
        (1e4, 1e4),
        (5e5, 0.5),
    ];
    let mut betas = Vec::new();
    let mut beta_inverses = Vec::new();
    for &(a, b) in &pairs {
        for x in [1e-6, 0.01, 0.1, 0.3, 0.5, 0.7, 0.9, 0.99, 0.999_999] {
            let tails = incomplete_beta(a, b, x);
            betas.extend([tails.lower, tails.upper]);
        }
        for &p in &probabilities[2..] {
            beta_inverses.extend([
                incomplete_beta_inverse(a, b, p, false),
                incomplete_beta_inverse(a, b, p, true),
            ]);
        }
    }
    lines.push(series("special/incomplete_beta", &betas));
    lines.push(series("special/incomplete_beta_inverse", &beta_inverses));
    lines
}

/// The random stream (release 0.3.0, step 4): Philox blocks at fixed counters and keys, and the
/// first uniforms of several streams, so that the four targets prove the same draws everywhere.
fn random_cases() -> Vec<Line> {
    use zeff_numeric::random::{Stream, philox4x32_10};
    let mut words: Vec<f64> = Vec::new();
    for (counter, key) in [
        ([0u32, 0, 0, 0], [0u32, 0]),
        ([u32::MAX; 4], [u32::MAX; 2]),
        (
            [0x243f_6a88, 0x85a3_08d3, 0x1319_8a2e, 0x0370_7344],
            [0xa409_3822, 0x299f_31d0],
        ),
        ([7, 0, 3, 1], [20_261_019, 0]),
    ] {
        words.extend(philox4x32_10(counter, key).map(f64::from));
    }
    let mut lines = vec![series("random/philox", &words)];
    let mut uniforms = Vec::new();
    for (seed, ordinal, pass) in [
        (20_261_019u64, 1u32, 0u32),
        (20_261_019, 2, 0),
        (42, 1, 3),
        (u64::MAX, 7, 0),
    ] {
        let mut stream = Stream::new(seed, ordinal, pass);
        uniforms.extend((0..16).map(|_| stream.uniform()));
    }
    lines.push(series("random/uniform", &uniforms));
    lines
}

/// The distribution functions (release 0.3.0, steps 7 to 10): density, both tails, quantile and
/// the first draws of each family at fixed parameters, so that the four targets prove the same
/// values and the same draws everywhere.
fn distribution_cases() -> Vec<Line> {
    use zeff_core::value::Distribution;
    use zeff_dist::families;
    use zeff_numeric::random::Stream;
    let xs: [f64; 14] = [
        -40.0, -5.0, -1.0, -1e-10, 0.0, 1e-10, 0.25, 0.5, 1.0, 1.96, 3.0, 10.0, 40.0, 1e6,
    ];
    let ps: [f64; 9] = [
        1e-300,
        1e-10,
        0.001,
        0.025,
        0.5,
        0.975,
        0.999,
        1.0 - 1e-6,
        1.0 - 1e-10,
    ];
    let families: [(&'static str, &[(&'static str, f64)]); 18] = [
        ("normal", &[("mean", 0.0), ("sd", 1.0)]),
        ("normal", &[("mean", 100.0), ("sd", 15.0)]),
        ("lognormal", &[("meanlog", 0.0), ("sdlog", 1.0)]),
        ("lognormal", &[("meanlog", 3.0), ("sdlog", 0.5)]),
        ("uniform", &[("min", 2.0), ("max", 5.0)]),
        ("exponential", &[("rate", 1.0)]),
        ("exponential", &[("rate", 0.1)]),
        ("t", &[("df", 3.0)]),
        ("t", &[("df", 24.0)]),
        ("t", &[("df", 1e6)]),
        ("chi2", &[("df", 5.0)]),
        ("f", &[("df1", 3.0), ("df2", 20.0)]),
        ("gamma", &[("shape", 2.0), ("rate", 3.0)]),
        ("gamma", &[("shape", 100.0), ("rate", 0.1)]),
        ("beta", &[("a", 2.0), ("b", 3.0)]),
        ("binomial", &[("n", 10.0), ("p", 0.3)]),
        ("poisson", &[("lambda", 2.0)]),
        ("negbinomial", &[("size", 3.0), ("p", 0.4)]),
    ];
    let mut values = Vec::new();
    let mut draws = Vec::new();
    for (name, params) in families {
        let d = Distribution::Family {
            name,
            params: params.to_vec(),
        };
        for &x in &xs {
            let tails = families::cdf(&d, x);
            values.extend([families::pdf(&d, x), tails.lower, tails.upper]);
        }
        for &p in &ps {
            values.push(families::quantile(&d, p));
        }
        let mut stream = Stream::new(20_261_019, 1, 0);
        draws.extend((0..8).map(|_| families::draw(&d, &mut stream)));
    }
    // A truncation and a mixture (step 10), on the same grids.
    let truncated = Distribution::Truncated {
        inner: Box::new(Distribution::Family {
            name: "normal",
            params: vec![("mean", 0.0), ("sd", 1.0)],
        }),
        lower: -1.0,
        upper: 2.0,
    };
    let mixture = Distribution::Mixture {
        parts: vec![
            (
                Distribution::Family {
                    name: "normal",
                    params: vec![("mean", 0.0), ("sd", 1.0)],
                },
                0.3,
            ),
            (
                Distribution::Family {
                    name: "normal",
                    params: vec![("mean", 3.0), ("sd", 0.5)],
                },
                0.7,
            ),
        ],
    };
    for d in [truncated, mixture] {
        for &x in &xs {
            let tails = families::cdf(&d, x);
            values.extend([families::pdf(&d, x), tails.lower, tails.upper]);
        }
        for &p in &ps {
            values.push(families::quantile(&d, p));
        }
        let mut stream = Stream::new(20_261_019, 2, 0);
        draws.extend((0..8).map(|_| families::draw(&d, &mut stream)));
    }
    vec![series("dist/values", &values), series("dist/draws", &draws)]
}

/// The linear-algebra kernel (release 0.4.0, step 2): the reduction shape, the blocked QR with
/// its limited pivoting, the triangular inverse, the Cholesky factor and the singular values, on
/// generated data with magnitudes up to 1e12, so that the four targets prove the same bits.
fn kernels_cases() -> Vec<Line> {
    use zeff_kernels::cholesky::cholesky;
    use zeff_kernels::product::gram;
    use zeff_kernels::qr::least_squares;
    use zeff_kernels::svd::{condition_number, singular_values};
    use zeff_kernels::triangular::inverse_upper;
    use zeff_kernels::{Matrix, dot, sum, sum_of_squares};
    let mut rng = SplitMix64(0x0400_0002_5EED_0001);
    let values: Vec<f64> = (0..10_007)
        .map(|i| {
            let u = rng.unit() - 0.5;
            if i % 7 == 0 { u * 1e12 } else { u }
        })
        .collect();
    let others: Vec<f64> = (0..10_007).map(|_| rng.unit() * 1e7 - 5e6).collect();
    let mut lines = vec![series(
        "kernels/reductions",
        &[sum(&values), dot(&values, &others), sum_of_squares(&others)],
    )];
    // A 1,000 × 6 design: an intercept, two uniform columns, one at 1e9, one at 1e12, and a
    // column that is the sum of the two uniform ones, which the pivoting leaves out; the rows go
    // through the tall-skinny QR in blocks of 256, so four block factors are folded.
    let n = 1000;
    let mut columns = vec![vec![1.0; n]];
    for scale in [1.0, 1.0, 1e9, 1e12] {
        columns.push((0..n).map(|_| rng.unit() * scale).collect());
    }
    let duplicate: Vec<f64> = (0..n).map(|i| columns[1][i] + columns[2][i]).collect();
    columns.push(duplicate);
    let x = Matrix::from_columns(&columns);
    let y: Vec<f64> = (0..n)
        .map(|i| {
            2.0 + 0.5 * x.get(i, 1) - 0.25 * x.get(i, 2)
                + 1e-9 * x.get(i, 3)
                + 1e-12 * x.get(i, 4)
                + (rng.unit() - 0.5)
        })
        .collect();
    // The demeaning kernel (release 0.6.0, step 17): a three-effect unbalanced panel of 400 rows,
    // two columns at unit scale and one at 1e12, the demeaned values and the pass count.
    let demeaned = {
        use zeff_kernels::demean::{Effect, Options, demean};
        let mut rng2 = SplitMix64(0x0600_0017_5EED_0002);
        let (mut unit_of, mut period_of, mut region_of) = (Vec::new(), Vec::new(), Vec::new());
        for u in 0..60usize {
            let stay = if u % 9 == 0 {
                1
            } else {
                2 + (rng2.unit() * 7.0) as usize
            };
            let start = (rng2.unit() * 10.0) as usize;
            for t in 0..stay {
                unit_of.push(u);
                period_of.push((start + t) % 10);
                region_of.push(u % 4);
            }
        }
        let rows = unit_of.len();
        let a: Vec<f64> = (0..rows).map(|_| rng2.unit() - 0.5).collect();
        let b: Vec<f64> = (0..rows).map(|i| rng2.unit() + unit_of[i] as f64).collect();
        let c: Vec<f64> = (0..rows).map(|_| (rng2.unit() - 0.5) * 1e12).collect();
        let out = demean(
            &[&a, &b, &c],
            &[
                Effect {
                    group_of_row: &unit_of,
                    groups: 60,
                },
                Effect {
                    group_of_row: &period_of,
                    groups: 10,
                },
                Effect {
                    group_of_row: &region_of,
                    groups: 4,
                },
            ],
            None,
            Options::default(),
        );
        let mut all: Vec<f64> = out.columns.concat();
        all.push(out.passes as f64);
        all.push(out.singletons_dropped as f64);
        all
    };
    lines.push(series("kernels/demean", &demeaned));
    let fit = least_squares(&x, &y, 1e-7, 256);
    let mut summary: Vec<f64> = fit
        .coefficients
        .iter()
        .map(|c| c.unwrap_or(f64::NAN))
        .collect();
    summary.push(fit.residual_sum_of_squares);
    summary.push(fit.rank as f64);
    summary.extend(fit.order.iter().map(|&o| o as f64));
    lines.push(series("kernels/least_squares", &summary));
    lines.push(series("kernels/r_factor", fit.r.data()));
    lines.push(series(
        "kernels/inverse",
        inverse_upper(&fit.r).expect("accepted pivots").data(),
    ));
    let kept = Matrix::from_columns(
        &fit.order[..fit.rank]
            .iter()
            .map(|&j| x.column(j).to_vec())
            .collect::<Vec<_>>(),
    );
    lines.push(series(
        "kernels/cholesky",
        cholesky(&gram(&kept))
            .expect("a positive-definite Gram matrix")
            .data(),
    ));
    let mut spectrum = singular_values(&x);
    spectrum.push(condition_number(&fit.r));
    lines.push(series("kernels/singular_values", &spectrum));
    lines
}

/// The regressions of release 0.4.0 (step 10): three fits through the whole engine, on the
/// shipped `wages` and on an in-memory frame, their printed summaries and messages digested, so
/// that the four targets prove the same estimates to the printed digit and the same bits in the
/// fit's values.
fn regress_cases() -> Vec<Line> {
    use std::path::Path;
    use zeff_engine::fs::MemoryFileSystem;
    use zeff_engine::project::Project;
    use zeff_engine::run::run_script;
    let fs = MemoryFileSystem::new();
    fs.add_file(
        "/r/zeff.toml",
        format!(
            "[project]\nname = \"r\"\nzeff = \"{}\"\nedition = \"preview\"\n",
            zeff_engine::VERSION
        ),
    );
    let project = match Project::open(&fs, Path::new("/r")) {
        Ok(project) => project,
        Err(_) => return vec![text("regress/project", "the project did not open")],
    };
    let scripts = [
        (
            "arithmetic",
            "dataset(wages).reg(log(wage) ~ educ + exper + exper^2 + female)\n",
        ),
        (
            "categories",
            "dataset(wages).reg(log(wage) ~ category(industry, contrast: sum) + educ * female, level: 0.9)\n",
        ),
        (
            "collinear",
            "let w = dataset(wages).add(twice = educ * 2, big = hours * 1e9)\nw.reg(wage ~ educ + twice + big + exper - 1)\n",
        ),
        (
            "clustered",
            "let w = dataset(wages)\nw.reg(log(wage) ~ educ + exper + female, cluster: worker)\nw.reg(log(wage) ~ educ + exper + female, se: hc3)\n",
        ),
        (
            "weighted",
            "let w = dataset(wages)\nw.reg(log(wage) ~ educ + female, weights: hours)\nw.reg(log(wage) ~ educ + female, weights: exper, weight_type: frequency)\n",
        ),
        // Fixed effects (release 0.6.0, step 18): one, two and interacted effects, clustered
        // errors with an effect nested in the clusters, the absorbed regressors and the
        // `report` table with its `Fixed effects` row.
        (
            "fe",
            "let w = dataset(wages)\nlet m1 = w.reg(log(wage) ~ educ + exper + exper^2 + female, fe: firm)\nm1\nlet m2 = w.reg(log(wage) ~ educ + exper + exper^2 + female, fe: industry + year, cluster: worker)\nm2\nlet m3 = w.reg(log(wage) ~ educ + exper + exper^2 + female, fe: worker + year, cluster: worker)\nm3\nw.reg(log(wage) ~ educ + female, fe: industry:year, weights: hours)\nm2.r_squared()\nreport([m1, m2, m3], stats: [n_obs, r_squared, r_squared_within])\nm1.predict(newdata: w.head(5))\n",
        ),
        // Event-study terms (release 0.6.0, step 19): the leads and lags of a staggered panel
        // with never-treated units under two-way fixed effects, another reference period, and
        // the refusal of a period absent from the data.
        (
            "event",
            "let d = frame(unit = [1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4], year = [2010, 2011, 2012, 2013, 2010, 2011, 2012, 2013, 2010, 2011, 2012, 2013, 2010, 2011, 2012, 2013], treat = [2011, 2011, 2011, 2011, 2012, 2012, 2012, 2012, 2013, 2013, 2013, 2013, NA, NA, NA, NA], y = [1.0, 2.1, 2.4, 2.6, 1.5, 1.4, 2.6, 2.9, 0.8, 0.9, 1.1, 2.0, 1.2, 1.3, 1.1, 1.4])\nlet m = d.reg(y ~ event(year, treat), fe: unit + year)\nm\nreport([m])\nd.reg(y ~ event(year, treat, ref: 0), fe: unit + year)\nd.reg(y ~ event(year, treat, ref: 9), fe: unit + year)\n",
        ),
        // The result interface (step 15): the summary with its options, the estimates frame, the
        // mappings and the intervals of one fit.
        (
            "interface",
            "let m = dataset(wages).reg(log(wage) ~ educ + exper + exper^2 + female)\nm.summary(standardized: true, fit_measures: true)\nm.estimates()\nm.std_errors()\nm.p_values(educ)\nm.ci(educ, level: 0.9)\nm.ci()\nm.n_obs()\n",
        ),
    ];
    let mut lines = Vec::new();
    for (name, script) in scripts {
        let outcome = run_script(&fs, &project, "scripts/main.zeff", script);
        lines.push(text(
            &format!("regress/{name}/stdout"),
            &outcome.output.stdout,
        ));
        lines.push(text(
            &format!("regress/{name}/stderr"),
            &outcome.output.stderr,
        ));
    }
    lines
}

/// The control flow of release 0.5.0 (step 2): `if` and `case` through the whole engine, a
/// `case` over one value and row by row over the shipped `wages`, inside `add` and `where`, with
/// the rows no arm matched, their printed output and messages digested.
fn control_cases() -> Vec<Line> {
    use std::path::Path;
    use zeff_engine::fs::MemoryFileSystem;
    use zeff_engine::project::Project;
    use zeff_engine::run::run_script;
    let fs = MemoryFileSystem::new();
    fs.add_file(
        "/c/zeff.toml",
        format!(
            "[project]\nname = \"c\"\nzeff = \"{}\"\nedition = \"preview\"\n",
            zeff_engine::VERSION
        ),
    );
    // A module of the project (step 8), brought in by the `modules` case.
    fs.add_file(
        "/c/functions/clean.zeff",
        "/// Share of rows where a variable is missing.\n/// data: The frame to check.\n/// var: The variable whose missing values are counted.\n/// returns: The share, a Float between 0 and 1.\n/// > dataset(wages).share_missing(wage)\n/// 0.0114599\nfn share_missing(data: Frame, var: Var) -> Float {\n    data.where(is_missing(var)).n_rows() / data.n_rows()\n}\n\n/// Twice a number, private to this file.\n/// x: The number.\n/// returns: Its double.\n/// > _twice(2)\n/// 4\nfn _twice(x: Float) -> Float { x * 2 }\n\n/// Four times a number.\n/// x: The number.\n/// returns: Four times x.\n/// > quadruple(2)\n/// 8\nfn quadruple(x: Float) -> Float { _twice(_twice(x)) }\n",
    );
    let project = match Project::open(&fs, Path::new("/c")) {
        Ok(project) => project,
        Err(_) => return vec![text("control/project", "the project did not open")],
    };
    let scripts = [
        // The schema pass (release 0.6.0, step 20): a misspelt variable in the last line of a
        // script is reported before the first statement runs, and a frame literal's columns
        // are known; a file that does not exist is checked at run time.
        (
            "check",
            "let w = dataset(wages)\nw.mean(wage)\nlet f = frame(x = [1, 2], y = [3, 4])\nf.mean(x)\nlet m = import(\"data/absent.csv\")\nw.reg(log(wage) ~ educ + expr, fe: firm)\n",
        ),
        // The schema pass, part 2 (step 21): a mistake after a chain of verbs, caught before
        // the first statement runs.
        (
            "schemas",
            "let w = dataset(wages)\nw.mean(wage)\nlet s = w.where(year > 2015).add(lw = log(wage)).rename(pay = wage).select([worker, year, pay, lw])\ns.aggregate(m = mean(lw), by: year).mean(m)\ns.merge(dataset(firms).select([firm, size]), on: firm)\n",
        ),
        (
            "case",
            "let w = dataset(wages)\nlet reps = if w.n_rows() < 500 { 5000 } else { 1000 }\nreps\nlet b = w.add(band = case { wage < 15 => \"low\", wage < 25 => \"mid\", _ => \"high\" }, k = case { hours > 45 => wage * 1.5, hours > 35 => wage, educ > 12 => 2.5 })\nb.summary([k])\nb.where(case { hours > 40 => true, _ => false }).mean(wage)\ncase { w.n_rows() > 5 => \"big\", _ => \"small\" }\n",
        ),
        // The modules of step 8: a module's functions brought in by `use`, the full forms, and a
        // private helper reached through a public one.
        (
            "modules",
            "use functions::clean\nlet w = dataset(wages)\n[w.share_missing(wage), w.share_missing(hours), quadruple(3), clean::quadruple(1.5), functions::share_missing(w, exper)]\n",
        ),
        // The methods and function values of step 7: every function as a method, and a
        // function passed to a parameter of type `Fn` and called through it.
        (
            "methods",
            "let w = dataset(wages)\nw[\"wage\"].log().is_missing().print()\n(-2.5).abs()\n16.sqrt()\n2.max(7, 3)\n/// Applies a function to a float.\nfn applyf(f: Fn(Float) -> Float, x: Float) -> Float { f(x) }\n/// Halves.\nfn half(x: Float) -> Float { x / 2 }\n[applyf(abs, -3), applyf(sqrt, 16), applyf(half, 9)]\n",
        ),
        // The functions of step 6: the specification's `share_missing` and `robustness` (with
        // `cluster:` and no `fe:`), a default evaluated per call and an early `return`.
        (
            "share_missing",
            "let w = dataset(wages)\n/// Share of rows where a variable is missing.\nfn share_missing(data: Frame, var: Var) -> Float {\n    data.where(is_missing(var)).n_rows() / data.n_rows()\n}\n/// Three standard robustness checks for one specification.\nfn robustness(data: Frame, f: Formula, cluster: Var = worker) -> List<RegFit> {\n    [data.reg(f, se: robust), data.reg(f, cluster: cluster), data.where(year >= 2015).reg(f, se: robust)]\n}\n/// Winsorised share.\nfn band(x: Float, top: Float = x * 2) -> Str {\n    if x > top { return \"above\" }\n    \"within\"\n}\nw.share_missing(wage)\nreport(w.robustness(log(wage) ~ educ + exper))\n[band(1), band(3, top: 2)]\n",
        ),
        // The failed values of step 5: fits that fail inside `attempt`, their printed form,
        // `report` skipping them and a loop that guards on `is(Failed)`.
        (
            "attempt",
            "let w = dataset(wages)\nlet fits = [for cut in [10, 20, 30, 1000] {\n    attempt(w.where(wage > cut).reg(log(wage) ~ educ + female))\n}]\nfits[4]\nreport(fits)\n[for m in fits {\n    if m.is(Failed) { continue }\n    m.coef(educ)\n}]\n",
        ),
        // The collection functions of step 11 on lists, strings and a column: lengths in code
        // points, the ends, a new list by `append`, a reversed copy, and failed items counted.
        (
            "collections_basics",
            "let xs = [3, 1, 2]\nlen(xs)\nfirst(xs)\nlast(xs)\nappend(xs, 9)\nreversed(xs)\nlen(\"caf\\u{e9}\")\nlen(\"e\\u{301}\")\nlen(\"\\u{1F600}\")\nlet w = dataset(wages)\nlen(w[\"wage\"])\nlen([1 => \"a\", 2 => \"b\"])\nappend([1.5], 2)\nlet d = frame(x = [1, 2, 3, 4, 5], y = [2, 4, 7, 8, 11])\nlet fits = [for cut in [0, 2, 10] { attempt(d.where(x > cut).reg(y ~ x)) }]\nlen(fits)\n",
        ),
        // `table` of release 0.6.0, step 16 (open decision #72): a table from lists and from a
        // frame's columns, with the labels, and `ci()` at the fit's level (open decision #67).
        (
            "table",
            "table(title: \"sales by year\", columns: {year: [2010, 2011, 2012], sales: [120.5, 130, NA], region: [\"North\", \"South\", \"West\"], flag: [true, false, true], day: [date(\"2024-01-01\"), NA, date(\"1999-12-31\")]})\nlet w = dataset(wages).where(worker <= 2)\ntable(title: \"two workers\", columns: {worker: w[\"worker\"], year: w[\"year\"], wage: w[\"wage\"], female: w[\"female\"], industry: w[\"industry\"]})\ntable(title: \"one row\", columns: {k: [1]})\nlet m90 = dataset(wages).reg(log(wage) ~ educ + exper + female, level: 0.9)\nm90.ci()\nm90.ci(educ)\nm90.ci(educ, level: 0.95)\n",
        ),
        // `vif` of step 21: the factors of a textbook model, a categorical regressor's columns, a
        // fit without an intercept, and one regressor.
        (
            "vif",
            "let w = dataset(wages)\nlet m1 = w.reg(log(wage) ~ educ + exper + exper^2 + female)\nm1.vif()\nlet m2 = w.reg(log(wage) ~ educ + industry)\nm2.vif()\nlet m3 = w.reg(log(wage) ~ educ + exper - 1)\nm3.vif()\nw.reg(log(wage) ~ educ).vif()\n",
        ),
        // `skewness` and `kurtosis` of step 20: the four forms on a small sample, a symmetric one,
        // two values, no spread, and the wages.
        (
            "moments",
            "let d = frame(x = [2, 4, 4, 4, 5, 5, 7, 9])\nd.skewness(x)\nd.skewness(x, adjusted: false)\nd.kurtosis(x)\nd.kurtosis(x, excess: false, adjusted: false)\nd.kurtosis(x, excess: false)\nframe(y = [-5, 0, 5, 0, -5, 5]).skewness(y)\nframe(y = [1, 2]).skewness(y)\nframe(y = [1, 2]).kurtosis(y, adjusted: false)\nframe(y = [3, 3, 3, 3]).skewness(y)\nlet w = dataset(wages)\nw.skewness(wage)\nw.kurtosis(wage)\nw.skewness(educ, adjusted: false)\n",
        ),
        // `iqr` and `mad` of step 19: the nine quantile types, the constant, one value, two values,
        // an infinity, and the wages.
        (
            "spread",
            "let d = frame(x = [1, 2, 4, 7, 11])\nd.iqr(x)\nd.iqr(x, type: 6)\nd.iqr(x, type: 8)\nd.mad(x)\nd.mad(x, constant: 1)\nframe(y = [5]).mad(y)\nframe(y = [1, 3]).mad(y, constant: 1)\nframe(y = [1, Inf, 3]).iqr(y)\nlet w = dataset(wages)\nw.iqr(wage)\nw.mad(wage)\nw.iqr(educ, type: 2)\n",
        ),
        // `drop_missing` of step 18: the rows kept in order, the note naming the variables, a
        // range, nothing dropped, and every row dropped.
        (
            "drop_missing",
            "let d = frame(id = [1, 2, 3, 4], x = [1, NA, 3, 4], y = [NA, 2, 3, NA], z = [1, 1, 1, 1])\nd.drop_missing([x, y])\nd.drop_missing([x])\nd.drop_missing([z])\nd.drop_missing([id..z])\nd.where(is_missing(x)).drop_missing([x])\nlet w = dataset(wages)\nw.drop_missing([wage, hours]).n_rows()\n",
        ),
        // `row_mean` and `row_sum` of step 17 inside `add`: a list and a range of variables,
        // `min_valid:`, rows with every value missing, and a dataset's scale.
        (
            "rows",
            "let d = frame(q1 = [5, 4, NA, 1.5], q2 = [3, NA, NA, 2.5], q3 = [4, 4, NA, 1e12])\nd.add(m = row_mean([q1, q2, q3]), m2 = row_mean([q1..q3], min_valid: 2), s = row_sum([q1, q2, q3]))\nlet w = dataset(wages)\nw.add(hw = row_sum([hours, wage]), pay = row_mean([wage, educ, exper], min_valid: 3)).summary([hw, pay])\n",
        ),
        // The column functions of the data package written in Zeff (step 16): `between` on a
        // column with `NA`, `center` and `standardize` on columns, inside `add` and over a dataset.
        (
            "scale",
            "let d = frame(x = [2, 4, 9, NA])\nbetween(d[\"x\"], 3, 9)\ncenter(d[\"x\"])\nstandardize(d[\"x\"])\nd.add(c = center(x), z = standardize(x), inside = between(x, 3, 9))\nlet w = dataset(wages)\nw.add(z = standardize(wage), c = center(educ), prime = between(age, 25, 54)).summary([z, c, prime])\nstandardize(frame(k = [2, 2, 2])[\"k\"])\n",
        ),
        // `assert`, `n_missing` and `fill_missing` of step 15: a check that holds, the counts of
        // missing values on lists and a column, and filling with the type joined.
        (
            "missing",
            "assert(1 + 1 == 2, \"arithmetic\")\nn_missing([1, NA, 3, NA])\nn_missing([1, 2])\nlet d = frame(x = [1, NA, 3], s = [\"a\", NA, \"c\"])\nn_missing(d[\"x\"])\nfill_missing(d[\"x\"], 0)\nfill_missing(d[\"s\"], \"none\")\nfill_missing(2.5, 0)\nn_missing(fill_missing(d[\"x\"], 0))\nlet w = dataset(wages)\nn_missing(w[\"wage\"])\nassert(n_missing(fill_missing(w[\"wage\"], 0)) == 0, \"filled\")\n",
        ),
        // The first functions written in Zeff (step 14): `clamp`, `logit` and `inv_logit` at
        // the edges and on columns, with the warnings of their bodies placed at the call.
        (
            "logit",
            "clamp(5, 0, 3)\nclamp(-1.5, 0, 3)\nclamp(2, 5, 1)\nlogit(0.5)\nlogit(0.9)\nlogit(0)\nlogit(1)\ninv_logit(0)\ninv_logit(2.5)\ninv_logit(-800)\ninv_logit(40)\nlet w = dataset(wages)\nw.add(c = clamp(wage, 10, 50), p = inv_logit(educ - 12), q = logit(female * 0.5 + 0.25)).summary([c, p, q])\n",
        ),
        // `any`, `all` and `seq` of step 13: the truth table over lists and a Bool column, and
        // sequences with fractional steps, counting down, and the count fixed by the tolerance.
        (
            "collections_seq",
            "any([false, NA])\nany([true, NA])\nall([true, NA])\nall([false, NA])\nany([])\nall([])\nlet w = dataset(wages)\nany(w[\"wage\"] > 100)\nall(w[\"educ\"] >= 0)\nseq(1, 5)\nseq(0, 1, step: 0.25)\nseq(10, 0, step: -5)\nseq(0, 1, step: 0.1)\nseq(1e7, 1e7 + 1, step: 0.2)\nseq(1, 1)\nlen(seq(0, 1e12, step: 1e11))\nseq(2, 11, step: 3)\n",
        ),
        // The order functions of step 12: `unique` keeping first occurrences, `sorted` both ways
        // with `NA` last, `is_in` on values and on a column, `keys` and `equal`.
        (
            "collections_order",
            "let xs = [3, NA, 1, 3, -0.0, 0.0, 2, NA]\nunique(xs)\nsorted(xs)\nsorted(xs, desc: true)\nsorted([\"b\", \"a\", \"\\u{e9}\", \"B\"])\nsorted([true, false, true])\nis_in(NA, [NA])\nis_in(2, xs)\nis_in(5, xs)\nkeys({mu: 1, sd: 2})\nkeys([2 => \"b\", 1 => \"a\"])\nequal([1, 2], [1, 2])\nequal({a: 1, b: 2}, {b: 2, a: 1})\nequal([1 => \"a\"], [1 => \"b\"])\nequal(1, \"1\")\nlet w = dataset(wages)\nw.add(f = is_in(educ, [12, 16])).summary([f])\n",
        ),
        // `iterate` of step 10: Newton's square root to convergence, the path under `keep: true`
        // and the stop at `max:` with its warning.
        (
            "iterate",
            "/// One Newton step.\nfn newton(s: Record, a: Float) -> Record { {x: (s.x + a / s.x) / 2} }\n/// Close enough.\nfn close(old: Record, new: Record) -> Bool { abs(new.x - old.x) < 1e-12 }\niterate(start: {x: 1.0}, step: newton, args: {a: 2.0}, done: close)\niterate(start: {x: 1.0}, step: newton, args: {a: 1e12}, done: close, keep: true)\niterate(start: {x: 1.0}, step: newton, args: {a: 2.0}, done: close, max: 2)\n",
        ),
        // The loops of step 3: the draws of three passes under a seed, a `[for]` with its jumps,
        // and a loop over variables fitting a formula with the variable in two places.
        (
            "for_draws",
            "seed(20261030)\nlet draws = [for s in 1..3 { dist.normal().draw(4) }]\ndraws[1]\ndraws[2]\ndraws[3]\nlet picked = [for i in 1..20 {\n    if i > 12 { break }\n    if mod(i, 3) == 0 { continue }\n    i * i\n}]\npicked\nlet w = dataset(wages)\n[for y in vars(wage, hours) { w.reg(log(y) ~ educ + y:female).coef(educ) }]\n",
        ),
        // Dates and times as computable types (release 0.6.0, step 2): conversions from text
        // with and without a format, an offset applied, comparisons, `Date - Date`, dates in a
        // frame through `where`, `add`, `case`, `recode`, `update` and `sort`, and the
        // collection functions on them.
        (
            "dates",
            "let d = to(\"2026-10-16\", Date)\nd\nto(\"16/10/2026\", Date(\"%d/%m/%Y\"))\nto(\"2026-02-30\", Date)\nto(\"2026-10-16T09:30:00+02:00\", DateTime)\nd - to(\"1970-01-01\", Date)\nd > to(\"2026-01-01\", Date)\nto(d, DateTime)\nto(to(\"2026-10-16T23:59:59\", DateTime), Date)\nlet f = frame(id = [1, 2, 3], sold = [d, to(\"2024-02-29\", Date), to(\"1999-12-31\", Date)])\nf.where(sold > to(\"2000-01-01\", Date)).add(days = sold - to(\"2000-01-01\", Date))\nf.add(era = case { sold < to(\"2000-01-01\", Date) => \"old\", _ => \"new\" }, d2 = case { id == 1 => to(\"1970-01-01\", Date), _ => sold })\nf.recode(sold, [to(\"1999-12-31\", Date) => to(\"2000-01-01\", Date)]).update(sold = to(\"2030-01-01\", Date), where: id == 2).sort(sold)\nsorted([d, to(\"0001-01-01\", Date), to(\"9999-12-31\", Date)])\nmin(d, to(\"2024-02-29\", Date))\nn_distinct(f[\"sold\"])\n",
        ),
        // Deliberate exceptions (release 0.7.0, step 2): a pragma above a statement, at the end
        // of its first line, in a loop and in a function silences its warning there and nowhere
        // else; the statements without one keep theirs.
        ("pragma", PRAGMA_SCRIPT),
        // A note, a coded warning and a variable's name (release 0.7.0, step 3.1): `note`
        // with and without a code, `warn` with a code written bare and as text, a pragma that
        // allows the coded warning, and `name_of` in a message, from the script and from a
        // function.
        ("notes", NOTES_SCRIPT),
        // The enclosing frame (release 0.7.0, step 3.3): the functions that read the frame
        // around the call, `n()` and the row functions, inside `add`, `update`, `aggregate`
        // and `where`, under `by:` where the group's rows are the frame, from the script and
        // through a project function, and the Z0401 of a call that stands in no frame method.
        // The row functions move to Zeff at step 4 on this piece, and the digests must not
        // move with them.
        ("enclosing", ENCLOSING_SCRIPT),
        // A function literal with `map`, `filter` and `reduce` (release 0.7.0, step 3b): the
        // probabilities i/n, a literal reading a binding and a loop variable, a named function
        // as the value, the method form, nested literals, and a Failed item under `attempt`.
        ("map", MAP_SCRIPT),
    ];
    let mut lines = Vec::new();
    for (name, script) in scripts {
        let outcome = run_script(&fs, &project, "scripts/main.zeff", script);
        lines.push(text(
            &format!("control/{name}/stdout"),
            &outcome.output.stdout,
        ));
        lines.push(text(
            &format!("control/{name}/stderr"),
            &outcome.output.stderr,
        ));
    }
    // The same script under `--deny-warnings`: it stops at the first warning that no pragma
    // allows, and what it printed and said up to there is digested.
    let denied = zeff_engine::run::run_script_with(
        &fs,
        &project,
        "scripts/main.zeff",
        PRAGMA_SCRIPT,
        &zeff_engine::run::RunOptions {
            deny_warnings: true,
            ..zeff_engine::run::RunOptions::default()
        },
    );
    lines.push(text("control/pragma_denied/stdout", &denied.output.stdout));
    lines.push(text(
        "control/pragma_denied/stderr",
        &format!("{}exit {}\n", denied.output.stderr, denied.exit_code),
    ));
    lines
}

/// The script of the `map` case: the three list functions over literals and named functions.
const MAP_SCRIPT: &str = "let k = 8\nmap(seq(1, k - 1), fn(i) i / k)\nlet offset = 10\nmap([1, 2, 3], fn(i) i * i + offset)\nfilter([3, NA, 1, 2], fn(x) !is_missing(x) && x > 1)\nprint(reduce([1, 2, 3, 4], fn(acc, x) acc + x, 0))\nprint(reduce([1.5, 2.5, 4], fn(acc, x) acc * x, 1.0))\n/// Twice.\nfn twice(x: Int) -> Int {\n    x * 2\n}\n[1, 2, 3].map(twice)\n[for m in 1..2 { map([1, 2], fn(i) i * m) }]\nmap([\"b\", \"aa\"], len)\nlet w = dataset(wages)\nmap([1, 2], fn(i) i * w.n_rows())\nlet failed = attempt([1][2])\nprint(attempt(map([1, failed], fn(i) i)).is(Failed))\nfilter([1, NA], fn(x) x > 0)\n";

/// The script of the `enclosing` case: `n()`, `row_mean` and `row_sum` wherever a frame
/// method gives them a frame, and once where none does.
const ENCLOSING_SCRIPT: &str = "let d = frame(q1 = [5, 4, NA, 2], q2 = [3, NA, NA, 2], q3 = [4, 4, 2, NA], s = [1.5, 2.5, NA, 4.5], g = [\"a\", \"a\", \"b\", \"b\"])\nd.add(rows = n(), score = row_mean([q1, q2, q3]), total = row_sum([q1, q2, q3]))\nd.add(rows = n(), score = row_mean([q1, q2, q3], min_valid: 2), by: g)\nd.aggregate(rows = n(), by: g)\nd.update(s = row_sum([q1, q2, q3]) / n(), where: is_missing(s))\nd.where(row_sum([q1, q2, q3]) > n())\nlet w = dataset(wages)\nw.aggregate(rows = n(), mean_wage = mean(wage), by: female)\nw.add(share = wage / n()).summary([share])\n/// The row count, through a function, which sees no frame.\nfn rows() -> Int {\n    n()\n}\nd.add(k = rows())\n";

/// The script of the `notes` case: the notes and coded warnings of section 5.4 from a script
/// and from a function, one allowed by a pragma, and a variable's name written into a message.
const NOTES_SCRIPT: &str = "let w = dataset(wages)\nnote(\"the wage data of the shipped example\")\nnote(\"one pass over the rows\", code: W0703)\nwarn(\"fewer than a thousand workers per industry\", code: W0205)\nwarn(\"the same, as text\", code: \"W0205\")\n// zeff: allow(W0205) the industry counts are known to be small\nwarn(\"allowed, so not printed\", code: W0205)\nprint(name_of(educ))\n/// Says which variable it looked at.\nfn looked_at(f: Frame, v: Var) -> Int {\n    note(name_of(v))\n    warn(\"a coded warning from a function\", code: W0201)\n    f.n_rows()\n}\nw.looked_at(educ)\n";

/// The script of the `pragma` cases: the logarithm of experience, which is zero for the workers
/// in their first year (W0204), allowed in four places and left to warn in two.
const PRAGMA_SCRIPT: &str = "let w = dataset(wages)\n// zeff: allow(W0204) a worker in the first year has no experience\nlet a = w.add(l = log(exper))\na.summary([l])\nlet b = w.add(l = log(exper)) // zeff: allow(W0204) the same, at the end of the line\nb.n_rows()\nfor i in [1, 2] {\n    // zeff: allow(W0204) in each pass\n    print(w.add(l = log(exper) * i).n_rows())\n}\n/// The rows of a frame after its logarithm is taken.\nfn logged(f: Frame) -> Int {\n    // zeff: allow(W0204) inside a function\n    f.add(l = log(exper)).n_rows()\n}\nlogged(w)\nprint(\"allowed so far\")\nlet c = w.add(l = log(exper))\nc.n_rows()\nlet d = w.add(l = log(exper - 1))\n";

/// The `time` package (release 0.6.0, step 3): parsing with and without a format, an offset
/// applied, and the six date parts on dates, on a time and on a column, so that the four targets
/// prove the calendar arithmetic identical.
fn time_cases() -> Vec<Line> {
    use std::path::Path;
    use zeff_engine::fs::MemoryFileSystem;
    use zeff_engine::project::Project;
    use zeff_engine::run::run_script;
    let fs = MemoryFileSystem::new();
    fs.add_file(
        "/t/zeff.toml",
        format!(
            "[project]\nname = \"t\"\nzeff = \"{}\"\nedition = \"preview\"\n",
            zeff_engine::VERSION
        ),
    );
    // The clock of the case: 2026-11-01 09:30:00, as the examples' clock is.
    fs.set_clock(zeff_engine::examples::EXAMPLES_CLOCK);
    let project = match Project::open(&fs, Path::new("/t")) {
        Ok(project) => project,
        Err(_) => return vec![text("time/parts", "the project did not open")],
    };
    let script = "date(\"2026-10-16\")\ndate(\"16/10/2026\", format: \"%d/%m/%Y\")\ndate(\"Oct 16, 2026\", format: \"%b %d, %Y\")\ndate(\"2026-02-30\")\ndatetime(\"2026-10-16T09:30:00.25\")\ndatetime(\"2026-10-16T09:30:00+02:00\")\ndatetime(\"2026-10-16\")\nlet d = date(\"2021-01-03\")\n[year(d), month(d), day(d), weekday(d), quarter(d), week(d)]\nlet t = datetime(\"1999-12-31T23:59:59\")\n[year(t), month(t), day(t), weekday(t), quarter(t), week(t)]\nlet f = frame(s = [\"2026-01-01\", \"2024-12-30\", \"0001-01-01\", \"9999-12-31\", \"no\"])\nf.add(d = date(s)).add(y = year(d), m = month(d), dd = day(d), wd = weekday(d), q = quarter(d), w = week(d))\n";
    let outcome = run_script(&fs, &project, "scripts/main.zeff", script);
    let arithmetic = "let d = date(\"2026-01-31\")\nadd_days(d, 30)\nadd_days(d, -31)\nadd_months(d, 1)\nadd_months(date(\"2024-01-31\"), 1)\nadd_months(d, -11)\nadd_months(date(\"9999-12-31\"), 1)\ndays_between(d, date(\"2026-03-01\"))\ndays_between(datetime(\"2026-01-31T23:00\"), datetime(\"2026-02-02T22:59\"))\nmonths_between(d, date(\"2026-02-28\"))\nmonths_between(date(\"2026-03-31\"), d)\nadd_days(datetime(\"2026-01-31T09:30\"), 1)\nlet f = frame(d = [d, date(\"2024-02-29\"), date(\"0001-01-01\")])\nf.add(later = add_months(d, 12), gap = days_between(d, date(\"2026-11-01\")), back = add_days(d, -1))\ntoday()\nnow()\ndays_between(today(), date(\"2027-01-01\"))\n";
    let moved = run_script(&fs, &project, "scripts/main.zeff", arithmetic);
    // `format` of the `text` package (step 5): decimals, separators, the plain form of a large
    // number, dates by a pattern, on values and on columns.
    let formatted = "format(1234567.891, digits: 2)\nformat(1234567.891, digits: 2, big_mark: \",\")\nformat(-0.0001, digits: 2)\nformat(2.675, digits: 2)\nformat(1e16, digits: 1)\nformat(1 / 3)\nformat(123456789, big_mark: \" \")\nformat(42, digits: 3)\nformat(date(\"2026-10-16\"), pattern: \"%A %d %B %Y, day %j\")\nformat(datetime(\"2026-10-16T09:05:07.5\"), pattern: \"%y%m%d %H:%M:%S\")\nformat(date(\"0001-01-01\"))\nlet f = frame(x = [1250.5, 980, -3.14159], d = [date(\"2026-01-05\"), date(\"2024-02-29\"), date(\"9999-12-31\")])\nf.add(t = format(x, digits: 2, big_mark: \",\"), m = format(d, pattern: \"%b %Y\"), p = format(x))\n";
    let printed = run_script(&fs, &project, "scripts/main.zeff", formatted);
    vec![
        text("time/parts/stdout", &outcome.output.stdout),
        text("time/parts/stderr", &outcome.output.stderr),
        text("time/arithmetic/stdout", &moved.output.stdout),
        text("time/arithmetic/stderr", &moved.output.stderr),
        text("text/format/stdout", &printed.output.stdout),
        text("text/format/stderr", &printed.output.stderr),
    ]
}

/// The panel declaration (release 0.6.0, step 7): declared on a literal frame and on the shipped
/// `wages`, the header, its survival and dropping through the verbs, and its refusals, so that
/// the four targets prove the index and the messages identical.
fn panel_cases() -> Vec<Line> {
    use std::path::Path;
    use zeff_engine::fs::MemoryFileSystem;
    use zeff_engine::project::Project;
    use zeff_engine::run::run_script;
    let fs = MemoryFileSystem::new();
    fs.add_file(
        "/n/zeff.toml",
        format!(
            "[project]\nname = \"n\"\nzeff = \"{}\"\nedition = \"preview\"\n",
            zeff_engine::VERSION
        ),
    );
    let project = match Project::open(&fs, Path::new("/n")) {
        Ok(project) => project,
        Err(_) => return vec![text("panel/declare", "the project did not open")],
    };
    let script = "let sales = frame(firm = [2, 1, 2, 1, 3], year = [2021, 2020, 2020, 2021, 2020], revenue = [12.5, 8, 11, 9, NA])\nlet p = sales.panel(unit: firm, time: year)\np\np.where(year > 2020).rename(company = firm)\np.sort(revenue).head(3)\np.drop(year)\np.add(firm = 1)\nlet w = dataset(wages).where(year >= 2019).panel(unit: worker, time: year)\nw.describe()\nw.select([worker, wage]).n_rows()\nlet d = frame(firm = [\"a\", \"a\", \"b\"], day = [to(\"2026-01-02\", Date), to(\"2026-01-01\", Date), to(\"2026-01-01\", Date)]).panel(unit: firm, time: day)\nd.sort(day)\nframe(firm = [1, 1], year = [2020, 2020]).panel(unit: firm, time: year)\n";
    let outcome = run_script(&fs, &project, "scripts/main.zeff", script);
    // The panel functions (step 8): lags, leads and differences across a gap, a Date panel, a
    // fill forward within units and by row order, and the refusals.
    let lags = "let p = frame(firm = [2, 1, 2, 1, 3, 2], year = [2021, 2020, 2020, 2021, 2020, 2023], revenue = [12.5, 8, 11, NA, 7, 15]).panel(unit: firm, time: year)\np.add(prev = lag(revenue), next = lead(revenue), two = lag(revenue, n: 2), change = diff(revenue), filled = fill_forward(revenue))\nlet d = frame(firm = [\"a\", \"a\", \"a\", \"b\"], day = [date(\"2026-01-01\"), date(\"2026-01-02\"), date(\"2026-01-04\"), date(\"2026-01-01\")], v = [1, 2, 3, 4]).panel(unit: firm, time: day)\nd.add(prev = lag(v), change = diff(v))\nframe(g = [1, 1, 2, 2], v = [NA, 3, NA, NA]).add(f = fill_forward(v))\nframe(g = [1, 1, 2, 2], v = [5, NA, 7, NA]).add(f = fill_forward(v), by: g)\nlet w = dataset(wages).where(worker <= 3)\nw.add(prev = lag(wage), growth = diff(wage)).select([worker, year, wage, prev, growth])\nframe(x = [1, 2]).add(y = lag(x))\n";
    let moved = run_script(&fs, &project, "scripts/main.zeff", lags);
    // The column functions (step 9): running sums and products with an `NA` and within groups,
    // the four tie rules, and bins with open ends, both closures, labels and a value outside.
    let running = "let s = frame(shop = [\"a\", \"a\", \"b\", \"b\", \"b\"], sales = [120, 80, NA, 150, 90], growth = [1.1, 0.9, 1.25, 1.0, 0.8])\ns.add(to_date = running_sum(sales), index = running_product(growth))\ns.add(so_far = running_sum(sales), by: shop)\nlet r = frame(name = [\"Ann\", \"Bo\", \"Cy\", \"Di\", \"Ed\"], score = [72.5, 90, 72.5, NA, 60])\nr.add(avg = rank(score), low = rank(score, ties: min), high = rank(score, ties: max), first = rank(score, ties: first))\nlet pay = frame(wage = [18.08, 8.5, 20.93, 15.68, 26.39, 33.07, NA])\npay.add(band = bin(wage, breaks: [-Inf, 10, 20, Inf]), left = bin(wage, breaks: [0, 10, 20], labels: [\"low\", \"high\"], closed: left), narrow = bin(wage, breaks: [10, 20]), fine = bin(wage, breaks: [0, 2.5, 1000, 123456, Inf]))\nframe(x = [9223372036854775800, 10]).add(s = running_sum(x))\n";
    let ran = run_script(&fs, &project, "scripts/main.zeff", running);
    // `quantile_group` and `winsorize` (step 10): quartiles and tertiles under two definitions,
    // ties at the cut points, groups within `by:`, and the capped values at two shares.
    let grouped = "let d = frame(g = [1, 1, 1, 1, 2, 2, 2, 2], x = [5, 1, 9, 3, 7, NA, 2, 8])\nd.add(quartile = quantile_group(x, n: 4), third = quantile_group(x, n: 3, type: 1), within = quantile_group(x, n: 2), by: g)\nframe(x = [1, 1, 1, 1, 2]).add(q = quantile_group(x, n: 4))\nlet w = dataset(wages).head(12)\nw.add(decile = quantile_group(wage, n: 10), capped = winsorize(wage, at: 0.1), one = winsorize(wage)).select([worker, year, wage, decile, capped, one])\nframe(x = [1.0, 2]).add(w = winsorize(x, at: 0.6))\n";
    let grouped_outcome = run_script(&fs, &project, "scripts/main.zeff", grouped);
    // `distinct` and `duplicates` (step 11): keys with repeats and `NA`, several keys, and the
    // table; `sample` under explicit seeds, without and with replacement, by count and by share.
    let distinct = "let sales = frame(firm = [2, 1, 2, 1, 3, NA, NA], year = [2021, 2020, 2020, 2020, 2021, 2020, 2020], revenue = [12.5, 8, 11, 9, 7, 1, 1])\nsales.distinct(firm)\nsales.distinct([firm, year])\nsales.distinct([firm, year, revenue])\nsales.duplicates(firm)\nsales.duplicates([firm, year])\nsales.duplicates(revenue)\nlet w = dataset(wages).where(worker <= 40)\nw.distinct(worker).select([worker, year, industry])\nw.duplicates(industry)\n";
    let distinct_outcome = run_script(&fs, &project, "scripts/main.zeff", distinct);
    let sampled = "let w = dataset(wages)\nw.sample(n: 8, seed: 20261015).select([worker, year, wage])\nw.sample(frac: 0.0005, seed: 7).select([worker, year, wage])\nw.sample(n: 6, with_replacement: true, seed: 3).select([worker, year, wage])\nw.head(5).sample(n: 7, with_replacement: true, seed: 11).select([worker, year])\nw.sample(n: 0, seed: 1).n_rows()\n";
    let sampled_outcome = run_script(&fs, &project, "scripts/main.zeff", sampled);
    // `aggregate` (step 12): reducers and arithmetic over them by one and two keys, `NA` as a
    // group, a definition using an earlier one, no `by:`, and a panel's note.
    let aggregated = "let sales = frame(firm = [2, 1, 2, 1, 3, 1, NA], year = [2021, 2020, 2020, 2020, 2021, 2021, 2021], revenue = [12.5, 8, 11, 9, NA, 10, 4])\nsales.aggregate(total = sum(revenue), firms = n_distinct(firm), rows = n(), by: year)\nsales.aggregate(avg = mean(revenue), spread = sd(revenue), se = spread / sqrt(n()), by: firm)\nsales.aggregate(avg = mean(revenue), rows = n(), by: [firm, year])\nsales.aggregate(total = sum(revenue), rows = n())\nlet w = dataset(wages).where(worker <= 30)\nw.aggregate(mean_wage = mean(wage), median_hours = median(hours), top = quantile(wage, p: 0.9), workers = n_distinct(worker), by: industry)\nw.aggregate(mean_wage = mean(wage), rows = n(), by: year)\n";
    let aggregated_outcome = run_script(&fs, &project, "scripts/main.zeff", aggregated);
    // `merge` (step 13): the three joins with unmatched rows on each side and `NA` keys, two
    // keys, suffixes, the shipped wages with firms, and the refusals.
    let merged = "let sales = frame(firm = [2, 1, 3, 1, NA], year = [2021, 2020, 2021, 2021, 2020], revenue = [12.5, 8, 7, 9, 1])\nlet names = frame(firm = [1, 2, 4, NA], name = [\"Acme\", \"Bolt\", \"Dyne\", \"None\"], revenue = [0, 0, 0, 0])\nsales.merge(names.drop(revenue), on: firm)\nsales.merge(names.drop(revenue), on: firm, join: inner)\nsales.merge(names.drop(revenue), on: firm, join: full)\nsales.merge(names, on: firm, suffixes: [\"_s\", \"_n\"])\nlet by_year = frame(firm = [1, 1, 2], year = [2020, 2021, 2021], target = [10, 11, 12])\nsales.merge(by_year, on: [firm, year], relation: many_to_one)\nlet w = dataset(wages).where(worker <= 20).merge(dataset(firms), on: [firm, industry], relation: many_to_one)\nw.select([worker, year, firm, size, founded]).head(5)\nsales.merge(names, on: firm)\nsales.merge(by_year, on: firm)\nsales.merge(names.drop(revenue), on: firm, relation: one_to_one)\n";
    let merged_outcome = run_script(&fs, &project, "scripts/main.zeff", merged);
    // `append`, `to_long` and `to_wide` (step 14): a stack with a column on each side only and
    // Int meeting Float, the long form of a wide frame by Int names, the wide form of a long one
    // with an absent combination, the round trip, and the refusals.
    let reshaped = "let this_year = frame(firm = [1, 2], revenue = [12.5, 8], region = [\"North\", \"South\"])\nlet last_year = frame(firm = [1, 3], revenue = [11, 7], founded = [date(\"1990-01-05\"), date(\"2001-11-30\")])\nthis_year.append(last_year)\nlet sales = frame(firm = [1, 2], region = [\"North\", \"South\"], sales_2010 = [120, 80], sales_2011 = [130, NA])\nlet long = sales.to_long(\"sales_\", id: firm, names_to: \"year\", values_to: \"sales\", names_type: Int)\nlong\nlong.to_wide(values_from: sales, id: [firm, region], names_from: year)\nframe(unit = [3, 1, 3, 1, 2], period = [2, 1, 1, 2, 1], y = [0.5, 1.25, -2, 3.5, NA]).to_wide(values_from: y, id: unit, names_from: period)\nsales.to_long(\"sales_\", id: firm, names_to: \"year\", values_to: \"sales\", names_type: Date)\nframe(id = [1, 2], year = [2010, 2011]).append(frame(id = [3], year = [\"2012\"]))\nframe(firm = [1, 1], year = [2010, 2010], sales = [1, 2]).to_wide(values_from: sales, id: firm, names_from: year)\n";
    let reshaped_outcome = run_script(&fs, &project, "scripts/main.zeff", reshaped);
    // `save` (step 15): the sample table of every type as CSV and TSV, with a byte-order mark
    // and with codes, and the refusals; the bytes of the files are part of the report.
    let saved = "let s = frame(id = [1, 2, 3], x = [1.5, -0.0, NA], flag = [true, false, NA], note = [\"plain\", \"with, comma\", \"two\\nlines\"], day = [date(\"2024-01-01\"), date(\"1970-01-01\"), NA], level = [\"low\", \"high\", \"mid\"]).add(level = category(level, levels: [\"low\", \"mid\", \"high\"]))\ns.save(\"sample.csv\")\ns.save(\"output/sample.tsv\")\ns.save(\"output/sample_bom.csv\", bom: true, codes: true)\ns.save(\"output/sample.parquet\")\ns.save(\"output/sample\")\n";
    let saved_outcome = run_script(&fs, &project, "scripts/main.zeff", saved);
    let saved_file = |name: &str| {
        zeff_engine::fs::FileSystem::read(&fs, Path::new(&format!("/n/output/{name}")))
            .unwrap_or_else(|_| b"(not written)".to_vec())
    };
    vec![
        text("panel/declare/stdout", &outcome.output.stdout),
        text("panel/declare/stderr", &outcome.output.stderr),
        text("panel/lags/stdout", &moved.output.stdout),
        text("panel/lags/stderr", &moved.output.stderr),
        text("panel/running/stdout", &ran.output.stdout),
        text("panel/running/stderr", &ran.output.stderr),
        text(
            "functions/quantile_group/stdout",
            &grouped_outcome.output.stdout,
        ),
        text(
            "functions/quantile_group/stderr",
            &grouped_outcome.output.stderr,
        ),
        text("verbs/distinct/stdout", &distinct_outcome.output.stdout),
        text("verbs/distinct/stderr", &distinct_outcome.output.stderr),
        text("verbs/sample/stdout", &sampled_outcome.output.stdout),
        text("verbs/sample/stderr", &sampled_outcome.output.stderr),
        text("verbs/aggregate/stdout", &aggregated_outcome.output.stdout),
        text("verbs/aggregate/stderr", &aggregated_outcome.output.stderr),
        text("verbs/merge/stdout", &merged_outcome.output.stdout),
        text("verbs/merge/stderr", &merged_outcome.output.stderr),
        text("verbs/reshape/stdout", &reshaped_outcome.output.stdout),
        text("verbs/reshape/stderr", &reshaped_outcome.output.stderr),
        text("verbs/save/stdout", &saved_outcome.output.stdout),
        text("verbs/save/stderr", &saved_outcome.output.stderr),
        bytes("verbs/save/sample.csv", &saved_file("sample.csv")),
        bytes("verbs/save/sample.tsv", &saved_file("sample.tsv")),
        bytes("verbs/save/sample_bom.csv", &saved_file("sample_bom.csv")),
    ]
}

/// A built-in written in Zeff (release 0.5.0, step 9): the engine's fixture `builtin_fixture`,
/// registered beside the shipped functions, run as a call, a method, with its default and on a
/// column, so that the four targets prove the loader and the lifted body identical.
fn builtin_fixture_cases() -> Vec<Line> {
    use std::path::Path;
    use zeff_engine::fixture::run_fixture_script;
    use zeff_engine::fs::MemoryFileSystem;
    use zeff_engine::project::Project;
    let fs = MemoryFileSystem::new();
    fs.add_file(
        "/b/zeff.toml",
        format!(
            "[project]\nname = \"b\"\nzeff = \"{}\"\nedition = \"preview\"\n",
            zeff_engine::VERSION
        ),
    );
    let project = match Project::open(&fs, Path::new("/b")) {
        Ok(project) => project,
        Err(_) => {
            return vec![text(
                "functions/builtin_fixture",
                "the project did not open",
            )];
        }
    };
    let script = "print(builtin_fixture(-3))\nprint(builtin_fixture(1.5, scale: 3))\nprint((-2.5).builtin_fixture())\nlet w = dataset(wages).head(5)\nw.add(y = builtin_fixture(wage - 20), z = builtin_fixture(wage, scale: hours))[\"y\"]\nw.add(y = builtin_fixture(wage - 20), z = builtin_fixture(wage, scale: hours))[\"z\"]\n";
    let outcome = run_fixture_script(&fs, &project, "scripts/main.zeff", script);
    vec![
        text("functions/builtin_fixture/stdout", &outcome.output.stdout),
        text("functions/builtin_fixture/stderr", &outcome.output.stderr),
    ]
}

/// The exported files of release 0.4.0 (step 13): the sample table as Word and LaTeX, so that the
/// four targets prove the bytes identical.
fn export_cases() -> Vec<Line> {
    let sample = zeff_results::export::sample();
    vec![
        bytes("export/sample.docx", &zeff_results::export::docx(&sample)),
        text("export/sample.tex", &zeff_results::export::tex(&sample)),
    ]
}

fn all_cases() -> Vec<Line> {
    let mut lines = reducer_cases();
    lines.extend(canary_cases());
    lines.extend(maths_cases());
    lines.extend(special_cases());
    lines.extend(random_cases());
    lines.extend(distribution_cases());
    lines.extend(regress_cases());
    lines.extend(control_cases());
    lines.extend(time_cases());
    lines.extend(panel_cases());
    lines.extend(builtin_fixture_cases());
    lines.extend(export_cases());
    lines.extend(kernels_cases());
    lines.extend(milestone_cases());
    lines.extend(sample_cases());
    lines
}

/// The published report of this release: `expected.txt`, compiled in, so that `zeff selftest` can
/// compare any installation with it (Engine and Project Specification, sections 2.1 and 8.3,
/// point 3, "Proof").
pub const PUBLISHED: &str = include_str!("../expected.txt");

/// One case of the self-test: its digest in the published report and in this build.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelftestCase {
    pub name: String,
    /// `None` when the published report has no such case (a build newer than its report).
    pub published: Option<u64>,
    pub actual: u64,
}

impl SelftestCase {
    pub fn matches(&self) -> bool {
        self.published == Some(self.actual)
    }
}

/// `zeff selftest`: every case of this build against the published report. The environment
/// variable `ZEFF_SELFTEST_FORCE_MISMATCH`, naming a case, flips that case's digest: a test hook,
/// so that the path that names a differing case is exercised.
pub fn selftest() -> Vec<SelftestCase> {
    let published: std::collections::HashMap<&str, u64> = PUBLISHED
        .lines()
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .filter_map(|l| {
            let (name, digest) = l.split_once(' ')?;
            Some((name, u64::from_str_radix(digest, 16).ok()?))
        })
        .collect();
    let forced = std::env::var("ZEFF_SELFTEST_FORCE_MISMATCH").ok();
    all_cases()
        .into_iter()
        .map(|line| {
            let actual = if forced.as_deref() == Some(line.name.as_str()) {
                line.digest ^ 1
            } else {
                line.digest
            };
            SelftestCase {
                published: published.get(line.name.as_str()).copied(),
                name: line.name,
                actual,
            }
        })
        .collect()
}

/// The report of this build: a `name digest` line per case in a fixed order, each single-result
/// case followed by a comment line holding the result's bits.
/// The engine's start, as a user first meets it: the registry built, a one-line script parsed,
/// checked and run on an in-memory project. The cold-start measurement (`tools/bundle/measure.cjs`,
/// risk R4) times this call, not the whole report, whose length grows with every case.
pub fn start() -> String {
    use std::path::Path;
    use zeff_engine::fs::MemoryFileSystem;
    use zeff_engine::project::Project;
    use zeff_engine::run::run_script;
    let fs = MemoryFileSystem::new();
    fs.add_file(
        "/s/zeff.toml",
        format!(
            "[project]\nname = \"s\"\nzeff = \"{}\"\nedition = \"preview\"\n",
            zeff_engine::VERSION
        ),
    );
    let Ok(project) = Project::open(&fs, Path::new("/s")) else {
        return "the project did not open".to_string();
    };
    let outcome = run_script(&fs, &project, "scripts/start.zeff", "print(1 + 1)\n");
    format!("{}{}", outcome.output.stdout, outcome.output.stderr)
}

pub fn report() -> String {
    let mut text = String::new();
    for line in all_cases() {
        text.push_str(&format!("{} {:016x}\n", line.name, line.digest));
        if let Some(detail) = line.detail {
            text.push_str(&format!("# {} bits {detail:016x}\n", line.name));
        }
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cases_only(text: &str) -> Vec<&str> {
        text.lines()
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .collect()
    }

    #[test]
    fn this_build_reproduces_the_committed_report() {
        let report = report();
        let expected = include_str!("../expected.txt");
        assert_eq!(
            cases_only(&report),
            cases_only(expected),
            "a result changed: if that is intended, regenerate expected.txt with \
             `cargo run -p zeff-conformance > crates/zeff-conformance/expected.txt`"
        );
    }

    #[test]
    fn the_report_is_the_same_twice() {
        assert_eq!(report(), report());
    }

    #[test]
    fn the_start_prints_two() {
        assert_eq!(start(), "2\n");
    }

    /// The milestone script ran into a check error from 18 October 2026 (a line `-a`, which the
    /// grammar refuses) to 23 October 2026, its digests being those of the failure on every
    /// target alike; this test keeps the case honest.
    #[test]
    fn the_milestone_script_runs_to_the_end() {
        let outcome = milestone_run().expect("the milestone project opens");
        assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
        assert!(!outcome.output.stdout.is_empty());
        assert!(
            !outcome.output.stderr.contains("error["),
            "{}",
            outcome.output.stderr
        );
    }

    #[test]
    fn every_sample_project_prints_its_stored_output_byte_for_byte() {
        for sample in SAMPLES {
            let outcome = sample_run(sample);
            assert_eq!(
                outcome.exit_code, 0,
                "{}: {}",
                sample.name, outcome.output.stderr
            );
            assert_eq!(
                outcome.output.stdout, sample.stdout,
                "the output of samples/{} changed; regenerate expected/ with `zeff run` there when the change is intended",
                sample.name
            );
            assert_eq!(outcome.output.stderr, sample.stderr, "{}", sample.name);
        }
    }

    #[test]
    fn every_case_has_its_own_name() {
        let report = report();
        let names: Vec<&str> = cases_only(&report)
            .iter()
            .map(|l| l.split(' ').next().unwrap())
            .collect();
        let mut sorted = names.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), names.len());
    }

    #[test]
    fn a_not_a_number_has_one_pattern() {
        assert_eq!(bits(f64::NAN), bits(-f64::NAN));
        assert_eq!(bits(f64::NAN), bits(f64::from_bits(0x7ff8_0000_0000_0001)));
    }
}
