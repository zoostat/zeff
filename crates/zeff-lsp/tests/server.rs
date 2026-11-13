//! The server, message by message (release 0.7.0, step 34b): every message it knows, and what
//! it does with those it does not.

use serde_json::{Value, json};
use std::io::Cursor;
use zeff_engine::VERSION;
use zeff_engine::fs::MemoryFileSystem;
use zeff_lsp::{Incoming, Server, read_message, serve, write_message};

const MAIN: &str = "file:///work/demo/scripts/main.zeff";
const CLEAN: &str = "file:///work/demo/functions/clean.zeff";

/// No message at all.
const NOTHING: [Value; 0] = [];

/// A module with one function and its complete page.
fn module(name: &str) -> String {
    format!(
        "/// Twice a number.\n///\n/// The number doubled.\n///\n/// x: The number.\n/// returns: The number times two, a Float.\n/// > {name}(2.0)\n/// 4\nfn {name}(x: Float) -> Float {{\n    x * 2\n}}\n"
    )
}

fn project() -> MemoryFileSystem {
    let fs = MemoryFileSystem::new();
    fs.add_file(
        "/work/demo/zeff.toml",
        format!("[project]\nname = \"demo\"\nzeff = \"{VERSION}\"\nedition = \"preview\"\n"),
    );
    fs.add_file(
        "/work/demo/data/small.csv",
        "a,b,c\n1,2.5,x\n2,3.5,y\n3,,x\n",
    );
    fs.add_file("/work/demo/functions/clean.zeff", module("twice"));
    fs.add_file(
        "/work/demo/scripts/main.zeff",
        "use functions::clean\nprint(twice(2))\n",
    );
    fs
}

/// A server and its files, with what it answers to a message.
struct Session {
    fs: MemoryFileSystem,
    server: Server,
}

impl Session {
    fn new() -> Session {
        Session {
            fs: project(),
            server: Server::new(),
        }
    }

    /// A server after `initialize` and `initialized`.
    fn started() -> Session {
        let mut session = Session::new();
        let answers =
            session.send(&json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}}));
        assert_eq!(answers.len(), 1);
        assert_eq!(
            session.send(&json!({"jsonrpc": "2.0", "method": "initialized", "params": {}})),
            NOTHING
        );
        session
    }

    fn raw(&mut self, body: &[u8]) -> (Vec<Value>, Option<i32>) {
        let answer = self.server.handle(&self.fs, body);
        let messages = answer
            .messages
            .iter()
            .map(|m| serde_json::from_str(m).expect("the server writes JSON"))
            .collect();
        (messages, answer.exit)
    }

    fn send(&mut self, message: &Value) -> Vec<Value> {
        let (messages, exit) = self.raw(message.to_string().as_bytes());
        assert_eq!(exit, None);
        messages
    }

    fn notify(&mut self, method: &str, params: Value) -> Vec<Value> {
        self.send(&json!({"jsonrpc": "2.0", "method": method, "params": params}))
    }

    fn open(&mut self, uri: &str, text: &str) -> Vec<Value> {
        self.notify(
            "textDocument/didOpen",
            json!({"textDocument": {"uri": uri, "languageId": "zeff", "version": 1, "text": text}}),
        )
    }

    fn change(&mut self, uri: &str, version: i64, text: &str) -> Vec<Value> {
        self.notify(
            "textDocument/didChange",
            json!({"textDocument": {"uri": uri, "version": version}, "contentChanges": [{"text": text}]}),
        )
    }

    fn close(&mut self, uri: &str) -> Vec<Value> {
        self.notify(
            "textDocument/didClose",
            json!({"textDocument": {"uri": uri}}),
        )
    }
}

/// The codes of the diagnostics a `publishDiagnostics` carries, with the document it is of.
fn told(message: &Value) -> (String, Vec<String>) {
    assert_eq!(
        message["method"], "textDocument/publishDiagnostics",
        "{message}"
    );
    let codes = message["params"]["diagnostics"]
        .as_array()
        .expect("a list")
        .iter()
        .map(|d| d["code"].as_str().unwrap_or("").to_string())
        .collect();
    (
        message["params"]["uri"]
            .as_str()
            .expect("a uri")
            .to_string(),
        codes,
    )
}

