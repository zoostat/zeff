//! The console session (Engine and Project Specification, section 2.1, "The console"): the
//! statements typed into `zeff repl`, kept as the session script `.zeff/session.zeff` of the
//! project, so that bindings stay immutable and the session replays like any script. The console
//! itself (prompts, the `:` commands) is in the command-line crate; this module is the part Zemp's
//! console shares: the list of statements, their file, and running them.

use crate::fs::FileSystem;
use crate::project::Project;
use crate::run::{Outcome, RunOptions, run_script_with};
use zeff_diagnostics::{Diagnostics, Severity};
use zeff_syntax::ast::Item;

/// The session script, relative to the project folder.
pub const SESSION_FILE: &str = ".zeff/session.zeff";

/// Where a session that could not be read is kept.
pub const PREVIOUS_FILE: &str = ".zeff/session.previous.zeff";

/// The statements of a session, in order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Session {
    statements: Vec<String>,
}

/// How a session was opened.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Opened {
    /// No session file: a new session.
    Fresh,
    /// The file's statements, this many, were read.
    Resumed(usize),
    /// The file could not be read as statements; it was moved to `PREVIOUS_FILE` and the session
    /// starts afresh.
    SetAside(String),
}

/// The statements of `text`, each as the text of one item (a statement, a `use` or a `fn`),
/// or the parse errors.
pub fn split_statements(text: &str) -> Result<Vec<String>, Diagnostics> {
    let parsed = zeff_syntax::parse(SESSION_FILE, text);
    if parsed.diagnostics.has_errors() {
        return Err(parsed.diagnostics);
    }
    Ok(parsed
        .script
        .items
        .iter()
        .map(|item| {
            let span = match item {
                Item::Use(u) => u.span,
                Item::Fn(f) => f.span,
                Item::Statement(s) => s.span,
            };
            text[span.start..span.end].trim_end().to_string()
        })
        .collect())
}

/// Whether `text` is a statement still being typed: a bracket not yet closed, or a line that
/// ends with an operator waiting for its right side.
pub fn is_incomplete(text: &str) -> bool {
    let parsed = zeff_syntax::parse(SESSION_FILE, text);
    parsed.diagnostics.iter().any(|d| {
        d.severity == Severity::Error
            && (d.code == "Z0302" || (d.code == "Z0301" && d.message.contains("found the end of")))
    })
}

impl Session {
    /// A session with no statements.
    pub fn new() -> Self {
        Session::default()
    }

    /// The session of `project`: its session file read, if there is one.
    pub fn open(fs: &dyn FileSystem, project: &Project) -> Result<(Session, Opened), String> {
        let path = project.root.join(SESSION_FILE);
        if !fs.is_file(&path) {
            return Ok((Session::new(), Opened::Fresh));
        }
        let text = fs
            .read_to_string(&path)
            .map_err(|e| format!("cannot read {SESSION_FILE}: {e}"))?;
        match split_statements(&text) {
            Ok(statements) => {
                let n = statements.len();
                Ok((Session { statements }, Opened::Resumed(n)))
            }
            Err(diagnostics) => {
                fs.write(&project.root.join(PREVIOUS_FILE), text.as_bytes())
                    .map_err(|e| format!("cannot write {PREVIOUS_FILE}: {e}"))?;
                let first = diagnostics
                    .iter()
                    .next()
                    .map(|d| d.message.clone())
                    .unwrap_or_default();
                Ok((Session::new(), Opened::SetAside(first)))
            }
        }
    }

    pub fn statements(&self) -> &[String] {
        &self.statements
    }

    pub fn len(&self) -> usize {
        self.statements.len()
    }

    pub fn is_empty(&self) -> bool {
        self.statements.is_empty()
    }

    /// The session as a script: one statement after another, each ending in a line break.
    pub fn text(&self) -> String {
        let mut text = String::new();
        for statement in &self.statements {
            text.push_str(statement);
            text.push('\n');
        }
        text
    }

    pub fn push(&mut self, statement: String) {
        self.statements.push(statement);
    }

    pub fn pop(&mut self) -> Option<String> {
        self.statements.pop()
    }

    /// Write the session file.
    pub fn save(&self, fs: &dyn FileSystem, project: &Project) -> std::io::Result<()> {
        let folder = project.root.join(".zeff");
        if !fs.is_dir(&folder) {
            fs.create_dir(&folder)?;
        }
        fs.write(&project.root.join(SESSION_FILE), self.text().as_bytes())
    }

    /// Run the whole session as a script; the outcome lists what each statement wrote.
    pub fn run(&self, fs: &dyn FileSystem, project: &Project, options: &RunOptions) -> Outcome {
        run_script_with(fs, project, SESSION_FILE, &self.text(), options)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fs::MemoryFileSystem;

    #[test]
    fn statements_are_split_and_incompleteness_is_told() {
        let statements = split_statements("let x = 1\n// a comment\nx\n  .describe()\n").unwrap();
        assert_eq!(statements, ["let x = 1", "x\n  .describe()"]);
        assert!(split_statements("").unwrap().is_empty());
        assert!(split_statements("let = ").is_err());
        assert!(is_incomplete("print(1"));
        assert!(is_incomplete("let y = [1, 2"));
        assert!(is_incomplete("let x = 1 +"));
        assert!(!is_incomplete("let x = 1"));
        assert!(!is_incomplete("let = 1"));
    }

    #[test]
    fn a_session_is_saved_and_resumed_or_set_aside() {
        let fs = MemoryFileSystem::new();
        fs.add_dir("/home/me");
        let project =
            crate::scratch::open_scratch(&fs, std::path::Path::new("/home/me/.zeff")).unwrap();
        let (mut session, opened) = Session::open(&fs, &project).unwrap();
        assert_eq!(opened, Opened::Fresh);
        session.push("let x = 1".into());
        session.push("x + 1".into());
        session.save(&fs, &project).unwrap();
        assert_eq!(
            fs.read_to_string(&project.root.join(SESSION_FILE)).unwrap(),
            "let x = 1\nx + 1\n"
        );
        let (again, opened) = Session::open(&fs, &project).unwrap();
        assert_eq!(opened, Opened::Resumed(2));
        assert_eq!(again, session);
        let outcome = again.run(&fs, &project, &RunOptions::default());
        assert_eq!(outcome.exit_code, 0);
        assert_eq!(outcome.output.statements.len(), 2);
        assert_eq!(outcome.output.statements[1].stdout, "2\n");
        fs.write(&project.root.join(SESSION_FILE), b"let = \n")
            .unwrap();
        let (fresh, opened) = Session::open(&fs, &project).unwrap();
        assert!(fresh.is_empty());
        assert!(matches!(opened, Opened::SetAside(_)));
        assert!(fs.is_file(&project.root.join(PREVIOUS_FILE)));
    }
}
