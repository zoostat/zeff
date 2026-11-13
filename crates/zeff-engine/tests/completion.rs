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
    // The module is sound: nothing is said of it.
    assert_eq!(
        diagnostics(&fs, &Documents::new(), Path::new(CLEAN)),
        Vec::new()
    );
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
        CompletionKind::Variable,
        CompletionKind::Word,
        CompletionKind::Argument,
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
    // Each in full where they are few; the words and the argument names always.
    let few = found.items.len() <= 6;
    for item in &found.items {
        let of_the_call = matches!(
            item.kind,
            CompletionKind::Variable | CompletionKind::Word | CompletionKind::Argument
        );
        if few || of_the_call {
            let written = if item.insert == item.label {
                String::new()
            } else {
                format!(" (written `{}`)", item.insert)
            };
            out.push_str(&format!(
                "    {}{written} | {} | {}\n",
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
            "let top = 1\nlet moved = map([1.0, 2], fn(tick) tick + t|)\n".to_string(),
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
fn the_corpus_of_cursor_places_inside_a_call() {
    let fs = project();
    let data = "let d = import(\"data/small.csv\")\n";
    let places: Vec<(&str, String)> = vec![
        // Where an argument begins: the names of the arguments not yet given, then the
        // names in reach.
        (MAIN, format!("{data}d.reg(b ~ a, |\n")),
        (MAIN, format!("{data}d.reg(b ~ a, cl|\n")),
        // Less those given, before the place and after it.
        (MAIN, format!("{data}d.reg(b ~ a, cluster: c, w|)\n")),
        (MAIN, format!("{data}d.reg(b ~ a, s|, se: robust)\n")),
        // After the name of an argument that takes one of a fixed list of words: those
        // words and nothing else.
        (MAIN, format!("{data}d.reg(b ~ a, se: |\n")),
        (MAIN, format!("{data}d.reg(b ~ a, se: h|)\n")),
        (MAIN, format!("{data}d.merge(d, on: a, join: |\n")),
        (MAIN, format!("{data}d.mean(a, weight_type: |)\n")),
        // An argument that takes a number or a word: the words, then the names in reach.
        (MAIN, format!("{data}let k = 12\nd.histogram(a, bins: |)\n")),
        // Words inside a list.
        (MAIN, format!("{data}d.summary([a, b], stats: [|\n")),
        (MAIN, format!("{data}d.summary([a, b], stats: [n, m|])\n")),
        // The first place of a method, and of the same function written before its value.
        (MAIN, format!("{data}d.mean(|\n")),
        (MAIN, format!("{data}mean(d, |\n")),
        (MAIN, format!("{data}mean(|\n")),
        // A function of a namespace, one written with its package, and a call inside a call.
        (MAIN, "let z = dist.normal(|\n".to_string()),
        (MAIN, format!("{data}data::sd(d, a, w|\n")),
        (MAIN, format!("{data}print(d.n_rows(), d.mean(a, |\n")),
        // A name several functions share: the one that takes the value, a frame or a plot.
        (MAIN, format!("{data}d.histogram(a, |\n")),
        (MAIN, format!("{data}d.plot(x: a).histogram(|\n")),
        (
            MAIN,
            format!("{data}let p = d.plot(x: a)\nhistogram(p, |\n"),
        ),
        // The project's functions: after a value, before it, and with their module.
        (
            MAIN,
            format!("use functions::clean\n{data}d.share_gone(|\n"),
        ),
        (
            MAIN,
            format!("use functions::clean\n{data}share_gone(d, v|\n"),
        ),
        (MAIN, "let y = clean::twice(|\n".to_string()),
        // Not where an argument begins: no argument names.
        (MAIN, format!("{data}d.mean(a + s|\n")),
        (MAIN, format!("{data}d.add(k = s|\n")),
        // A chain broken over lines.
        (
            MAIN,
            format!("{data}d\n    .where(a > 1)\n    .reg(b ~ a,\n         se: c|\n"),
        ),
        // A function that is not one, and brackets that are no call.
        (MAIN, "nosuch(wh|\n".to_string()),
        (MAIN, "let x = (wh|\n".to_string()),
        (MAIN, "fn local(wh|\n".to_string()),
    ];
    let mut all = String::new();
    for (path, marked) in &places {
        all.push_str(&shown(&fs, path, marked));
        all.push_str("----\n");
    }
    insta::assert_snapshot!("corpus_inside_a_call", all);
}

#[test]
fn the_corpus_of_cursor_places_where_a_variable_is_written() {
    let fs = project();
    let data = "let d = import(\"data/small.csv\")\n";
    let places: Vec<(&str, String)> = vec![
        // An argument that takes a variable, several, terms, a formula.
        (MAIN, format!("{data}d.mean(|\n")),
        (MAIN, format!("{data}d.mean(b|)\n")),
        (MAIN, format!("{data}d.summary([a, |\n")),
        (MAIN, format!("{data}d.reg(|\n")),
        (MAIN, format!("{data}d.reg(b ~ |\n")),
        (MAIN, format!("{data}d.reg(b ~ a + |, se: robust)\n")),
        (MAIN, format!("{data}d.reg(b ~ a, fe: |\n")),
        (MAIN, format!("{data}d.reg(b ~ a, cluster: |\n")),
        (MAIN, format!("{data}d.plot(x: a, y: |\n")),
        (MAIN, format!("{data}let other = d\nd.merge(other, on: |\n")),
        // A condition and a definition, at any depth of brackets and of calls inside them.
        (MAIN, format!("{data}d.where(|\n")),
        (MAIN, format!("{data}d.where(a > 1 & (b < 3 & |\n")),
        (MAIN, format!("{data}d.add(k = |\n")),
        (MAIN, format!("{data}d.add(k = log(|\n")),
        (MAIN, format!("{data}d.aggregate(m = mean(|), by: c)\n")),
        // At that point of the chain: after `where`, `add`, `rename`, `select`, `drop` and
        // `aggregate`.
        (MAIN, format!("{data}d.where(a > 1).mean(|\n")),
        (
            MAIN,
            format!("{data}d.add(k = a * 2, when = date(\"2024-03-01\")).mean(|\n"),
        ),
        (MAIN, format!("{data}d.rename(z = a).mean(|\n")),
        (MAIN, format!("{data}d.select([c, a]).mean(|\n")),
        (MAIN, format!("{data}d.drop([a]).mean(|\n")),
        (
            MAIN,
            format!("{data}d.aggregate(m = mean(b), n = n(), by: c).mean(|\n"),
        ),
        (
            MAIN,
            format!("{data}d\n    .where(a > 1)\n    .add(k = a * 2)\n    .reg(k ~ |\n"),
        ),
        // A frame bound under another name, a frame written out, a shipped dataset.
        (
            MAIN,
            format!("{data}let kept = d.where(a > 1).add(k = a * 2)\nkept.mean(|\n"),
        ),
        (
            MAIN,
            "frame(x = [1.0, 2], who = [\"a\", \"b\"], ok = [true, false]).mean(|\n".to_string(),
        ),
        (
            MAIN,
            "let w = dataset(wages)\nw.reg(log(wage) ~ |\n".to_string(),
        ),
        // What else is known of a variable without running (open decision #145). A shipped
        // dataset: the label and the count of missing values of each. Through a step that
        // keeps every row the counts stay; after one that does not, the labels alone.
        (MAIN, "dataset(wages).select([wage, hours, industry]).sort(wage).mean(|\n".to_string()),
        (MAIN, "dataset(wages).where(year == 2015).select([wage, hours, industry]).mean(|\n".to_string()),
        (MAIN, "dataset(wages).select([wage, hours]).head(100).mean(|\n".to_string()),
        // Under a new name, the same variable; made anew, another of which nothing is known.
        (MAIN, "dataset(wages).select([wage, hours]).rename(pay = wage).mean(|\n".to_string()),
        (MAIN, "dataset(wages).select([wage, hours]).add(lwage = log(wage), hours = hours / 40).mean(|\n".to_string()),
        // A label written in the script is known at once, and an empty one removes it.
        (MAIN, "dataset(wages).select([wage, hours]).label(wage, \"Pay by the hour\").label(hours, \"\").mean(|\n".to_string()),
        (MAIN, format!("{data}d.label(b, \"Score at entry\").mean(|\n")),
        // The groups of an aggregate keep their labels; what it computes is new.
        (MAIN, "dataset(wages).aggregate(pay = mean(wage), by: [industry, year]).mean(|\n".to_string()),
        // The function written before its frame.
        (MAIN, format!("{data}mean(d, |\n")),
        (MAIN, format!("{data}reg(d, b ~ a, cluster: |\n")),
        // A function of the project that takes a variable.
        (
            MAIN,
            format!("use functions::clean\n{data}d.share_gone(|\n"),
        ),
        // Where no variable is written: an argument that takes a number, a word or a text;
        // a call that is on no frame.
        (MAIN, format!("{data}d.head(|\n")),
        (MAIN, format!("{data}d.reg(b ~ a, level: |\n")),
        (MAIN, format!("{data}d.reg(b ~ a, se: |\n")),
        (MAIN, format!("{data}print(d.mean(a), |\n")),
        (MAIN, format!("{data}let m = 3\nlet x = round(m|\n")),
        // A frame whose variables the checker does not know before a run: a file that is not
        // there, a step whose columns depend on the data, a parameter of a function.
        (MAIN, "import(\"data/none.csv\").mean(|\n".to_string()),
        (
            MAIN,
            format!(
                "{data}let wide = d.to_wide(id: a, names_from: c, values_from: b)\nwide.mean(|\n"
            ),
        ),
        (
            CLEAN,
            "/// T.\nfn share_gone(data: Frame, var: Var) -> Float {\n    data.mean(|\n}\n"
                .to_string(),
        ),
        // A file outside any project.
        (
            "/elsewhere/try.zeff",
            "let d = frame(x = [1.0, 2], y = [3, 4])\nd.reg(y ~ |\n".to_string(),
        ),
    ];
    let mut all = String::new();
    for (path, marked) in &places {
        if *path != MAIN {
            all.push_str(&format!("in {path}\n"));
        }
        let found = at_cursor(&fs, path, marked);
        let mut out = format!("> {}", marked.replace('\n', "\n> "));
        out.truncate(out.trim_end_matches(['>', ' ']).len());
        // The variables in full; of the rest, how many of each kind.
        let variables: Vec<String> = found
            .items
            .iter()
            .filter(|item| item.kind == CompletionKind::Variable)
            .map(|item| match (item.detail.as_str(), item.summary.as_str()) {
                ("", "") => item.label.clone(),
                (detail, "") => format!("{} {detail}", item.label),
                (detail, summary) => format!("{} {detail} ({summary})", item.label),
            })
            .collect();
        out.push_str(&format!(
            "variables: {}\n",
            if variables.is_empty() {
                "none".to_string()
            } else {
                variables.join(", ")
            }
        ));
        let count = |kind: CompletionKind| found.items.iter().filter(|i| i.kind == kind).count();
        out.push_str(&format!(
            "then: {} words, {} argument names, {} names in reach, {} functions\n",
            count(CompletionKind::Word),
            count(CompletionKind::Argument),
            count(CompletionKind::Binding),
            count(CompletionKind::Method) + count(CompletionKind::Function),
        ));
        all.push_str(&out);
        all.push_str("----\n");
    }
    insta::assert_snapshot!("corpus_of_variables", all);
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

/// The completions of one kind at a place, by their labels.
fn of_kind(found: &Completions, kind: CompletionKind) -> Vec<&str> {
    found
        .items
        .iter()
        .filter(|item| item.kind == kind)
        .map(|item| item.label.as_str())
        .collect()
}

#[test]
fn the_names_of_the_arguments_not_yet_given() {
    let fs = project();
    let data = "let d = import(\"data/small.csv\")\n";
    // In the order of the usage line, before every other name, each written `name: `.
    let found = at_cursor(&fs, MAIN, &format!("{data}d.reg(b ~ a, |\n"));
    assert_eq!(
        of_kind(&found, CompletionKind::Argument),
        ["fe", "se", "cluster", "weights", "weight_type", "level"]
    );
    // The place takes terms (`fe:`), so the frame's variables come first; then the names.
    let after: Vec<CompletionKind> = found
        .items
        .iter()
        .map(|item| item.kind)
        .filter(|kind| *kind != CompletionKind::Variable)
        .take(7)
        .collect();
    assert_eq!(after[..6], [CompletionKind::Argument; 6]);
    assert_ne!(after[6], CompletionKind::Argument);
    let se = found
        .items
        .iter()
        .find(|item| item.label == "se")
        .expect("se");
    assert_eq!(
        (se.insert.as_str(), se.detail.as_str()),
        ("se: ", "classical | robust | hc2 | hc3")
    );
    assert!(
        se.summary.starts_with("The kind of standard error"),
        "{}",
        se.summary
    );
    // The first place of a method is its second parameter; of the function written before
    // its value, its first.
    let found = at_cursor(&fs, MAIN, &format!("{data}d.reg(|\n"));
    assert_eq!(of_kind(&found, CompletionKind::Argument)[0], "formula");
    let found = at_cursor(&fs, MAIN, &format!("{data}reg(|\n"));
    assert_eq!(
        of_kind(&found, CompletionKind::Argument)[..2],
        ["data", "formula"]
    );
    // Given by name before the place or after it, or in its place: not offered again.
    let found = at_cursor(
        &fs,
        MAIN,
        &format!("{data}d.reg(b ~ a, cluster: c, |, level: 0.9)\n"),
    );
    assert_eq!(
        of_kind(&found, CompletionKind::Argument),
        ["fe", "se", "weights", "weight_type"]
    );
    // Every name offered is one the checker knows as an argument.
    for item in at_cursor(&fs, MAIN, &format!("{data}d.reg(b ~ a, |\n"))
        .items
        .iter()
        .filter(|item| item.kind == CompletionKind::Argument)
    {
        let text = format!("{data}d.reg(b ~ a, {}1)\n", item.insert);
        let mut open = Documents::new();
        open.open(MAIN, text.clone(), 1);
        for finding in diagnostics(&fs, &open, Path::new(MAIN)) {
            assert!(
                !finding.message.contains("has no argument called"),
                "{text}: {}",
                finding.message
            );
        }
    }
    // Not where the value of an argument is under way, nor inside a list, nor after `name:`.
    for marked in [
        format!("{data}d.mean(a + |\n"),
        format!("{data}d.summary([a, |\n"),
        format!("{data}d.reg(b ~ a, level: |\n"),
    ] {
        let found = at_cursor(&fs, MAIN, &marked);
        assert_eq!(
            of_kind(&found, CompletionKind::Argument),
            Vec::<&str>::new(),
            "{marked}"
        );
        // The names in reach are still offered there.
        assert!(
            of_kind(&found, CompletionKind::Binding).contains(&"d"),
            "{marked}"
        );
    }
}

#[test]
fn the_words_of_a_fixed_list() {
    let fs = project();
    let data = "let d = import(\"data/small.csv\")\n";
    // After the name of an argument that takes only such a word: the words, in the order the
    // function gives them, and nothing else.
    let found = at_cursor(&fs, MAIN, &format!("{data}d.reg(b ~ a, se: |\n"));
    assert_eq!(labels(&found), ["classical", "robust", "hc2", "hc3"]);
    assert!(
        found
            .items
            .iter()
            .all(|item| item.kind == CompletionKind::Word && item.insert == item.label)
    );
    assert_eq!(found.items[0].detail, "se: classical | robust | hc2 | hc3");
    // Each is accepted by the checker where it is offered.
    for item in &found.items {
        let text = format!("{data}d.reg(b ~ a, se: {})\n", item.label);
        let mut open = Documents::new();
        open.open(MAIN, text.clone(), 1);
        let found = diagnostics(&fs, &open, Path::new(MAIN));
        assert!(found.is_empty(), "{text}: {}", found[0].message);
    }
    // An argument that takes a number or a word: the words first, then what else may stand.
    let found = at_cursor(&fs, MAIN, &format!("{data}d.histogram(a, bins: |)\n"));
    assert_eq!(labels(&found)[..3], ["sturges", "scott", "fd"]);
    assert!(of_kind(&found, CompletionKind::Binding).contains(&"d"));
    // In its place, without its name.
    let found = at_cursor(&fs, MAIN, &format!("{data}d.histogram(a, s|)\n"));
    assert_eq!(of_kind(&found, CompletionKind::Word), ["sturges", "scott"]);
    // Inside a list of words.
    let found = at_cursor(
        &fs,
        MAIN,
        &format!("{data}d.summary([a, b], stats: [mean, s|])\n"),
    );
    assert_eq!(labels(&found), ["sd"]);
}

/// Every fixed list of every function of the engine: after `name: ` of the argument, the
/// words offered are the words its type gives, and the names offered at the first place are
/// the function's parameters.
#[test]
fn every_fixed_list_of_every_function() {
    let fs = project();
    let registry = zeff_engine::registry::registry().expect("the registry");
    let mut lists = 0;
    let mut seen: Vec<&str> = Vec::new();
    for f in registry.functions() {
        // Of several functions of one name the first is the one a bare call names here.
        // Inside `dataset(` the shipped datasets are offered, and nothing else.
        if seen.contains(&f.spec.name) || f.spec.name == "dataset" {
            continue;
        }
        seen.push(f.spec.name);
        let found = at_cursor(&fs, MAIN, &format!("{}(|\n", f.spec.name));
        let names: Vec<&str> = f.spec.params.iter().map(|p| p.name).collect();
        assert_eq!(
            of_kind(&found, CompletionKind::Argument),
            names,
            "{}",
            f.spec.name
        );
        for p in &f.spec.params {
            let words = zeff_ir::Ty::parse(p.ty).map_or(Vec::new(), |ty| ty.words());
            if words.is_empty() {
                continue;
            }
            lists += 1;
            let found = at_cursor(&fs, MAIN, &format!("{}({}: |\n", f.spec.name, p.name));
            assert_eq!(
                of_kind(&found, CompletionKind::Word),
                words,
                "{}({}:",
                f.spec.name,
                p.name
            );
        }
    }
    assert!(lists >= 30, "{lists} fixed lists");
}

#[test]
fn the_function_of_a_call_is_the_one_the_checker_would_choose() {
    let fs = project();
    let data = "let d = import(\"data/small.csv\")\n";
    // `histogram` of a frame takes `by:`; the layer of a plot takes `data:` and no `by:`.
    let frame = at_cursor(&fs, MAIN, &format!("{data}d.histogram(a, |\n"));
    assert!(of_kind(&frame, CompletionKind::Argument).contains(&"by"));
    for marked in [
        format!("{data}d.plot(x: a).histogram(|\n"),
        format!("{data}let p = d.plot(x: a)\np.histogram(|\n"),
        format!("{data}let p = d.plot(x: a)\nhistogram(p, |\n"),
    ] {
        let layer = at_cursor(&fs, MAIN, &marked);
        let names = of_kind(&layer, CompletionKind::Argument);
        assert!(
            names.contains(&"data") && !names.contains(&"by"),
            "{marked}: {names:?}"
        );
    }
    // A function of the project, by its own page; one that is not brought in is not known
    // by its short name, and is by its module's.
    let own = at_cursor(
        &fs,
        MAIN,
        &format!("use functions::clean\n{data}d.share_gone(|\n"),
    );
    let var = own
        .items
        .iter()
        .find(|item| item.kind == CompletionKind::Argument)
        .expect("an argument");
    assert_eq!(
        (
            var.label.as_str(),
            var.insert.as_str(),
            var.detail.as_str(),
            var.summary.as_str()
        ),
        ("var", "var: ", "Var", "The variable.")
    );
    let unknown = at_cursor(&fs, MAIN, &format!("{data}share_gone(d, |\n"));
    assert_eq!(
        of_kind(&unknown, CompletionKind::Argument),
        Vec::<&str>::new()
    );
    let by_module = at_cursor(&fs, MAIN, &format!("{data}clean::share_gone(d, |\n"));
    assert_eq!(of_kind(&by_module, CompletionKind::Argument), ["var"]);
    // The module as typed in the editor, not as saved.
    let mut open = Documents::new();
    open.open(CLEAN, MODULE.replace("var: Var", "which: Var"), 2);
    let text = format!("use functions::clean\n{data}d.share_gone(");
    open.open(MAIN, text.clone(), 1);
    let found = completions(&fs, &open, Path::new(MAIN), position(&text, text.len()));
    assert_eq!(of_kind(&found, CompletionKind::Argument), ["which"]);
}

#[test]
fn the_variables_of_the_frame_come_first_and_can_be_written_there() {
    let fs = project();
    let data = "let d = import(\"data/small.csv\")\n";
    let found = at_cursor(&fs, MAIN, &format!("{data}d.mean(|\n"));
    // In the order of the frame, before the argument names and everything else, each with
    // its type.
    let first: Vec<(&str, CompletionKind, &str)> = found.items[..4]
        .iter()
        .map(|item| (item.label.as_str(), item.kind, item.detail.as_str()))
        .collect();
    assert_eq!(
        first,
        [
            ("a", CompletionKind::Variable, "Int"),
            ("b", CompletionKind::Variable, "Float"),
            ("c", CompletionKind::Variable, "Str"),
            ("var", CompletionKind::Argument, "Var"),
        ]
    );
    // Each is a variable the checker finds in the frame, at that point of the chain.
    for chain in [
        "d",
        "d.rename(z = a)",
        "d.aggregate(m = mean(b), by: c)",
        "d.drop([b])",
    ] {
        let found = at_cursor(&fs, MAIN, &format!("{data}{chain}.select([|\n"));
        let variables = of_kind(&found, CompletionKind::Variable);
        assert!(!variables.is_empty(), "{chain}");
        for variable in variables {
            let text = format!("{data}{chain}.select([{variable}])\n");
            let mut open = Documents::new();
            open.open(MAIN, text.clone(), 1);
            let said = diagnostics(&fs, &open, Path::new(MAIN));
            assert!(said.is_empty(), "{text}: {}", said[0].message);
        }
    }
    // After a dot a variable is not what follows.
    let found = at_cursor(&fs, MAIN, &format!("{data}d.where(a.|\n"));
    assert_eq!(
        of_kind(&found, CompletionKind::Variable),
        Vec::<&str>::new()
    );
}

#[test]
fn the_frame_is_the_one_of_the_innermost_call_on_a_frame() {
    let fs = project();
    let two = "let d = import(\"data/small.csv\")\nlet other = frame(p = [1.0, 2], q = [3, 4])\n";
    // A call on another frame inside an argument: that frame's variables.
    let found = at_cursor(&fs, MAIN, &format!("{two}d.add(k = other.mean(|\n"));
    assert_eq!(of_kind(&found, CompletionKind::Variable), ["p", "q"]);
    // A function of a number inside an argument: the frame of the call it stands in.
    let found = at_cursor(&fs, MAIN, &format!("{two}other.add(k = sqrt(abs(|\n"));
    assert_eq!(of_kind(&found, CompletionKind::Variable), ["p", "q"]);
    // Past a call that is closed, the outer one again.
    let found = at_cursor(&fs, MAIN, &format!("{two}d.add(k = other.mean(p) + |\n"));
    assert_eq!(of_kind(&found, CompletionKind::Variable), ["a", "b", "c"]);
    // An argument of the inner call that takes no variable: none, though the outer would.
    let found = at_cursor(&fs, MAIN, &format!("{two}d.add(k = other.head(|\n"));
    assert_eq!(
        of_kind(&found, CompletionKind::Variable),
        Vec::<&str>::new()
    );
}

/// A step whose columns depend on the data has none before a run; after one, those the run
/// recorded, marked (section 11, "from last run").
#[test]
fn the_variables_of_a_step_checked_at_run_time_are_those_of_the_last_run() {
    use zeff_engine::project::Project;
    use zeff_engine::run::run_script;
    let fs = project();
    fs.set_clock(1_000);
    fs.add_file(
        "/work/demo/data/sales.csv",
        "firm,year,sales\n1,2010,5\n1,2011,6\n2,2010,7\n2,2011,9\n",
    );
    let script = "let long = import(\"data/sales.csv\")\nlet wide = long.to_wide(id: firm, names_from: year, values_from: sales)\n";
    let before = at_cursor(&fs, MAIN, &format!("{script}wide.mean(|\n"));
    assert_eq!(
        of_kind(&before, CompletionKind::Variable),
        Vec::<&str>::new()
    );
    let project = Project::open(&fs, Path::new("/work/demo")).expect("the project");
    let outcome = run_script(&fs, &project, "scripts/main.zeff", script);
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    let after = at_cursor(&fs, MAIN, &format!("{script}wide.mean(|\n"));
    let variables: Vec<(&str, &str, &str)> = after
        .items
        .iter()
        .filter(|item| item.kind == CompletionKind::Variable)
        .map(|item| {
            (
                item.label.as_str(),
                item.detail.as_str(),
                item.summary.as_str(),
            )
        })
        .collect();
    assert_eq!(
        variables,
        [
            ("firm", "Int", "0 missing, from last run"),
            ("sales_2010", "Int", "0 missing, from last run"),
            ("sales_2011", "Int", "0 missing, from last run"),
        ]
    );
    // The statement changed since the run: its record is of another statement.
    let changed = script.replace("id: firm", "id: [firm]");
    let found = at_cursor(&fs, MAIN, &format!("{changed}wide.mean(|\n"));
    assert_eq!(
        of_kind(&found, CompletionKind::Variable),
        Vec::<&str>::new()
    );
}

/// What a run recorded of the frames its `let`s bound is shown at the next completion: each
/// variable's count of missing values and its label, marked as the last run's (open decision
/// #145). Before any run a file's variables have their types alone.
#[test]
fn the_counts_and_labels_of_a_frame_the_last_run_bound() {
    use zeff_engine::project::Project;
    use zeff_engine::run::run_script;
    let fs = project();
    fs.set_clock(1_000);
    let script = "let d = import(\"data/small.csv\")\nlet named = d.label(a, \"Entry number\").where(a > 1)\nprint(named.mean(b))\n";
    let said = |marked: &str| -> Vec<(String, String, String)> {
        at_cursor(&fs, MAIN, marked)
            .items
            .iter()
            .filter(|item| item.kind == CompletionKind::Variable)
            .map(|item| {
                (
                    item.label.clone(),
                    item.detail.clone(),
                    item.summary.clone(),
                )
            })
            .collect()
    };
    let row = |name: &str, ty: &str, summary: &str| {
        (name.to_string(), ty.to_string(), summary.to_string())
    };
    // Never run: the types, and the label the script writes.
    assert_eq!(
        said(&format!("{script}d.mean(|\n")),
        [
            row("a", "Int", ""),
            row("b", "Float", ""),
            row("c", "Str", "")
        ]
    );
    assert_eq!(
        said(&format!("{script}named.mean(|\n")),
        [
            row("a", "Int", "Entry number"),
            row("b", "Float", ""),
            row("c", "Str", "")
        ]
    );
    let project = Project::open(&fs, Path::new("/work/demo")).expect("the project");
    let outcome = run_script(&fs, &project, "scripts/main.zeff", script);
    assert_eq!(outcome.exit_code, 0, "{}", outcome.output.stderr);
    // Run: the file has one missing value, in `b`.
    assert_eq!(
        said(&format!("{script}d.mean(|\n")),
        [
            row("a", "Int", "0 missing, from last run"),
            row("b", "Float", "1 missing, from last run"),
            row("c", "Str", "0 missing, from last run"),
        ]
    );
    // The frame bound after `where` is a selection of the first: of its variables the run
    // gathered only `b`, which it took the mean of, and only what the run had in hand is
    // recorded. Recording never makes a frame hold more than the run made it hold.
    assert_eq!(
        said(&format!("{script}named.mean(|\n")),
        [
            row("a", "Int", "Entry number"),
            row("b", "Float", "1 missing, from last run"),
            row("c", "Str", ""),
        ]
    );
    // What is known apart, for an interface that shows it in columns of its own.
    let found = at_cursor(&fs, MAIN, &format!("{script}d.mean(|\n"));
    let b = found.items[1].variable.clone().expect("a variable");
    assert_eq!(
        (b.ty.as_str(), b.missing, b.label.as_str(), b.from_last_run),
        ("Float", Some(1), "", true)
    );
    assert!(
        found
            .items
            .iter()
            .filter(|item| item.kind != CompletionKind::Variable)
            .all(|item| item.variable.is_none())
    );
    // Through a step that keeps every row the counts stay; after `where` they are not known,
    // and nothing is said in their place.
    assert_eq!(
        said(&format!("{script}d.select([b]).sort(b).mean(|\n")),
        [row("b", "Float", "1 missing, from last run")]
    );
    assert_eq!(
        said(&format!("{script}d.where(a > 1).mean(|\n")),
        [
            row("a", "Int", ""),
            row("b", "Float", ""),
            row("c", "Str", "")
        ]
    );
    // The statement changed since the run: its record is another statement's.
    let changed = script.replace(
        "let d = import(\"data/small.csv\")",
        "let d = import(\"data/small.csv\", header: true)",
    );
    assert_eq!(
        said(&format!("{changed}d.mean(|\n")),
        [
            row("a", "Int", ""),
            row("b", "Float", ""),
            row("c", "Str", "")
        ]
    );
    // A second run that binds the same frames writes nothing: the cache file is as it was.
    let cache = Path::new("/work/demo/.zeff/schemas/schemas.json");
    let before = zeff_engine::fs::FileSystem::read_to_string(&fs, cache).expect("the cache");
    assert!(before.contains("\"frames\": ["), "{before}");
    fs.set_clock(2_000);
    let again = run_script(&fs, &project, "scripts/main.zeff", script);
    assert_eq!(again.exit_code, 0);
    assert_eq!(
        zeff_engine::fs::FileSystem::read_to_string(&fs, cache).expect("the cache"),
        before
    );
}

/// A shipped dataset is known whole when the engine is built: what is offered of its
/// variables is what the dataset holds.
#[test]
fn the_counts_and_labels_of_a_shipped_dataset_are_those_of_its_data() {
    let fs = project();
    let registry = zeff_engine::registry::registry().expect("the registry");
    for dataset in registry.datasets() {
        let name = dataset.spec.name;
        let found = at_cursor(&fs, MAIN, &format!("dataset({name}).select([|\n"));
        let variables: Vec<_> = found
            .items
            .iter()
            .filter_map(|item| {
                item.variable
                    .as_ref()
                    .map(|facts| (item.label.as_str(), facts))
            })
            .collect();
        // Its page lists each variable with its type, its label and its missing values.
        assert_eq!(variables.len(), dataset.spec.format.len(), "{name}");
        for ((variable, facts), row) in variables.iter().zip(&dataset.spec.format) {
            assert_eq!(*variable, row.name, "{name}");
            // The page says more of a category: `Category (12 levels)`.
            assert!(
                row.ty.starts_with(&facts.ty) && !facts.ty.is_empty(),
                "{name}.{variable}"
            );
            assert_eq!(facts.label, row.label, "{name}.{variable}");
            assert_eq!(facts.missing, Some(row.missing as u64), "{name}.{variable}");
            assert!(!facts.from_last_run);
        }
    }
}
