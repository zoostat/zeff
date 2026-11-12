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

/// The sample of the plots of release 0.7.0 (step 21d): six figures of the shipped wages, each
/// exported as SVG, whose digests prove that every target writes the same bytes.
pub const PLOTS: Sample = Sample {
    name: "plots",
    manifest: include_str!("../../../samples/plots/zeff.toml"),
    data: &[],
    modules: &[],
    script: include_str!("../../../samples/plots/scripts/main.zeff"),
    stdout: include_str!("../../../samples/plots/expected/stdout.txt"),
    stderr: include_str!("../../../samples/plots/expected/stderr.txt"),
};

/// Every sample project, oldest first; each adds three cases to the report, and one more for
/// every file it writes under `output/`.
pub const SAMPLES: [&Sample; 7] = [
    &FIRST_ENGINE,
    &DATA_CLEANING,
    &SIMULATION,
    &REGRESSION,
    &USER_FUNCTIONS,
    &PANEL,
    &PLOTS,
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
    // The studentized range (release 0.7.0, step 18) in a series of its own, on a short grid:
    // every value is a double integral.
    let mut tukey = Vec::new();
    for (groups, df) in [(3.0, 20.0), (10.0, 5.0), (40.0, 1e6)] {
        let d = Distribution::Family {
            name: "tukey",
            params: vec![("groups", groups), ("df", df)],
        };
        for x in [0.5, 2.0, 3.5, 6.0, 30.0] {
            let tails = families::cdf(&d, x);
            tukey.extend([families::pdf(&d, x), tails.lower, tails.upper]);
        }
        tukey.extend([families::quantile(&d, 0.05), families::quantile(&d, 0.95)]);
        let mut stream = Stream::new(20_261_019, 3, 0);
        tukey.push(families::draw(&d, &mut stream));
    }
    vec![
        series("dist/values", &values),
        series("dist/draws", &draws),
        series("dist/tukey", &tukey),
    ]
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
        // A parameter of several types (release 0.7.0, step 3.2, shipped at step 4.1 with
        // `fill_missing` in Zeff): each scalar type, `NA` typed and bare, Int with Float both
        // ways, the column forms, and the join inside `add`.
        ("several_types", SEVERAL_TYPES_SCRIPT),
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

/// The script of the `several_types` case: `fill_missing`, written in Zeff on a parameter of
/// several types, under every type and column form a call may take.
const SEVERAL_TYPES_SCRIPT: &str = "fill_missing(NA, 3)\nfill_missing(to(NA, Int), 0)\nfill_missing(true, false)\nfill_missing(to(NA, Bool), true)\nfill_missing(\"a\", \"b\")\nfill_missing(to(NA, Str), \"none\")\nfill_missing(1, 2.5)\nfill_missing(2.5, 1)\nfill_missing(to(NA, Float), 1)\nfill_missing(to(NA, Int), 1.5)\nfill_missing(to(\"2026-01-01\", Date), to(\"2000-01-01\", Date))\nfill_missing(to(NA, Date), to(\"2000-01-01\", Date))\nlet d = frame(i = [1, NA, 3], f = [1.5, NA, 2], s = [\"a\", NA, \"c\"], b = [true, NA, false])\nfill_missing(d[\"i\"], 0)\nfill_missing(d[\"f\"], 0)\nfill_missing(d[\"i\"], 0.5)\nfill_missing(d[\"s\"], \"none\")\nfill_missing(d[\"b\"], false)\nfill_missing(NA, d[\"i\"])\nfill_missing(d[\"f\"], d[\"i\"])\nd.add(i2 = fill_missing(i, 0), f2 = fill_missing(f, i), s2 = fill_missing(s, \"-\"))\nlet w = dataset(wages)\nw.add(wage2 = fill_missing(wage, 0)).summary([wage2])\n";

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
    // The text functions of release 0.7.0, step 5: case by simple mapping, trimming of Unicode
    // white space, prefix and suffix tests, slices and padding by scalar values, and joining,
    // on values and on a frame's text column.
    let basics = "upper(\"caf\u{e9} Stra\u{df}e \u{fb01}sh\")\nlower(\"\u{c9}COLE \u{3a3}\u{3af}\u{3c3}\u{3c5}\u{3c6}\u{3bf}\u{3c2}\")\nlen(upper(\"e\u{301}\"))\ntrim(\"\u{a0} padded\t\u{2003}\")\nstarts_with(\"Hello, World\", \"Hello\")\nends_with(\"caf\u{e9}\", \"\u{e9}\")\nslice(\"Hello, World\", start: 8, len: 50)\nslice(\"\u{65e5}\u{672c}\u{8a9e}\", start: 2, len: 2)\npad(\"7\", width: 3, fill: \"0\")\npad(\"abcdef\", width: 3)\njoin([\"a\", NA, \"c\"], sep: \"/\")\njoin(map([1.5, 2], fn(x) format(x, digits: 1)), sep: \" and \")\nlet d = frame(name = [\"  Ada \", \"grace\", NA], code = [\"ab-1\", \"cd-22\", \"ef-3\"])\nd.add(clean = upper(trim(name)), prefix = slice(code, start: 1, len: 2), two = ends_with(code, \"2\"), padded = pad(code, width: 6, fill: \".\"))\nd.where(starts_with(code, \"c\"))\nslice(\"abc\", start: 0, len: 1)\n";
    let texts = run_script(&fs, &project, "scripts/main.zeff", basics);
    // The pattern functions (step 6): literal and regular-expression matching, case folding,
    // replacing, extracting, splitting, on values and on a frame's text column, and a pattern
    // that cannot be read.
    let patterns = "contains(\"Hello, World\", \"World\")\ncontains(\"Hello, World\", \"WORLD\", ignore_case: true)\ncontains(\"abc123\", \"[0-9]+\", regex: true)\ncontains(\"Stra\u{df}e\", \"STRASSE\", ignore_case: true)\nreplace(\"a.b.c\", \".\", \"-\")\nreplace(\"a.b.c\", \".\", \"-\", regex: true)\nreplace(\"  two  words \", \"\\\\s+\", \" \", regex: true)\nreplace(\"abc\", \"\", \"-\", regex: true)\nextract(\"abc123def45\", \"[0-9]+\")\nextract(\"Hello\", \"xyz\")\nsplit(\"a,b,\", \",\")\nsplit(\"abc\", \"\")\nlet d = frame(code = [\"ab-1\", \"CD-22\", NA, \"ef-333\"])\nd.add(digits = extract(code, \"[0-9]+\"), letters = replace(code, \"[^a-z]\", \"\", regex: true), has_two = contains(code, \"2\"), cd = contains(code, \"cd\", ignore_case: true))\nd.where(contains(code, \"^[a-z]+-\\\\d$\", regex: true))\ncontains(\"abc\", \"(ab\", regex: true)\n";
    let patterned = run_script(&fs, &project, "scripts/main.zeff", patterns);
    vec![
        text("time/parts/stdout", &outcome.output.stdout),
        text("time/parts/stderr", &outcome.output.stderr),
        text("time/arithmetic/stdout", &moved.output.stdout),
        text("time/arithmetic/stderr", &moved.output.stderr),
        text("text/format/stdout", &printed.output.stdout),
        text("text/format/stderr", &printed.output.stderr),
        text("text/basics/stdout", &texts.output.stdout),
        text("text/basics/stderr", &texts.output.stderr),
        text("text/patterns/stdout", &patterned.output.stdout),
        text("text/patterns/stderr", &patterned.output.stderr),
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
    // `tabulate` (release 0.7.0, step 8): one- and two-way with the three percentages, `NA` as
    // its own row and column, labelled values, a Date variable and an empty frame.
    let tabulated = "let wages = dataset(wages)\nwages.tabulate(industry)\nwages.tabulate([female, union], percent: row)\nlet small = frame(sex = [\"f\", \"m\", NA, \"f\", \"m\", \"f\"], smoker = [true, false, true, NA, false, false], day = [to(\"2026-01-02\", Date), to(\"2026-01-01\", Date), to(\"2026-01-02\", Date), NA, to(\"2026-01-01\", Date), to(\"2026-01-02\", Date)])\nsmall.tabulate(sex)\nsmall.tabulate([sex, smoker])\nsmall.tabulate([sex, smoker], percent: col)\nsmall.tabulate([day, sex], percent: total)\nsmall.where(sex == \"z\").tabulate(sex)\n";
    let tabulated_outcome = run_script(&fs, &project, "scripts/main.zeff", tabulated);
    // `t_test` (release 0.7.0, step 9): the three forms with their hypothesis blocks, one-sided
    // alternatives and intervals, another level, the pooled test with Cohen's d, frequency
    // weights, text groups, and the note on rows dropped.
    let tested = "let wages = dataset(wages).add(lwage = log(wage))\nwages.t_test(lwage, group: female)\nwages.t_test(lwage, mu: 3, alternative: greater)\nwages.t_test(lwage, group: union, equal_var: true, effect: d, level: 0.99)\nlet pairs = frame(before = [0.7, -1.6, -0.2, -1.2, -0.1, 3.4, 3.7, 0.8, 0.0, 2.0], after = [1.9, 0.8, 1.1, 0.1, -0.1, 4.4, 5.5, 1.6, 4.6, 3.4])\npairs.t_test(before, after, paired: true)\npairs.t_test(before, after, paired: true, alternative: less, level: 0.9)\nlet w = frame(x = [1.0, 2.0, 4.0, NA], f = [2, 1, 3, 1], s = [\"a\", \"a\", \"b\", \"b\"])\nw.t_test(x, mu: 1, weights: f, weight_type: frequency)\nw.t_test(x, group: s)\n";
    let tested_outcome = run_script(&fs, &project, "scripts/main.zeff", tested);
    // `proportion_test` and `chi2_test` (release 0.7.0, step 10): one and two proportions with
    // the three intervals, the correction and one-sided alternatives; independence on a wide
    // table and a corrected 2 × 2; goodness of fit; an empty cell's warning; frequency weights.
    let counted = "let wages = dataset(wages)\nwages.proportion_test(union, p: 0.3)\nwages.proportion_test(union, p: 0.3, correct: true, ci: exact, alternative: greater)\nwages.proportion_test(union, group: female)\nwages.proportion_test(union, group: female, ci: wald, correct: true, level: 0.99)\nwages.chi2_test(industry, female)\nwages.chi2_test(union, female, correct: true)\nwages.chi2_test(female, p: [0.5, 0.5])\nlet small = frame(a = [\"x\", \"x\", \"y\", \"y\", \"z\", NA], b = [1, 2, 1, 2, 1, 2], w = [1, 1, 2, 1, 1, 1])\nsmall.chi2_test(a, b)\nsmall.chi2_test(a, b, weights: w, weight_type: frequency)\nlet coin = frame(heads = [true, true, false, true, true, true, false, true, true, false])\ncoin.proportion_test(heads, p: 0.5, ci: wald, alternative: less)\n";
    let counted_outcome = run_script(&fs, &project, "scripts/main.zeff", counted);
    // `wilcoxon_test` (release 0.7.0, step 11): the exact signed rank without and with ties and
    // zeros, the approximation forced, one-sided alternatives and other levels, the paired form,
    // the exact and approximate rank sum, and the wages' rank sum with ties.
    let ranked = "let d = frame(x = [1.83, 0.50, 1.62, 2.48, 1.68, 1.88, 1.55, 3.06, 1.30], y = [0.878, 0.647, 0.598, 2.05, 1.06, 1.29, 1.06, 3.14, 1.29])\nd.wilcoxon_test(x, mu: 1)\nd.wilcoxon_test(x, mu: 1, exact: false, level: 0.99)\nd.wilcoxon_test(x, y, paired: true, alternative: greater)\nlet t = frame(v = [3, 1, 4, 1, 5, 9, 2, 6, 5, 3, 5, 8, 9, 7, 9])\nt.wilcoxon_test(v, mu: 5)\nt.wilcoxon_test(v, mu: 5, exact: false, alternative: less)\nlet g = frame(s = [0.80, 0.83, 1.89, 1.04, 1.45, 1.38, 1.91, 1.64, 0.73, 1.46, 1.15, 0.88, 0.90, 0.74, 1.21], grp = [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2])\ng.wilcoxon_test(s, group: grp)\ng.wilcoxon_test(s, group: grp, alternative: greater, level: 0.9, exact: false)\nlet w = dataset(wages).where(worker <= 400).add(lwage = log(wage))\nw.wilcoxon_test(lwage, group: female)\n";
    let ranked_outcome = run_script(&fs, &project, "scripts/main.zeff", ranked);
    // `kruskal_test` and `p_adjust` (release 0.7.0, steps 12, 18b and 18c): three and two groups
    // without ties, heavy ties, a Category grouping variable with the note on rows dropped,
    // frequency weights, the eight adjustments with an `NA` kept in place, and the adjustment
    // as a column inside `add`, over the whole frame and within the groups of `by:`.
    let compared = "let g = frame(s = [2.9, 3.0, 2.5, 2.6, 3.2, 3.8, 2.7, 4.0, 2.4, 2.8, 3.4, 3.7, 2.2, 2.0], site = [1, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 3])\ng.kruskal_test(s, group: site)\ng.where(site < 3).kruskal_test(s, group: site)\nlet t = frame(v = [3, 1, 4, 1, 5, 9, 2, 6, 5, 3, 5, 8, 9, 7, 9], k = [\"a\", \"b\", \"c\", \"a\", \"b\", \"c\", \"a\", \"b\", \"c\", \"a\", \"b\", \"c\", \"a\", \"b\", \"c\"], w = [1, 2, 1, 1, 1, 3, 1, 1, 1, 2, 1, 1, 1, 1, 1])\nt.kruskal_test(v, group: k)\nt.kruskal_test(v, group: k, weights: w, weight_type: frequency)\nlet wages = dataset(wages).add(lwage = log(wage))\nwages.kruskal_test(lwage, group: industry)\np_adjust([0.01, 0.04, 0.03, 0.005, 0.2])\np_adjust([0.01, 0.04, 0.03, 0.005, 0.2], method: bonferroni)\np_adjust([0.01, NA, 0.03, 0.005, 0.2], method: bh)\np_adjust([0.01, NA, 0.03, 0.005, 0.2], method: by)\np_adjust([0.01, 0.04, 0.03, 0.005, 0.2], method: hochberg)\np_adjust([0.01, 0.04, 0.03, 0.005, 0.2], method: hommel)\np_adjust([0.01, 0.04, 0.03, 0.005, 0.2], method: sidak)\np_adjust([0.011, 0.012, 0.013, 0.014, 0.6, 0.6], method: hommel)\np_adjust([0.01, NA, 0.03, 0.005, 0.2], method: none)\nlet tests = frame(outcome = [\"wage\", \"wage\", \"wage\", \"hours\", \"hours\", \"hours\", \"hours\"], arm = [1, 2, 3, 1, 2, 3, 4], p = [0.01, 0.04, 0.03, 0.2, NA, 0.001, 0.02])\ntests.add(holm = p_adjust(p), bh = p_adjust(p, method: bh))\ntests.add(holm = p_adjust(p), hommel = p_adjust(p, method: hommel), by: outcome)\ntests.where(arm < 4).add(sidak = p_adjust(p, method: sidak), fdr = p_adjust(p, method: by), by: outcome)\np_adjust(tests[\"p\"], method: hochberg)\n";
    let compared_outcome = run_script(&fs, &project, "scripts/main.zeff", compared);
    let exact = "let wages = dataset(wages)\nwages.fisher_test(union, female)\nwages.fisher_test(union, female, alternative: greater)\nlet t = frame(a = [\"x\", \"x\", \"x\", \"x\", \"y\", \"y\", \"y\", \"y\"], b = [1, 1, 1, 2, 1, 2, 2, 2])\nt.fisher_test(a, b)\nt.fisher_test(a, b, alternative: less)\nlet z = frame(a = [\"x\", \"x\", \"x\", \"x\", \"x\", \"y\", \"y\", \"y\", \"y\", \"y\", \"y\", \"y\"], b = [1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2])\nz.fisher_test(a, b)\nlet m = frame(r = [\"a\", \"a\", \"a\", \"a\", \"a\", \"a\", \"b\", \"b\", \"b\", \"b\", \"c\", \"c\", \"c\", \"c\", \"c\", \"c\"], c = [1, 1, 2, 2, 2, 2, 1, 1, 1, 2, 1, 2, 2, 2, 2, 2])\nm.fisher_test(r, c)\nlet w = frame(r = [\"a\", \"a\", \"b\", \"b\", \"c\", \"c\"], c = [1, 2, 1, 2, 1, 2], w = [3, 1, 1, 3, 0, 0])\nw.fisher_test(r, c, weights: w, weight_type: frequency)\nlet d = frame(a = [\"x\", \"x\", NA, \"y\", \"y\", \"x\"], b = [1, 2, 1, 1, 2, 1])\nd.fisher_test(a, b)\n";
    let exact_outcome = run_script(&fs, &project, "scripts/main.zeff", exact);
    // A `TestResult` read as values (release 0.7.0, step 12b; open decision #94): the six
    // methods on a t test, a chi-squared test, a Wald test and an exact test, the widened
    // `estimates` of a test and of a fit, the sentence built with `format` and `join`, and the
    // refusal on a fit.
    let fields = "let wages = dataset(wages).add(lwage = log(wage))\nlet t = wages.t_test(lwage, group: female)\nt.statistic()\nt.df()\nt.effect_size()\nt.h0()\nt.h1()\nt.alternative()\nt.estimates()\nlet c = wages.chi2_test(union, female)\nc.df()\nc.effect_size()\nc.estimates()\nlet m = wages.reg(lwage ~ educ + female)\nm.estimates()\nm.test(educ == 0).df()\nwages.fisher_test(union, female).statistic()\njoin([\"t(\", format(t.df()[1], digits: 1), \") = \", format(t.statistic(), digits: 2), \", g = \", format(t.effect_size(), digits: 3)])\nm.df()\n";
    let fields_outcome = run_script(&fs, &project, "scripts/main.zeff", fields);
    // `shapiro_test` (release 0.7.0, step 14): fourteen values, the exact case of three, tied
    // values, small sizes on both sides of the approximation's branch, NA, a subset of the shipped
    // wages, the result read as values, and the refusal of the whole dataset.
    let normality = "let g = frame(s = [2.9, 3.0, 2.5, 2.6, 3.2, 3.8, 2.7, 4.0, 2.4, 2.8, 3.4, 3.7, 2.2, 2.0])\ng.shapiro_test(s)\nframe(x = [1.2, 3.4, 2.2]).shapiro_test(x)\nframe(x = [1, 1, 2]).shapiro_test(x)\nframe(x = [1, 2, 3, 4, 10]).shapiro_test(x)\nframe(x = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]).shapiro_test(x)\nframe(x = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 40]).shapiro_test(x)\nframe(x = [1.5, 2.5, NA, 4.5, 5.5, 6.5, 7.5, 2.0, 3.0, 9.0, 1.0, 2.5]).shapiro_test(x)\nlet wages = dataset(wages).where(year == 2010).add(lwage = log(wage))\nwages.shapiro_test(lwage)\nwages.shapiro_test(educ).statistic()\nwages.shapiro_test(educ).p_values()\ndataset(wages).shapiro_test(educ)\n";
    let normality_outcome = run_script(&fs, &project, "scripts/main.zeff", normality);
    // `ks_test` (release 0.7.0, step 15): the exact case against the uniform and the normal,
    // ties, the Lilliefors form, one value, NA, a subset of the shipped wages against a normal, a
    // truncated normal and the Poisson, the result read as values, and the refusal of
    // `estimated: true` on another family.
    let kolmogorov = "let g = frame(u = [0.11, 0.32, 0.45, 0.58, 0.63, 0.77, 0.84, 0.92])\ng.ks_test(u, dist.uniform())\ng.ks_test(u, dist.normal(mean: 0.5, sd: 0.3))\nframe(x = [1.2, 3.4, 2.2, 2.2, 5.1]).ks_test(x, dist.normal(mean: 2, sd: 1))\nframe(x = [1.2, 3.4, 2.2, 0.8, 5.1, 2.9, 3.3]).ks_test(x, dist.normal(), estimated: true)\nframe(x = [0.4]).ks_test(x, dist.uniform())\nframe(x = [1.5, 2.5, NA, 4.5, 5.5, 6.5, 7.5]).ks_test(x, dist.exponential(rate: 0.25))\nlet wages = dataset(wages).where(year == 2010).add(lwage = log(wage))\nwages.ks_test(lwage, dist.normal(mean: 3, sd: 0.5))\nwages.ks_test(lwage, dist.normal(), estimated: true)\nwages.ks_test(lwage, dist.truncated(dist.normal(mean: 3, sd: 0.5), lower: 1.5, upper: 4.6))\nwages.ks_test(educ, dist.poisson(lambda: 12))\nwages.ks_test(lwage, dist.normal(mean: 3, sd: 0.5)).statistic()\nwages.ks_test(lwage, dist.normal(mean: 3, sd: 0.5)).p_values()\nwages.ks_test(lwage, dist.exponential(), estimated: true)\n";
    let kolmogorov_outcome = run_script(&fs, &project, "scripts/main.zeff", kolmogorov);
    // `anova`, one-way (release 0.7.0, step 16): Welch's F and the classical table on three
    // groups, two groups, text groups with frequency and analytic weights, NA, a group of one under
    // the classical form, the shipped wages by industry, the result read as values, omega and
    // epsilon squared under `effect:` (step 17b) with the note of a negative estimate, and the
    // refusal of a group of one under Welch's form.
    let one_way = "let g = frame(s = [2.9, 3.0, 2.5, 2.6, 3.2, 3.8, 2.7, 4.0, 2.4, 2.8, 3.4, 3.7, 2.2, 2.0], site = [1, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 3])\ng.anova(s, group: site)\ng.anova(s, group: site, equal_var: true)\ng.where(site < 3).anova(s, group: site)\nlet t = frame(v = [3, 1, 4, 1, 5, 9, 2, 6, 5, 3, 5, 8, 9, 7, 9], k = [\"a\", \"b\", \"c\", \"a\", \"b\", \"c\", \"a\", \"b\", \"c\", \"a\", \"b\", \"c\", \"a\", \"b\", \"c\"], w = [1, 2, 1, 1, 1, 3, 1, 1, 1, 2, 1, 1, 1, 1, 1])\nt.anova(v, group: k, weights: w, weight_type: frequency)\nt.anova(v, group: k, weights: w, weight_type: analytic, equal_var: true)\nlet m = frame(x = [1.5, 2.5, NA, 4.5, 5.5, 6.5, 7.5, 3.5], g = [1, 1, 2, 2, NA, 3, 3, 2])\nm.anova(x, group: g)\nframe(x = [1.5, 2.5, 3.5, 4.5, 9.0], g = [1, 1, 2, 2, 3]).anova(x, group: g, equal_var: true)\nlet wages = dataset(wages).add(lwage = log(wage))\nwages.anova(lwage, group: industry)\nlet a = wages.where(year == 2010).anova(lwage, group: industry, equal_var: true)\na\na.df()\na.effect_size()\ng.anova(s, group: site, effect: omega)\ng.anova(s, group: site, equal_var: true, effect: epsilon)\nt.anova(v, group: k, weights: w, weight_type: frequency, effect: omega)\nwages.anova(lwage, group: industry, effect: epsilon)\nwages.anova(lwage, group: industry, effect: omega).effect_size()\nframe(x = [1.5, 2.5, 3.5, 4.5, 9.0], g = [1, 1, 2, 2, 3]).anova(x, group: g)\n";
    let one_way_outcome = run_script(&fs, &project, "scripts/main.zeff", one_way);
    // `anova`, factorial (release 0.7.0, step 17): the three types of sums of squares on an
    // unbalanced layout with a covariate, the terms in two orders, frequency and analytic weights,
    // an empty cell, a repeated variable, the shipped wages by industry and sex, the result read
    // as values, partial omega and epsilon squared under `effect:` (step 17b) with the note
    // naming the rows whose estimate is negative, and the refusal of type 3 where a cell is empty.
    let factorial = "let d = frame(y = [3.1, 4.2, 5.0, 6.3, 2.2, 7.9, 4.4, 5.1, 6.6, 3.8, 4.9, 7.2], a = [\"p\", \"p\", \"p\", \"p\", \"q\", \"q\", \"q\", \"q\", \"r\", \"r\", \"r\", \"r\"], b = [\"u\", \"v\", \"u\", \"v\", \"u\", \"v\", \"u\", \"v\", \"u\", \"v\", \"u\", \"v\"], x = [1.0, 2.5, 0.3, 4.1, 2.2, 3.3, 1.9, 0.7, 2.8, 3.9, 1.1, 2.0], w = [1, 2, 1, 0, 1, 3, 1, 1, 2, 1, 1, 1]).add(a = category(a), b = category(b))\nd.anova(y ~ a * b)\nd.anova(y ~ a * b + x, type: 1)\nd.anova(y ~ x + a * b, type: 1)\nd.anova(y ~ a * b + x, type: 3)\nd.anova(y ~ a + x, weights: w, weight_type: frequency)\nd.anova(y ~ a + x, weights: w, weight_type: analytic)\nd.where(!(a == \"r\" && b == \"v\")).anova(y ~ a * b)\nd.add(x2 = 2 * x).anova(y ~ x + x2)\nlet wages = dataset(wages).add(lwage = log(wage))\nwages.anova(lwage ~ industry * category(female) + educ)\nwages.anova(lwage ~ industry * category(female) + educ, type: 3)\nlet t = d.anova(y ~ a * b + x)\nt.statistic()\nt.df()\nt.p_values()\nt.estimates()\nd.anova(y ~ a * b, effect: omega)\nd.anova(y ~ a * b + x, type: 3, effect: epsilon)\nd.anova(y ~ a + x, weights: w, weight_type: frequency, effect: omega)\nwages.anova(lwage ~ industry * category(female) + educ, effect: omega)\nd.anova(y ~ a * b + x, effect: epsilon).estimates()\nd.where(!(a == \"r\" && b == \"v\")).anova(y ~ a * b, type: 3)\n";
    let factorial_outcome = run_script(&fs, &project, "scripts/main.zeff", factorial);
    // `posthoc` (release 0.7.0, step 19): the comparisons after Welch's and the classical one-way
    // `anova` under Tukey's and Scheffé's methods and the adjusted t-tests, single-step and
    // step-wise, with the note where the pooled methods follow Welch's F; two groups; frequency
    // and analytic weights; a group of one on the pooled variance; the 66 comparisons of the
    // shipped wages by industry, on the studentized range with 18,000 degrees of freedom; the
    // result read as values; after a factorial `anova` (step 19b) the marginal means of a main
    // effect and the cells of an interaction, a covariate inside `log()` that interacts with the
    // term compared, an empty cell left out, and the shipped wages by industry with union and
    // experience in the model; and the refusal after another test.
    let comparisons = "let g = frame(s = [2.9, 3.0, 2.5, 2.6, 3.2, 3.8, 2.7, 4.0, 2.4, 2.8, 3.4, 3.7, 2.2, 2.0], site = [1, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 3])\ng.anova(s, group: site).posthoc()\ng.anova(s, group: site, equal_var: true).posthoc(method: scheffe)\ng.anova(s, group: site).posthoc(method: holm)\ng.anova(s, group: site).posthoc(method: sidak)\ng.anova(s, group: site, equal_var: true).posthoc(method: hommel)\ng.anova(s, group: site).posthoc(method: by)\ng.where(site < 3).anova(s, group: site).posthoc(method: none)\nlet t = frame(v = [3, 1, 4, 1, 5, 9, 2, 6, 5, 3, 5, 8, 9, 7, 9], k = [\"a\", \"b\", \"c\", \"a\", \"b\", \"c\", \"a\", \"b\", \"c\", \"a\", \"b\", \"c\", \"a\", \"b\", \"c\"], w = [1, 2, 1, 1, 1, 3, 1, 1, 1, 2, 1, 1, 1, 1, 1])\nt.anova(v, group: k, weights: w, weight_type: frequency).posthoc(method: hochberg)\nt.anova(v, group: k, weights: w, weight_type: analytic, equal_var: true).posthoc(method: bonferroni)\nframe(x = [1.5, 2.5, 3.5, 4.5, 9.0], g = [1, 1, 2, 2, 3]).anova(x, group: g, equal_var: true).posthoc()\nlet wages = dataset(wages).add(lwage = log(wage))\nwages.anova(lwage, group: industry, equal_var: true).posthoc()\nwages.anova(lwage, group: industry).posthoc(method: bh)\nlet p = g.anova(s, group: site, equal_var: true).posthoc(method: bonferroni)\np.estimates()\np.ci()\np.statistic()\np.df()\np.p_values()\nlet f = frame(gain = [4.2, 3.9, 4.6, 4.1, 4.4, 5.1, 5.6, 4.9, 5.4, 5.8, 4.5, 4.0, 4.8, 4.3, 5.0, 4.7], dose = [\"low\", \"low\", \"low\", \"low\", \"mid\", \"mid\", \"mid\", \"mid\", \"mid\", \"high\", \"high\", \"high\", \"high\", \"high\", \"low\", \"mid\"], diet = [\"a\", \"b\", \"a\", \"b\", \"a\", \"b\", \"a\", \"b\", \"a\", \"b\", \"a\", \"b\", \"a\", \"b\", \"b\", \"b\"], age = [3, 5, 4, 6, 5, 7, 6, 4, 5, 3, 6, 4, 7, 5, 4, 6]).add(dose = category(dose, levels: [\"low\", \"mid\", \"high\"]), diet = category(diet))\nlet m = f.anova(gain ~ dose * diet + age)\nm.posthoc(term: dose)\nm.posthoc(term: diet, method: scheffe)\nm.posthoc(term: \"dose:diet\", method: holm)\nf.anova(gain ~ dose * log(age)).posthoc(method: sidak)\nf.where(!(dose == \"high\" && diet == \"a\")).anova(gain ~ dose * diet).posthoc(term: \"dose:diet\", method: bonferroni)\nwages.anova(lwage ~ industry * category(union) + exper).posthoc(term: industry)\nwages.anova(lwage ~ industry * exper).posthoc(method: by)\ng.kruskal_test(s, group: site).posthoc()\n";
    let comparisons_outcome = run_script(&fs, &project, "scripts/main.zeff", comparisons);
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
        text("verbs/tabulate/stdout", &tabulated_outcome.output.stdout),
        text("verbs/tabulate/stderr", &tabulated_outcome.output.stderr),
        text("tests/t_test/stdout", &tested_outcome.output.stdout),
        text("tests/t_test/stderr", &tested_outcome.output.stderr),
        text("tests/counts/stdout", &counted_outcome.output.stdout),
        text("tests/counts/stderr", &counted_outcome.output.stderr),
        text("tests/wilcoxon/stdout", &ranked_outcome.output.stdout),
        text("tests/wilcoxon/stderr", &ranked_outcome.output.stderr),
        text("tests/kruskal/stdout", &compared_outcome.output.stdout),
        text("tests/kruskal/stderr", &compared_outcome.output.stderr),
        text("tests/fisher/stdout", &exact_outcome.output.stdout),
        text("tests/fisher/stderr", &exact_outcome.output.stderr),
        text("tests/fields/stdout", &fields_outcome.output.stdout),
        text("tests/fields/stderr", &fields_outcome.output.stderr),
        text("tests/shapiro/stdout", &normality_outcome.output.stdout),
        text("tests/shapiro/stderr", &normality_outcome.output.stderr),
        text("tests/ks/stdout", &kolmogorov_outcome.output.stdout),
        text("tests/ks/stderr", &kolmogorov_outcome.output.stderr),
        text("tests/anova_oneway/stdout", &one_way_outcome.output.stdout),
        text("tests/anova_oneway/stderr", &one_way_outcome.output.stderr),
        text(
            "tests/anova_factorial/stdout",
            &factorial_outcome.output.stdout,
        ),
        text(
            "tests/anova_factorial/stderr",
            &factorial_outcome.output.stderr,
        ),
        text("tests/posthoc/stdout", &comparisons_outcome.output.stdout),
        text("tests/posthoc/stderr", &comparisons_outcome.output.stderr),
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

/// The plots of release 0.7.0 (step 20): the printed scenes of the shipped wages as a script
/// draws them, without a layer, with points, jittered under a seed, with a second layer of
/// another frame, of one variable, of very large and very small values, and a refusal; and the
/// scene itself as JSON, with every limit, break and jittered position to the last bit, so that
/// the four targets prove a figure's content identical before any renderer draws it.
fn plot_cases() -> Vec<Line> {
    use std::path::Path;
    use std::rc::Rc;
    use zeff_core::value::{Aesthetic, Frame, Mark, PlotLayer, PlotMapping};
    use zeff_engine::fs::MemoryFileSystem;
    use zeff_engine::project::Project;
    use zeff_engine::run::run_script;
    use zeff_numeric::random::Stream;
    let fs = MemoryFileSystem::new();
    fs.add_file(
        "/g/zeff.toml",
        format!(
            "[project]\nname = \"g\"\nzeff = \"{}\"\nedition = \"preview\"\n",
            zeff_engine::VERSION
        ),
    );
    let project = match Project::open(&fs, Path::new("/g")) {
        Ok(project) => project,
        Err(_) => return vec![text("plot/points", "the project did not open")],
    };
    let script = "seed(20261111)\nlet w = dataset(wages)\nw.plot(x: educ, y: wage)\nw.plot(x: educ, y: wage).points()\nw.plot(x: educ, y: wage).points(alpha: 0.3, jitter: 0.4)\nlet top = w.where(wage > 100)\nw.plot(x: exper, y: hours).points(jitter: 0.25).points(data: top, size: 8)\nw.plot(x: hours)\nframe(a = [1000000, 3000000], b = [0.00001, 0.00003]).plot(x: a, y: b).points()\nw.plot(x: industry, y: wage)\n";
    let outcome = run_script(&fs, &project, "scripts/main.zeff", script);
    let scene = match zeff_datasets::table("wages") {
        Some(table) => {
            let frame = Frame {
                table: Rc::new(table),
                name: Some("wages".to_string()),
            };
            let mapping = |aesthetic, variable: &str| PlotMapping {
                aesthetic,
                variable: variable.to_string(),
                computed: None,
            };
            let plot = zeff_plot::scene::start(
                frame,
                vec![mapping(Aesthetic::X, "educ"), mapping(Aesthetic::Y, "wage")],
                Vec::new(),
            );
            let layer = PlotLayer {
                mark: Mark::Points,
                data: None,
                alpha: 0.3,
                size: None,
                jitter: 0.4,
                nudge: (0.0, 0.0),
                width: None,
                stream: Some(Stream::new(20_261_111, 2, 0)),
                computed: Vec::new(),
                mappings: Vec::new(),
                constants: Vec::new(),
            };
            zeff_plot::scene::with_layer(&plot, layer).1.to_json()
        }
        None => "the wages did not load".to_string(),
    };
    // Step 20b: positions mapped from expressions, printed as a script draws them, and the scene
    // of the logarithm of the wage, whose every position is a value of the engine's own `log`.
    let script = "seed(20261111)\nlet w = dataset(wages)\nw.plot(x: educ, y: log(wage)).points()\nw.plot(x: exper^2, y: wage / hours).points()\nw.plot(x: sqrt(exper), y: exp(educ / 10)).points(jitter: 0.3)\nlet top = w.where(wage > 100)\nw.plot(x: log10(hours), y: log(wage, base: 2)).points(alpha: 0.3).points(data: top, size: 8)\nw.plot(x: educ, y: log(wage - 20)).points()\nw.plot(x: category(educ), y: log(wage))\n";
    let expression = run_script(&fs, &project, "scripts/main.zeff", script);
    let logarithm = match zeff_datasets::table("wages") {
        Some(table) => {
            use zeff_columns::Column;
            use zeff_core::value::{ColumnExpr, PlotComputed};
            let logs: Option<Vec<Option<f64>>> = table
                .column("wage")
                .and_then(|wage| wage.as_floats())
                .map(|wage| wage.map(|v| v.map(zeff_numeric::maths::log)).collect());
            match logs {
                Some(logs) => {
                    let frame = Frame {
                        table: Rc::new(table),
                        name: Some("wages".to_string()),
                    };
                    let plot = zeff_plot::scene::start(
                        frame,
                        vec![
                            PlotMapping {
                                aesthetic: Aesthetic::X,
                                variable: "educ".to_string(),
                                computed: None,
                            },
                            PlotMapping {
                                aesthetic: Aesthetic::Y,
                                variable: "log(wage)".to_string(),
                                computed: Some(PlotComputed {
                                    expr: ColumnExpr::Var("wage".to_string()),
                                    values: Rc::new(Column::floats(logs).0),
                                }),
                            },
                        ],
                        Vec::new(),
                    );
                    let layer = PlotLayer {
                        mark: Mark::Points,
                        data: None,
                        alpha: 1.0,
                        size: None,
                        jitter: 0.0,
                        nudge: (0.0, 0.0),
                        width: None,
                        stream: None,
                        computed: Vec::new(),
                        mappings: Vec::new(),
                        constants: Vec::new(),
                    };
                    zeff_plot::scene::with_layer(&plot, layer).1.to_json()
                }
                None => "the wage is not a Float".to_string(),
            }
        }
        None => "the wages did not load".to_string(),
    };
    // Step 20c: colour by categories, on the continuous ramp and as one colour for every mark,
    // printed as a script draws them, and the two scenes with their colour scales and legends.
    let script = "let w = dataset(wages)\nw.plot(x: educ, y: wage, color: female).points()\nw.plot(x: educ, y: wage, color: exper, fill: \"#48b\").points(alpha: 0.3)\nw.plot(x: educ, y: wage, color: industry).points()\nlet top = w.where(wage > 100)\nw.plot(x: educ, y: wage, color: union, fill: female).points().points(data: top, size: 8)\nw.plot(x: educ, y: wage, color: \"steelblue\").points()\nw.plot(x: educ, y: wage, color: \"steel blue\")\n";
    let colour = run_script(&fs, &project, "scripts/main.zeff", script);
    let coloured = match zeff_datasets::table("wages") {
        Some(table) => {
            use zeff_core::value::{PlotConstant, PlotSetting};
            let frame = Frame {
                table: Rc::new(table),
                name: Some("wages".to_string()),
            };
            let mapping = |aesthetic, variable: &str| PlotMapping {
                aesthetic,
                variable: variable.to_string(),
                computed: None,
            };
            let layer = || PlotLayer {
                mark: Mark::Points,
                data: None,
                alpha: 1.0,
                size: None,
                jitter: 0.0,
                nudge: (0.0, 0.0),
                width: None,
                stream: None,
                computed: Vec::new(),
                mappings: Vec::new(),
                constants: Vec::new(),
            };
            let categories = zeff_plot::scene::start(
                frame.clone(),
                vec![
                    mapping(Aesthetic::X, "educ"),
                    mapping(Aesthetic::Y, "wage"),
                    mapping(Aesthetic::Color, "industry"),
                ],
                Vec::new(),
            );
            let ramp = zeff_plot::scene::start(
                frame,
                vec![
                    mapping(Aesthetic::X, "educ"),
                    mapping(Aesthetic::Y, "wage"),
                    mapping(Aesthetic::Color, "exper"),
                ],
                vec![PlotConstant {
                    aesthetic: Aesthetic::Fill,
                    value: PlotSetting::Colour {
                        hex: "#4488BB".to_string(),
                        written: "#48b".to_string(),
                    },
                }],
            );
            format!(
                "{}\n{}",
                zeff_plot::scene::with_layer(&categories, layer())
                    .1
                    .to_json(),
                zeff_plot::scene::with_layer(&ramp, layer()).1.to_json()
            )
        }
        None => "the wages did not load".to_string(),
    };
    // Step 20d: a size by area, a shape by categories, the constants of both, a group and a
    // label, printed as a script draws them, and the scene with its size and shape scales, its
    // one legend for a variable shown twice and a second layer of a size of its own.
    let script = "let w = dataset(wages)\nw.plot(x: educ, y: wage, size: exper).points(alpha: 0.3)\nw.plot(x: educ, y: wage, shape: industry).points()\nlet top = w.where(wage > 100)\nw.plot(x: educ, y: wage, color: union, size: hours, shape: union).points(alpha: 0.2).points(data: top, size: 8)\nw.plot(x: educ, y: wage, size: 3, shape: \"diamond_open\", group: worker, label: firm).points()\nw.plot(x: educ, y: wage, shape: \"circle open\")\n";
    let size_shape = run_script(&fs, &project, "scripts/main.zeff", script);
    let sized = match zeff_datasets::table("wages") {
        Some(table) => {
            let frame = Frame {
                table: Rc::new(table),
                name: Some("wages".to_string()),
            };
            let mapping = |aesthetic, variable: &str| PlotMapping {
                aesthetic,
                variable: variable.to_string(),
                computed: None,
            };
            let layer = |size| PlotLayer {
                mark: Mark::Points,
                data: None,
                alpha: 1.0,
                size,
                jitter: 0.0,
                nudge: (0.0, 0.0),
                width: None,
                stream: None,
                computed: Vec::new(),
                mappings: Vec::new(),
                constants: Vec::new(),
            };
            let plot = zeff_plot::scene::start(
                frame,
                vec![
                    mapping(Aesthetic::X, "educ"),
                    mapping(Aesthetic::Y, "wage"),
                    mapping(Aesthetic::Color, "union"),
                    mapping(Aesthetic::Size, "exper"),
                    mapping(Aesthetic::Shape, "union"),
                    mapping(Aesthetic::Group, "worker"),
                    mapping(Aesthetic::Label, "firm"),
                ],
                Vec::new(),
            );
            let plot = zeff_plot::scene::with_layer(&plot, layer(None)).0;
            zeff_plot::scene::with_layer(&plot, layer(Some(8.0)))
                .1
                .to_json()
        }
        None => "the wages did not load".to_string(),
    };
    // Lines (step 23c): means joined from year to year, a line for each worker, a line broken
    // where the wage is missing; what the script prints, and the scene and the drawing of the
    // wages of each worker as lines of a type for each sex.
    let script = "let w = dataset(wages)\nlet u = w.aggregate(wage = mean(wage), by: [year, union])\nu.plot(x: year, y: wage, color: union).points().lines(linetype: union)\nlet few = w.where(worker <= 40)\nfew.plot(x: year, y: log(wage), group: worker).lines(width: 0.5)\nfew.plot(x: exper, y: wage).lines(group: worker, color: female)\nfew.plot(x: year, y: wage, group: worker).step(direction: mid, linetype: \"dotted\")\n";
    let lined = run_script(&fs, &project, "scripts/main.zeff", script);
    let lines = match zeff_datasets::table("wages") {
        Some(table) => {
            use std::rc::Rc;
            use zeff_core::value::{Aesthetic, Frame, Mark, PlotLayer, PlotMapping};
            use zeff_plot::layout::{JOURNAL, Size, layout};
            let mapping = |aesthetic, variable: &str| PlotMapping {
                aesthetic,
                variable: variable.to_string(),
                computed: None,
            };
            let plot = zeff_plot::scene::start(
                Frame {
                    table: Rc::new(table),
                    name: Some("wages".to_string()),
                },
                vec![
                    mapping(Aesthetic::X, "year"),
                    mapping(Aesthetic::Y, "wage"),
                    mapping(Aesthetic::Group, "worker"),
                ],
                Vec::new(),
            );
            let layer = PlotLayer {
                mark: Mark::Lines,
                data: None,
                alpha: 1.0,
                size: None,
                jitter: 0.0,
                nudge: (0.0, 0.0),
                width: Some(0.25),
                stream: None,
                computed: Vec::new(),
                mappings: vec![mapping(Aesthetic::Linetype, "female")],
                constants: Vec::new(),
            };
            let scene = zeff_plot::scene::with_layer(&plot, layer.clone()).1;
            let drawing = match layout(&scene, Size::TwoColumn, &JOURNAL) {
                Ok(drawing) => drawing.describe(),
                Err(why) => format!("{why:?}\n"),
            };
            // The same rows as steps that go upright first (step 23c2).
            let stepped = PlotLayer {
                mark: Mark::Step(zeff_core::value::Stairs::UprightFirst),
                ..layer
            };
            let steps = zeff_plot::scene::with_layer(&plot, stepped).1;
            let stairs = match layout(&steps, Size::TwoColumn, &JOURNAL) {
                Ok(drawing) => drawing.describe(),
                Err(why) => format!("{why:?}\n"),
            };
            format!(
                "{}\n=== drawing\n{drawing}=== steps\n{}\n=== drawing\n{stairs}",
                scene.to_json(),
                steps.to_json()
            )
        }
        None => "the wages did not load".to_string(),
    };
    // The kinds of scale of an axis (step 25a): the wages against schooling with the wage on
    // a log axis and the schooling on a square-root one, as a scene and as a drawing, where
    // every place is the logarithm or the root of a value, computed by the engine's own
    // functions.
    let scaled = match zeff_datasets::table("wages") {
        Some(table) => {
            use std::rc::Rc;
            use zeff_core::value::{
                Aesthetic, AxisKind, Frame, Mark, PlotAxis, PlotLayer, PlotMapping,
            };
            use zeff_plot::layout::{JOURNAL, Size, layout};
            let mapping = |aesthetic, variable: &str| PlotMapping {
                aesthetic,
                variable: variable.to_string(),
                computed: None,
            };
            let mut plot = zeff_plot::scene::start(
                Frame {
                    table: Rc::new(table),
                    name: Some("wages".to_string()),
                },
                vec![mapping(Aesthetic::X, "educ"), mapping(Aesthetic::Y, "wage")],
                Vec::new(),
            );
            plot.axes = vec![
                PlotAxis::of(Aesthetic::Y, AxisKind::Log),
                PlotAxis::of(Aesthetic::X, AxisKind::Sqrt),
            ];
            let layer = PlotLayer {
                mark: Mark::Points,
                data: None,
                alpha: 0.3,
                size: None,
                jitter: 0.0,
                nudge: (0.0, 0.0),
                width: None,
                stream: None,
                computed: Vec::new(),
                mappings: Vec::new(),
                constants: Vec::new(),
            };
            let scene = zeff_plot::scene::with_layer(&plot, layer).1;
            let drawing = match layout(&scene, Size::TwoColumn, &JOURNAL) {
                Ok(drawing) => drawing.describe(),
                Err(why) => format!("{why:?}\n"),
            };
            format!("{}\n=== drawing\n{drawing}", scene.to_json())
        }
        None => "the wages did not load".to_string(),
    };
    // Filled marks (step 23d): a band of two standard errors around the yearly means and an
    // area under the means of each group, what the script prints, and each figure as the
    // bytes of its SVG, its PDF and its PNG, where a see-through fill is drawn three ways.
    let script = "let w = dataset(wages).drop_missing([wage])\nlet m = w.aggregate(se = sd(wage) / sqrt(n()), wage = mean(wage), by: year)\nlet u = w.aggregate(wage = mean(wage), by: [year, union])\nlet band = m.plot(x: year, y: wage).ribbon(low: wage - 2 * se, high: wage + 2 * se).lines()\nband\nlet areas = u.plot(x: year, y: wage, color: union, fill: union).area().lines()\nareas\nu.plot(x: year).ribbon(low: wage * 0.9, high: wage * 1.1, group: union, fill: \"steelblue\")\nu.plot(x: year, y: wage, fill: union).area(position: stack)\nu.where(year != 2015 || union == 1).plot(x: year, y: wage, fill: union).area(position: fill)\nexport(band, \"band.svg\")\nexport(band, \"band.pdf\")\nexport(band, \"band.png\")\nexport(areas, \"areas.svg\")\nexport(areas, \"areas.pdf\")\nexport(areas, \"areas.png\")\n";
    let filled = run_script(&fs, &project, "scripts/main.zeff", script);
    let mut fills = vec![
        text("plot/fills/stdout", &filled.output.stdout),
        text("plot/fills/stderr", &filled.output.stderr),
    ];
    for name in [
        "band.svg",
        "band.pdf",
        "band.png",
        "areas.svg",
        "areas.pdf",
        "areas.png",
    ] {
        let data = zeff_engine::fs::FileSystem::read(&fs, Path::new(&format!("/g/output/{name}")))
            .unwrap_or_default();
        fills.push(bytes(&format!("plot/fills/{name}"), &data));
    }
    // Reference lines (step 23e1): at a number and at the values of a variable of another
    // frame, coloured by it, over the points; what the script prints and the figure in the
    // three formats.
    let script = "let w = dataset(wages)\nlet m = w.aggregate(wage = mean(wage), by: union)\nlet few = w.where(worker <= 40)\nlet p = few.plot(x: educ, y: wage).points(alpha: 0.5).hline(data: m, at: wage, color: union).vline(at: 12).vline(at: 16, style: dash_dot, color: \"firebrick\", width: 1)\np\nfew.plot(x: educ).vline(at: educ, style: solid, width: 0.25)\nexport(p, \"rules.svg\")\nexport(p, \"rules.pdf\")\nexport(p, \"rules.png\")\n";
    let ruled = run_script(&fs, &project, "scripts/main.zeff", script);
    fills.push(text("plot/rules/stdout", &ruled.output.stdout));
    fills.push(text("plot/rules/stderr", &ruled.output.stderr));
    for name in ["rules.svg", "rules.pdf", "rules.png"] {
        let data = zeff_engine::fs::FileSystem::read(&fs, Path::new(&format!("/g/output/{name}")))
            .unwrap_or_default();
        fills.push(bytes(&format!("plot/rules/{name}"), &data));
    }
    // A rug (step 23e2): on both sides under the points, coloured as they are, and along one
    // axis by itself; what the script prints and the first figure in the three formats.
    let script = "let w = dataset(wages).where(worker <= 60)\nlet p = w.plot(x: educ, y: wage, color: union).points(alpha: 0.5).rug(sides: both, alpha: 0.4)\np\nw.plot(x: wage).rug(alpha: 0.3)\nw.plot(x: educ, y: wage).rug(sides: y, color: \"firebrick\")\nexport(p, \"rug.svg\")\nexport(p, \"rug.pdf\")\nexport(p, \"rug.png\")\n";
    let rugged = run_script(&fs, &project, "scripts/main.zeff", script);
    fills.push(text("plot/rug/stdout", &rugged.output.stdout));
    fills.push(text("plot/rug/stderr", &rugged.output.stderr));
    for name in ["rug.svg", "rug.pdf", "rug.png"] {
        let data = zeff_engine::fs::FileSystem::read(&fs, Path::new(&format!("/g/output/{name}")))
            .unwrap_or_default();
        fills.push(bytes(&format!("plot/rug/{name}"), &data));
    }
    // An axis of categories (step 24a): the wages by industry as points, the means of the
    // industries joined, and a category on each axis; what the script prints and the first
    // figure in the three formats, with a label under each place.
    let script = "seed(20261112)\nlet w = dataset(wages).where(worker <= 200)\nlet p = w.plot(x: industry, y: wage, color: union).points(alpha: 0.5, jitter: 0.25)\np\nlet m = w.aggregate(wage = mean(wage), by: industry)\nm.plot(x: industry, y: wage).points().lines()\nw.plot(x: wage, y: industry).rug()\nexport(p, \"categories.svg\")\nexport(p, \"categories.pdf\")\nexport(p, \"categories.png\")\n";
    let placed = run_script(&fs, &project, "scripts/main.zeff", script);
    fills.push(text("plot/categories/stdout", &placed.output.stdout));
    fills.push(text("plot/categories/stderr", &placed.output.stderr));
    for name in ["categories.svg", "categories.pdf", "categories.png"] {
        let data = zeff_engine::fs::FileSystem::read(&fs, Path::new(&format!("/g/output/{name}")))
            .unwrap_or_default();
        fills.push(bytes(&format!("plot/categories/{name}"), &data));
    }
    // Bars (step 24b): the counts of the industries, stacked by union membership, side by
    // side and as shares, and the mean wage of each industry as a bar; what the script prints
    // and the stacked figure in the three formats.
    let script = "let w = dataset(wages)\nw.plot(x: industry).bars()\nlet p = w.plot(x: industry, fill: union).bars()\np\nw.plot(x: industry, fill: union).bars(position: dodge, width: 0.8)\nw.plot(x: industry, fill: union).bars(position: fill)\nlet m = w.aggregate(wage = mean(wage), by: industry)\nm.plot(x: industry, y: wage).bars(fill: \"steelblue\", alpha: 0.8)\nw.plot(x: educ).bars()\nw.plot(x: industry, color: union).bars(position: dodge)\nexport(p, \"bars.svg\")\nexport(p, \"bars.pdf\")\nexport(p, \"bars.png\")\n";
    let barred = run_script(&fs, &project, "scripts/main.zeff", script);
    fills.push(text("plot/bars/stdout", &barred.output.stdout));
    fills.push(text("plot/bars/stderr", &barred.output.stderr));
    for name in ["bars.svg", "bars.pdf", "bars.png"] {
        let data = zeff_engine::fs::FileSystem::read(&fs, Path::new(&format!("/g/output/{name}")))
            .unwrap_or_default();
        fills.push(bytes(&format!("plot/bars/{name}"), &data));
    }
    // Tiles (step 24c): the mean wage of each industry in each year as a heat map on a
    // continuous scale of fills, and the same cells by a category; what the script prints
    // and the heat map in the three formats, with the bar of the ramp for its legend.
    let script = "let w = dataset(wages)\nlet m = w.aggregate(wage = mean(wage), by: [year, industry])\nlet p = m.plot(x: year, y: industry, fill: wage).tile()\np\nm.add(high = wage > 28).plot(x: year, y: industry, fill: high).tile(alpha: 0.8)\nm.plot(x: year, y: wage).tile()\nexport(p, \"tiles.svg\")\nexport(p, \"tiles.pdf\")\nexport(p, \"tiles.png\")\n";
    let tiled = run_script(&fs, &project, "scripts/main.zeff", script);
    fills.push(text("plot/tiles/stdout", &tiled.output.stdout));
    fills.push(text("plot/tiles/stderr", &tiled.output.stderr));
    for name in ["tiles.svg", "tiles.pdf", "tiles.png"] {
        let data = zeff_engine::fs::FileSystem::read(&fs, Path::new(&format!("/g/output/{name}")))
            .unwrap_or_default();
        fills.push(bytes(&format!("plot/tiles/{name}"), &data));
    }
    // Text (step 24d): the mean wage of each industry as a bar with its value written on it,
    // and the names of a few rows beside their points; what the script prints and the first
    // figure in the three formats, where the labels are set in the embedded face.
    let script = "let w = dataset(wages)\nlet m = w.aggregate(wage = mean(wage), by: industry).add(shown = round(wage, 1))\nlet p = m.plot(x: industry, y: wage).bars(fill: \"steelblue\").text(label: shown, nudge: [0, 5], size: 7)\np\nlet few = w.where(worker <= 3 && year == 2015)\nfew.plot(x: educ, y: wage, color: union).points().text(label: worker, nudge: [6, 0])\nexport(p, \"text.svg\")\nexport(p, \"text.pdf\")\nexport(p, \"text.png\")\n";
    let written = run_script(&fs, &project, "scripts/main.zeff", script);
    fills.push(text("plot/text/stdout", &written.output.stdout));
    fills.push(text("plot/text/stderr", &written.output.stderr));
    for name in ["text.svg", "text.pdf", "text.png"] {
        let data = zeff_engine::fs::FileSystem::read(&fs, Path::new(&format!("/g/output/{name}")))
            .unwrap_or_default();
        fills.push(bytes(&format!("plot/text/{name}"), &data));
    }
    // The scales of the axes (step 25b): the wages on a log axis, on square-root axes, within
    // limits and with breaks and labels of the script's own; what the script prints and the
    // log figure in the three formats.
    let script = "let w = dataset(wages)\nlet p = w.plot(x: educ, y: wage, color: union).points(alpha: 0.3).scale_y(log)\np\nw.plot(x: educ, y: wage).points(alpha: 0.3).scale_y(sqrt).scale_x(sqrt)\nw.plot(x: educ, y: wage).points(alpha: 0.3).scale_y(limits: [0, 60], breaks: [0, 20, 40, 60], labels: [\"none\", \"20\", \"40\", \"60\"]).scale_x(limits: [10, NA])\nw.plot(x: union, y: wage).points(alpha: 0.3)\nexport(p, \"scales.svg\")\nexport(p, \"scales.pdf\")\nexport(p, \"scales.png\")\n";
    let scaled_script = run_script(&fs, &project, "scripts/main.zeff", script);
    fills.push(text("plot/scales/stdout", &scaled_script.output.stdout));
    fills.push(text("plot/scales/stderr", &scaled_script.output.stderr));
    for name in ["scales.svg", "scales.pdf", "scales.png"] {
        let data = zeff_engine::fs::FileSystem::read(&fs, Path::new(&format!("/g/output/{name}")))
            .unwrap_or_default();
        fills.push(bytes(&format!("plot/scales/{name}"), &data));
    }
    // An axis of dates (step 25c1): the firms against the day each was founded, the count
    // of firms founded in each year as bars at the first day of the year, and a series of
    // times within one day; what the script prints and the first figure in the three formats.
    let script = "let f = dataset(firms)\nlet p = f.plot(x: founded, y: size).points(alpha: 0.4)\np\nlet c = f.add(y = year(founded)).aggregate(by: y, firms = n()).add(start = add_months(date(\"1970-01-01\"), (y - 1970) * 12))\nc.plot(x: start, y: firms).bars()\nlet t = frame(read = [\"2024-03-05 09:30:00\", \"2024-03-05 12:00:00\", \"2024-03-05 16:45:00\"], level = [1, 4, 2]).add(when = datetime(read))\nt.plot(x: when, y: level).step().points()\nlet e = frame(read = [\"1973-10-17\", \"2008-09-15\"]).add(day = date(read))\nf.plot(x: founded, y: size).points(alpha: 0.4).vline(data: e, at: day)\nexport(p, \"dates.svg\")\nexport(p, \"dates.pdf\")\nexport(p, \"dates.png\")\n";
    let dated = run_script(&fs, &project, "scripts/main.zeff", script);
    fills.push(text("plot/dates/stdout", &dated.output.stdout));
    fills.push(text("plot/dates/stderr", &dated.output.stderr));
    for name in ["dates.svg", "dates.pdf", "dates.png"] {
        let data = zeff_engine::fs::FileSystem::read(&fs, Path::new(&format!("/g/output/{name}")))
            .unwrap_or_default();
        fills.push(bytes(&format!("plot/dates/{name}"), &data));
    }
    // The scale of dates (step 25c2): the firms within limits said as dates, with a pattern
    // for the labels, breaks of the script's own and a line at one date; what the script
    // prints and the first figure in the three formats.
    let script = "let f = dataset(firms)\nlet crash = date(\"2008-09-15\")\nlet p = f.plot(x: founded, y: size).points(alpha: 0.4).scale_x(date, limits: [date(\"1970-01-01\"), NA], labels: \"%b %Y\").vline(at: date(\"1973-10-17\")).vline(at: crash, color: \"firebrick\", style: solid)\np\nf.plot(x: founded, y: size).points(alpha: 0.4).scale_x(breaks: [date(\"1973-10-17\"), crash], labels: [\"oil\", \"Lehman\"])\nf.plot(x: size, y: founded).points(alpha: 0.4).scale_y(date, limits: [date(\"1990-01-01\"), date(\"1992-01-01\")])\nexport(p, \"datescale.svg\")\nexport(p, \"datescale.pdf\")\nexport(p, \"datescale.png\")\n";
    let date_scaled = run_script(&fs, &project, "scripts/main.zeff", script);
    fills.push(text("plot/datescale/stdout", &date_scaled.output.stdout));
    fills.push(text("plot/datescale/stderr", &date_scaled.output.stderr));
    for name in ["datescale.svg", "datescale.pdf", "datescale.png"] {
        let data = zeff_engine::fs::FileSystem::read(&fs, Path::new(&format!("/g/output/{name}")))
            .unwrap_or_default();
        fills.push(bytes(&format!("plot/datescale/{name}"), &data));
    }
    // An expression in a mapping that calls the functions of `time` and `text` (step
    // 25c3a): the year of a date on a linear axis, text made of a category on an axis of
    // categories, a date moved by a year on a scale of dates; what the script prints and
    // the first figure in the three formats.
    let script = "let f = dataset(firms)\nlet p = f.plot(x: year(founded), y: size).points(alpha: 0.4)\np\nf.plot(x: slice(industry, 1, 3), y: size).points(alpha: 0.4)\nf.plot(x: add_days(founded, 365), y: size).points(alpha: 0.4)\nf.plot(x: quarter(founded)).bars()\nf.plot(x: size, y: contains(industry, \"a\")).points()\nexport(p, \"mapped.svg\")\nexport(p, \"mapped.pdf\")\nexport(p, \"mapped.png\")\n";
    let widened = run_script(&fs, &project, "scripts/main.zeff", script);
    fills.push(text("plot/mapped/stdout", &widened.output.stdout));
    fills.push(text("plot/mapped/stderr", &widened.output.stderr));
    for name in ["mapped.svg", "mapped.pdf", "mapped.png"] {
        let data = zeff_engine::fs::FileSystem::read(&fs, Path::new(&format!("/g/output/{name}")))
            .unwrap_or_default();
        fills.push(bytes(&format!("plot/mapped/{name}"), &data));
    }
    // The four functions of `core` written in Zeff, in a mapping (step 25c3b): the log odds
    // of a share with the warning of a share of zero, a probability from a number, a value
    // held in a range and a missing value filled; what the script prints.
    let script = "let d = frame(share = [0.1, 0.5, 0.9, 0, NA], x = [-2, 0.5, 7, 1, 2], k = [1, NA, 3, 4, 5], s = [\"a\", NA, \"b\", \"a\", \"b\"])\nd.plot(x: x, y: logit(share)).points()\nd.plot(x: x, y: inv_logit(x)).lines()\nd.plot(x: clamp(x, 0, 3), y: x).points()\nd.plot(x: x, y: fill_missing(k, 0)).points()\nd.plot(x: x, y: fill_missing(s, \"none\")).points()\n";
    let inlined = run_script(&fs, &project, "scripts/main.zeff", script);
    fills.push(text("plot/zeffmapped/stdout", &inlined.output.stdout));
    fills.push(text("plot/zeffmapped/stderr", &inlined.output.stderr));
    // `case` in a mapping (step 25c3c): a wage held at 100, three groups of schooling as
    // text on an axis of categories, and a `case` with no catch-all, whose unmatched rows are
    // counted; what the script prints and the first figure in the three formats.
    let script = "let w = dataset(wages)\nlet p = w.plot(x: case { educ < 12 => \"less\", educ == 12 => \"school\", educ > 12 => \"more\" }, y: case { wage > 100 => 100, _ => wage }).points(alpha: 0.2)\np\nw.plot(x: educ, y: case { wage > 20 => 1 }).points()\nexport(p, \"case.svg\")\nexport(p, \"case.pdf\")\nexport(p, \"case.png\")\n";
    let chosen = run_script(&fs, &project, "scripts/main.zeff", script);
    fills.push(text("plot/case/stdout", &chosen.output.stdout));
    fills.push(text("plot/case/stderr", &chosen.output.stderr));
    for name in ["case.svg", "case.pdf", "case.png"] {
        let data = zeff_engine::fs::FileSystem::read(&fs, Path::new(&format!("/g/output/{name}")))
            .unwrap_or_default();
        fills.push(bytes(&format!("plot/case/{name}"), &data));
    }
    // The scales of colour, size and shape (step 25d): two colours of the script for two
    // categories, a ramp through three colours for a number, viridis for twelve categories,
    // sizes read from the smallest value to the largest, and shapes of the script's own;
    // what the script prints and the ramp in the three formats.
    let script = "let w = dataset(wages).where(year == 2015)\nw.plot(x: educ, y: wage, color: union).scale(color: [\"grey\", \"firebrick\"]).points(alpha: 0.5)\nlet p = w.plot(x: educ, y: wage, color: hours).scale(color: [\"white\", \"steelblue\", \"navy\"]).points()\np\nw.plot(x: educ, y: wage, color: industry).scale(color: viridis).points()\nw.plot(x: educ, y: wage, size: hours, shape: union).scale(size: [2, 12], shape: [\"circle_open\", \"cross\"]).points(alpha: 0.4)\nw.plot(x: industry, color: union).scale(color: [\"grey\", \"black\"]).bars(position: dodge)\nexport(p, \"palette.svg\")\nexport(p, \"palette.pdf\")\nexport(p, \"palette.png\")\n";
    let paletted = run_script(&fs, &project, "scripts/main.zeff", script);
    fills.push(text("plot/palette/stdout", &paletted.output.stdout));
    fills.push(text("plot/palette/stderr", &paletted.output.stderr));
    for name in ["palette.svg", "palette.pdf", "palette.png"] {
        let data = zeff_engine::fs::FileSystem::read(&fs, Path::new(&format!("/g/output/{name}")))
            .unwrap_or_default();
        fills.push(bytes(&format!("plot/palette/{name}"), &data));
    }
    // The place of the legend (step 25e1): above the panel, under it and not drawn, with
    // the bar of a continuous scale lying above the panel; what the script prints, the keys
    // above the panel in the three formats and the lying bar as a figure.
    let script = "let w = dataset(wages).where(year == 2015)\nlet p = w.plot(x: educ, y: wage, color: union).points(alpha: 0.5).legend(top)\np\nw.plot(x: educ, y: wage, color: union).points(alpha: 0.5).legend(bottom)\nw.plot(x: educ, y: wage, color: union).points(alpha: 0.5).legend(none)\nlet q = w.plot(x: educ, y: wage, color: hours).points().legend(top)\nexport(p, \"legend.svg\")\nexport(p, \"legend.pdf\")\nexport(p, \"legend.png\")\nexport(q, \"legend_ramp.svg\")\n";
    let legended = run_script(&fs, &project, "scripts/main.zeff", script);
    fills.push(text("plot/legend/stdout", &legended.output.stdout));
    fills.push(text("plot/legend/stderr", &legended.output.stderr));
    for name in ["legend.svg", "legend.pdf", "legend.png", "legend_ramp.svg"] {
        let data = zeff_engine::fs::FileSystem::read(&fs, Path::new(&format!("/g/output/{name}")))
            .unwrap_or_default();
        fills.push(bytes(&format!("plot/legend/{name}"), &data));
    }
    // The names of the lines at their ends (step 25e2): the mean wage of each industry by
    // year as twelve lines, each named at its end in its colour, the names moved apart
    // where they would touch; what the script prints and the figure in the three formats.
    let script = "let w = dataset(wages)\nlet m = w.aggregate(wage = mean(wage), by: [year, industry])\nlet p = m.plot(x: year, y: wage, color: industry).scale(color: viridis).lines().legend(direct)\np\nlet u = w.aggregate(wage = mean(wage), by: [year, union])\nu.plot(x: year, y: wage, color: union).lines().points().legend(direct)\nu.plot(x: year, y: wage, color: union).points().legend(direct)\nexport(p, \"direct.svg\")\nexport(p, \"direct.pdf\")\nexport(p, \"direct.png\")\n";
    let ended = run_script(&fs, &project, "scripts/main.zeff", script);
    fills.push(text("plot/direct/stdout", &ended.output.stdout));
    fills.push(text("plot/direct/stderr", &ended.output.stderr));
    for name in ["direct.svg", "direct.pdf", "direct.png"] {
        let data = zeff_engine::fs::FileSystem::read(&fs, Path::new(&format!("/g/output/{name}")))
            .unwrap_or_default();
        fills.push(bytes(&format!("plot/direct/{name}"), &data));
    }
    // The words of a plot (step 26a): a title, a subtitle and a caption, the titles of an
    // axis and of the legend of colour in the words of the script, and a title taken away;
    // what the script prints and the titled figure in the three formats.
    let script = "let w = dataset(wages).where(year == 2015)\nlet p = w.plot(x: educ, y: log(wage), color: union).points(alpha: 0.5).labels(title: \"Schooling and wages in 2015\", subtitle: \"Hourly wages of 2,152 workers\", y: \"log wage\", color: \"In a union\", caption: \"Source: the wages data shipped with the engine.\")\np\np.labels(x: \"\", title: \"\")\np.legend(top)\nexport(p, \"titled.svg\")\nexport(p, \"titled.pdf\")\nexport(p, \"titled.png\")\n";
    let titled = run_script(&fs, &project, "scripts/main.zeff", script);
    fills.push(text("plot/titled/stdout", &titled.output.stdout));
    fills.push(text("plot/titled/stderr", &titled.output.stderr));
    for name in ["titled.svg", "titled.pdf", "titled.png"] {
        let data = zeff_engine::fs::FileSystem::read(&fs, Path::new(&format!("/g/output/{name}")))
            .unwrap_or_default();
        fills.push(bytes(&format!("plot/titled/{name}"), &data));
    }
    // The themes (step 26b): one plot in `minimal`, on a slide and at a base size of 12
    // points; what the script prints, the minimal figure in the three formats and the slide
    // as a figure of the size of a slide.
    let script = "let w = dataset(wages).where(year == 2015)\nlet p = w.plot(x: educ, y: wage, color: union).points(alpha: 0.5).labels(title: \"Schooling and wages\")\nlet m = p.theme(minimal)\nm\nlet s = p.theme(slide)\ns\np.theme(base_size: 12)\nexport(m, \"theme.svg\")\nexport(m, \"theme.pdf\")\nexport(m, \"theme.png\")\nexport(s, \"theme_slide.svg\", size: slide)\n";
    let themed = run_script(&fs, &project, "scripts/main.zeff", script);
    fills.push(text("plot/theme/stdout", &themed.output.stdout));
    fills.push(text("plot/theme/stderr", &themed.output.stderr));
    for name in ["theme.svg", "theme.pdf", "theme.png", "theme_slide.svg"] {
        let data = zeff_engine::fs::FileSystem::read(&fs, Path::new(&format!("/g/output/{name}")))
            .unwrap_or_default();
        fills.push(bytes(&format!("plot/theme/{name}"), &data));
    }
    // An annotation, and the labels of a crowded axis (step 26c): a text at a place of a
    // series, at a category and at a date; five regions whose labels are broken onto two or
    // three lines; what the script prints, the annotated series in the three formats and
    // the broken labels as a figure.
    let script = "let w = dataset(wages)\nlet u = w.aggregate(wage = mean(wage), by: [year, union])\nlet p = u.plot(x: year, y: wage, color: union).lines().annotate(text: \"the gap widens\", x: 2017, y: 34, size: 10)\np\nlet f = dataset(firms)\nf.plot(x: founded, y: size).points(alpha: 0.3).annotate(text: \"oil crisis\", x: date(\"1973-10-17\"), y: 38)\nlet s = frame(group = [\"New south region\", \"Old north region\", \"Far east region\", \"Mid west region\", \"The islands\"], n = [12, 30, 22, 8, 15])\nlet q = s.plot(x: group, y: n).bars().annotate(text: \"largest\", x: \"Old north region\", y: 31)\nq\nexport(p, \"annotated.svg\")\nexport(p, \"annotated.pdf\")\nexport(p, \"annotated.png\")\nexport(q, \"wrapped.svg\")\n";
    let annotated = run_script(&fs, &project, "scripts/main.zeff", script);
    fills.push(text("plot/annotate/stdout", &annotated.output.stdout));
    fills.push(text("plot/annotate/stderr", &annotated.output.stderr));
    for name in [
        "annotated.svg",
        "annotated.pdf",
        "annotated.png",
        "wrapped.svg",
    ] {
        let data = zeff_engine::fs::FileSystem::read(&fs, Path::new(&format!("/g/output/{name}")))
            .unwrap_or_default();
        fills.push(bytes(&format!("plot/annotate/{name}"), &data));
    }
    // Flipped coordinates (step 27a): the mean wage of each industry as bars that lie along
    // the page, every name written; a series with a reference line and a note, turned; what
    // the script prints, the lying bars in the three formats and the turned series as a
    // figure.
    let script = "let w = dataset(wages)\nlet m = w.where(year == 2015).aggregate(wage = mean(wage), by: [industry])\nlet p = m.plot(x: industry, y: wage).bars().coord(flip)\np\nlet u = w.aggregate(wage = mean(wage), by: [year, union])\nlet q = u.plot(x: year, y: wage, color: union).lines().hline(at: 30).annotate(text: \"thirty an hour\", x: 2012, y: 31).coord(flip)\nq\nq.coord(cartesian)\nexport(p, \"flipped.svg\")\nexport(p, \"flipped.pdf\")\nexport(p, \"flipped.png\")\nexport(q, \"turned.svg\")\n";
    let flipped = run_script(&fs, &project, "scripts/main.zeff", script);
    fills.push(text("plot/coord/stdout", &flipped.output.stdout));
    fills.push(text("plot/coord/stderr", &flipped.output.stderr));
    for name in ["flipped.svg", "flipped.pdf", "flipped.png", "turned.svg"] {
        let data = zeff_engine::fs::FileSystem::read(&fs, Path::new(&format!("/g/output/{name}")))
            .unwrap_or_default();
        fills.push(bytes(&format!("plot/coord/{name}"), &data));
    }
    // Fixed coordinates (step 27b): the hours of work against the age of each worker with
    // a unit of each the same length, and with a unit of `y` twice as long; a log axis,
    // where a power of ten is a unit; what the script prints, the first figure in the three
    // formats and the one with a legend above it as a figure of two columns.
    let script = "let w = dataset(wages).where(year == 2015)\nlet p = w.plot(x: age, y: hours, color: union).points(alpha: 0.4).coord(fixed)\np\nlet q = p.coord(fixed, ratio: 2).legend(top)\nq\nw.plot(x: wage, y: educ).points().scale_x(log).coord(fixed, ratio: 0.1)\nexport(p, \"fixed.svg\")\nexport(p, \"fixed.pdf\")\nexport(p, \"fixed.png\")\nexport(q, \"fixed_wide.svg\", size: two_column)\n";
    let fixed = run_script(&fs, &project, "scripts/main.zeff", script);
    fills.push(text("plot/fixed/stdout", &fixed.output.stdout));
    fills.push(text("plot/fixed/stderr", &fixed.output.stderr));
    for name in ["fixed.svg", "fixed.pdf", "fixed.png", "fixed_wide.svg"] {
        let data = zeff_engine::fs::FileSystem::read(&fs, Path::new(&format!("/g/output/{name}")))
            .unwrap_or_default();
        fills.push(bytes(&format!("plot/fixed/{name}"), &data));
    }
    // Polar coordinates (step 27c2): the mean wage of each industry as wedges of one
    // circle, every name round the rim; the calls of a day round a clock, whose two ends
    // share the top; the wage by year and union as two rings; what the script prints, the
    // wedges in the three formats and the two others as figures.
    let script = "let w = dataset(wages)\nlet m = w.where(year == 2015).aggregate(wage = mean(wage), by: [industry])\nlet p = m.plot(x: industry, y: wage).bars().coord(polar)\np\nlet h = frame(hour = [0, 3, 6, 9, 12, 15, 18, 21, 24], calls = [2, 1, 4, 12, 15, 11, 9, 5, 2])\nlet q = h.plot(x: hour, y: calls).area(alpha: 0.4).lines().scale_x(breaks: [0, 6, 12, 18, 24]).coord(polar)\nq\nlet u = w.aggregate(wage = mean(wage), by: [year, union])\nlet r = u.plot(x: year, y: union, fill: wage).tile().coord(polar).theme(minimal)\nr\nexport(p, \"polar.svg\")\nexport(p, \"polar.pdf\")\nexport(p, \"polar.png\")\nexport(q, \"clock.svg\")\nexport(r, \"rings.svg\", size: two_column)\n";
    let wound = run_script(&fs, &project, "scripts/main.zeff", script);
    fills.push(text("plot/polar/stdout", &wound.output.stdout));
    fills.push(text("plot/polar/stderr", &wound.output.stderr));
    for name in [
        "polar.svg",
        "polar.pdf",
        "polar.png",
        "clock.svg",
        "rings.svg",
    ] {
        let data = zeff_engine::fs::FileSystem::read(&fs, Path::new(&format!("/g/output/{name}")))
            .unwrap_or_default();
        fills.push(bytes(&format!("plot/polar/{name}"), &data));
    }
    // Panels (step 27d3): the wage against schooling in 2015, parted by union membership
    // and coloured by sex, and parted by industry into twelve panels; stacked bars by year
    // within each panel, flipped; what the script prints, the two panels in the three
    // formats, the twelve as a figure of two columns and the bars as a figure.
    let script = "let w = dataset(wages)\nlet y = w.where(year == 2015)\nlet p = y.plot(x: educ, y: wage, color: female).points(alpha: 0.4).facet(by: union)\np\nlet q = y.plot(x: educ, y: wage).points(alpha: 0.3).facet(by: industry)\nq\nlet m = w.aggregate(wage = mean(wage), by: [year, union, female])\nlet r = m.plot(x: year, y: wage, fill: female).bars().facet(by: union).coord(flip).legend(top)\nr\nexport(p, \"panels.svg\")\nexport(p, \"panels.pdf\")\nexport(p, \"panels.png\")\nexport(q, \"twelve.svg\", size: two_column)\nexport(r, \"lying.svg\", size: two_column)\n";
    let parted = run_script(&fs, &project, "scripts/main.zeff", script);
    fills.push(text("plot/facet/stdout", &parted.output.stdout));
    fills.push(text("plot/facet/stderr", &parted.output.stderr));
    for name in [
        "panels.svg",
        "panels.pdf",
        "panels.png",
        "twelve.svg",
        "lying.svg",
    ] {
        let data = zeff_engine::fs::FileSystem::read(&fs, Path::new(&format!("/g/output/{name}")))
            .unwrap_or_default();
        fills.push(bytes(&format!("plot/facet/{name}"), &data));
    }
    let mut cases = vec![
        text("plot/lines/stdout", &lined.output.stdout),
        text("plot/lines/stderr", &lined.output.stderr),
        text("plot/lines/scene", &lines),
        text("plot/axes/scene", &scaled),
        text("plot/points/stdout", &outcome.output.stdout),
        text("plot/points/stderr", &outcome.output.stderr),
        text("plot/points/scene.json", &scene),
        text("plot/expression/stdout", &expression.output.stdout),
        text("plot/expression/stderr", &expression.output.stderr),
        text("plot/expression/scene.json", &logarithm),
        text("plot/colour/stdout", &colour.output.stdout),
        text("plot/colour/stderr", &colour.output.stderr),
        text("plot/colour/scene.json", &coloured),
        text("plot/size_shape/stdout", &size_shape.output.stdout),
        text("plot/size_shape/stderr", &size_shape.output.stderr),
        text("plot/size_shape/scene.json", &sized),
        text("plot/layout/drawings", &layout_drawings()),
        text("plot/svg/figures", &svg_figures()),
        bytes("plot/pdf/plain", &pdf_figures(false)),
        bytes("plot/pdf/figures", &pdf_figures(true)),
        bytes("plot/png/dots", &png_figures(false)),
        bytes("plot/png/figures", &png_figures(true)),
        bytes("plot/svg/picture", &picture_figure(false)),
        bytes("plot/pdf/picture", &picture_figure(true)),
        bytes("plot/lines/svg", &line_types(0)),
        bytes("plot/lines/pdf", &line_types(1)),
        bytes("plot/lines/png", &line_types(2)),
        text("plot/fonts/metrics", &font_metrics()),
        text("plot/fonts/texts", &font_texts()),
        text("plot/fonts/outlines", &font_outlines()),
    ];
    cases.extend(fills);
    cases
}

/// Two plots of the shipped wages, one plain and one with a colour, a size and a shape and so
/// with two legends, each laid out under the journal theme at the three named sizes and at a
/// size in millimetres (release 0.7.0, steps 21b and 21c).
fn sample_drawings() -> Vec<(String, Result<zeff_plot::drawing::Drawing, String>)> {
    use std::rc::Rc;
    use zeff_core::value::{Aesthetic, Frame, Mark, PlotLayer, PlotMapping};
    use zeff_plot::layout::{JOURNAL, Size, layout};
    let Some(table) = zeff_datasets::table("wages") else {
        return vec![(
            "wages".to_string(),
            Err("the wages did not load".to_string()),
        )];
    };
    let frame = Frame {
        table: Rc::new(table),
        name: Some("wages".to_string()),
    };
    let mapping = |aesthetic, variable: &str| PlotMapping {
        aesthetic,
        variable: variable.to_string(),
        computed: None,
    };
    let layer = PlotLayer {
        mark: Mark::Points,
        data: None,
        alpha: 0.4,
        size: None,
        jitter: 0.0,
        nudge: (0.0, 0.0),
        width: None,
        stream: None,
        computed: Vec::new(),
        mappings: Vec::new(),
        constants: Vec::new(),
    };
    let plain = zeff_plot::scene::start(
        frame.clone(),
        vec![mapping(Aesthetic::X, "educ"), mapping(Aesthetic::Y, "wage")],
        Vec::new(),
    );
    let full = zeff_plot::scene::start(
        frame,
        vec![
            mapping(Aesthetic::X, "exper"),
            mapping(Aesthetic::Y, "wage"),
            mapping(Aesthetic::Color, "industry"),
            mapping(Aesthetic::Shape, "industry"),
            mapping(Aesthetic::Size, "hours"),
        ],
        Vec::new(),
    );
    let mut out = Vec::new();
    for (name, plot) in [("plain", plain), ("full", full)] {
        let scene = zeff_plot::scene::with_layer(&plot, layer.clone()).1;
        for (page, size) in [
            ("one_column", Size::OneColumn),
            ("two_column", Size::TwoColumn),
            ("slide", Size::Slide),
            ("120 by 75 mm", Size::Millimetres(120.0, 75.0)),
        ] {
            out.push((
                format!("{name}, {page}"),
                layout(&scene, size, &JOURNAL).map_err(|why| format!("{why:?}")),
            ));
        }
    }
    out
}

/// The layout of a figure (step 21b): the sample drawings read as text, every place to a
/// thousandth of a point.
fn layout_drawings() -> String {
    let mut out = String::new();
    for (name, drawing) in sample_drawings() {
        out.push_str(&format!("=== {name}\n"));
        match drawing {
            Ok(drawing) => out.push_str(&drawing.describe()),
            Err(why) => out.push_str(&format!("{why}\n")),
        }
    }
    out
}

/// The SVG of a figure (step 21c): the bytes of each sample drawing, its text drawn as the
/// outlines of the embedded fonts.
fn svg_figures() -> String {
    let mut out = String::new();
    for (name, drawing) in sample_drawings() {
        out.push_str(&format!("=== {name}\n"));
        match drawing {
            Ok(drawing) => out.push_str(&zeff_plot::svg::write(&drawing)),
            Err(why) => out.push_str(&format!("{why}\n")),
        }
    }
    out
}

/// The PDF of a figure (step 22b1): the bytes of each sample drawing as a file of one page,
/// its text set in the embedded fonts, which it carries. With `packed` the streams of each
/// file are compressed, as `export` writes them; without, they are as they are, so that a
/// difference between two machines says whether it lies in what is drawn or in how it is
/// compressed.
fn pdf_figures(packed: bool) -> Vec<u8> {
    let mut out = Vec::new();
    for (name, drawing) in sample_drawings() {
        out.extend_from_slice(format!("=== {name}\n").as_bytes());
        match drawing {
            Ok(drawing) if packed => out.extend_from_slice(&zeff_plot::pdf::write(&drawing)),
            Ok(drawing) => out.extend_from_slice(&zeff_plot::pdf::write_plain(&drawing)),
            Err(why) => out.extend_from_slice(format!("{why}\n").as_bytes()),
        }
    }
    out
}

/// The PNG of a figure (step 22b2): the sample drawings of one column and of 120 mm by 75,
/// drawn in dots at 300 to the inch by the engine's own reckoning. With `written` each is
/// the PNG file `export` writes; without, the dots themselves, so that a difference between
/// two machines says whether it lies in what is drawn or in how it is compressed.
fn png_figures(written: bool) -> Vec<u8> {
    use zeff_plot::raster::{DOTS, render};
    let mut out = Vec::new();
    for (name, drawing) in sample_drawings() {
        if !(name.ends_with("one_column") || name.ends_with("mm")) {
            continue;
        }
        out.extend_from_slice(format!("=== {name}\n").as_bytes());
        match drawing.and_then(|d| render(&d, DOTS).map_err(|why| format!("{why:?}"))) {
            Ok(picture) if written => {
                out.extend_from_slice(&zeff_plot::png::write(&picture, DOTS).unwrap_or_default());
            }
            Ok(picture) => {
                out.extend_from_slice(
                    format!("{} by {}\n", picture.width, picture.height).as_bytes(),
                );
                out.extend_from_slice(&picture.rgb);
            }
            Err(why) => out.extend_from_slice(format!("{why}\n").as_bytes()),
        }
    }
    out
}

/// A layer drawn in dots inside a figure of lines (step 22b3b; open decision #118, F4): the
/// plain plot of the shipped wages on a page of one column, its one layer flagged, which no
/// script of this release can do, written as SVG or as PDF.
fn picture_figure(as_pdf: bool) -> Vec<u8> {
    use std::rc::Rc;
    use zeff_core::value::{Aesthetic, Frame, Mark, PlotLayer, PlotMapping};
    use zeff_plot::layout::{JOURNAL, Size, layout};
    let Some(table) = zeff_datasets::table("wages") else {
        return b"the wages did not load\n".to_vec();
    };
    let mapping = |aesthetic, variable: &str| PlotMapping {
        aesthetic,
        variable: variable.to_string(),
        computed: None,
    };
    let plot = zeff_plot::scene::start(
        Frame {
            table: Rc::new(table),
            name: Some("wages".to_string()),
        },
        vec![mapping(Aesthetic::X, "educ"), mapping(Aesthetic::Y, "wage")],
        Vec::new(),
    );
    let layer = PlotLayer {
        mark: Mark::Points,
        data: None,
        alpha: 0.4,
        size: None,
        jitter: 0.0,
        nudge: (0.0, 0.0),
        width: None,
        stream: None,
        computed: Vec::new(),
        mappings: Vec::new(),
        constants: Vec::new(),
    };
    let mut scene = zeff_plot::scene::with_layer(&plot, layer).1;
    for panel in &mut scene.panels {
        for layer in &mut panel.layers {
            layer.raster = true;
        }
    }
    let drawing = match layout(&scene, Size::OneColumn, &JOURNAL) {
        Ok(drawing) => drawing,
        Err(why) => return format!("{why:?}\n").into_bytes(),
    };
    if as_pdf {
        zeff_plot::pdf::write(&drawing)
    } else {
        zeff_plot::svg::write(&drawing).into_bytes()
    }
}

/// Lines with a type (step 23b; decision #118, F9): a line of each of the six types, a bent
/// line and two outlines that are not whole, written as SVG (0), as PDF (1) and as a PNG at
/// 300 dots to the inch (2), so that the targets prove they cut a line alike.
fn line_types(format: u8) -> Vec<u8> {
    use zeff_plot::drawing::{Drawing, Head, Item, LineType, Primitive, Step, Stroke};
    let line = |dash: LineType, width: f64, colour: &str| Stroke {
        colour: colour.to_string(),
        width,
        dash,
    };
    let unnamed = |primitive: Primitive| Item {
        name: None,
        primitive,
    };
    let mut items = vec![unnamed(Primitive::Rectangle {
        at: (0.0, 0.0),
        width: 120.0,
        height: 80.0,
        radius: 0.0,
        fill: Some("#FFFFFF".to_string()),
        stroke: None,
    })];
    for (i, dash) in LineType::ALL.into_iter().enumerate() {
        let y = 8.0 + 8.0 * i as f64;
        items.push(unnamed(Primitive::Segment {
            from: (10.0, y),
            to: (110.0, y + 0.37 * i as f64),
            stroke: line(dash, 0.5 + 0.25 * i as f64, "#222222"),
            heads: (Head::Plain, Head::Plain),
        }));
    }
    for (i, dash) in LineType::ALL.into_iter().enumerate() {
        let x = 10.0 + 17.0 * i as f64;
        items.push(unnamed(Primitive::Path {
            steps: vec![
                Step::Move((x, 74.0)),
                Step::Line((x + 6.5, 58.3)),
                Step::Curve((x + 9.0, 55.0), (x + 12.0, 70.0), (x + 14.0, 60.0)),
                Step::Close,
            ],
            stroke: Some(line(dash, 0.75, "#0072B2")),
            fill: None,
            opacity: if i % 2 == 0 { 1.0 } else { 0.5 },
        }));
    }
    let drawing = Drawing {
        width: 120.0,
        height: 80.0,
        items,
        notes: Vec::new(),
    };
    match format {
        0 => zeff_plot::svg::write(&drawing).into_bytes(),
        1 => zeff_plot::pdf::write(&drawing),
        _ => zeff_plot::raster::render(&drawing, zeff_plot::raster::DOTS)
            .ok()
            .and_then(|picture| zeff_plot::png::write(&picture, zeff_plot::raster::DOTS))
            .unwrap_or_default(),
    }
}

/// The fonts of a figure (release 0.7.0, step 21a): for each embedded face, its heights, the
/// glyph of every character it draws and the advance of every glyph.
fn font_metrics() -> String {
    use zeff_plot::font::{FaceName, face};
    let mut out = String::new();
    for name in FaceName::ALL {
        let font = face(name);
        out.push_str(&format!(
            "{} em {} ascender {} descender {} gap {} capitals {} x {} glyphs {}\n",
            name.file(),
            font.units_per_em(),
            font.ascender(),
            font.descender(),
            font.line_gap(),
            font.cap_height(),
            font.x_height(),
            font.glyphs()
        ));
        for character in font.characters() {
            let glyph = font.glyph(character).unwrap_or(0);
            out.push_str(&format!("{:04X} {glyph}\n", u32::from(character)));
        }
        for glyph in 0..font.glyphs() {
            out.push_str(&format!("{glyph} {}\n", font.advance(glyph)));
        }
    }
    out
}

/// Texts set in each face: what a figure writes, pairs the fonts kern, letters with marks
/// written joined and apart, a joined letter drawn by its parts, and characters no face has;
/// each as its glyphs, their places and its width at three sizes.
fn font_texts() -> String {
    use zeff_plot::font::{FaceName, face};
    let texts = [
        "Hourly wage",
        "Years of schooling",
        "log(wage)",
        "Coefficient and 95% confidence interval",
        "0 20 40 60 80 100",
        "\u{2212}0.25",
        "p < 0.001",
        "R\u{b2} = 0.47",
        "AVATAR To Ty P. F, r. y, 7. (j f)",
        "The quick brown fox jumps over the lazy dog",
        "\u{c9}cole, Stra\u{df}e, Malm\u{f6}, \u{141}\u{f3}d\u{17a}",
        "cafe\u{301} caf\u{e9}",
        "\u{1e7} g\u{30c}",
        "\u{3b1} \u{3b2} \u{3a3} \u{3c7}\u{b2}",
        "\u{4e2d}\u{6587} a\tb",
        "",
    ];
    let mut out = String::new();
    for name in FaceName::ALL {
        let font = face(name);
        for text in texts {
            let shaped = font.shape(text);
            out.push_str(&format!("{} {text:?} width {}", name.file(), shaped.width));
            for placed in &shaped.glyphs {
                out.push_str(&format!(" {}@{}", placed.glyph, placed.x));
            }
            out.push_str(&format!(" lacking {:?}", shaped.missing));
            for size in [8.0, 9.5, 24.0] {
                out.push_str(&format!(" {:016x}", font.width(text, size).to_bits()));
            }
            out.push('\n');
        }
        // Every pair of the letters, digits and signs of ASCII, kerned.
        let ascii: Vec<u16> = (33_u8..127)
            .filter_map(|code| font.glyph(char::from(code)))
            .collect();
        for &left in &ascii {
            for &right in &ascii {
                let kern = font.kern(left, right);
                if kern != 0 {
                    out.push_str(&format!("{left} {right} {kern}\n"));
                }
            }
        }
    }
    out
}

/// The outline of every glyph of every face, each coordinate by its bits.
fn font_outlines() -> String {
    use zeff_plot::font::{FaceName, Segment, face};
    let mut out = String::new();
    let bits = |x: f64, y: f64| format!("{:016x},{:016x}", x.to_bits(), y.to_bits());
    for name in FaceName::ALL {
        let font = face(name);
        for glyph in 0..font.glyphs() {
            out.push_str(&format!("{} {glyph}", name.file()));
            match font.outline(glyph) {
                Ok(steps) => {
                    for step in steps {
                        out.push_str(&match step {
                            Segment::Move(x, y) => format!(" M{}", bits(x, y)),
                            Segment::Line(x, y) => format!(" L{}", bits(x, y)),
                            Segment::Curve(c, p) => {
                                format!(" Q{} {}", bits(c.0, c.1), bits(p.0, p.1))
                            }
                            Segment::Close => " Z".to_string(),
                        });
                    }
                }
                Err(error) => out.push_str(&format!(" {error:?}")),
            }
            out.push('\n');
        }
    }
    out
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
    lines.extend(plot_cases());
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
