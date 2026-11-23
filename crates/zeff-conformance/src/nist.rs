//! The Statistical Reference Datasets of the National Institute of Standards and Technology
//! (release 0.9.0, step 36; open decision #153): problems whose answers NIST certifies to
//! fifteen significant digits, computed independently of any statistical package. They are
//! the one check of the suite that owes nothing to a reference program.
//!
//! Each problem is read from NIST's own file as it is served (`tests/reference/nist`, with
//! its source note), its data are written as a CSV file into a project in memory, and a Zeff
//! script imports that file and computes every certified value through the whole engine: the
//! reader of numbers, the function, and the figures written in full by `to(x, Str)`. What is
//! recorded for each certified value is its log relative error after McCullough (1998),
//! `-log10(|q - c| / |c|)`: the number of leading digits of the certified value `c` that the
//! computed value `q` has right. It runs from 0 to 15, the digits NIST certifies; where `c`
//! is zero the absolute error stands in the place of the relative one; a value the engine did
//! not give (a term dropped as collinear) has 0.
//!
//! Two conformance lines stand for a problem: the digest of what its script printed, and the
//! digest of its table of log relative errors to one decimal. Every target must give both,
//! so the accuracy recorded here is the accuracy on every platform.

/// The kind of problem, as NIST files them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Linear least-squares regression (`lls`).
    Regression,
}

impl Kind {
    fn folder(self) -> &'static str {
        match self {
            Kind::Regression => "lls",
        }
    }
}

/// One problem: its name, its kind and NIST's file.
#[derive(Debug, Clone, Copy)]
pub struct Problem {
    pub kind: Kind,
    pub name: &'static str,
    pub file: &'static str,
}

macro_rules! problem {
    ($kind:expr, $folder:literal, $name:literal) => {
        Problem {
            kind: $kind,
            name: $name,
            file: include_str!(concat!(
                "../../../tests/reference/nist/",
                $folder,
                "/",
                $name,
                ".dat"
            )),
        }
    };
}

/// The eleven linear-regression problems, in NIST's order of difficulty: lower (Norris,
/// Pontius), average (NoInt1, NoInt2), higher (Filip, Longley, Wampler1 to Wampler5).
pub const REGRESSION: [Problem; 11] = [
    problem!(Kind::Regression, "lls", "Norris"),
    problem!(Kind::Regression, "lls", "Pontius"),
    problem!(Kind::Regression, "lls", "NoInt1"),
    problem!(Kind::Regression, "lls", "NoInt2"),
    problem!(Kind::Regression, "lls", "Filip"),
    problem!(Kind::Regression, "lls", "Longley"),
    problem!(Kind::Regression, "lls", "Wampler1"),
    problem!(Kind::Regression, "lls", "Wampler2"),
    problem!(Kind::Regression, "lls", "Wampler3"),
    problem!(Kind::Regression, "lls", "Wampler4"),
    problem!(Kind::Regression, "lls", "Wampler5"),
];

/// The digits NIST certifies, and so the most a log relative error can say.
pub const CERTIFIED_DIGITS: f64 = 15.0;

/// The log relative error of `computed` against `certified`: the number of leading digits
/// that agree, from 0 to 15. Equal values have 15. Against a certified zero the absolute
/// error is used, as McCullough does. A value that is missing or not a number has 0.
///
/// A certified infinity has no relative error and gives `None`: the F of an exact fit
/// (Wampler1 and Wampler2) is infinite only where the residuals are exactly zero, which no
/// arithmetic in sixteen digits gives; the residual sum of squares, certified as zero beside
/// it, is the value that says how near the fit came.
pub fn log_relative_error(computed: f64, certified: f64) -> Option<f64> {
    if certified.is_infinite() {
        return None;
    }
    Some(digits_agreeing(computed, certified))
}

fn digits_agreeing(computed: f64, certified: f64) -> f64 {
    if computed.is_nan() || certified.is_nan() {
        return 0.0;
    }
    if computed == certified {
        return CERTIFIED_DIGITS;
    }
    if computed.is_infinite() {
        return 0.0;
    }
    let error = if certified == 0.0 {
        computed.abs()
    } else {
        ((computed - certified) / certified).abs()
    };
    (-zeff_numeric::maths::log10(error)).clamp(0.0, CERTIFIED_DIGITS)
}

