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
    let mut digest = Digest::new();
    for chunk in value.as_bytes().chunks(8) {
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
fn milestone_cases() -> Vec<Line> {
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
    let script = "let wages = import(\"data/wages.csv\")\nwages.mean(wage)\nwages.mean(hours)\nprint(wages, [1, 2.5], 1e-7)\nwages.mean(id)\nlet built = frame(x = [1, 2.5, NA], flag = [true, false, true], name = [\"a\", \"b\", \"c\"])\nbuilt\nbuilt.mean(x)\nbuilt.mean(flag)\nwages.n_rows()\nwages.n_cols()\nwages.variables()\nbuilt.variables()\nwages.describe()\nbuilt.describe()\nwages.sd(wage)\nwages.sd(hours)\nwages.sd(id)\nbuilt.sd(x)\nbuilt.sd(flag)\nwages.summary()\nbuilt.summary([x, flag], stats: [n, mean, max])\nprint(wages.summary([wage]), NA, -0.0, 1e15, [true, NA])\nlet a = 7\nlet b = 2.5\na / 3\na ^ 0.5\nb ^ a\n(a + b) * (a - b)\n1e300 * 1e10\n0.1 + 0.2 == 0.3\n2..6\n[1 => \"one\", 2 => \"two\"][2]\n{x: 1.5, y: [1, 2]}.y[2]\n-a\n!(a > b)\nwages[\"wage\"] * 2\nwages[\"hours\"] / wages[\"id\"]\nwages[\"wage\"] > 25 && wages[\"id\"] != 3\nframe(ratio = wages[\"wage\"] / 10, big = wages[\"wage\"] >= 27.75).describe()\n";
    let outcome = match Project::open(&fs, Path::new("/p")) {
        Ok(project) => run_script(&fs, &project, "scripts/main.zeff", script),
        Err(_) => return vec![text("milestone/project", "could not open")],
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

/// Every sample project, oldest first; each adds three cases to the report.
pub const SAMPLES: [&Sample; 2] = [&FIRST_ENGINE, &DATA_CLEANING];

/// Run a sample project on an in-memory copy of its files.
pub fn sample_run(sample: &Sample) -> zeff_engine::run::Outcome {
    use std::path::Path;
    use zeff_engine::fs::MemoryFileSystem;
    use zeff_engine::interpreter::Output;
    use zeff_engine::project::Project;
    use zeff_engine::run::{Outcome, run_script};

    let fs = MemoryFileSystem::new();
    fs.add_file("/s/zeff.toml", sample.manifest);
    for (file, bytes) in sample.data {
        fs.add_file(format!("/s/data/{file}"), *bytes);
    }
    fs.add_file("/s/scripts/main.zeff", sample.script);
    match Project::open(&fs, Path::new("/s")) {
        Ok(project) => run_script(&fs, &project, "scripts/main.zeff", sample.script),
        Err(_) => Outcome {
            output: Output::default(),
            exit_code: 4,
        },
    }
}

fn sample_cases() -> Vec<Line> {
    SAMPLES
        .iter()
        .flat_map(|sample| {
            let outcome = sample_run(sample);
            // The first sample keeps the case names of release 0.1.0.
            let prefix = if sample.name == "first_engine" {
                "sample".to_string()
            } else {
                sample.name.to_string()
            };
            vec![
                text(&format!("{prefix}/stdout"), &outcome.output.stdout),
                text(&format!("{prefix}/stderr"), &outcome.output.stderr),
                scalar(
                    format!("{prefix}/exit_code"),
                    Some(f64::from(outcome.exit_code)),
                ),
            ]
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
    let families: [(&'static str, &[(&'static str, f64)]); 15] = [
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
        draws.extend((0..8).map(|_| families::draw(&d, stream.uniform())));
    }
    vec![series("dist/values", &values), series("dist/draws", &draws)]
}

fn all_cases() -> Vec<Line> {
    let mut lines = reducer_cases();
    lines.extend(canary_cases());
    lines.extend(maths_cases());
    lines.extend(special_cases());
    lines.extend(random_cases());
    lines.extend(distribution_cases());
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
