//! Signature help (release 0.7.0, step 35e; Engine and Project Specification, section 12):
//! for a place inside the brackets of a call, the usage line of the function, the parameter
//! the place is in, and that parameter's line of the help page. A corpus of cursor places,
//! each a text with `|` where the cursor stands, is a stored text.

use std::path::Path;
use zeff_engine::VERSION;
use zeff_engine::fs::MemoryFileSystem;
use zeff_engine::language::{Documents, Position, SignatureHelp, position, signature_help};

const MAIN: &str = "/work/demo/scripts/main.zeff";
const CLEAN: &str = "/work/demo/functions/clean.zeff";

const MODULE: &str = "/// The share of rows a variable is missing.
///
/// The missing values of the variable over the rows of the frame.
///
/// data: The frame.
/// var: The variable.
/// digits: How many decimals to keep.
/// returns: The share, a Float.
/// > frame(x = [1.0, NA]).share_gone(x)
/// 0.5
fn share_gone(data: Frame, var: Var, digits: Int = 3) -> Float {
    let values = data[name_of(var)]
    round(n_missing(values) / data.n_rows(), digits)
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

/// The signature help at the `|` of a text, typed into a document.
fn at_cursor(fs: &MemoryFileSystem, path: &str, marked: &str) -> Option<SignatureHelp> {
    assert_eq!(marked.matches('|').count(), 1, "one cursor: {marked}");
    let cursor = marked.find('|').expect("a cursor");
    let text = format!("{}{}", &marked[..cursor], &marked[cursor + 1..]);
    let mut open = Documents::new();
    open.open(path, text.clone(), 1);
    signature_help(fs, &open, Path::new(path), position(&text, cursor))
}

/// A stretch of a text by UTF-16 code units, as the ranges of the parameters count.
fn stretch(text: &str, (from, to): (u32, u32)) -> String {
    let units: Vec<u16> = text.encode_utf16().collect();
    String::from_utf16_lossy(&units[from as usize..to as usize])
}

/// The help as a text: the usage line with the parameter the place is in between `[[` and
/// `]]`, and that parameter's line of the page.
fn shown(fs: &MemoryFileSystem, path: &str, marked: &str) -> String {
    let mut out = format!("> {}", marked.replace('\n', "\n> "));
    out.truncate(out.trim_end_matches(['>', ' ']).len());
    if !out.ends_with('\n') {
        out.push('\n');
    }
    let Some(help) = at_cursor(fs, path, marked) else {
        out.push_str("no signature help\n");
        return out;
    };
    match help.active {
        Some(active) => {
            let parameter = &help.parameters[active];
            let units: Vec<u16> = help.usage.encode_utf16().collect();
            let (from, to) = (parameter.range.0 as usize, parameter.range.1 as usize);
            out.push_str(&format!(
                "{}[[{}]]{}\n",
                String::from_utf16_lossy(&units[..from]),
                String::from_utf16_lossy(&units[from..to]),
                String::from_utf16_lossy(&units[to..])
            ));
            out.push_str(&format!("    {}: {}\n", parameter.name, parameter.help));
        }
        None => out.push_str(&format!("{}\n    in no parameter\n", help.usage)),
    }
    out.push_str(&format!("    ({})\n", help.summary));
    out
}

#[test]
fn the_corpus_of_cursor_places() {
    let fs = project();
    let data = "let d = import(\"data/small.csv\")\n";
    let places: Vec<(&str, String)> = vec![
        // A method: each place in turn, and the same while a word is typed.
        (MAIN, format!("{data}d.reg(|\n")),
        (MAIN, format!("{data}d.reg(b ~ a|\n")),
        (MAIN, format!("{data}d.reg(b ~ a, |\n")),
        (MAIN, format!("{data}d.reg(b ~ a, c, ro|\n")),
        // A named argument, wherever it stands; and one the function has not.
        (MAIN, format!("{data}d.reg(b ~ a, cluster: |\n")),
        (MAIN, format!("{data}d.reg(b ~ a, level: 0.9, se: rob|)\n")),
        (MAIN, format!("{data}d.reg(b ~ a, nosuch: |\n")),
        // The same function written before its frame: the frame is its first argument.
        (MAIN, format!("{data}reg(|\n")),
        (MAIN, format!("{data}reg(d, |\n")),
        // More values than the function takes.
        (MAIN, format!("{data}d.n_rows(1, |\n")),
        // What a function takes beyond its parameters: further values, and definitions.
        (MAIN, "print(1, 2, |\n".to_string()),
        (MAIN, format!("{data}d.add(k = a * 2, m = |\n")),
        (MAIN, format!("{data}d.aggregate(m = mean(b), by: |\n")),
        // A call inside a call: the innermost; past its closing bracket, the outer again.
        (MAIN, format!("{data}print(d.mean(a, |\n")),
        (MAIN, format!("{data}print(d.mean(a), |\n")),
        (MAIN, format!("{data}d.add(k = round(a / 3, |\n")),
        // Inside a list and inside a text between quotes the place is still in its call.
        (MAIN, format!("{data}d.summary([a, |\n")),
        (MAIN, "let d = import(\"data/sm|\n".to_string()),
        // A function of a namespace, one written with its package, a name two functions
        // share.
        (MAIN, "let z = dist.normal(0, |\n".to_string()),
        (MAIN, format!("{data}data::sd(d, |\n")),
        (MAIN, format!("{data}d.histogram(a, |\n")),
        (MAIN, format!("{data}d.plot(x: a).histogram(|\n")),
        // A function of the project, with the lines of its `///` page: after a value,
        // before it, with its module.
        (MAIN, format!("use functions::clean\n{data}d.share_gone(|\n")),
        (MAIN, format!("use functions::clean\n{data}share_gone(d, b, |\n")),
        (MAIN, format!("{data}clean::share_gone(d, digits: |\n")),
        // A chain broken over lines.
        (MAIN, format!("{data}d\n    .where(a > 1)\n    .reg(b ~ a,\n         se: |\n")),
        // No help: outside a call, past its end, in a comment, in the brackets of a
        // definition, in brackets that are no call, for a function that is not one.
        (MAIN, format!("{data}d.me|\n")),
        (MAIN, format!("{data}d.mean(a)|\n")),
        (MAIN, format!("{data}// d.mean(|\n")),
        (MAIN, "fn local(alpha: Float, |\n".to_string()),
        (MAIN, "let x = (1 + |\n".to_string()),
        (MAIN, "nosuch(|\n".to_string()),
        // In a module, and in a file outside any project.
        (
            CLEAN,
            "/// T.\nfn share_gone(data: Frame, var: Var) -> Float {\n    round(data.n_rows() / 3, |\n}\n"
                .to_string(),
        ),
        ("/elsewhere/try.zeff", "let d = frame(x = [1.0, 2])\nd.mean(x, by: |\n".to_string()),
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
fn the_parameter_the_place_is_in() {
    let fs = project();
    let data = "let d = import(\"data/small.csv\")\n";
    let help = at_cursor(&fs, MAIN, &format!("{data}d.reg(b ~ a, se: |\n")).expect("help");
    assert_eq!(
        help.usage,
        "data.reg(formula, fe: Terms, se: classical | robust | hc2 | hc3, cluster: Var, weights: Var, weight_type: frequency | analytic | sampling = sampling, level: Float = 0.95)"
    );
    assert_eq!(
        help.summary,
        "Linear regression by least squares, with fixed effects"
    );
    // Every parameter of the usage line, the frame before the dot first, each where it
    // stands.
    let names: Vec<&str> = help.parameters.iter().map(|p| p.name.as_str()).collect();
    assert_eq!(
        names,
        [
            "data",
            "formula",
            "fe",
            "se",
            "cluster",
            "weights",
            "weight_type",
            "level"
        ]
    );
    let written: Vec<String> = help
        .parameters
        .iter()
        .map(|p| stretch(&help.usage, p.range))
        .collect();
    assert_eq!(
        written,
        [
            "data",
            "formula",
            "fe: Terms",
            "se: classical | robust | hc2 | hc3",
            "cluster: Var",
            "weights: Var",
            "weight_type: frequency | analytic | sampling = sampling",
            "level: Float = 0.95"
        ]
    );
    assert_eq!(help.active, Some(3));
    assert!(
        help.parameters[3]
            .help
            .starts_with("The kind of standard error")
    );
    // A value in its place: counted after the frame, which stands before the dot.
    let active = |marked: String| at_cursor(&fs, MAIN, &marked).expect("help").active;
    assert_eq!(active(format!("{data}d.reg(|\n")), Some(1));
    assert_eq!(active(format!("{data}d.reg(b ~ a, c|\n")), Some(2));
    // Written before its frame, the frame is the first value.
    assert_eq!(active(format!("{data}reg(|\n")), Some(0));
    assert_eq!(active(format!("{data}reg(d, b ~ |\n")), Some(1));
    // A name the function has not, and more values than it takes: in no parameter.
    assert_eq!(active(format!("{data}d.reg(b ~ a, nosuch: |\n")), None);
    assert_eq!(active(format!("{data}d.n_rows(1, |\n")), None);
    // Beyond the parameters of a function that takes more, the `...` it ends with.
    let help = at_cursor(&fs, MAIN, "print(1, 2, |\n").expect("help");
    let last = help.parameters.last().expect("a parameter");
    assert_eq!(
        (last.name.as_str(), help.active),
        ("...", Some(help.parameters.len() - 1))
    );
    assert_eq!(stretch(&help.usage, last.range), "...");
    let help = at_cursor(&fs, MAIN, &format!("{data}d.add(k = |\n")).expect("help");
    let last = help.parameters.last().expect("a parameter");
    assert_eq!(help.active, Some(help.parameters.len() - 1));
    assert!(stretch(&help.usage, last.range).starts_with("name = "));
}

#[test]
fn a_function_of_the_project_is_shown_by_its_own_page() {
    let fs = project();
    let data = "let d = import(\"data/small.csv\")\n";
    let help = at_cursor(
        &fs,
        MAIN,
        &format!("use functions::clean\n{data}d.share_gone(b, |\n"),
    )
    .expect("help");
    assert_eq!(
        help.usage,
        "share_gone(data: Frame, var: Var, digits: Int) -> Float"
    );
    assert_eq!(help.summary, "The share of rows a variable is missing.");
    let lines: Vec<(&str, String, &str)> = help
        .parameters
        .iter()
        .map(|p| {
            (
                p.name.as_str(),
                stretch(&help.usage, p.range),
                p.help.as_str(),
            )
        })
        .collect();
    assert_eq!(
        lines,
        [
            ("data", "data: Frame".to_string(), "The frame."),
            ("var", "var: Var".to_string(), "The variable."),
            (
                "digits",
                "digits: Int".to_string(),
                "How many decimals to keep."
            ),
        ]
    );
    // After the frame and one value, the third parameter.
    assert_eq!(help.active, Some(2));
    // The module as typed in the editor, not as saved.
    let mut open = Documents::new();
    open.open(CLEAN, MODULE.replace("digits", "places"), 2);
    let text = format!("use functions::clean\n{data}d.share_gone(b, ");
    open.open(MAIN, text.clone(), 1);
    let help =
        signature_help(&fs, &open, Path::new(MAIN), position(&text, text.len())).expect("help");
    assert_eq!(help.parameters[2].name, "places");
    // Not brought in by `use`, it is no function here by its short name.
    assert_eq!(
        at_cursor(&fs, MAIN, &format!("{data}share_gone(d, |\n")),
        None
    );
}

#[test]
fn where_there_is_no_help() {
    let fs = project();
    for marked in [
        "let x = |\n",
        "print(1)|\n",
        "// print(|\n",
        "let x = 1 // print(|\n",
        "fn local(alpha: Float, |\n",
        "let x = (1 + |\n",
        "nosuch(|\n",
        "let d = [1, |\n",
        "let r = {a: |\n",
    ] {
        assert_eq!(at_cursor(&fs, MAIN, marked), None, "{marked}");
    }
    // A document neither open nor on disk; a place past the end of one that is.
    let none = signature_help(
        &fs,
        &Documents::new(),
        Path::new("/work/demo/scripts/none.zeff"),
        Position {
            line: 0,
            character: 0,
        },
    );
    assert_eq!(none, None);
    let mut open = Documents::new();
    open.open(MAIN, "print(", 1);
    let far = signature_help(
        &fs,
        &open,
        Path::new(MAIN),
        Position {
            line: 9,
            character: 9,
        },
    );
    assert!(far.is_some());
}
