//! `zeff audit` (Language Reference, section 11.1, "Known-issues list and `zeff audit`"; open
//! decision #206; release 0.9.0, step 22c): the recorded runs of a project read against the
//! list of known wrong results, and each run that an issue affects said in the form the
//! reference gives: "your run of 12 October used `reg` with `weights:`, affected by issue
//! 123, fixed in 0.9.2".
//!
//! A run is affected by an issue when the engine that made it is one the issue affects and
//! the run's script calls one of the issue's functions, with one of its options written when
//! the issue names options. The audit says so and no more: whether the analysis is in fact
//! wrong depends on what the issue says of the data, which the sentence under each finding
//! repeats.

use crate::known_issues::Issue;
use crate::runs::RunRecord;

/// Runs of one script on one engine that one issue affects through one call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Affected {
    pub issue: u64,
    pub summary: String,
    pub affected_when: String,
    /// The version that corrects the issue, when one does.
    pub fixed: Option<String>,
    pub script: String,
    pub engine: String,
    pub function: String,
    /// The options of the issue that the call writes; empty when the issue is about every call.
    pub options: Vec<String>,
    /// How many runs, and the days of the first and the last that have one.
    pub runs: usize,
    pub first: Option<String>,
    pub last: Option<String>,
}

/// A known wrong result of the engine a project pins.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OfThePin {
    pub issue: u64,
    pub functions: Vec<String>,
    pub options: Vec<String>,
    pub fixed: Option<String>,
}

/// What an audit finds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    /// The runs read and the issues they were read against.
    pub runs: usize,
    pub issues: usize,
    pub affected: Vec<Affected>,
    /// The engine the project pins and its known wrong results.
    pub pinned: String,
    pub of_the_pin: Vec<OfThePin>,
}

/// Read `records`, in the order of the runs, against `issues`.
pub fn audit(records: &[RunRecord], issues: &[Issue], pinned: &str) -> Report {
    let mut affected: Vec<Affected> = Vec::new();
    for record in records {
        for issue in issues
            .iter()
            .filter(|issue| issue.affects_version(&record.engine))
        {
            let Some(call) = record
                .calls
                .iter()
                .find(|call| issue.affects_call(&call.function, &call.options))
            else {
                continue;
            };
            let options: Vec<String> = issue
                .options
                .iter()
                .filter(|option| call.options.contains(option))
                .cloned()
                .collect();
            let known = affected.iter_mut().find(|found| {
                found.issue == issue.number
                    && found.script == record.script
                    && found.engine == record.engine
                    && found.function == call.function
                    && found.options == options
            });
            match known {
                Some(found) => {
                    found.runs += 1;
                    if record.on.is_some() {
                        found.first = found.first.take().or(record.on.clone());
                        found.last = record.on.clone();
                    }
                }
                None => affected.push(Affected {
                    issue: issue.number,
                    summary: issue.summary.clone(),
                    affected_when: issue.affected_when.clone(),
                    fixed: issue.fixed.clone(),
                    script: record.script.clone(),
                    engine: record.engine.clone(),
                    function: call.function.clone(),
                    options,
                    runs: 1,
                    first: record.on.clone(),
                    last: record.on.clone(),
                }),
            }
        }
    }
    Report {
        runs: records.len(),
        issues: issues.len(),
        affected,
        pinned: pinned.to_string(),
        of_the_pin: issues
            .iter()
            .filter(|issue| issue.affects_version(pinned))
            .map(|issue| OfThePin {
                issue: issue.number,
                functions: issue.functions.clone(),
                options: issue.options.clone(),
                fixed: issue.fixed.clone(),
            })
            .collect(),
    }
}

const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

/// A day as it is said, `12 October`, with its year when that is not `this_year`.
pub fn day_said(day: &str, this_year: Option<&str>) -> String {
    let mut parts = day.split('-');
    let (Some(year), Some(month), Some(date)) = (parts.next(), parts.next(), parts.next()) else {
        return day.to_string();
    };
    let (Ok(month), Ok(date)) = (month.parse::<usize>(), date.parse::<u32>()) else {
        return day.to_string();
    };
    let Some(month) = month.checked_sub(1).and_then(|at| MONTHS.get(at)) else {
        return day.to_string();
    };
    if this_year == Some(year) {
        format!("{date} {month}")
    } else {
        format!("{date} {month} {year}")
    }
}

/// `reg` with `weights:`, or `mad` alone.
pub fn call_said(function: &str, options: &[String]) -> String {
    if options.is_empty() {
        return format!("`{function}`");
    }
    let options: Vec<String> = options
        .iter()
        .map(|option| format!("`{option}:`"))
        .collect();
    let listed = match options.as_slice() {
        [one] => one.clone(),
        [most @ .., last] => format!("{} and {last}", most.join(", ")),
        [] => String::new(),
    };
    format!("`{function}` with {listed}")
}

/// How an issue ends a sentence: `fixed in 0.9.2`, or that nothing corrects it yet.
fn fixed_said(fixed: Option<&str>) -> String {
    match fixed {
        Some(version) => format!("fixed in {version}"),
        None => "not yet corrected".to_string(),
    }
}

