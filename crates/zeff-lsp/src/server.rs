//! The server: one message in, the messages it answers with out.
//!
//! A message with a `method` and an `id` is a request and is always answered, with a result or
//! with an error of the protocol. One with a `method` and no `id` is a notification, which the
//! protocol does not let a server answer; where one cannot be acted on, the editor is told by a
//! `window/logMessage`. One without a `method` is the editor's answer to a request, and this
//! server makes none.

use crate::uri::path_of;
use serde_json::{Map, Value, json};
use std::collections::BTreeMap;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};
use zeff_engine::fs::FileSystem;
use zeff_engine::language::{Documents, Finding, Level, Position, Range, diagnostics, offset};

/// The errors of JSON-RPC and of the protocol that this server gives.
pub mod code {
    /// The body is not JSON.
    pub const PARSE_ERROR: i64 = -32700;
    /// The body is JSON and not a message, or a request the state of the server does not
    /// allow.
    pub const INVALID_REQUEST: i64 = -32600;
    /// A request this server does not serve.
    pub const METHOD_NOT_FOUND: i64 = -32601;
    /// A request before `initialize`.
    pub const SERVER_NOT_INITIALIZED: i64 = -32002;
}

/// Where the conversation stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    /// Before `initialize`.
    New,
    /// Between `initialize` and `shutdown`.
    Running,
    /// After `shutdown`: only `exit` is expected.
    ShutDown,
}

/// What a message leaves to be done.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Answer {
    /// The bodies of the messages to send, in order.
    pub messages: Vec<String>,
    /// The code the program ends with, when the message was `exit`.
    pub exit: Option<i32>,
}

/// The language server of one editor.
#[derive(Debug)]
pub struct Server {
    state: State,
    documents: Documents,
    /// The name the editor gave each open document, to answer with the same.
    uris: BTreeMap<PathBuf, String>,
    /// What the editor was last told of each open document.
    told: BTreeMap<PathBuf, Vec<Finding>>,
}

impl Default for Server {
    fn default() -> Self {
        Server::new()
    }
}

fn position(at: Position) -> Value {
    json!({ "line": at.line, "character": at.character })
}

fn range(range: Range) -> Value {
    json!({ "start": position(range.start), "end": position(range.end) })
}

/// A place the editor sent, where it is one.
fn sent_range(value: &Value) -> Option<Range> {
    let place = |value: &Value| {
        Some(Position {
            line: u32::try_from(value.get("line")?.as_u64()?).ok()?,
            character: u32::try_from(value.get("character")?.as_u64()?).ok()?,
        })
    };
    Some(Range {
        start: place(value.get("start")?)?,
        end: place(value.get("end")?)?,
    })
}

/// A finding as the protocol's `Diagnostic`. Its text is the message, the mark of section 11
/// where the message does not carry it, and each suggestion on a line of its own.
fn diagnostic(uri: &str, finding: &Finding) -> Value {
    let mut message = finding.message.clone();
    if let Some(mark) = &finding.mark
        && !message.contains(mark.as_str())
    {
        message.push_str(&format!(" ({mark})"));
    }
    for help in &finding.help {
        message.push_str("\nhelp: ");
        message.push_str(help);
    }
    let mut out = Map::new();
    out.insert("range".into(), range(finding.range));
    // 1 an error, 2 a warning, 3 information.
    let severity = match finding.level {
        Level::Error => 1,
        Level::Warning => 2,
        Level::Note => 3,
    };
    out.insert("severity".into(), json!(severity));
    if !finding.code.is_empty() {
        out.insert("code".into(), json!(finding.code));
    }
    out.insert("source".into(), json!("zeff"));
    out.insert("message".into(), json!(message));
    if !finding.related.is_empty() {
        let related: Vec<Value> = finding
            .related
            .iter()
            .map(|(at, text)| {
                json!({ "location": { "uri": uri, "range": range(*at) }, "message": text })
            })
            .collect();
        out.insert("relatedInformation".into(), Value::Array(related));
    }
    Value::Object(out)
}

fn response(id: &Value, result: Value) -> String {
    json!({ "jsonrpc": "2.0", "id": id, "result": result }).to_string()
}

fn error(id: &Value, code: i64, message: &str) -> String {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } }).to_string()
}

/// The answer to a body that is no message at all: an error that names no request.
pub fn refusal(code: i64, message: &str) -> String {
    error(&Value::Null, code, message)
}

