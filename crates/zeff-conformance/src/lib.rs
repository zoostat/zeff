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
    let script = "let wages = import(\"data/wages.csv\")\nwages.mean(wage)\nwages.mean(hours)\nprint(wages, [1, 2.5], 1e-7)\nwages.mean(id)\nlet built = frame(x = [1, 2.5, NA], flag = [true, false, true], name = [\"a\", \"b\", \"c\"])\nbuilt\nbuilt.mean(x)\nbuilt.mean(flag)\nwages.n_rows()\nwages.n_cols()\nwages.variables()\nbuilt.variables()\nwages.describe()\nbuilt.describe()\nwages.sd(wage)\nwages.sd(hours)\nwages.sd(id)\nbuilt.sd(x)\nbuilt.sd(flag)\nwages.summary()\nbuilt.summary([x, flag], stats: [n, mean, max])\nprint(wages.summary([wage]), NA, -0.0, 1e15, [true, NA])\n";
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

fn all_cases() -> Vec<Line> {
    let mut lines = reducer_cases();
    lines.extend(canary_cases());
    lines.extend(milestone_cases());
    lines
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