/// The two line numbers a file gives for one of its parts, `Data (lines 61 to 76)` or
/// `Data            : lines 61 to 260`: the first line of the header that names the part and
/// the word `lines`.
fn lines_of(file: &str, part: &str) -> Option<(usize, usize)> {
    file.lines().take(12).find_map(|line| {
        let at = line.find(part)?;
        let rest = &line[at + part.len()..];
        let after = &rest[rest.find("lines")? + "lines".len()..];
        let mut numbers = after
            .split(|c: char| !c.is_ascii_digit())
            .filter(|token| !token.is_empty())
            .filter_map(|token| token.parse::<usize>().ok());
        Some((numbers.next()?, numbers.next()?))
    })
}

/// The lines of a part of the file, by the numbers the file itself gives (counted from one).
fn part<'a>(file: &'a str, name: &str) -> Vec<&'a str> {
    let Some((first, last)) = lines_of(file, name) else {
        return Vec::new();
    };
    file.lines()
        .skip(first.saturating_sub(1))
        .take(last + 1 - first)
        .collect()
}

/// A number as NIST writes it: `0.358191792925910E-01`, `1.60080000000000E+02`, `Infinity`.
fn number(token: &str) -> Option<f64> {
    if token == "Infinity" {
        return Some(f64::INFINITY);
    }
    let first = token.chars().next()?;
    if !(first.is_ascii_digit() || first == '-' || first == '+' || first == '.') {
        return None;
    }
    token.parse().ok()
}

/// The rows of data of a file, each its numbers as written.
pub fn data(problem: &Problem) -> Vec<Vec<&'static str>> {
    part(problem.file, "Data")
        .into_iter()
        .map(|line| line.split_whitespace().collect::<Vec<_>>())
        .filter(|row| !row.is_empty())
        .collect()
}

/// A certified value: what it is, and the value.
#[derive(Debug, Clone, PartialEq)]
pub struct Certified {
    pub what: String,
    pub value: f64,
}

/// The certified values of a regression problem, in the order the script prints them: each
/// parameter's estimate and its standard deviation; the residual standard deviation and R²;
/// the sums of squares, the mean squares and the F of the table of variance.
fn certified_regression(problem: &Problem) -> Vec<Certified> {
    let mut parameters = Vec::new();
    let (mut sd, mut r_squared) = (None, None);
    let (mut regression, mut residual) = (Vec::new(), Vec::new());
    for line in part(problem.file, "Certified Values") {
        let tokens: Vec<&str> = line.split_whitespace().collect();
        let numbers: Vec<f64> = tokens.iter().skip(1).filter_map(|t| number(t)).collect();
        match tokens.first().copied() {
            Some(name)
                if name.starts_with('B')
                    && name[1..].chars().all(|c| c.is_ascii_digit())
                    && name.len() > 1
                    && numbers.len() == 2 =>
            {
                parameters.push((name.to_string(), numbers[0], numbers[1]));
            }
            Some("Standard") if tokens.get(1) == Some(&"Deviation") && tokens.len() == 3 => {
                sd = number(tokens[2]);
            }
            Some("R-Squared") => r_squared = numbers.first().copied(),
            Some("Regression") if numbers.len() == 4 => regression = numbers[1..].to_vec(),
            Some("Residual") if numbers.len() == 3 => residual = numbers[1..].to_vec(),
            _ => {}
        }
    }
    let mut values = Vec::new();
    for (name, estimate, deviation) in parameters {
        values.push(Certified {
            what: format!("{name} estimate"),
            value: estimate,
        });
        values.push(Certified {
            what: format!("{name} standard deviation"),
            value: deviation,
        });
    }
    let mut one = |what: &str, value: Option<f64>| {
        values.push(Certified {
            what: what.to_string(),
            value: value.unwrap_or(f64::NAN),
        });
    };
    one("residual standard deviation", sd);
    one("R-squared", r_squared);
    one("regression sum of squares", regression.first().copied());
    one("regression mean square", regression.get(1).copied());
    one("F statistic", regression.get(2).copied());
    one("residual sum of squares", residual.first().copied());
    one("residual mean square", residual.get(1).copied());
    values
}

/// The certified values of a problem, in the order its script prints the computed ones.
pub fn certified(problem: &Problem) -> Vec<Certified> {
    match problem.kind {
        Kind::Regression => certified_regression(problem),
    }
}

