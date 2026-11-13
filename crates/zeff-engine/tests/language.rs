//! The language services, part 1 (release 0.7.0, step 34a; Engine and Project Specification,
//! section 12): the documents an editor has open, and the diagnostics of a document, the
//! checker's own, placed by line and by UTF-16 column as the Language Server Protocol counts.

use proptest::prelude::*;
use std::path::Path;
use zeff_engine::VERSION;
use zeff_engine::fs::{FileSystem, MemoryFileSystem};
use zeff_engine::language::{
    Documents, Finding, Level, Position, Range, diagnostics, offset, position,
};

const MAIN: &str = "/work/demo/scripts/main.zeff";
const CLEAN: &str = "/work/demo/functions/clean.zeff";

/// A module with one function and its complete page.
fn module(name: &str) -> String {
    format!(
        "/// Twice a number.\n///\n/// The number doubled.\n///\n/// x: The number.\n/// returns: The number times two, a Float.\n/// > {name}(2.0)\n/// 4\nfn {name}(x: Float) -> Float {{\n    x * 2\n}}\n"
    )
}

/// A project with a data file, a module and a script.
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
    fs.add_file(CLEAN, module("twice"));
    fs.add_file(MAIN, "use functions::clean\nprint(twice(2))\n");
    fs
}

fn place(range: Range) -> String {
    format!(
        "{}:{}-{}:{}",
        range.start.line, range.start.character, range.end.line, range.end.character
    )
}

/// The findings as a text, one to a paragraph, the places counted from 0 as the protocol does.
fn shown(findings: &[Finding]) -> String {
    let mut all = String::new();
    for finding in findings {
        let level = match finding.level {
            Level::Error => "error",
            Level::Warning => "warning",
            Level::Note => "note",
        };
        let code = if finding.code.is_empty() {
            String::new()
        } else {
            format!("[{}]", finding.code)
        };
        all.push_str(&format!(
            "{} {level}{code}: {}\n",
            place(finding.range),
            finding.message
        ));
        if !finding.label.is_empty() {
            all.push_str(&format!("    here: {}\n", finding.label));
        }
        for (range, text) in &finding.related {
            all.push_str(&format!("    also {}: {text}\n", place(*range)));
        }
        for help in &finding.help {
            all.push_str(&format!("    help: {help}\n"));
        }
        if let Some(mark) = &finding.mark {
            all.push_str(&format!("    mark: {mark}\n"));
        }
    }
    if findings.is_empty() {
        all.push_str("nothing\n");
    }
    all
}

/// The diagnostics of a text typed into a document, the document open in the editor.
fn typed(fs: &MemoryFileSystem, path: &str, text: &str) -> Vec<Finding> {
    let mut open = Documents::new();
    open.open(path, text, 1);
    diagnostics(fs, &open, Path::new(path))
}

fn at(line: u32, character: u32) -> Position {
    Position { line, character }
}

