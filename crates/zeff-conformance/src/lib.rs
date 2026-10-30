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
    script: include_str!("../../../samples/data_cleaning/scripts/main.zeff"),
    stdout: include_str!("../../../samples/data_cleaning/expected/stdout.txt"),
    stderr: include_str!("../../../samples/data_cleaning/expected/stderr.txt"),
};

/// The sample of release 0.3.0: a simulation and its p-values end to end, with no data file.
pub const SIMULATION: Sample = Sample {
    name: "simulation",
    manifest: include_str!("../../../samples/simulation/zeff.toml"),
    data: &[],
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
    script: include_str!("../../../samples/regression/scripts/main.zeff"),
    stdout: include_str!("../../../samples/regression/expected/stdout.txt"),
    stderr: include_str!("../../../samples/regression/expected/stderr.txt"),
};

/// Every sample project, oldest first; each adds three cases to the report, and one more for
/// every file it writes under `output/`.
pub const SAMPLES: [&Sample; 4] = [&FIRST_ENGINE, &DATA_CLEANING, &SIMULATION, &REGRESSION];

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
    lines
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