/// The model of a regression problem as NIST states it: the terms beside the intercept, and
/// whether there is an intercept. One regressor is `x`; Longley's six are `x1` to `x6`.
fn model(name: &str) -> (Vec<String>, bool) {
    let powers = |degree: usize| {
        (1..=degree)
            .map(|k| {
                if k == 1 {
                    "x".to_string()
                } else {
                    format!("x^{k}")
                }
            })
            .collect::<Vec<_>>()
    };
    match name {
        "Norris" => (powers(1), true),
        "Pontius" => (powers(2), true),
        "NoInt1" | "NoInt2" => (powers(1), false),
        "Filip" => (powers(10), true),
        "Longley" => ((1..=6).map(|i| format!("x{i}")).collect(), true),
        // Wampler1 to Wampler5: a polynomial of degree five.
        _ => (powers(5), true),
    }
}

/// The CSV file of a problem's data, with the names its script uses.
pub fn csv(problem: &Problem) -> String {
    let rows = data(problem);
    let width = rows.first().map_or(0, Vec::len);
    let names: Vec<String> = match problem.kind {
        Kind::Regression if width == 2 => vec!["y".to_string(), "x".to_string()],
        Kind::Regression => std::iter::once("y".to_string())
            .chain((1..width).map(|i| format!("x{i}")))
            .collect(),
    };
    let mut text = names.join(",");
    text.push('\n');
    for row in rows {
        text.push_str(&row.join(","));
        text.push('\n');
    }
    text
}

/// The script of a regression problem. It prints one figure to a line, in the order of
/// `certified`: for each parameter the estimate and its standard error, read from the fit;
/// R² from the fit; and the rest of NIST's table from the fit's residuals and fitted values
/// by the engine's own sums, since `reg` prints no table of variance: the residual sum of
/// squares and what follows from it, and the regression sum of squares about the mean (about
/// zero where the model has no intercept, as NIST defines it there).
fn regression_script(problem: &Problem) -> String {
    let (terms, intercept) = model(problem.name);
    let rows = data(problem).len();
    let parameters = terms.len() + usize::from(intercept);
    let df_regression = if intercept {
        parameters - 1
    } else {
        parameters
    };
    let df_residual = rows - parameters;
    let formula = if intercept {
        format!("y ~ {}", terms.join(" + "))
    } else {
        format!("y ~ {} - 1", terms.join(" + "))
    };
    let mut script = format!(
        "let d = import(\"data/{}.csv\")\nlet m = d.reg({formula})\n",
        problem.name
    );
    let mut named: Vec<String> = Vec::new();
    if intercept {
        named.push("(intercept)".to_string());
    }
    named.extend(terms);
    for term in &named {
        script.push_str(&format!(
            "print(to(m.coef(`{term}`), Str))\nprint(to(m.std_errors(`{term}`), Str))\n"
        ));
    }
    let about = if intercept { "(f - mean(y))^2" } else { "f^2" };
    script.push_str(&format!(
        "let s = d.add(e = m.residuals(), f = m.fitted()).add(e2 = e^2, f2 = {about})\n\
         let rss = s.sum(e2)\n\
         let ssr = s.sum(f2)\n\
         print(to(sqrt(rss / {df_residual}), Str))\n\
         print(to(m.fit_measures().r_squared, Str))\n\
         print(to(ssr, Str))\n\
         print(to(ssr / {df_regression}, Str))\n\
         print(to((ssr / {df_regression}) / (rss / {df_residual}), Str))\n\
         print(to(rss, Str))\n\
         print(to(rss / {df_residual}, Str))\n"
    ));
    script
}

/// The script that computes a problem's certified values, one to a line.
pub fn script(problem: &Problem) -> String {
    match problem.kind {
        Kind::Regression => regression_script(problem),
    }
}

/// A figure as the script printed it: a number, `Inf`, or `NA` for a value the engine did
/// not give.
fn printed(line: &str) -> f64 {
    match line.trim() {
        "Inf" => f64::INFINITY,
        "-Inf" => f64::NEG_INFINITY,
        text => text.parse().unwrap_or(f64::NAN),
    }
}

/// What a problem gave: what its script printed, what it said, and for each certified value
/// the computed one with its log relative error.
#[derive(Debug, Clone)]
pub struct Solved {
    pub problem: Problem,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub values: Vec<(Certified, f64, Option<f64>)>,
}

/// A log relative error to one decimal, or `n/a` where it is not defined.
fn shown(error: Option<f64>) -> String {
    error.map_or("n/a".to_string(), |e| format!("{e:.1}"))
}