#[test]
fn a_place_is_a_line_and_a_column_of_utf16_code_units() {
    // A face (outside the basic plane: four bytes, two code units), a Greek letter (two bytes,
    // one unit), a letter with a combining mark (two characters), and the three line ends.
    let text = "let s = \"😀 π\"\r\nlet t = \"e\u{301}\"\rlast\nend";
    let byte = |needle: &str| text.find(needle).expect("in the text");
    assert_eq!(position(text, 0), at(0, 0));
    assert_eq!(position(text, byte("😀")), at(0, 9));
    // After the face the column is two further, not four and not one.
    assert_eq!(position(text, byte(" π")), at(0, 11));
    assert_eq!(position(text, byte("π") + "π".len()), at(0, 13));
    // Inside the face: the place of the face.
    assert_eq!(position(text, byte("😀") + 2), at(0, 9));
    // The end of the first line, before and between its two line-end characters.
    assert_eq!(position(text, byte("\r\n")), at(0, 14));
    assert_eq!(position(text, byte("\r\n") + 1), at(0, 14));
    assert_eq!(position(text, byte("let t")), at(1, 0));
    assert_eq!(position(text, byte("\u{301}")), at(1, 10));
    // A carriage return alone ends a line too.
    assert_eq!(position(text, byte("last")), at(2, 0));
    assert_eq!(position(text, byte("end")), at(3, 0));
    assert_eq!(position(text, text.len()), at(3, 3));
    assert_eq!(position(text, text.len() + 40), at(3, 3));

    // And back.
    assert_eq!(offset(text, at(0, 9)), byte("😀"));
    assert_eq!(offset(text, at(0, 11)), byte(" π"));
    // A column between the two code units of the face is the face.
    assert_eq!(offset(text, at(0, 10)), byte("😀"));
    // Past the end of a line: its end, before the line-end characters.
    assert_eq!(offset(text, at(0, 99)), byte("\r\n"));
    assert_eq!(offset(text, at(1, 99)), byte("\rlast"));
    assert_eq!(offset(text, at(2, 2)), byte("last") + 2);
    assert_eq!(offset(text, at(3, 3)), text.len());
    // Past the last line: the end of the text.
    assert_eq!(offset(text, at(9, 0)), text.len());
    assert_eq!(offset("", at(0, 0)), 0);
    assert_eq!(position("", 0), at(0, 0));
}

proptest! {
    /// For any text, the place of a byte names that byte again, and places go forward with
    /// the bytes; a byte inside a character or inside the pair of a line end is the one
    /// before it.
    #[test]
    fn a_place_names_its_byte_again(text in "[a-zπé😀𝑥 \t\r\n\"]{0,40}") {
        let bytes = text.as_bytes();
        let mut before = at(0, 0);
        for byte in 0..=text.len() {
            let here = position(&text, byte);
            prop_assert!(here >= before, "{:?} after {:?} at byte {}", here, before, byte);
            before = here;
            let inside_a_pair = byte > 0 && bytes[byte - 1] == b'\r' && bytes.get(byte) == Some(&b'\n');
            if text.is_char_boundary(byte) && !inside_a_pair {
                prop_assert_eq!(offset(&text, here), byte, "{:?}", here);
            } else {
                prop_assert!(offset(&text, here) < byte);
            }
        }
        // Any place at all names a byte where a character begins, or the end.
        for line in 0..4 {
            for character in 0..12 {
                prop_assert!(text.is_char_boundary(offset(&text, at(line, character))));
            }
        }
    }
}

#[test]
fn the_documents_an_editor_has_open_hold_the_text_as_last_typed() {
    let mut open = Documents::new();
    assert!(open.get(Path::new(MAIN)).is_none());
    // A change to a document that was never opened is not kept.
    assert!(!open.change(Path::new(MAIN), "print(1)\n", 2));
    assert!(open.get(Path::new(MAIN)).is_none());
    open.open(MAIN, "print(1)\n", 1);
    open.open(CLEAN, module("twice"), 1);
    assert!(open.change(Path::new(MAIN), "print(2)\n", 7));
    let document = open.get(Path::new(MAIN)).expect("open");
    assert_eq!(
        (document.text.as_str(), document.version),
        ("print(2)\n", 7)
    );
    // By their paths, in order.
    let paths: Vec<&Path> = open.paths().collect();
    assert_eq!(paths, [Path::new(CLEAN), Path::new(MAIN)]);
    assert!(open.close(Path::new(MAIN)));
    assert!(!open.close(Path::new(MAIN)));
    assert!(open.get(Path::new(MAIN)).is_none());
    // Opened again, a document begins with the text it is opened with.
    open.open(MAIN, "print(3)\n", 1);
    assert_eq!(open.get(Path::new(MAIN)).expect("open").text, "print(3)\n");
}

