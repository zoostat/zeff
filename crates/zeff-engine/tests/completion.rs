//! Completion, part 1 (release 0.7.0, step 35a; Engine and Project Specification, section 12):
//! after a dot, what may be written as a method of the value before it, by the value's type;
//! at the start of a word, the names in reach. A corpus of cursor places, each a text with `|`
//! where the cursor stands, is a stored text.

use std::path::Path;
use zeff_engine::VERSION;
use zeff_engine::fs::MemoryFileSystem;
use zeff_engine::language::{
    CompletionKind, Completions, Documents, Position, Range, completions, diagnostics, position,
};

const MAIN: &str = "/work/demo/scripts/main.zeff";
const CLEAN: &str = "/work/demo/functions/clean.zeff";

/// A module with a function of a number and a function of a frame, each with its page.
const MODULE: &str = "/// Twice a number.
///
/// The number doubled.
///
/// x: The number.
/// returns: The number times two, a Float.
/// > twice(2.0)
/// 4
fn twice(x: Float) -> Float {
    x * 2
}

/// The share of rows a variable is missing.
///
/// The missing values of the variable over the rows of the frame.
///
/// data: The frame.
/// var: The variable.
/// returns: The share, a Float.
/// > frame(x = [1.0, NA]).share_gone(x)
/// 0.5
fn share_gone(data: Frame, var: Var) -> Float {
    let n = data.n_rows()
    data.n_missing(var) / n
}
";

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
    fs.add_file(CLEAN, MODULE);
    fs
}

/// The completions at the `|` of a text, typed into a document.
fn at_cursor(fs: &MemoryFileSystem, path: &str, marked: &str) -> Completions {
    assert_eq!(marked.matches('|').count(), 1, "one cursor: {marked}");
    let cursor = marked.find('|').expect("a cursor");
    let text = format!("{}{}", &marked[..cursor], &marked[cursor + 1..]);
    let mut open = Documents::new();
    open.open(path, text.clone(), 1);
    completions(fs, &open, Path::new(path), position(&text, cursor))
}

fn labels(found: &Completions) -> Vec<&str> {
    found.items.iter().map(|item| item.label.as_str()).collect()
}

/// The completions of a place as a text: the word replaced, the labels by kind, and each
/// completion in full where they are few.
fn shown(fs: &MemoryFileSystem, path: &str, marked: &str) -> String {
    let found = at_cursor(fs, path, marked);
    let mut out = format!("> {}", marked.replace('\n', "\n> "));
    out.truncate(out.trim_end_matches(['>', ' ']).len());
    out.push_str(&format!(
        "replace {}:{}-{}:{}, {} offered\n",
        found.replace.start.line,
        found.replace.start.character,
        found.replace.end.line,
        found.replace.end.character,
        found.items.len()
    ));
    for kind in [
        CompletionKind::Field,
        CompletionKind::Method,
        CompletionKind::Function,
        CompletionKind::Binding,
        CompletionKind::Dataset,
    ] {
        let of_kind: Vec<&str> = found
            .items
            .iter()
            .filter(|item| item.kind == kind)
            .map(|item| item.label.as_str())
            .collect();
        if !of_kind.is_empty() {
            out.push_str(&format!(
                "  {kind:?} ({}): {}\n",
                of_kind.len(),
                of_kind.join(" ")
            ));
        }
    }
    if found.items.len() <= 6 {
        for item in &found.items {
            out.push_str(&format!(
                "    {} | {} | {}\n",
                item.label, item.detail, item.summary
            ));
        }
    }
    out
}