/// A line for the editor's log: 1 an error, 2 a warning.
fn log(kind: u8, message: &str) -> String {
    json!({
        "jsonrpc": "2.0",
        "method": "window/logMessage",
        "params": { "type": kind, "message": message },
    })
    .to_string()
}

/// The diagnostics of a check, or `None` where the check itself failed: a fault of the engine,
/// which must not end the server.
fn checked(check: impl FnOnce() -> Vec<Finding>) -> Option<Vec<Finding>> {
    catch_unwind(AssertUnwindSafe(check)).ok()
}

impl Server {
    pub fn new() -> Server {
        Server {
            state: State::New,
            documents: Documents::new(),
            uris: BTreeMap::new(),
            told: BTreeMap::new(),
        }
    }

    /// The documents the editor has open.
    pub fn documents(&self) -> &Documents {
        &self.documents
    }

    /// One message's body in; what to send back, and whether to end.
    pub fn handle(&mut self, fs: &dyn FileSystem, body: &[u8]) -> Answer {
        let mut answer = Answer::default();
        let Ok(value) = serde_json::from_slice::<Value>(body) else {
            answer
                .messages
                .push(refusal(code::PARSE_ERROR, "the message is not JSON"));
            return answer;
        };
        let Some(message) = value.as_object() else {
            let what = if value.is_array() {
                "a batch of messages is not part of the Language Server Protocol"
            } else {
                "a message is a JSON object"
            };
            answer.messages.push(refusal(code::INVALID_REQUEST, what));
            return answer;
        };
        // What a request is answered under: a number or a text.
        let id = message
            .get("id")
            .filter(|id| id.is_number() || id.is_string());
        if message.get("jsonrpc").and_then(Value::as_str) != Some("2.0") {
            answer.messages.push(error(
                id.unwrap_or(&Value::Null),
                code::INVALID_REQUEST,
                "a message says `\"jsonrpc\": \"2.0\"`",
            ));
            return answer;
        }
        let Some(method) = message.get("method").and_then(Value::as_str) else {
            // The editor's answer to a request; this server makes none, so it is passed over.
            // Anything else without a method is no message.
            if !(message.contains_key("result") || message.contains_key("error")) {
                answer.messages.push(error(
                    id.unwrap_or(&Value::Null),
                    code::INVALID_REQUEST,
                    "a message has a `method`, or is the answer to a request",
                ));
            }
            return answer;
        };
        let params = message.get("params").unwrap_or(&Value::Null);
        match id {
            Some(id) => answer.messages.push(self.request(id, method)),
            None => self.notification(fs, method, params, &mut answer),
        }
        answer
    }

    /// A request, answered.
    fn request(&mut self, id: &Value, method: &str) -> String {
        match (self.state, method) {
            (State::New, "initialize") => {
                self.state = State::Running;
                response(
                    id,
                    json!({
                        "capabilities": {
                            // Places are counted in UTF-16 code units, the protocol's own
                            // way, which every editor has.
                            "positionEncoding": "utf-16",
                            // A document opened and closed is told, and a change comes as
                            // the whole text (1).
                            "textDocumentSync": { "openClose": true, "change": 1 },
                        },
                        "serverInfo": { "name": "zeff", "version": zeff_engine::VERSION },
                    }),
                )
            }
            (State::New, _) => error(
                id,
                code::SERVER_NOT_INITIALIZED,
                &format!("`{method}` before `initialize`: the server is not initialised yet"),
            ),
            (State::Running, "initialize") => {
                error(id, code::INVALID_REQUEST, "`initialize` was already sent")
            }
            (State::Running, "shutdown") => {
                self.state = State::ShutDown;
                response(id, Value::Null)
            }
            (State::Running, _) => error(
                id,
                code::METHOD_NOT_FOUND,
                &format!("`{method}` is not a request this server answers"),
            ),
            (State::ShutDown, _) => error(
                id,
                code::INVALID_REQUEST,
                &format!("`{method}` after `shutdown`: only `exit` is expected"),
            ),
        }
    }