impl Affected {
    /// The sentence of section 11.1: "your run of 12 October used `reg` with `weights:`,
    /// affected by issue 123, fixed in 0.9.2".
    pub fn sentence(&self, this_year: Option<&str>) -> String {
        let when = match (&self.last, &self.first, self.runs) {
            (Some(last), _, 1) => format!("your run of {}", day_said(last, this_year)),
            (Some(last), Some(first), runs) => format!(
                "your run of {}, and {} before it since {},",
                day_said(last, this_year),
                if runs == 2 {
                    "1 run".to_string()
                } else {
                    format!("{} runs", runs - 1)
                },
                day_said(first, this_year)
            ),
            (_, _, 1) => "a run with no day recorded".to_string(),
            (_, _, runs) => format!("{runs} runs with no day recorded"),
        };
        format!(
            "{when} used {}, affected by issue {}, {}",
            call_said(&self.function, &self.options),
            self.issue,
            fixed_said(self.fixed.as_deref())
        )
    }
}

impl Report {
    /// The report as it is printed.
    pub fn text(&self, engine: &str, this_year: Option<&str>) -> String {
        let mut text = String::new();
        if self.runs == 0 {
            text.push_str(
                "this project has no recorded run: runs are recorded from Zeff 0.9.0, by `zeff run`\n",
            );
        }
        for found in &self.affected {
            text.push_str(&format!(
                "{}\n  {} on Zeff {}: {}\n  an analysis is affected when {}\n\n",
                found.sentence(this_year),
                found.script,
                found.engine,
                found.summary,
                found.affected_when
            ));
        }
        if self.runs > 0 && self.affected.is_empty() {
            text.push_str("no recorded run of this project is affected by a known issue\n");
        }
        match self.of_the_pin.as_slice() {
            [] => text.push_str(&format!(
                "Zeff {}, which this project pins, has no known wrong result\n",
                self.pinned
            )),
            issues => {
                let listed: Vec<String> = issues
                    .iter()
                    .map(|issue| {
                        // Any one of an issue's options makes a call one of its calls.
                        let functions: Vec<String> = issue
                            .functions
                            .iter()
                            .map(|function| {
                                call_said(function, &issue.options).replace(" and ", " or ")
                            })
                            .collect();
                        format!(
                            "issue {} ({}), {}",
                            issue.issue,
                            functions.join(", "),
                            fixed_said(issue.fixed.as_deref())
                        )
                    })
                    .collect();
                text.push_str(&format!(
                    "Zeff {}, which this project pins, has {}: {}\n",
                    self.pinned,
                    if issues.len() == 1 {
                        "1 known wrong result".to_string()
                    } else {
                        format!("{} known wrong results", issues.len())
                    },
                    listed.join("; ")
                ));
                // The engine to move to: the newest that corrects any of them.
                let newest = issues
                    .iter()
                    .filter_map(|issue| issue.fixed.as_deref())
                    .max_by_key(|version| crate::known_issues::ordered(version));
                if let Some(newest) = newest {
                    text.push_str(&format!(
                        "  the pin does not move by itself: `zeff engine use {newest}` runs the project on both engines, says what differs, and moves it\n"
                    ));
                }
            }
        }
        text.push_str(&format!(
            "{} read against the {} known to Zeff {engine}\n",
            match self.runs {
                1 => "1 run".to_string(),
                runs => format!("{runs} runs"),
            },
            match self.issues {
                1 => "1 issue".to_string(),
                issues => format!("{issues} issues"),
            }
        ));
        text
    }

    /// The report as one JSON record.
    pub fn json(&self, engine: &str) -> String {
        use zeff_diagnostics::json_string;
        let text_or_null = |text: Option<&str>| text.map_or("null".to_string(), json_string);
        let names = |names: &[String]| {
            format!(
                "[{}]",
                names
                    .iter()
                    .map(|name| json_string(name))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };
        let affected: Vec<String> = self
            .affected
            .iter()
            .map(|found| {
                format!(
                    "{{\"issue\": {}, \"fixed\": {}, \"script\": {}, \"engine\": {}, \"function\": {}, \"options\": {}, \"runs\": {}, \"first\": {}, \"last\": {}}}",
                    found.issue,
                    text_or_null(found.fixed.as_deref()),
                    json_string(&found.script),
                    json_string(&found.engine),
                    json_string(&found.function),
                    names(&found.options),
                    found.runs,
                    text_or_null(found.first.as_deref()),
                    text_or_null(found.last.as_deref())
                )
            })
            .collect();
        let of_the_pin: Vec<String> = self
            .of_the_pin
            .iter()
            .map(|issue| {
                format!(
                    "{{\"issue\": {}, \"functions\": {}, \"options\": {}, \"fixed\": {}}}",
                    issue.issue,
                    names(&issue.functions),
                    names(&issue.options),
                    text_or_null(issue.fixed.as_deref())
                )
            })
            .collect();
        format!(
            "{{\"engine\": {}, \"runs\": {}, \"issues\": {}, \"affected\": [{}], \"pinned\": {}, \"pinned_issues\": [{}]}}",
            json_string(engine),
            self.runs,
            self.issues,
            affected.join(", "),
            json_string(&self.pinned),
            of_the_pin.join(", ")
        )
    }
}