#[test]
fn the_corpus_of_cursor_places() {
    let fs = project();
    let data = "let d = import(\"data/small.csv\")\n";
    let places: Vec<(&str, String)> = vec![
        // After a dot, by the type of the value: a frame; a frame after a step of a chain; a
        // word begun.
        (MAIN, format!("{data}d.|\n")),
        (MAIN, format!("{data}let m = d.where(a > 1).|\n")),
        (MAIN, format!("{data}d.me|\n")),
        // A number, a text, a date, a list, a record, a mapping.
        (MAIN, format!("{data}let m = d.mean(a)\nm.|\n")),
        (MAIN, "let s = \"text\"\ns.|\n".to_string()),
        (MAIN, "let day = date(\"2024-03-01\")\nday.|\n".to_string()),
        (MAIN, "let xs = [1.0, 2, 3]\nxs.|\n".to_string()),
        (MAIN, "let r = {n: 3, name: \"x\"}\nr.|\n".to_string()),
        (MAIN, "let r = {n: 3, name: \"x\"}\nr.n|\n".to_string()),
        // A column, inside a frame's method and bound.
        (MAIN, format!("{data}d.add(k = a.|)\n")),
        (MAIN, format!("{data}let col = d[\"b\"]\ncol.|\n")),
        // A fit, a table of a fit, a test, a plot, a distribution.
        (MAIN, format!("{data}let fit = d.reg(b ~ a)\nfit.|\n")),
        (MAIN, format!("{data}let fit = d.reg(b ~ a)\nfit.coef().|\n")),
        (MAIN, format!("{data}d.t_test(b, mu: 3).|\n")),
        (MAIN, format!("{data}d.plot(x: a, y: b).|\n")),
        (MAIN, "let z = dist.normal(0, 1)\nz.|\n".to_string()),
        // A value the checker cannot type.
        (MAIN, "nothing.|\n".to_string()),
        // A chain broken over lines; brackets not yet closed; a call already written after
        // the word.
        (MAIN, format!("{data}d\n    .where(a > 1)\n    .ag|\n")),
        (MAIN, format!("{data}print(d.mean(a), d.n_|\n")),
        (MAIN, format!("{data}let xs = [d.n_rows(), d.n_c|\n")),
        (MAIN, format!("{data}d.me|an(a)\n")),
        // With the rest of the document as it is, statements after the place included.
        (MAIN, format!("{data}let k = d.n_|\nprint(k)\nlet later = [1, 2\n")),
        // A namespace of the engine's functions.
        (MAIN, "let z = dist.|\n".to_string()),
        (MAIN, "let z = dist.no|\n".to_string()),
        // The project's functions after a dot: the one that takes the value first.
        (MAIN, format!("use functions::clean\n{data}d.sh|\n")),
        (MAIN, "use functions::clean\nlet x = 2.0\nx.tw|\n".to_string()),
        // Not brought in by `use`: not offered.
        (MAIN, "let x = 2.0\nx.tw|\n".to_string()),
        // No completion: a number, a range, a text between quotes, a comment.
        (MAIN, "let x = 1.|\n".to_string()),
        (MAIN, "for i in 1..|\n".to_string()),
        (MAIN, "let s = \"d.|\"\n".to_string()),
        (MAIN, "// d.|\n".to_string()),
        (MAIN, "let n = 12|\n".to_string()),
        // At a word: the shipped datasets inside `dataset(`.
        (MAIN, "let d = dataset(|\n".to_string()),
        (MAIN, "let d = dataset(wa|)\n".to_string()),
        // At a word: the names in reach, the project's functions, the engine's.
        (MAIN, "use functions::clean\nlet y = tw|\n".to_string()),
        (
            MAIN,
            "let wages = dataset(wages)\nlet wide = 3\nfor i in 1..3 {\n    let within = i\n    print(w|)\n}\n"
                .to_string(),
        ),
        // After the loop its variable and what it bound are out of reach.
        (
            MAIN,
            "let wide = 3\nfor index in 1..3 {\n    let inner = index\n}\nprint(in|)\n".to_string(),
        ),
        // Of two of one name the later, with its type.
        (
            MAIN,
            "let total = 3\nlet total = \"three\"\nprint(tota|)\n".to_string(),
        ),
        // Inside a function written with a name: its parameters and its own bindings, not
        // the script's.
        (
            MAIN,
            "let alone = 1\nfn local(alpha: Float, also: Int) -> Float {\n    let always = alpha\n    al|\n}\n"
                .to_string(),
        ),
        // A function literal sees what is bound where it is written.
        (
            MAIN,
            "let top = 1\nlet f = fn(tick) tick + t|\n".to_string(),
        ),
        // In a module, and in a file outside any project.
        (
            CLEAN,
            "/// T.\nfn share_gone(data: Frame, var: Var) -> Float {\n    let n = data.n_|\n    1.0\n}\n"
                .to_string(),
        ),
        (
            CLEAN,
            "/// T.\nfn share_gone(data: Frame, var: Var) -> Float {\n    let n = data.n_rows()\n    d|\n}\n"
                .to_string(),
        ),
        (
            "/elsewhere/try.zeff",
            "let d = frame(x = [1.0, 2])\nd.n_|\n".to_string(),
        ),
    ];
    let mut all = String::new();
    for (path, marked) in &places {
        if *path != MAIN {
            all.push_str(&format!("in {path}\n"));
        }
        all.push_str(&shown(&fs, path, marked));
        all.push_str("----\n");
    }
    insta::assert_snapshot!("corpus", all);
}