#[test]
fn a_script_is_checked_as_it_is_typed_not_as_it_was_saved() {
    let fs = project();
    let mut open = Documents::new();
    // Not open: the file on disk, which is right.
    assert_eq!(
        shown(&diagnostics(&fs, &open, Path::new(MAIN))),
        "nothing\n"
    );
    // Open, with a name misspelt: said at once, though nothing was saved.
    open.open(MAIN, "use functions::clean\nprint(twise(2))\n", 1);
    let found = diagnostics(&fs, &open, Path::new(MAIN));
    assert_eq!(found.len(), 1, "{}", shown(&found));
    assert_eq!(found[0].code, "Z0103");
    assert_eq!(found[0].level, Level::Error);
    assert_eq!(found[0].message, "no function is called `twise`");
    assert_eq!(found[0].label, "not a function of Zeff or of this project");
    assert_eq!(found[0].help, ["did you mean `twice`?"]);
    assert_eq!(
        found[0].range,
        Range {
            start: at(1, 6),
            end: at(1, 11)
        }
    );
    assert_eq!(
        fs.read_to_string(Path::new(MAIN)).expect("on disk"),
        "use functions::clean\nprint(twice(2))\n"
    );
    // Corrected: nothing. Closed: the file on disk again.
    assert!(open.change(
        Path::new(MAIN),
        "use functions::clean\nprint(twice(3))\n",
        2
    ));
    assert_eq!(
        shown(&diagnostics(&fs, &open, Path::new(MAIN))),
        "nothing\n"
    );
    assert!(open.change(Path::new(MAIN), "print(nope)\n", 3));
    assert_eq!(diagnostics(&fs, &open, Path::new(MAIN)).len(), 1);
    open.close(Path::new(MAIN));
    assert_eq!(
        shown(&diagnostics(&fs, &open, Path::new(MAIN))),
        "nothing\n"
    );
}

#[test]
fn a_column_after_a_letter_outside_the_basic_plane_is_counted_in_utf16() {
    let fs = project();
    // The semicolon is the 14th character of its line and begins at byte 17; the protocol
    // counts the face as two, so the editor is told column 14.
    let found = typed(&fs, MAIN, "let d = 1\nlet s = \"😀 π\"; print(d)\n");
    assert_eq!(found.len(), 1, "{}", shown(&found));
    assert_eq!(found[0].code, "Z0315");
    assert_eq!(
        found[0].range,
        Range {
            start: at(1, 14),
            end: at(1, 15)
        }
    );
    // A name misspelt after a face on the same line.
    let found = typed(
        &fs,
        MAIN,
        "let d = frame(x = [1.0, 2])\nprint(\"😀😀\", d.mean(xx))\n",
    );
    assert_eq!(found.len(), 1, "{}", shown(&found));
    assert_eq!(found[0].code, "Z0101");
    assert_eq!(
        found[0].range,
        Range {
            start: at(1, 21),
            end: at(1, 23)
        }
    );
}

#[test]
fn the_messages_of_a_script_of_a_project() {
    let fs = project();
    let scripts = [
        // A statement half typed: the brackets that are open.
        "let d = frame(x = [1.0, 2\n",
        // Half typed, at an operator and at a dot.
        "let x = 1 +\n",
        "let d = import(\"data/small.csv\")\nd.\n",
        // A call half typed inside a block.
        "for i in 1..3 {\n    print(i\n}\n",
        // A variable that the file does not hold, with the suggestion; a function misspelt.
        "let d = import(\"data/small.csv\")\nd.mean(aa)\nd.maen(a)\n",
        // The marks of section 11: a step whose columns the data decide, and a file that is
        // not there yet.
        "let d = import(\"data/small.csv\")\nlet wide = d.to_wide(id: a, names_from: c, values_from: b)\nlet m = import(\"data/none.csv\")\nm.mean(zzz)\n",
        // Diagnostics that point at two places.
        "/// One.\nfn f(x: Float) -> Float { x }\n/// Two.\nfn f(x: Float) -> Float { x + 1 }\nlet d = import(\"data/small.csv\")\nlet a = 3\nd.add(k = a)\nlet y = if d.n_rows() > 2 { 1.5 } else { \"few\" }\n",
        // Warnings: a binding that hides a function, and a function without its summary.
        "let mean = 3\nfn half(x: Float) -> Float { x / 2 }\nprint(half(mean))\n",
        // The project's function with the wrong type.
        "use functions::clean\nprint(twice(\"two\"))\n",
        // Nothing wrong.
        "use functions::clean\nlet d = import(\"data/small.csv\")\nprint(twice(d.mean(b)))\n",
    ];
    let mut all = String::new();
    for script in scripts {
        all.push_str(&format!("> {}", script.replace('\n', "\n> ")));
        all.truncate(all.len() - 2);
        all.push_str(&shown(&typed(&fs, MAIN, script)));
        all.push_str("----\n");
    }
    insta::assert_snapshot!("script_of_a_project", all);
}