impl Solved {
    /// The table of the problem: one line for each certified value, with its log relative
    /// error to one decimal. This is the text whose digest is the problem's conformance line.
    pub fn table(&self) -> String {
        let mut text = String::new();
        for (certified, _, error) in &self.values {
            text.push_str(&format!("{:<32}{:>5}\n", certified.what, shown(*error)));
        }
        if self.values.is_empty() {
            text.push_str("the script did not run\n");
        }
        text
    }

    /// The smallest log relative error of the certified values whose name ends in `suffix`
    /// (`estimate`, `standard deviation`), or of them all for an empty one.
    pub fn least(&self, suffix: &str) -> f64 {
        self.values
            .iter()
            .filter(|(c, _, _)| c.what.ends_with(suffix))
            .filter_map(|(_, _, e)| *e)
            .fold(CERTIFIED_DIGITS, f64::min)
    }
}

/// Run one problem through the engine.
pub fn solve(problem: &Problem) -> Solved {
    use std::path::Path;
    use zeff_engine::fs::MemoryFileSystem;
    use zeff_engine::project::Project;
    let fs = MemoryFileSystem::new();
    fs.add_file(
        "/nist/zeff.toml",
        format!(
            "[project]\nname = \"nist\"\nzeff = \"{}\"\nedition = \"preview\"\n",
            zeff_engine::VERSION
        ),
    );
    fs.add_file(
        format!("/nist/data/{}.csv", problem.name).as_str(),
        csv(problem),
    );
    let failed = |stderr: &str| Solved {
        problem: *problem,
        stdout: String::new(),
        stderr: stderr.to_string(),
        exit_code: 2,
        values: Vec::new(),
    };
    let Ok(project) = Project::open(&fs, Path::new("/nist")) else {
        return failed("the project did not open");
    };
    let outcome = crate::run_script(&fs, &project, "scripts/main.zeff", &script(problem));
    let certified = certified(problem);
    let computed: Vec<f64> = outcome.output.stdout.lines().map(printed).collect();
    let values = if outcome.exit_code == 0 && computed.len() == certified.len() {
        certified
            .into_iter()
            .zip(computed)
            .map(|(c, q)| {
                let error = log_relative_error(q, c.value);
                (c, q, error)
            })
            .collect()
    } else {
        Vec::new()
    };
    Solved {
        problem: *problem,
        stdout: outcome.output.stdout,
        stderr: outcome.output.stderr,
        exit_code: outcome.exit_code,
        values,
    }
}

/// Every problem, solved, in NIST's order.
pub fn solved() -> Vec<Solved> {
    REGRESSION.iter().map(solve).collect()
}

/// The conformance lines of the problems: for each, the digest of what its script printed
/// and the digest of its table of log relative errors.
pub(crate) fn cases() -> Vec<crate::Line> {
    let mut lines = Vec::new();
    for solved in solved() {
        let name = format!(
            "nist/{}/{}",
            solved.problem.kind.folder(),
            solved.problem.name
        );
        lines.push(crate::text(&format!("{name}/stdout"), &solved.stdout));
        lines.push(crate::text(&format!("{name}/lre"), &solved.table()));
    }
    lines
}

/// The whole record as one text, for the stored table: each problem with what the engine
/// said of it, and each certified value with the computed one and its log relative error.
pub fn record() -> String {
    let mut text = String::new();
    for solved in solved() {
        text.push_str(&format!(
            "## {} · {}\n",
            solved.problem.kind.folder(),
            solved.problem.name
        ));
        for line in solved.stderr.lines() {
            // The messages of the run, without the note on the import, which every problem has.
            if line.starts_with("note: data/") {
                continue;
            }
            text.push_str(&format!("  | {line}\n"));
        }
        if solved.values.is_empty() {
            text.push_str(&format!(
                "  the script did not run (exit code {})\n",
                solved.exit_code
            ));
        }
        text.push_str(&format!(
            "  {:<32}{:>26}{:>26}{:>6}\n",
            "certified value", "NIST", "Zeff", "LRE"
        ));
        for (certified, computed, error) in &solved.values {
            text.push_str(&format!(
                "  {:<32}{:>26}{:>26}{:>6}\n",
                certified.what,
                format!("{:e}", certified.value),
                if computed.is_nan() {
                    "not given".to_string()
                } else {
                    format!("{computed:e}")
                },
                shown(*error),
            ));
        }
        if !solved.values.is_empty() {
            text.push_str(&format!(
                "  least: estimates {:.1}, standard deviations {:.1}, all {:.1}\n",
                solved.least(" estimate"),
                solved.least(" standard deviation"),
                solved.least(""),
            ));
        }
        text.push('\n');
    }
    text
}