#[test]
fn after_a_frame_the_methods_of_a_frame_and_no_others() {
    let fs = project();
    let found = at_cursor(&fs, MAIN, "let d = import(\"data/small.csv\")\nd.|\n");
    let names = labels(&found);
    for method in ["where", "mean", "reg", "summary", "to_long", "plot", "save"] {
        assert!(names.contains(&method), "{method} is not offered");
    }
    // A function of a number, a method of a fit, a method of a plot, a function of a
    // namespace: none is written after a frame.
    for other in ["sqrt", "coef", "facet", "upper", "dist.normal", "normal"] {
        assert!(!names.contains(&other), "{other} is offered");
    }
    // The methods come first, each kind in the order of the letters.
    let kinds: Vec<CompletionKind> = found.items.iter().map(|item| item.kind).collect();
    assert!(kinds.is_sorted());
    let methods: Vec<&str> = found
        .items
        .iter()
        .filter(|item| item.kind == CompletionKind::Method)
        .map(|item| item.label.as_str())
        .collect();
    assert!(methods.is_sorted() && methods.len() > 40);
    // Each with its usage line and the summary of its page.
    let mean = found
        .items
        .iter()
        .find(|item| item.label == "mean")
        .expect("mean");
    assert!(
        mean.detail.starts_with("data.mean(var, by: Vars"),
        "{}",
        mean.detail
    );
    assert_eq!(mean.summary, "Mean of a variable");
    // Nothing is replaced where no word is begun.
    assert_eq!(
        found.replace,
        Range {
            start: Position {
                line: 1,
                character: 2
            },
            end: Position {
                line: 1,
                character: 2
            }
        }
    );
}

#[test]
fn a_column_is_offered_the_functions_of_its_values() {
    let fs = project();
    // A function of a number takes each row of a column of numbers (section 3.7, "Columns
    // and scalars"), so `sqrt` follows a column; a method of a frame does not.
    let found = at_cursor(
        &fs,
        MAIN,
        "let d = import(\"data/small.csv\")\nd.add(k = b.|)\n",
    );
    let names = labels(&found);
    for function in ["sqrt", "log", "round", "lag", "is_missing"] {
        assert!(names.contains(&function), "{function} is not offered");
    }
    for other in ["where", "reg", "upper", "coef"] {
        assert!(!names.contains(&other), "{other} is offered");
    }
}

/// What is offered after a value can be written there: the checker knows each name, and takes
/// the value as its first argument. (The arguments that follow are not given here, and their
/// absence is not what this test is about.)
#[test]
fn what_is_offered_after_a_value_is_accepted_by_the_checker() {
    let fs = project();
    let values = [
        ("let v = import(\"data/small.csv\")\n", 40),
        ("let v = 2.5\n", 30),
        ("let v = \"text\"\n", 20),
        ("let v = [1.0, 2, 3]\n", 10),
        ("let v = import(\"data/small.csv\").reg(b ~ a)\n", 15),
        ("let v = import(\"data/small.csv\").plot(x: a, y: b)\n", 15),
        ("let v = date(\"2024-03-01\")\n", 5),
    ];
    for (bound, at_least) in values {
        let found = at_cursor(&fs, MAIN, &format!("use functions::clean\n{bound}v.|\n"));
        assert!(
            found.items.len() >= at_least,
            "{bound}: {}",
            found.items.len()
        );
        for item in &found.items {
            let text = format!("use functions::clean\n{bound}v.{}()\n", item.label);
            let mut open = Documents::new();
            open.open(MAIN, text.clone(), 1);
            for finding in diagnostics(&fs, &open, Path::new(MAIN)) {
                assert!(
                    finding.code != "Z0103" && !finding.message.contains("cannot be called on"),
                    "{}: {}",
                    text,
                    finding.message
                );
            }
        }
    }
}