#[test]
fn a_module_is_checked_and_the_script_sees_it_as_it_is_typed() {
    let fs = project();
    let mut open = Documents::new();
    // The module as saved is right.
    assert_eq!(
        shown(&diagnostics(&fs, &open, Path::new(CLEAN))),
        "nothing\n"
    );
    // Its function renamed in the editor, not saved: the module is still right, and the
    // script, which calls the old name, is told so.
    open.open(CLEAN, module("double"), 1);
    assert_eq!(
        shown(&diagnostics(&fs, &open, Path::new(CLEAN))),
        "nothing\n"
    );
    let script = diagnostics(&fs, &open, Path::new(MAIN));
    assert_eq!(script.len(), 1, "{}", shown(&script));
    assert_eq!(script[0].message, "no function is called `twice`");
    // A fault typed into the module is the module's, at its place there; the script shows
    // only what is its own.
    open.change(
        Path::new(CLEAN),
        module("twice").replace("x * 2", "y * 2"),
        2,
    );
    let found = diagnostics(&fs, &open, Path::new(CLEAN));
    assert_eq!(found.len(), 1, "{}", shown(&found));
    assert_eq!(found[0].code, "Z0101");
    assert_eq!(found[0].message, "nothing is called `y` here");
    assert_eq!(found[0].help, ["did you mean `x`?"]);
    assert_eq!(
        found[0].range,
        Range {
            start: at(9, 4),
            end: at(9, 5)
        }
    );
    assert_eq!(
        shown(&diagnostics(&fs, &open, Path::new(MAIN))),
        "nothing\n"
    );
    // A function without its page, half typed.
    open.change(
        Path::new(CLEAN),
        "fn f(x: Float) -> Float {\n    x * \n}\n",
        3,
    );
    let found = diagnostics(&fs, &open, Path::new(CLEAN));
    let codes: Vec<&str> = found.iter().map(|finding| finding.code.as_str()).collect();
    assert!(codes.contains(&"Z0319"), "{}", shown(&found));
    assert!(codes.contains(&"Z0301"), "{}", shown(&found));
    // Closed: the module on disk, and the script is right again.
    open.close(Path::new(CLEAN));
    assert_eq!(
        shown(&diagnostics(&fs, &open, Path::new(CLEAN))),
        "nothing\n"
    );
    assert_eq!(
        shown(&diagnostics(&fs, &open, Path::new(MAIN))),
        "nothing\n"
    );
}

#[test]
fn a_module_opened_and_not_yet_saved_is_a_module_of_the_project() {
    let fs = project();
    let mut open = Documents::new();
    let new = "/work/demo/functions/more.zeff";
    open.open(
        new,
        module("thrice")
            .replace("x * 2", "x * 3")
            .replace("/// 4", "/// 6"),
        1,
    );
    assert_eq!(shown(&diagnostics(&fs, &open, Path::new(new))), "nothing\n");
    open.open(MAIN, "use functions::more\nprint(thrice(2))\n", 1);
    assert_eq!(
        shown(&diagnostics(&fs, &open, Path::new(MAIN))),
        "nothing\n"
    );
    // Without the module open there is no such module.
    open.close(Path::new(new));
    let found = diagnostics(&fs, &open, Path::new(MAIN));
    assert!(!found.is_empty());
    assert!(found.iter().all(|finding| finding.level == Level::Error));
}

