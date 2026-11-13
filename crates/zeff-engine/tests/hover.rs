//! Hover, part 1 (release 0.7.0, step 36a; Engine and Project Specification, section 12):
//! on the name of a function its help page, on a variable of a frame its type, its label and
//! its count of missing values. A corpus of cursor places, each a text with `|` where the
//! cursor stands, is a stored text.

use std::path::Path;
use zeff_engine::VERSION;
use zeff_engine::fs::MemoryFileSystem;
use zeff_engine::language::{Documents, Hover, HoverKind, Position, Range, hover, position};

const MAIN: &str = "/work/demo/scripts/main.zeff";
const CLEAN: &str = "/work/demo/functions/clean.zeff";

const MODULE: &str = "/// The share of rows a variable is missing.
///
/// The missing values of the variable over the rows of the frame.
///
/// data: The frame.
/// var: The variable.
/// returns: The share, a Float.
/// > frame(x = [1.0, NA]).share_gone(x)
/// 0.5
fn share_gone(data: Frame, var: Var) -> Float {
    let values = data[name_of(var)]
    n_missing(values) / data.n_rows()
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
    assert_eq!(
        zeff_engine::language::diagnostics(&fs, &Documents::new(), Path::new(CLEAN)),
        Vec::new()
    );
    fs
}

/// The hover at the `|` of a text, typed into a document.
fn at_cursor(fs: &MemoryFileSystem, path: &str, marked: &str) -> Option<Hover> {
    assert_eq!(marked.matches('|').count(), 1, "one cursor: {marked}");
    let cursor = marked.find('|').expect("a cursor");
    let text = format!("{}{}", &marked[..cursor], &marked[cursor + 1..]);
    let mut open = Documents::new();
    open.open(path, text.clone(), 1);
    hover(fs, &open, Path::new(path), position(&text, cursor))
}

/// The hover as a text: the word marked, what it is, and what is shown; of a help page its
/// first lines and how many follow.
fn shown(fs: &MemoryFileSystem, path: &str, marked: &str) -> String {
    let mut out = format!("> {}", marked.replace('\n', "\n> "));
    out.truncate(out.trim_end_matches(['>', ' ']).len());
    if !out.ends_with('\n') {
        out.push('\n');
    }
    let Some(found) = at_cursor(fs, path, marked) else {
        out.push_str("nothing\n");
        return out;
    };
    let kind = match &found.kind {
        HoverKind::Function => "a function",
        HoverKind::Variable(_) => "a variable",
    };
    out.push_str(&format!(
        "{}:{}-{}:{}, {kind}\n",
        found.range.start.line,
        found.range.start.character,
        found.range.end.line,
        found.range.end.character
    ));
    let lines: Vec<&str> = found.text.lines().collect();
    let few = if matches!(found.kind, HoverKind::Function) {
        4
    } else {
        9
    };
    for line in lines.iter().take(few) {
        out.push_str(&format!("    {line}\n"));
    }
    if lines.len() > few {
        out.push_str(&format!("    (and {} lines more)\n", lines.len() - few));
    }
    out
}

