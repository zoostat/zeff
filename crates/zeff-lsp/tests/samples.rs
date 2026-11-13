//! The messages exchanged on the sample scripts (release 0.7.0, step 34b): a conversation with
//! an editor that opens the scripts and the modules of the sample projects, types a fault into
//! one, and puts it right. Every message in and every message out is a stored text.

use serde_json::{Value, json};
use std::io::Cursor;
use std::path::Path;
use zeff_engine::fs::MemoryFileSystem;
use zeff_lsp::{Server, serve, write_message};

/// A sample project in memory, at `/s/<name>`, with the scripts and modules it holds.
fn sample(fs: &MemoryFileSystem, name: &str) -> Vec<(String, String)> {
    let folder = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../samples")
        .join(name);
    let mut documents = Vec::new();
    let mut copy = |relative: &str, keep: bool| {
        let bytes = std::fs::read(folder.join(relative)).expect("a file of the sample");
        if keep {
            let text = String::from_utf8(bytes.clone()).expect("a script is text");
            documents.push((format!("file:///s/{name}/{relative}"), text));
        }
        fs.add_file(format!("/s/{name}/{relative}"), bytes);
    };
    copy("zeff.toml", false);
    for sub in ["scripts", "functions", "data"] {
        let mut files: Vec<String> = std::fs::read_dir(folder.join(sub))
            .map(|entries| {
                entries
                    .flatten()
                    .map(|entry| entry.file_name().to_string_lossy().into_owned())
                    .collect()
            })
            .unwrap_or_default();
        files.sort();
        for file in files {
            copy(
                &format!("{sub}/{file}"),
                sub != "data" && file.ends_with(".zeff"),
            );
        }
    }
    documents
}

/// A conversation: what the editor sends, and what the server answers to each message.
struct Conversation {
    fs: MemoryFileSystem,
    server: Server,
    /// The bodies sent, to be sent again over an input.
    sent: Vec<String>,
    /// The bodies answered.
    answered: Vec<String>,
    /// Both, as a text to store.
    told: String,
}

impl Conversation {
    fn new(fs: MemoryFileSystem) -> Conversation {
        Conversation {
            fs,
            server: Server::new(),
            sent: Vec::new(),
            answered: Vec::new(),
            told: String::new(),
        }
    }

    fn send(&mut self, message: Value) -> Option<i32> {
        // A script's text is long: the stored text says how long it is and shows its end.
        let mut shown = message.clone();
        for pointer in ["/params/textDocument/text", "/params/contentChanges/0/text"] {
            let Some(text) = shown.pointer_mut(pointer) else {
                continue;
            };
            let whole = text.as_str().expect("a text").to_string();
            let lines: Vec<&str> = whole.lines().collect();
            if lines.len() > 4 {
                *text = json!(format!(
                    "({} lines, the last: {})",
                    lines.len(),
                    lines.last().expect("a line")
                ));
            }
        }
        self.told.push_str(&format!(
            "--> {}\n",
            serde_json::to_string_pretty(&shown).expect("JSON")
        ));
        let body = message.to_string();
        let answer = self.server.handle(&self.fs, body.as_bytes());
        self.sent.push(body);
        for message in &answer.messages {
            let mut value: Value = serde_json::from_str(message).expect("the server writes JSON");
            // The version is the engine's, which changes with each release.
            if let Some(version) = value.pointer_mut("/result/serverInfo/version") {
                assert_eq!(version, zeff_engine::VERSION);
                *version = json!("(the version of the engine)");
            }
            // A help page in a hover: its first lines, and how many follow.
            if let Some(page) = value.pointer_mut("/result/contents/value")
                && let Some(text) = page.as_str()
                && text.lines().count() > 6
            {
                let lines: Vec<&str> = text.lines().collect();
                *page = json!(format!(
                    "{}\n(and {} lines more)",
                    lines[..6].join("\n"),
                    lines.len() - 6
                ));
            }
            // A long list of completions: its first four, and how many follow.
            if let Some(items) = value
                .pointer_mut("/result/items")
                .and_then(Value::as_array_mut)
                && items.len() > 4
            {
                let more = items.len() - 4;
                items.truncate(4);
                items.push(json!(format!("(and {more} more)")));
            }
            self.told.push_str(&format!(
                "<-- {}\n",
                serde_json::to_string_pretty(&value).expect("JSON")
            ));
        }
        self.answered.extend(answer.messages);
        if let Some(code) = answer.exit {
            self.told
                .push_str(&format!("the server ends with {code}\n"));
        }
        answer.exit
    }