#[test]
fn the_word_at_the_cursor_is_what_a_completion_replaces() {
    let fs = project();
    // The letters before the cursor choose; those after it are replaced with them.
    let found = at_cursor(
        &fs,
        MAIN,
        "let d = import(\"data/small.csv\")\nd.me|an(a)\n",
    );
    assert_eq!(labels(&found), ["mean", "median", "merge"]);
    assert_eq!(
        found.replace,
        Range {
            start: Position {
                line: 1,
                character: 2
            },
            end: Position {
                line: 1,
                character: 6
            }
        }
    );
    // Counted in UTF-16 after a letter outside the basic plane.
    let found = at_cursor(&fs, MAIN, "let s = \"😀\"; s.up|\n");
    assert_eq!(
        found.replace,
        Range {
            start: Position {
                line: 0,
                character: 16
            },
            end: Position {
                line: 0,
                character: 18
            }
        }
    );
    // Capitals and small letters are told apart.
    assert!(
        at_cursor(&fs, MAIN, "let x = PI|\n")
            .items
            .iter()
            .any(|item| item.label == "PI")
    );
    assert!(
        at_cursor(&fs, MAIN, "let x = pi|\n")
            .items
            .iter()
            .all(|item| item.label != "PI")
    );
}

#[test]
fn a_place_with_no_completion_and_a_document_that_cannot_be_read() {
    let fs = project();
    for marked in [
        "let x = 1.|\n",
        "let x = 12|\n",
        "for i in 1..|\n",
        "let s = \"d.|\"\n",
        "let s = \"a \\\" b.|\"\n",
        "// d.|\n",
        "let x = 1 // then x.|\n",
        "/// A page: data.|\nfn f(data: Frame) -> Int { data.n_rows() }\n",
    ] {
        assert_eq!(
            labels(&at_cursor(&fs, MAIN, marked)),
            Vec::<&str>::new(),
            "{marked}"
        );
    }
    // After a text between quotes that is closed, there is one.
    assert!(!at_cursor(&fs, MAIN, "let s = \"d.\".|\n").items.is_empty());
    // A document neither open nor on disk; a place past the end of one that is.
    let none = completions(
        &fs,
        &Documents::new(),
        Path::new("/work/demo/scripts/none.zeff"),
        Position {
            line: 0,
            character: 0,
        },
    );
    assert!(none.items.is_empty());
    let mut open = Documents::new();
    open.open(MAIN, "let d = import(\"data/small.csv\")\nd.", 1);
    let far = completions(
        &fs,
        &open,
        Path::new(MAIN),
        Position {
            line: 99,
            character: 99,
        },
    );
    assert!(far.items.iter().any(|item| item.label == "where"));
}

#[test]
fn the_text_in_the_editor_is_what_is_completed() {
    let fs = project();
    // The module renamed in the editor, not saved: the script is offered the new name.
    let mut open = Documents::new();
    open.open(CLEAN, MODULE.replace("twice", "double"), 2);
    let text = "use functions::clean\nlet x = 2.0\nx.";
    open.open(MAIN, text, 1);
    let found = completions(&fs, &open, Path::new(MAIN), position(text, text.len()));
    let names = labels(&found);
    assert!(
        names.contains(&"double") && !names.contains(&"twice"),
        "{names:?}"
    );
    // And a private function of a module is its own.
    open.change(Path::new(CLEAN), MODULE.replace("twice", "_twice"), 3);
    let found = completions(&fs, &open, Path::new(MAIN), position(text, text.len()));
    assert!(!labels(&found).contains(&"_twice"));
}