#[test]
fn the_corpus_of_cursor_places() {
    let fs = project();
    let data = "let d = import(\"data/small.csv\")\n";
    let places: Vec<(&str, String)> = vec![
        // The name of a function: a method, at its start, inside it and at its end.
        (MAIN, format!("{data}d.|mean(a)\n")),
        (MAIN, format!("{data}d.me|an(a)\n")),
        (MAIN, format!("{data}d.mean|(a)\n")),
        // Before its brackets are typed, and in a chain broken over lines.
        (MAIN, format!("{data}d.summ|ary\n")),
        (
            MAIN,
            format!("{data}d\n    .where(a > 1)\n    .re|g(b ~ a)\n"),
        ),
        // A function written before its value, one of a namespace, one with its package.
        (MAIN, "let x = sq|rt(2.0)\n".to_string()),
        (MAIN, "let z = dist.nor|mal(0, 1)\n".to_string()),
        (MAIN, format!("{data}data::s|d(d, a)\n")),
        // A name two functions share: by the value it is called on, or its first argument.
        (MAIN, format!("{data}d.histo|gram(a)\n")),
        (MAIN, format!("{data}d.plot(x: a).histo|gram()\n")),
        (
            MAIN,
            format!("{data}let p = d.plot(x: a)\nhisto|gram(p, bins: 5)\n"),
        ),
        // A function named and not called.
        (MAIN, "let roots = map([1.0, 4], sq|rt)\n".to_string()),
        // A function of the project: after a value, before it, with its module, and in its
        // own definition.
        (
            MAIN,
            format!("use functions::clean\n{data}d.share_|gone(b)\n"),
        ),
        (
            MAIN,
            format!("use functions::clean\n{data}share_|gone(d, b)\n"),
        ),
        (MAIN, format!("{data}clean::share_|gone(d, b)\n")),
        (CLEAN, MODULE.replace("fn share_gone(", "fn share_|gone(")),
        // A function inside an argument of a frame's method.
        (MAIN, format!("{data}d.add(k = lo|g(b))\n")),
        // A variable of a frame: in an argument, a formula, a condition, a definition.
        (MAIN, format!("{data}d.mean(|b)\n")),
        (MAIN, format!("{data}d.reg(b ~ |a, cluster: c)\n")),
        (MAIN, format!("{data}d.reg(b ~ a, cluster: c|)\n")),
        (MAIN, format!("{data}d.where(a > 1 & b| < 3)\n")),
        (MAIN, format!("{data}d.add(k = log(b|) + a)\n")),
        // At that point of the chain: under its new name, and no longer under its old.
        (MAIN, format!("{data}d.rename(z = a).mean(z|)\n")),
        (MAIN, format!("{data}d.rename(z = a).mean(a|)\n")),
        // What else is known of it: a shipped dataset's label and count of missing values;
        // through a step that keeps every row, and after one that does not.
        (MAIN, "dataset(wages).reg(log(wa|ge) ~ educ)\n".to_string()),
        (MAIN, "dataset(wages).mean(indus|try)\n".to_string()),
        (
            MAIN,
            "dataset(wages).where(year == 2015).mean(wa|ge)\n".to_string(),
        ),
        (
            MAIN,
            "dataset(wages).label(wage, \"Pay by the hour\").mean(wa|ge)\n".to_string(),
        ),
        // Nothing: a word that is neither, a number, a word of a fixed list, the name of an
        // argument, a comment, a text between quotes, no word at all.
        (MAIN, format!("{data}let k = 3\nprint(k|)\n")),
        (MAIN, "let x = 12|3\n".to_string()),
        (MAIN, format!("{data}d.reg(b ~ a, se: rob|ust)\n")),
        (MAIN, format!("{data}d.reg(b ~ a, clus|ter: c)\n")),
        (MAIN, "// sq|rt(2)\n".to_string()),
        (MAIN, "let s = \"sq|rt\"\n".to_string()),
        (MAIN, "let x = 1 + | 2\n".to_string()),
        (MAIN, "nosu|ch(1)\n".to_string()),
        // In a file outside any project.
        (
            "/elsewhere/try.zeff",
            "let d = frame(x = [1.0, NA])\nd.mean(|x)\n".to_string(),
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
fn on_a_function_its_help_page_as_zeff_help_prints_it() {
    let fs = project();
    let registry = zeff_engine::registry::registry().expect("the registry");
    let data = "let d = import(\"data/small.csv\")\n";
    let found = at_cursor(&fs, MAIN, &format!("{data}d.me|an(a)\n")).expect("a hover");
    assert_eq!(found.kind, HoverKind::Function);
    assert_eq!(
        found.text,
        zeff_engine::help::help_text(registry, "mean").expect("the page")
    );
    // The word marked is the name, wherever in it the cursor stands.
    let name = Range {
        start: Position {
            line: 1,
            character: 2,
        },
        end: Position {
            line: 1,
            character: 6,
        },
    };
    assert_eq!(found.range, name);
    for marked in ["d.|mean(a)", "d.mean|(a)", "d.m|ean(a)"] {
        let found = at_cursor(&fs, MAIN, &format!("{data}{marked}\n")).expect("a hover");
        assert_eq!(found.range, name, "{marked}");
    }
    // A function of a namespace is marked with its namespace.
    let found = at_cursor(&fs, MAIN, "let z = dist.nor|mal(0, 1)\n").expect("a hover");
    assert_eq!(
        found.range,
        Range {
            start: Position {
                line: 0,
                character: 8
            },
            end: Position {
                line: 0,
                character: 19
            },
        }
    );
    assert!(found.text.starts_with("dist.normal — "), "{}", found.text);
    // Of two functions of one name, the one that takes the value: a frame, or a plot.
    let of_a_frame = at_cursor(&fs, MAIN, &format!("{data}d.histo|gram(a)\n")).expect("a hover");
    let of_a_plot =
        at_cursor(&fs, MAIN, &format!("{data}d.plot(x: a).histo|gram()\n")).expect("a hover");
    assert!(
        of_a_frame.text.contains("data.histogram(var"),
        "{}",
        of_a_frame.text
    );
    assert!(
        of_a_plot.text.contains("p.histogram("),
        "{}",
        of_a_plot.text
    );
    let before = at_cursor(
        &fs,
        MAIN,
        &format!("{data}let p = d.plot(x: a)\nhisto|gram(p, bins: 5)\n"),
    )
    .expect("a hover");
    assert_eq!(before.text, of_a_plot.text);
    // A function of the project: its own page, as `zeff help` prints it in the project.
    let project =
        zeff_engine::project::Project::open(&fs, Path::new("/work/demo")).expect("the project");
    let page =
        zeff_engine::help::project_help(&fs, &project, registry, "share_gone").expect("its page");
    let found = at_cursor(
        &fs,
        MAIN,
        &format!("use functions::clean\n{data}d.share_|gone(b)\n"),
    )
    .expect("a hover");
    assert_eq!(found.text, page);
    // Not brought in, it is no function here by its short name.
    assert_eq!(
        at_cursor(&fs, MAIN, &format!("{data}d.share_|gone(b)\n")),
        None
    );
}

#[test]
fn on_a_variable_what_completion_says_of_it() {
    let fs = project();
    let found = at_cursor(&fs, MAIN, "dataset(wages).reg(log(wa|ge) ~ educ)\n").expect("a hover");
    assert_eq!(found.text, "wage: Float\nHourly wage\n211 missing");
    let HoverKind::Variable(facts) = &found.kind else {
        panic!("not a variable: {found:?}");
    };
    assert_eq!(
        (
            facts.ty.as_str(),
            facts.label.as_str(),
            facts.missing,
            facts.from_last_run
        ),
        ("Float", "Hourly wage", Some(211), false)
    );
    assert_eq!(
        found.range,
        Range {
            start: Position {
                line: 0,
                character: 23
            },
            end: Position {
                line: 0,
                character: 27
            },
        }
    );
    // A category by the name of its type; a variable of a file never run by its type alone.
    let found = at_cursor(&fs, MAIN, "dataset(wages).mean(indus|try)\n").expect("a hover");
    assert_eq!(found.text, "industry: Category\nIndustry\n0 missing");
    let found = at_cursor(
        &fs,
        MAIN,
        "let d = import(\"data/small.csv\")\nd.mean(b|)\n",
    )
    .expect("a hover");
    assert_eq!(found.text, "b: Float");
    // A name that is both a variable of the frame and bound in the script is the variable,
    // as the checker has it inside a frame's method.
    let found =
        at_cursor(&fs, MAIN, "let wage = 3\ndataset(wages).mean(wa|ge)\n").expect("a hover");
    assert!(matches!(found.kind, HoverKind::Variable(_)));
    // A name the frame does not hold is no variable.
    assert_eq!(at_cursor(&fs, MAIN, "dataset(wages).mean(wa|gee)\n"), None);
}

#[test]
fn where_there_is_nothing_to_show() {
    let fs = project();
    for marked in [
        "let x = 12|3\n",
        "// sq|rt(2)\n",
        "let s = \"sq|rt\"\n",
        "let x = 1 + | 2\n",
        "nosu|ch(1)\n",
        "let k = 3\nprint(k|)\n",
        "|\n",
    ] {
        assert_eq!(at_cursor(&fs, MAIN, marked), None, "{marked}");
    }
    // A document neither open nor on disk.
    let none = hover(
        &fs,
        &Documents::new(),
        Path::new("/work/demo/scripts/none.zeff"),
        Position {
            line: 0,
            character: 0,
        },
    );
    assert_eq!(none, None);
}