#[test]
fn a_file_outside_a_project_is_checked_alone() {
    let fs = project();
    let path = "/elsewhere/try.zeff";
    // Its syntax, its names and its types; of the file it reads nothing is known and nothing
    // is said, and a variable of that frame is left to the run.
    let found = typed(
        &fs,
        path,
        "let d = import(\"data/small.csv\")\nd.mean(aa)\nlet f = frame(x = [1.0, 2])\nf.mean(xx)\nf.maen(x)\nprint(nope)\nlet wide = f.to_wide(id: x, names_from: x, values_from: x)\n",
    );
    insta::assert_snapshot!("file_outside_a_project", shown(&found));
    // Half typed.
    let found = typed(&fs, path, "let x = [1, 2\n");
    assert_eq!(found.len(), 1, "{}", shown(&found));
    assert_eq!(found[0].code, "Z0302");
    // The modules of a project are not its own.
    let found = typed(&fs, path, "use functions::clean\n");
    assert_eq!(found.len(), 1, "{}", shown(&found));
    // On disk and not open, it is read from the disk.
    fs.add_file("/elsewhere/saved.zeff", "print(nope)\n");
    let found = diagnostics(&fs, &Documents::new(), Path::new("/elsewhere/saved.zeff"));
    assert_eq!(found.len(), 1, "{}", shown(&found));
    assert_eq!(found[0].code, "Z0101");
}

#[test]
fn a_document_that_cannot_be_read_and_a_project_that_cannot_be_opened() {
    let fs = project();
    // Neither open nor on disk.
    assert!(
        diagnostics(
            &fs,
            &Documents::new(),
            Path::new("/work/demo/scripts/none.zeff")
        )
        .is_empty()
    );
    // Not text.
    fs.add_file("/work/demo/scripts/bytes.zeff", [0xff, 0xfe, 0x00]);
    assert!(
        diagnostics(
            &fs,
            &Documents::new(),
            Path::new("/work/demo/scripts/bytes.zeff")
        )
        .is_empty()
    );
    // A project whose `zeff.toml` is wrong: the script is checked alone, and still checked.
    fs.add_file("/broken/zeff.toml", "[project\n");
    let found = typed(&fs, "/broken/scripts/main.zeff", "print(nope)\n");
    assert_eq!(found.len(), 1, "{}", shown(&found));
    assert_eq!(found[0].code, "Z0101");
}

/// The sample projects: every script and module, as an editor that opens it would show it.
/// None has an error, and what each is told is a stored text.
#[test]
fn the_messages_of_the_sample_scripts() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../samples");
    let mut all = String::new();
    for name in [
        "first_engine",
        "data_cleaning",
        "regression",
        "simulation",
        "user_functions",
        "panel",
        "plots",
    ] {
        let folder = root.join(name);
        let fs = MemoryFileSystem::new();
        fs.set_clock(1_000);
        let mut documents = Vec::new();
        let copy = |relative: &str| {
            if let Ok(bytes) = std::fs::read(folder.join(relative)) {
                fs.add_file(format!("/s/{relative}"), bytes);
            }
        };
        copy("zeff.toml");
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
                copy(&format!("{sub}/{file}"));
                if sub != "data" && file.ends_with(".zeff") {
                    documents.push(format!("{sub}/{file}"));
                }
            }
        }
        assert!(!documents.is_empty(), "{name} has no script");
        for document in documents {
            let found = diagnostics(&fs, &Documents::new(), &Path::new("/s").join(&document));
            assert!(
                found.iter().all(|finding| finding.level != Level::Error),
                "{name}/{document}:\n{}",
                shown(&found)
            );
            all.push_str(&format!("==== {name}/{document}\n{}", shown(&found)));
        }
    }
    insta::assert_snapshot!("sample_scripts", all);
}