fn error_code(message: &Value) -> i64 {
    message["error"]["code"]
        .as_i64()
        .unwrap_or_else(|| panic!("no error: {message}"))
}

#[test]
fn initialize_says_what_the_server_does() {
    let mut session = Session::new();
    let answers = session.send(&json!({
        "jsonrpc": "2.0", "id": 1, "method": "initialize",
        "params": {"processId": null, "rootUri": "file:///work/demo", "capabilities": {}},
    }));
    assert_eq!(
        answers,
        [json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "capabilities": {
                    "positionEncoding": "utf-16",
                    "textDocumentSync": {"openClose": true, "change": 1},
                },
                "serverInfo": {"name": "zeff", "version": VERSION},
            },
        })]
    );
    // The name a request is answered under may be a text.
    let answers = session.send(&json!({"jsonrpc": "2.0", "id": "two", "method": "initialize"}));
    assert_eq!(answers[0]["id"], "two");
    // It was already sent.
    assert_eq!(error_code(&answers[0]), -32600);
    assert_eq!(
        answers[0]["error"]["message"],
        "`initialize` was already sent"
    );
}

#[test]
fn before_initialize_a_request_is_refused_and_a_notification_dropped() {
    let mut session = Session::new();
    let answers = session.send(&json!({"jsonrpc": "2.0", "id": 5, "method": "shutdown"}));
    assert_eq!(answers.len(), 1);
    assert_eq!(error_code(&answers[0]), -32002);
    assert_eq!(answers[0]["id"], 5);
    assert_eq!(
        answers[0]["error"]["message"],
        "`shutdown` before `initialize`: the server is not initialised yet"
    );
    // A document opened too early is not opened.
    assert_eq!(session.open(MAIN, "print(nope)\n"), NOTHING);
    assert_eq!(session.server.documents().paths().count(), 0);
}

