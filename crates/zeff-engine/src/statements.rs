//! Each statement of a run, for an interface that says something beside every statement (open
//! decision #175, C5; release 0.9.0, step 26a): the name it binds, its result's type and
//! dimensions, what it printed and said, and what it depends on: the statements it reads, the
//! project functions it calls, and whether it sets the seed, draws from the run's stream,
//! reads a file or writes one. The record is the same from `zeff run --format json` and in
//! the outcome of a run of the engine's worker.
//!
//! What a statement depends on is read from the checked script and is there for every
//! statement, run or not; what its result is and whether it drew are of the run, and are
//! there for the statements that ran.

use crate::dependencies::graph_in;
use crate::interpreter::{Output, ResultFacts};
use zeff_diagnostics::json_string;
use zeff_ir::Program;
use zeff_registry::Registry;

/// One statement of the script's top level.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatementFacts {
    /// Its place among the statements, from 1, as the events of a run number them.
    pub index: usize,
    /// The lines it begins and ends on.
    pub line: u32,
    pub end_line: u32,
    /// Whether the run reached it: a run that stops leaves the statements after it not run.
    pub ran: bool,
    /// The name a `let` binds, run or not.
    pub name: Option<String>,
    /// What its result is, where it ran and gave one.
    pub result: Option<ResultFacts>,
    /// What it printed, and its warnings, notes and error in their short form.
    pub stdout: String,
    pub stderr: String,
    /// The earlier statements whose bindings it reads, by their places from 1.
    pub reads: Vec<usize>,
    /// The functions of the project it calls or names.
    pub calls: Vec<String>,
    /// From the checked script: it sets the seed; it draws from the run's stream; it reads a
    /// file; it writes one.
    pub seeds: bool,
    pub draws: bool,
    pub reads_file: bool,
    pub writes_file: bool,
    /// Whether it drew from the run's stream in this run.
    pub drew: bool,
}

/// The line, from 1, that the byte at `offset` of `text` is on.
fn line_of(text: &str, offset: usize) -> u32 {
    let upto = offset.min(text.len());
    1 + text.as_bytes()[..upto]
        .iter()
        .filter(|byte| **byte == b'\n')
        .count() as u32
}

/// The statements of `program`, the checked form of `text`, with what the run recorded of
/// each in `output`: the first statement of the script is the first the run recorded.
pub fn statement_facts(
    program: &Program,
    script: &str,
    text: &str,
    registry: &Registry,
    output: &Output,
) -> Vec<StatementFacts> {
    let graph = graph_in(program, script, Some(registry));
    graph
        .statements
        .iter()
        .enumerate()
        .map(|(at, node)| {
            let ran = output.statements.get(at);
            // A statement ends before the line break that follows it.
            let last = node.span.end.saturating_sub(1).max(node.span.start);
            StatementFacts {
                index: at + 1,
                line: line_of(text, node.span.start),
                end_line: line_of(text, last),
                ran: ran.is_some(),
                name: node
                    .defines
                    .map(|binding| program.bindings[binding].name.clone()),
                result: ran.and_then(|ran| ran.result.clone()),
                stdout: ran.map_or(String::new(), |ran| ran.stdout.clone()),
                stderr: ran.map_or(String::new(), |ran| ran.stderr.clone()),
                reads: node.reads.iter().map(|earlier| earlier + 1).collect(),
                calls: node
                    .calls
                    .iter()
                    .map(|id| {
                        let function = &program.functions[*id];
                        match &function.module {
                            Some(module) => format!("{module}::{}", function.name),
                            None => function.name.clone(),
                        }
                    })
                    .collect(),
                seeds: node.seeds,
                draws: node.draws,
                reads_file: node.reads_file,
                writes_file: node.writes_file,
                drew: ran.is_some_and(|ran| ran.drew),
            }
        })
        .collect()
}

fn count(n: Option<u64>) -> String {
    n.map_or("null".to_string(), |n| n.to_string())
}

impl StatementFacts {
    /// The statement as one JSON object.
    pub fn json(&self) -> String {
        let result = &self.result;
        let number = |pick: fn(&ResultFacts) -> Option<u64>| count(result.as_ref().and_then(pick));
        format!(
            "{{\"index\": {}, \"line\": {}, \"end_line\": {}, \"ran\": {}, \"name\": {}, \"type\": {}, \"rows\": {}, \"columns\": {}, \"length\": {}, \"n\": {}, \"stdout\": {}, \"stderr\": {}, \"reads\": [{}], \"calls\": [{}], \"seeds\": {}, \"draws\": {}, \"reads_file\": {}, \"writes_file\": {}, \"drew\": {}}}",
            self.index,
            self.line,
            self.end_line,
            self.ran,
            self.name.as_deref().map_or("null".to_string(), json_string),
            result
                .as_ref()
                .map_or("null".to_string(), |facts| json_string(&facts.ty)),
            number(|facts| facts.rows),
            number(|facts| facts.columns),
            number(|facts| facts.length),
            number(|facts| facts.n),
            json_string(&self.stdout),
            json_string(&self.stderr),
            self.reads
                .iter()
                .map(usize::to_string)
                .collect::<Vec<_>>()
                .join(", "),
            self.calls
                .iter()
                .map(|name| json_string(name))
                .collect::<Vec<_>>()
                .join(", "),
            self.seeds,
            self.draws,
            self.reads_file,
            self.writes_file,
            self.drew
        )
    }
}

/// The statements as a JSON list.
pub fn json(statements: &[StatementFacts]) -> String {
    format!(
        "[{}]",
        statements
            .iter()
            .map(StatementFacts::json)
            .collect::<Vec<_>>()
            .join(", ")
    )
}