    /// A notification, acted on.
    fn notification(
        &mut self,
        fs: &dyn FileSystem,
        method: &str,
        params: &Value,
        answer: &mut Answer,
    ) {
        if method == "exit" {
            // With success where `shutdown` came first, as the protocol has it.
            answer.exit = Some(if self.state == State::ShutDown { 0 } else { 1 });
            return;
        }
        if self.state != State::Running {
            // Before `initialize` and after `shutdown` a notification is dropped.
            return;
        }
        let document = params.get("textDocument");
        let uri = document.and_then(|d| d.get("uri")).and_then(Value::as_str);
        let version = document
            .and_then(|d| d.get("version"))
            .and_then(Value::as_i64)
            .unwrap_or(0);
        match (method, uri) {
            ("textDocument/didOpen", Some(uri)) => {
                let Some(text) = document.and_then(|d| d.get("text")).and_then(Value::as_str)
                else {
                    answer.messages.push(log(
                        2,
                        &format!("`{method}` without the text of {uri}: passed over"),
                    ));
                    return;
                };
                let path = path_of(uri);
                self.documents.open(path.clone(), text, version);
                self.uris.insert(path.clone(), uri.to_string());
                self.tell(fs, Some(&path), answer);
            }
            ("textDocument/didChange", Some(uri)) => {
                let path = path_of(uri);
                let Some(open) = self.documents.get(&path) else {
                    answer.messages.push(log(
                        2,
                        &format!("`{method}` for {uri}, which is not open: passed over"),
                    ));
                    return;
                };
                let mut text = open.text.clone();
                let changes = params.get("contentChanges").and_then(Value::as_array);
                for change in changes.into_iter().flatten() {
                    let Some(new) = change.get("text").and_then(Value::as_str) else {
                        continue;
                    };
                    match change.get("range").and_then(sent_range) {
                        // A part of the text, from an editor that sends parts though the
                        // whole was asked for.
                        Some(part) => {
                            let start = offset(&text, part.start);
                            let end = offset(&text, part.end).max(start);
                            text.replace_range(start..end, new);
                        }
                        None => text = new.to_string(),
                    }
                }
                self.documents.change(&path, text, version);
                self.tell(fs, Some(&path), answer);
            }
            ("textDocument/didClose", Some(uri)) => {
                let path = path_of(uri);
                if self.documents.close(&path) {
                    // Nothing is said of a document that is not open: what was said is taken
                    // back.
                    self.told.remove(&path);
                    let uri = self.uris.remove(&path).unwrap_or_else(|| uri.to_string());
                    answer.messages.push(
                        json!({
                            "jsonrpc": "2.0",
                            "method": "textDocument/publishDiagnostics",
                            "params": { "uri": uri, "diagnostics": [] },
                        })
                        .to_string(),
                    );
                    self.tell(fs, None, answer);
                }
            }
            ("textDocument/didOpen" | "textDocument/didChange" | "textDocument/didClose", None) => {
                answer.messages.push(log(
                    2,
                    &format!("`{method}` without the name of its document: passed over"),
                ));
            }
            // `initialized`, and whatever else an editor tells: nothing to do.
            _ => {}
        }
    }

    /// Tell the editor the diagnostics of the open documents: of `changed` always, and of
    /// each other where they are no longer what it was last told, since a change to a module
    /// changes what is said of the scripts that use it.
    fn tell(&mut self, fs: &dyn FileSystem, changed: Option<&Path>, answer: &mut Answer) {
        let paths: Vec<PathBuf> = self.documents.paths().map(Path::to_path_buf).collect();
        for path in paths {
            let uri = self.uris.get(&path).cloned().unwrap_or_default();
            let Some(found) = checked(|| diagnostics(fs, &self.documents, &path)) else {
                answer.messages.push(log(
                    1,
                    &format!(
                        "internal error while checking {uri}: this is a bug in Zeff, not in your script (Z9001)"
                    ),
                ));
                continue;
            };
            if Some(path.as_path()) != changed && self.told.get(&path) == Some(&found) {
                continue;
            }
            let version = self.documents.get(&path).map_or(0, |d| d.version);
            let list: Vec<Value> = found.iter().map(|f| diagnostic(&uri, f)).collect();
            answer.messages.push(
                json!({
                    "jsonrpc": "2.0",
                    "method": "textDocument/publishDiagnostics",
                    "params": { "uri": uri, "version": version, "diagnostics": list },
                })
                .to_string(),
            );
            self.told.insert(path, found);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::checked;

    #[test]
    fn a_check_that_fails_is_a_check_without_a_result() {
        let hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        let failed = checked(|| panic!("a fault of the engine"));
        std::panic::set_hook(hook);
        assert_eq!(failed, None);
        assert_eq!(checked(Vec::new), Some(Vec::new()));
    }
}