    fn open(&mut self, uri: &str, text: &str) {
        self.send(json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {"textDocument": {"uri": uri, "languageId": "zeff", "version": 1, "text": text}},
        }));
    }

    fn change(&mut self, uri: &str, version: i64, text: &str) {
        self.send(json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didChange",
            "params": {"textDocument": {"uri": uri, "version": version}, "contentChanges": [{"text": text}]},
        }));
    }

    fn close(&mut self, uri: &str) {
        self.send(json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didClose",
            "params": {"textDocument": {"uri": uri}},
        }));
    }
}

#[test]
fn the_messages_exchanged_on_the_sample_scripts() {
    let fs = MemoryFileSystem::new();
    fs.set_clock(1_000);
    let mut documents = Vec::new();
    for name in [
        "first_engine",
        "data_cleaning",
        "regression",
        "simulation",
        "user_functions",
        "panel",
        "plots",
    ] {
        documents.extend(sample(&fs, name));
    }
    assert_eq!(documents.len(), 9);
    let mut editor = Conversation::new(fs);
    editor.send(json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "processId": null,
            "rootUri": "file:///s",
            "capabilities": {"textDocument": {"hover": {"contentFormat": ["markdown", "plaintext"]}}},
        },
    }));
    editor.send(json!({"jsonrpc": "2.0", "method": "initialized", "params": {}}));
    // Every script and module of the samples opened: none has anything said of it.
    for (uri, text) in &documents {
        editor.open(uri, text);
    }
    // The sample with the project's own functions: its script, and the module it uses.
    let text_of = |uri: &str| -> String {
        let found = documents.iter().find(|(name, _)| name == uri);
        found.expect("a document of the samples").1.clone()
    };
    let script = "file:///s/user_functions/scripts/main.zeff";
    let module = "file:///s/user_functions/functions/share_missing.zeff";
    let (script_text, module_text) = (text_of(script), text_of(module));
    // Two faults typed at the end of the script: a variable the data do not hold, and a
    // statement half typed.
    editor.change(script, 2, &format!("{script_text}wages.mean(wagee)\n"));
    editor.change(
        script,
        3,
        &format!("{script_text}wages.mean(wagee)\nlet half = wages.summary([lwage, exper\n"),
    );
    // A call begun at the end of the script: what may stand in it, which begins with the
    // variables of the frame, each with its type, its label and its count of missing values;
    // and the usage line of the function, with the argument the cursor is in.
    let begun = format!("{script_text}wages.reg(lwage ~ ");
    let end = begun.lines().last().map_or(0, |line| line.chars().count());
    let place = json!({"line": begun.lines().count() - 1, "character": end});
    editor.change(script, 4, &begun);
    editor.send(json!({
        "jsonrpc": "2.0",
        "id": 10,
        "method": "textDocument/completion",
        "params": {"textDocument": {"uri": script}, "position": place},
    }));
    editor.send(json!({
        "jsonrpc": "2.0",
        "id": 11,
        "method": "textDocument/signatureHelp",
        "params": {"textDocument": {"uri": script}, "position": place},
    }));
    // After the name of an argument that takes one of a fixed list of words.
    let begun = format!("{script_text}wages.reg(lwage ~ educ, se: ");
    let end = begun.lines().last().map_or(0, |line| line.chars().count());
    let place = json!({"line": begun.lines().count() - 1, "character": end});
    editor.change(script, 5, &begun);
    editor.send(json!({
        "jsonrpc": "2.0",
        "id": 12,
        "method": "textDocument/completion",
        "params": {"textDocument": {"uri": script}, "position": place},
    }));
    // After a dot, with a word begun: the project's own function among the engine's.
    let begun = format!("{script_text}wages.sha");
    let end = begun.lines().last().map_or(0, |line| line.chars().count());
    let place = json!({"line": begun.lines().count() - 1, "character": end});
    editor.change(script, 6, &begun);
    editor.send(json!({
        "jsonrpc": "2.0",
        "id": 13,
        "method": "textDocument/completion",
        "params": {"textDocument": {"uri": script}, "position": place},
    }));
    // What is known of the word under the cursor: a frame bound in the script, a variable
    // of it, and a function of the project.
    let asked = format!("{script_text}wages.where(wage > 20).share_missing(hours)\n");
    let line = asked.lines().count() - 1;
    editor.change(script, 7, &asked);
    for (id, character) in [(14, 2), (15, 14), (16, 28)] {
        editor.send(json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "textDocument/hover",
            "params": {"textDocument": {"uri": script}, "position": {"line": line, "character": character}},
        }));
    }
    // The last line typed untidily, and formatted: alone, by a selection that stands in
    // it, and with the whole document, which changes nothing else.
    let untidy = format!("{script_text}wages.where( wage>20 ).share_missing( hours )\n");
    editor.change(script, 8, &untidy);
    editor.send(json!({
        "jsonrpc": "2.0",
        "id": 17,
        "method": "textDocument/rangeFormatting",
        "params": {
            "textDocument": {"uri": script},
            "range": {"start": {"line": line, "character": 3}, "end": {"line": line, "character": 3}},
            "options": {"tabSize": 4, "insertSpaces": true},
        },
    }));
    editor.send(json!({
        "jsonrpc": "2.0",
        "id": 18,
        "method": "textDocument/formatting",
        "params": {"textDocument": {"uri": script}, "options": {"tabSize": 4, "insertSpaces": true}},
    }));
    // A script that does not parse is left as it is, and the editor's log says why.
    editor.change(script, 9, &format!("{script_text}wages.where(wage > 20\n"));
    editor.send(json!({
        "jsonrpc": "2.0",
        "id": 19,
        "method": "textDocument/formatting",
        "params": {"textDocument": {"uri": script}, "options": {"tabSize": 4, "insertSpaces": true}},
    }));
    // The module's function renamed, not saved: the script is told what it now lacks.
    editor.change(
        module,
        2,
        &module_text.replace("share_missing", "share_absent"),
    );
    // The module as it was, and the script as it was: nothing is left to say.
    editor.change(module, 3, &module_text);
    editor.change(script, 10, &script_text);
    for (uri, _) in &documents {
        editor.close(uri);
    }
    // A request for a part of the server that a later release builds.
    editor.send(json!({
        "jsonrpc": "2.0",
        "id": 20,
        "method": "textDocument/definition",
        "params": {"textDocument": {"uri": script}, "position": {"line": 0, "character": 0}},
    }));
    editor.send(json!({"jsonrpc": "2.0", "id": 21, "method": "shutdown"}));
    assert_eq!(
        editor.send(json!({"jsonrpc": "2.0", "method": "exit"})),
        Some(0)
    );
    insta::assert_snapshot!("sample_scripts", editor.told);

    // The same conversation over an input and an output gives the same answers, byte for byte.
    let mut input = Vec::new();
    for body in &editor.sent {
        write_message(&mut input, body).expect("written");
    }
    let mut expected = Vec::new();
    for body in &editor.answered {
        write_message(&mut expected, body).expect("written");
    }
    let mut output = Vec::new();
    let code = serve(&editor.fs, &mut Cursor::new(input), &mut output);
    assert_eq!(code, 0);
    assert_eq!(
        String::from_utf8_lossy(&output),
        String::from_utf8_lossy(&expected)
    );
}