#[test]
fn shutdown_then_exit_ends_with_success_and_exit_alone_does_not() {
    let mut session = Session::started();
    let answers = session.send(&json!({"jsonrpc": "2.0", "id": 2, "method": "shutdown"}));
    assert_eq!(
        answers,
        [json!({"jsonrpc": "2.0", "id": 2, "result": null})]
    );
    // After it only `exit` is expected: a request is refused, a notification dropped.
    let answers = session.send(&json!({"jsonrpc": "2.0", "id": 3, "method": "shutdown"}));
    assert_eq!(error_code(&answers[0]), -32600);
    assert_eq!(
        answers[0]["error"]["message"],
        "`shutdown` after `shutdown`: only `exit` is expected"
    );
    assert_eq!(session.open(MAIN, "print(nope)\n"), NOTHING);
    let (messages, exit) = session.raw(br#"{"jsonrpc":"2.0","method":"exit"}"#);
    assert_eq!((messages, exit), (vec![], Some(0)));

    // Without `shutdown` before it, `exit` ends with 1; before `initialize` too.
    let mut session = Session::started();
    assert_eq!(
        session.raw(br#"{"jsonrpc":"2.0","method":"exit"}"#).1,
        Some(1)
    );
    let mut session = Session::new();
    assert_eq!(
        session.raw(br#"{"jsonrpc":"2.0","method":"exit"}"#).1,
        Some(1)
    );
}

#[test]
fn a_document_opened_changed_and_closed_is_told_its_diagnostics_each_time() {
    let mut session = Session::started();
    // Opened with a fault: told at once, under the version the editor gave.
    let answers = session.open(MAIN, "use functions::clean\nprint(twice(2))\nprint(nope)\n");
    assert_eq!(
        answers,
        [json!({
            "jsonrpc": "2.0",
            "method": "textDocument/publishDiagnostics",
            "params": {
                "uri": MAIN,
                "version": 1,
                "diagnostics": [{
                    "range": {"start": {"line": 2, "character": 6}, "end": {"line": 2, "character": 10}},
                    "severity": 1,
                    "code": "Z0101",
                    "source": "zeff",
                    "message": "nothing is called `nope` here",
                }],
            },
        })]
    );
    // Changed: the fault corrected, told again though nothing is left to say.
    let answers = session.change(MAIN, 2, "use functions::clean\nprint(twice(2))\n");
    assert_eq!(answers.len(), 1);
    assert_eq!(
        answers[0]["params"],
        json!({"uri": MAIN, "version": 2, "diagnostics": []})
    );
    // Changed again: a warning (2) and a note (3), a suggestion on a line of its own, the
    // mark of section 11 kept in the text.
    let answers = session.change(
        MAIN,
        3,
        "let mean = 3\nlet d = import(\"data/small.csv\")\nlet wide = d.to_wide(id: a, names_from: c, values_from: b)\nd.mean(aa)\n",
    );
    let list = answers[0]["params"]["diagnostics"]
        .as_array()
        .expect("a list");
    let seen: Vec<(i64, &str)> = list
        .iter()
        .map(|d| {
            (
                d["severity"].as_i64().unwrap(),
                d["code"].as_str().unwrap_or(""),
            )
        })
        .collect();
    assert_eq!(seen, [(2, "W0101"), (3, ""), (1, "Z0101")]);
    assert_eq!(
        list[0]["message"],
        "`mean` hides the function `data::mean`\nhelp: the function is still reachable as `data::mean`; a different name avoids the confusion"
    );
    assert!(list[1].get("code").is_none());
    assert!(
        list[1]["message"]
            .as_str()
            .unwrap()
            .starts_with("`to_wide` is checked at run time: "),
        "{}",
        list[1]["message"]
    );
    assert_eq!(
        list[2]["message"],
        "no variable `aa` in `d`\nhelp: did you mean `a`? If you meant a category level, write it in quotes: `\"a\"`"
    );
    // Closed: what was said is taken back, and the file on disk is the script again.
    let answers = session.close(MAIN);
    assert_eq!(
        answers,
        [json!({
            "jsonrpc": "2.0",
            "method": "textDocument/publishDiagnostics",
            "params": {"uri": MAIN, "diagnostics": []},
        })]
    );
    assert_eq!(session.server.documents().paths().count(), 0);
    // Closed twice: nothing.
    assert_eq!(session.close(MAIN), NOTHING);
}

#[test]
fn a_diagnostic_with_two_places_names_the_other() {
    let mut session = Session::started();
    let answers = session.open(
        MAIN,
        "/// One.\nfn f(x: Float) -> Float { x }\n/// Two.\nfn f(x: Float) -> Float { x + 1 }\n",
    );
    let list = &answers[0]["params"]["diagnostics"];
    assert_eq!(list[0]["code"], "Z0107");
    assert_eq!(
        list[0]["relatedInformation"],
        json!([{
            "location": {
                "uri": MAIN,
                "range": {"start": {"line": 1, "character": 3}, "end": {"line": 1, "character": 4}},
            },
            "message": "the first",
        }])
    );
}

#[test]
fn a_change_to_a_module_is_told_to_the_scripts_that_use_it() {
    let mut session = Session::started();
    assert_eq!(
        told(&session.open(MAIN, "use functions::clean\nprint(twice(2))\n")[0]),
        (MAIN.to_string(), vec![])
    );
    // The module opened as it is: only the module is told, the script has nothing new.
    let answers = session.open(CLEAN, &module("twice"));
    assert_eq!(answers.len(), 1);
    assert_eq!(told(&answers[0]), (CLEAN.to_string(), vec![]));
    // Its function renamed: the module is told (nothing), and the script, which now calls a
    // function that is not there.
    let answers = session.change(CLEAN, 2, &module("double"));
    assert_eq!(answers.len(), 2);
    assert_eq!(told(&answers[0]), (CLEAN.to_string(), vec![]));
    assert_eq!(
        told(&answers[1]),
        (MAIN.to_string(), vec!["Z0103".to_string()])
    );
    // A keystroke in the module that changes nothing for the script: the script is not told
    // again.
    let answers = session.change(CLEAN, 3, &format!("{}\n", module("double")));
    assert_eq!(answers.len(), 1);
    assert_eq!(told(&answers[0]).0, CLEAN);
    // The module closed without saving: the script is right again, and told so.
    let answers = session.close(CLEAN);
    assert_eq!(answers.len(), 2);
    assert_eq!(
        answers[0]["params"],
        json!({"uri": CLEAN, "diagnostics": []})
    );
    assert_eq!(told(&answers[1]), (MAIN.to_string(), vec![]));
}

#[test]
fn a_change_sent_as_a_part_of_the_text_is_put_in_its_place() {
    // The server asks for whole texts; an editor that sends parts all the same is followed,
    // its places counted in UTF-16.
    let mut session = Session::started();
    session.open(MAIN, "let s = \"😀\"\nprint(nope)\n");
    let answers = session.notify(
        "textDocument/didChange",
        json!({
            "textDocument": {"uri": MAIN, "version": 2},
            "contentChanges": [
                // `nope` becomes `s`, then a letter goes in after the face (two code units).
                {"range": {"start": {"line": 1, "character": 6}, "end": {"line": 1, "character": 10}}, "text": "s"},
                {"range": {"start": {"line": 0, "character": 11}, "end": {"line": 0, "character": 11}}, "text": "π"},
            ],
        }),
    );
    assert_eq!(told(&answers[0]), (MAIN.to_string(), vec![]));
    let path = zeff_lsp::uri::path_of(MAIN);
    let document = session.server.documents().get(&path).expect("open");
    assert_eq!(document.text, "let s = \"😀π\"\nprint(s)\n");
    assert_eq!(document.version, 2);
    // A place past the end is the end; a range written backwards changes nothing but its start.
    let answers = session.notify(
        "textDocument/didChange",
        json!({
            "textDocument": {"uri": MAIN, "version": 3},
            "contentChanges": [
                {"range": {"start": {"line": 99, "character": 0}, "end": {"line": 99, "character": 5}}, "text": "print(1)\n"},
                {"range": {"start": {"line": 1, "character": 5}, "end": {"line": 0, "character": 0}}, "text": ""},
            ],
        }),
    );
    assert_eq!(told(&answers[0]), (MAIN.to_string(), vec![]));
    let document = session.server.documents().get(&path).expect("open");
    assert_eq!(document.text, "let s = \"😀π\"\nprint(s)\nprint(1)\n");
}

#[test]
fn documents_by_other_names() {
    let mut session = Session::started();
    // A name with `%20` and letters outside ASCII is answered under the very name given.
    let odd = "file:///work/my%20notes/%C3%A9tude.zeff";
    let answers = session.open(odd, "print(nope)\n");
    assert_eq!(
        told(&answers[0]),
        (odd.to_string(), vec!["Z0101".to_string()])
    );
    // A document that is no file yet is checked alone.
    let answers = session.open(
        "untitled:Untitled-1",
        "let d = frame(x = [1.0, 2])\nd.mean(xx)\n",
    );
    assert_eq!(
        told(&answers[0]),
        ("untitled:Untitled-1".to_string(), vec!["Z0101".to_string()])
    );
    // A document that is not a script of the project's folders, in the project.
    let answers = session.open(
        "file:///work/demo/notes/try.zeff",
        "use functions::clean\nprint(twice(1))\n",
    );
    assert_eq!(told(&answers[0]).1, Vec::<String>::new());
}

#[test]
fn a_notification_that_cannot_be_acted_on_is_put_in_the_editors_log() {
    let mut session = Session::started();
    let log = |answers: Vec<Value>| -> String {
        assert_eq!(answers.len(), 1, "{answers:?}");
        assert_eq!(answers[0]["method"], "window/logMessage");
        assert_eq!(answers[0]["params"]["type"], 2);
        assert!(answers[0].get("id").is_none());
        answers[0]["params"]["message"]
            .as_str()
            .expect("a text")
            .to_string()
    };
    assert_eq!(
        log(session.change(MAIN, 2, "print(1)\n")),
        format!("`textDocument/didChange` for {MAIN}, which is not open: passed over")
    );
    assert_eq!(
        log(session.notify(
            "textDocument/didOpen",
            json!({"textDocument": {"uri": MAIN}})
        )),
        format!("`textDocument/didOpen` without the text of {MAIN}: passed over")
    );
    for method in [
        "textDocument/didOpen",
        "textDocument/didChange",
        "textDocument/didClose",
    ] {
        for params in [
            json!(null),
            json!({}),
            json!([1, 2]),
            json!({"textDocument": {"uri": 7}}),
        ] {
            assert_eq!(
                log(session.notify(method, params)),
                format!("`{method}` without the name of its document: passed over")
            );
        }
    }
    assert_eq!(session.server.documents().paths().count(), 0);
    // A change whose parts are not changes leaves the text as it was.
    session.open(MAIN, "print(1)\n");
    let answers = session.notify(
        "textDocument/didChange",
        json!({"textDocument": {"uri": MAIN, "version": 2}, "contentChanges": [7, {"range": 3}, {"text": 5}]}),
    );
    assert_eq!(told(&answers[0]), (MAIN.to_string(), vec![]));
    let answers = session.notify(
        "textDocument/didChange",
        json!({"textDocument": {"uri": MAIN, "version": 3}}),
    );
    assert_eq!(told(&answers[0]), (MAIN.to_string(), vec![]));
    let path = zeff_lsp::uri::path_of(MAIN);
    assert_eq!(
        session.server.documents().get(&path).expect("open").text,
        "print(1)\n"
    );
}

#[test]
fn what_the_server_does_not_serve() {
    let mut session = Session::started();
    // A request it does not answer is answered so; the parts that follow in later steps too.
    for method in [
        "textDocument/hover",
        "textDocument/completion",
        "workspace/symbol",
        "$/unknown",
        "",
    ] {
        let answers =
            session.send(&json!({"jsonrpc": "2.0", "id": 9, "method": method, "params": {}}));
        assert_eq!(answers.len(), 1);
        assert_eq!(error_code(&answers[0]), -32601, "{method}");
        assert_eq!(answers[0]["id"], 9);
        assert_eq!(
            answers[0]["error"]["message"],
            format!("`{method}` is not a request this server answers")
        );
    }
    // A notification it does not know, and the editor's answer to a request: nothing.
    for message in [
        json!({"jsonrpc": "2.0", "method": "$/cancelRequest", "params": {"id": 9}}),
        json!({"jsonrpc": "2.0", "method": "$/setTrace", "params": {"value": "off"}}),
        json!({"jsonrpc": "2.0", "method": "workspace/didChangeConfiguration", "params": {}}),
        json!({"jsonrpc": "2.0", "method": "textDocument/didSave", "params": {"textDocument": {"uri": MAIN}}}),
        json!({"jsonrpc": "2.0", "id": 4, "result": null}),
        json!({"jsonrpc": "2.0", "id": 4, "error": {"code": 1, "message": "no"}}),
    ] {
        assert_eq!(session.send(&message), NOTHING, "{message}");
    }
}

#[test]
fn a_body_that_is_no_message_is_answered_with_an_error_of_the_protocol() {
    let mut session = Session::started();
    let mut refused = |body: &[u8]| -> (i64, String, Value) {
        let (answers, exit) = session.raw(body);
        assert_eq!(exit, None);
        assert_eq!(answers.len(), 1, "{answers:?}");
        assert_eq!(answers[0]["jsonrpc"], "2.0");
        (
            error_code(&answers[0]),
            answers[0]["error"]["message"]
                .as_str()
                .expect("a text")
                .to_string(),
            answers[0]["id"].clone(),
        )
    };
    let not_json = (-32700, "the message is not JSON".to_string(), Value::Null);
    assert_eq!(refused(b""), not_json);
    assert_eq!(refused(b"{"), not_json);
    assert_eq!(refused(b"print(1)"), not_json);
    assert_eq!(refused(&[0xff, 0xfe, 0x00]), not_json);
    // Nested deeper than any message is.
    assert_eq!(refused("[".repeat(100_000).as_bytes()), not_json);
    // JSON, and no message.
    for body in ["7", "null", "\"exit\"", "true"] {
        assert_eq!(
            refused(body.as_bytes()),
            (
                -32600,
                "a message is a JSON object".to_string(),
                Value::Null
            ),
            "{body}"
        );
    }
    assert_eq!(
        refused(br#"[{"jsonrpc":"2.0","id":1,"method":"shutdown"}]"#),
        (
            -32600,
            "a batch of messages is not part of the Language Server Protocol".to_string(),
            Value::Null
        )
    );
    // An object that does not say it is a message; the request's name is kept where it has one.
    let says = "a message says `\"jsonrpc\": \"2.0\"`".to_string();
    assert_eq!(
        refused(br#"{"id":3,"method":"shutdown"}"#),
        (-32600, says.clone(), json!(3))
    );
    assert_eq!(
        refused(br#"{"jsonrpc":"1.0","method":"exit"}"#),
        (-32600, says.clone(), Value::Null)
    );
    assert_eq!(refused(b"{}"), (-32600, says, Value::Null));
    // Neither a method nor an answer; a method that is no text.
    let neither = "a message has a `method`, or is the answer to a request".to_string();
    assert_eq!(
        refused(br#"{"jsonrpc":"2.0","id":"a"}"#),
        (-32600, neither.clone(), json!("a"))
    );
    assert_eq!(
        refused(br#"{"jsonrpc":"2.0","method":7,"id":2}"#),
        (-32600, neither.clone(), json!(2))
    );
    assert_eq!(
        refused(br#"{"jsonrpc":"2.0"}"#),
        (-32600, neither, Value::Null)
    );
    // A name that is neither a number nor a text is no name: the message is a notification.
    let (answers, exit) = session.raw(br#"{"jsonrpc":"2.0","id":[1],"method":"shutdown"}"#);
    assert_eq!((answers, exit), (vec![], None));
    // None of it ended the server, which still answers.
    let answers = session.send(&json!({"jsonrpc": "2.0", "id": 8, "method": "shutdown"}));
    assert_eq!(
        answers,
        [json!({"jsonrpc": "2.0", "id": 8, "result": null})]
    );
}

/// The bodies of some messages, framed one after another.
fn framed(bodies: &[String]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for body in bodies {
        write_message(&mut bytes, body).expect("written");
    }
    bytes
}

/// The messages a server wrote.
fn written(bytes: &[u8]) -> Vec<Value> {
    let mut input = Cursor::new(bytes.to_vec());
    let mut messages = Vec::new();
    loop {
        match read_message(&mut input) {
            Incoming::Message(body) => messages.push(serde_json::from_slice(&body).expect("JSON")),
            Incoming::End => return messages,
            Incoming::Refused(why) => panic!("the server wrote what it would refuse: {why}"),
        }
    }
}

#[test]
fn a_conversation_over_an_input_and_an_output() {
    let fs = project();
    let bodies = [
        json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}}).to_string(),
        json!({"jsonrpc": "2.0", "method": "initialized", "params": {}}).to_string(),
        json!({"jsonrpc": "2.0", "method": "textDocument/didOpen", "params": {"textDocument": {"uri": MAIN, "version": 1, "text": "print(nope)\n"}}}).to_string(),
        "not JSON".to_string(),
        json!({"jsonrpc": "2.0", "id": 2, "method": "shutdown"}).to_string(),
        json!({"jsonrpc": "2.0", "method": "exit"}).to_string(),
        // After `exit` nothing is read.
        json!({"jsonrpc": "2.0", "id": 3, "method": "shutdown"}).to_string(),
    ];
    let mut input = Cursor::new(framed(&bodies));
    let mut output = Vec::new();
    assert_eq!(serve(&fs, &mut input, &mut output), 0);
    let messages = written(&output);
    assert_eq!(messages.len(), 4, "{messages:?}");
    assert_eq!(messages[0]["id"], 1);
    assert_eq!(
        told(&messages[1]),
        (MAIN.to_string(), vec!["Z0101".to_string()])
    );
    assert_eq!(error_code(&messages[2]), -32700);
    assert_eq!(
        messages[3],
        json!({"jsonrpc": "2.0", "id": 2, "result": null})
    );
    assert_eq!(input.position() as usize, framed(&bodies[..6]).len());

    // The input ends without `exit`: the server ends with 1, having answered what came.
    let mut input = Cursor::new(framed(&bodies[..3]));
    let mut output = Vec::new();
    assert_eq!(serve(&fs, &mut input, &mut output), 1);
    assert_eq!(written(&output).len(), 2);
    // Bytes that are no messages: each refusal is an error of the protocol, and the end of
    // the input is the end.
    let mut input = Cursor::new(b"hello\r\n\r\nContent-Length: x\r\n\r\n".to_vec());
    let mut output = Vec::new();
    assert_eq!(serve(&fs, &mut input, &mut output), 1);
    let messages = written(&output);
    assert_eq!(messages.len(), 2);
    assert_eq!(
        messages[0],
        json!({"jsonrpc": "2.0", "id": null, "error": {"code": -32700, "message": "a header line without `:`"}})
    );
    // An output nobody reads ends the server.
    struct Closed;
    impl std::io::Write for Closed {
        fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
            Err(std::io::ErrorKind::BrokenPipe.into())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut input = Cursor::new(framed(&bodies));
    assert_eq!(serve(&fs, &mut input, &mut Closed), 1);
}
