//! `cargo run -p xtask -- check-grammar`: the tree-sitter grammar held to the engine's parser
//! (release 0.7.0, step 37b; Engine and Project Specification, section 12, "Syntax
//! highlighting").
//!
//! Every script the repository holds that the engine's parser reads without an error (the
//! sample projects, the specification's sample script, the built-ins written in Zeff and
//! every script the conformance suite runs) is read by the grammar of
//! `editors/tree-sitter-zeff`: it must find no error, and its statements must begin and end
//! where the engine's parser has them. The grammar colours a script; this is what keeps it
//! from colouring another script than the engine runs.
//!
//! The grammar is run by its own tool, `tree-sitter`, which must be installed (the job that
//! runs this check installs it); the workspace links nothing of it.

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use zeff_syntax::ast::Item;

const GRAMMAR: &str = "editors/tree-sitter-zeff";

/// A place in a text as tree-sitter writes it: the line from 0, and the byte in the line.
type Place = (usize, usize);

fn place(text: &str, byte: usize) -> Place {
    let before = &text[..byte.min(text.len())];
    let line_start = before.rfind('\n').map_or(0, |end| end + 1);
    (before.matches('\n').count(), byte - line_start)
}

/// Where the engine's parser has each statement of a script begin and end. A function is
/// taken from its `fn`: its help lines stand above it, and the grammar keeps them as comments.
fn engine_statements(text: &str, items: &[Item]) -> Vec<(Place, Place)> {
    items
        .iter()
        .map(|item| {
            let (mut start, end) = match item {
                Item::Use(used) => (used.span.start, used.span.end),
                Item::Statement(statement) => (statement.span.start, statement.span.end),
                Item::Fn(function) => (function.span.start, function.span.end),
            };
            if matches!(item, Item::Fn(_)) {
                while text[start..]
                    .trim_start_matches([' ', '\t'])
                    .starts_with("///")
                {
                    start += text[start..]
                        .find('\n')
                        .map_or(text.len() - start, |at| at + 1);
                }
                start += text[start..].len() - text[start..].trim_start_matches([' ', '\t']).len();
            }
            (place(text, start), place(text, end))
        })
        .collect()
}

/// `[row, column]` at the start of a text.
fn written_place(text: &str) -> Option<(Place, &str)> {
    let rest = text.strip_prefix('[')?;
    let (row, rest) = rest.split_once(", ")?;
    let (column, rest) = rest.split_once(']')?;
    Some(((row.parse().ok()?, column.parse().ok()?), rest))
}

/// Where the grammar has each statement begin and end, from the tree its tool prints: the
/// children of the script, the comments among them left out. `Err` where the tree holds an
/// error, with the line that says so.
fn grammar_statements(tree: &str) -> Result<Vec<(Place, Place)>, String> {
    let mut found = Vec::new();
    for line in tree.lines() {
        if line.contains("(ERROR") || line.contains("(MISSING") {
            return Err(line.trim().to_string());
        }
        // A child of the script is written two spaces in.
        let Some(node) = line.strip_prefix("  (") else {
            continue;
        };
        if node.starts_with(' ') {
            continue;
        }
        let Some((kind, rest)) = node.split_once(' ') else {
            continue;
        };
        if matches!(kind, "comment" | "doc_comment" | "pragma") {
            continue;
        }
        let range = written_place(rest).and_then(|(start, rest)| {
            let (end, _) = written_place(rest.strip_prefix(" - ")?)?;
            Some((start, end))
        });
        match range {
            Some(range) => found.push(range),
            None => return Err(format!("a line of the tree that cannot be read: {line}")),
        }
    }
    Ok(found)
}

/// How the statements of the grammar differ from the engine's: the first that does, with
/// where each has it; `None` where they are the same.
fn difference(found: &[(Place, Place)], expected: &[(Place, Place)]) -> Option<String> {
    if found == expected {
        return None;
    }
    let first = found
        .iter()
        .zip(expected)
        .position(|(a, b)| a != b)
        .unwrap_or(found.len().min(expected.len()));
    let show = |list: &[(Place, Place)]| {
        list.get(first)
            .map_or("no statement".to_string(), |((a, b), (c, d))| {
                format!("line {} byte {} to line {} byte {}", a + 1, b, c + 1, d)
            })
    };
    Some(format!(
        "statement {} (the grammar has {} statements, the engine {}): the grammar has {}, the engine {}",
        first + 1,
        found.len(),
        expected.len(),
        show(found),
        show(expected)
    ))
}

/// The scripts to read, each with a name to report it by.
fn scripts() -> Vec<(String, String)> {
    let mut found: Vec<(String, String)> = Vec::new();
    let mut folders: Vec<PathBuf> = vec![PathBuf::from("docs/spec")];
    for (root, depth) in [("samples", 1), ("packages", 2)] {
        let mut level = vec![PathBuf::from(root)];
        for _ in 0..depth {
            level = level
                .iter()
                .flat_map(|folder| std::fs::read_dir(folder).into_iter().flatten().flatten())
                .map(|entry| entry.path())
                .filter(|path| path.is_dir())
                .collect();
        }
        for folder in level {
            folders.push(folder.join("scripts"));
            folders.push(folder.join("functions"));
        }
    }
    folders.sort();
    for folder in folders {
        let mut files: Vec<PathBuf> = std::fs::read_dir(&folder)
            .into_iter()
            .flatten()
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|e| e == "zeff"))
            .collect();
        files.sort();
        for file in files {
            if let Ok(text) = std::fs::read_to_string(&file) {
                found.push((file.display().to_string(), text));
            }
        }
    }
    for (at, text) in zeff_conformance::scripts().into_iter().enumerate() {
        found.push((format!("conformance script {}", at + 1), text));
    }
    found
}

const HIGHLIGHTS: &str = "editors/tree-sitter-zeff/queries/highlights.scm";
const WORDS_BEGIN: &str = "; BEGIN WORDS";
const WORDS_END: &str = "; END WORDS";

/// The fixed lists of the engine's functions, as the highlight queries need them.
struct OptionWords {
    /// For each argument name that takes one of a fixed list of words in some function
    /// (`se: classical | robust | hc2 | hc3`), the words it takes there; both by the letters.
    by_name: Vec<(String, Vec<String>)>,
    /// The words that some argument takes in a list (`stats: [mean, sd]`), by the letters.
    in_lists: Vec<String>,
}

fn option_words() -> Result<OptionWords, String> {
    let registry =
        zeff_engine::registry::registry().map_err(|_| "the registry does not build".to_string())?;
    let mut by_name: std::collections::BTreeMap<String, std::collections::BTreeSet<String>> =
        std::collections::BTreeMap::new();
    let mut in_lists: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for parameter in registry
        .functions()
        .iter()
        .flat_map(|f| f.spec.params.iter())
    {
        let Some(ty) = zeff_ir::Ty::parse(parameter.ty) else {
            continue;
        };
        let words = ty.words();
        if words.is_empty() {
            continue;
        }
        // The words a list of the type may hold: of the type itself, or of one of its
        // alternatives.
        let listed = |ty: &zeff_ir::Ty| match ty {
            zeff_ir::Ty::List(inner) => inner.words(),
            _ => Vec::new(),
        };
        in_lists.extend(listed(&ty));
        if let zeff_ir::Ty::Union(options) = &ty {
            in_lists.extend(options.iter().flat_map(listed));
        }
        by_name
            .entry(parameter.name.to_string())
            .or_default()
            .extend(words);
    }
    Ok(OptionWords {
        by_name: by_name
            .into_iter()
            .map(|(name, words)| (name, words.into_iter().collect()))
            .collect(),
        in_lists: in_lists.into_iter().collect(),
    })
}

/// The part of the highlight queries that is written from the registry. A bare word is an
/// option word where it is the whole value of an argument or of a record's field of a name
/// that takes such words, and is one of the words that name takes: `se: robust`,
/// `{se: robust}`. So a variable that happens to be called `x` is no option word in
/// `plot(x: x)`. The name is captured under the style it has anyway, a property: the
/// highlighter styles by every capture, the one a rule only tests among them.
///
/// A word in a list (`stats: [mean, sd]`) is an option word where some argument takes it in
/// a list, whatever the argument's name: a rule that also captured the name would fit once
/// for each word of the list with the name in common, and the highlighter keeps one such
/// fit, colouring one word of the list and no other.
fn words_section(words: &OptionWords) -> String {
    let quoted = |words: &[String]| {
        words
            .iter()
            .map(|word| format!("\"{word}\""))
            .collect::<Vec<_>>()
            .join(" ")
    };
    let mut out = String::new();
    for (name, words) in &words.by_name {
        for place in [
            "(named_argument name: (identifier) @property value: (qualified_name . (identifier) @constant.option .))",
            "(field name: (identifier) @property value: (qualified_name . (identifier) @constant.option .))",
        ] {
            out.push_str(&format!(
                "({place}\n  (#eq? @property \"{name}\") (#any-of? @constant.option {}))\n",
                quoted(words)
            ));
        }
    }
    if !words.in_lists.is_empty() {
        for place in [
            "(named_argument value: (list (qualified_name . (identifier) @constant.option .)))",
            "(field value: (list (qualified_name . (identifier) @constant.option .)))",
        ] {
            out.push_str(&format!(
                "({place}\n  (#any-of? @constant.option {}))\n",
                quoted(&words.in_lists)
            ));
        }
    }
    out
}

/// The highlight queries with their written part put in, or `Err` where the marks that
/// bound it are not there.
fn with_words(queries: &str, section: &str) -> Result<String, String> {
    let begin = queries
        .find(WORDS_BEGIN)
        .ok_or(format!("`{WORDS_BEGIN}` is not in {HIGHLIGHTS}"))?;
    let end = queries
        .find(WORDS_END)
        .filter(|end| *end > begin)
        .ok_or(format!("`{WORDS_END}` is not in {HIGHLIGHTS}"))?;
    Ok(format!(
        "{}{WORDS_BEGIN}\n{section}{}",
        &queries[..begin],
        &queries[end..]
    ))
}

/// `cargo run -p xtask -- grammar-words`: write the option words of the registry into the
/// highlight queries.
pub fn write_words() -> ExitCode {
    let written = option_words().and_then(|words| {
        let queries = std::fs::read_to_string(HIGHLIGHTS)
            .map_err(|why| format!("cannot read {HIGHLIGHTS}: {why}"))?;
        let new = with_words(&queries, &words_section(&words))?;
        std::fs::write(HIGHLIGHTS, new)
            .map_err(|why| format!("cannot write {HIGHLIGHTS}: {why}"))?;
        Ok(words.by_name.len())
    });
    match written {
        Ok(words) => {
            println!(
                "grammar: the option words of {words} argument names written into {HIGHLIGHTS}"
            );
            ExitCode::SUCCESS
        }
        Err(why) => {
            eprintln!("error: {why}");
            ExitCode::from(2)
        }
    }
}

/// Whether the option words in the highlight queries are the registry's.
fn words_are_current() -> Result<(), String> {
    let words = option_words()?;
    let queries = std::fs::read_to_string(HIGHLIGHTS)
        .map_err(|why| format!("cannot read {HIGHLIGHTS}: {why}"))?;
    if with_words(&queries, &words_section(&words))? == queries {
        Ok(())
    } else {
        Err(format!(
            "the option words of {HIGHLIGHTS} are not the registry's; `cargo run -p xtask -- grammar-words` writes them"
        ))
    }
}

pub fn check() -> ExitCode {
    if let Err(why) = words_are_current() {
        eprintln!("error: {why}");
        return ExitCode::from(1);
    }
    let grammar = Path::new(GRAMMAR);
    if !grammar.join("src/parser.c").is_file() {
        eprintln!(
            "error: {GRAMMAR}/src/parser.c is not there; run this from the root of the repository"
        );
        return ExitCode::from(2);
    }
    let Ok(root) = std::env::current_dir() else {
        eprintln!("error: cannot tell which folder this is");
        return ExitCode::from(2);
    };
    let scratch = root.join("target/grammar");
    let library = root.join("target/tree-sitter");
    for folder in [&scratch, &library] {
        if let Err(why) = std::fs::create_dir_all(folder) {
            eprintln!("error: cannot make {}: {why}", folder.display());
            return ExitCode::from(2);
        }
    }
    let scripts = scripts();
    let (mut read, mut refused_by_the_engine) = (0, 0);
    let mut wrong: Vec<String> = Vec::new();
    for (at, (name, text)) in scripts.iter().enumerate() {
        let parsed = zeff_syntax::parse("script.zeff", text);
        // A script written not to parse (the cases of the parser's own messages) has no
        // statements to compare.
        if parsed.diagnostics.has_errors() {
            refused_by_the_engine += 1;
            continue;
        }
        let file = scratch.join(format!("{at}.zeff"));
        if let Err(why) = std::fs::write(&file, text) {
            eprintln!("error: cannot write {}: {why}", file.display());
            return ExitCode::from(2);
        }
        // The tool keeps the parser it compiles in the repository's build folder.
        let output = Command::new("tree-sitter")
            .arg("parse")
            .arg(&file)
            .current_dir(grammar)
            .env("TREE_SITTER_LIBDIR", &library)
            .output();
        let output = match output {
            Ok(output) => output,
            Err(why) => {
                eprintln!(
                    "error: cannot run `tree-sitter` ({why}); it is installed with `cargo install tree-sitter-cli --version 0.27.2 --locked`"
                );
                return ExitCode::from(2);
            }
        };
        read += 1;
        let tree = String::from_utf8_lossy(&output.stdout);
        let expected = engine_statements(text, &parsed.script.items);
        match grammar_statements(&tree) {
            Err(why) => wrong.push(format!("{name}: the grammar finds an error: {why}")),
            Ok(found) => {
                if let Some(how) = difference(&found, &expected) {
                    wrong.push(format!("{name}: {how}"));
                }
            }
        }
    }
    println!(
        "grammar: {read} scripts read by the grammar, {refused_by_the_engine} left out that the engine's parser refuses; {} differ",
        wrong.len()
    );
    for line in &wrong {
        println!("  {line}");
    }
    // Too few scripts would mean the scripts were not found, not that the grammar is right.
    if read < 100 {
        eprintln!("error: only {read} scripts were found");
        return ExitCode::from(2);
    }
    if wrong.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_statements_of_a_printed_tree() {
        let tree = "(script [0, 0] - [9, 0]\n  (comment [0, 0] - [0, 12])\n  (let_statement [1, 0] - [1, 9]\n    name: (identifier [1, 4] - [1, 5])\n    value: (number [1, 8] - [1, 9]))\n  (doc_comment [2, 0] - [2, 9])\n  (function_definition [3, 0] - [5, 1]\n    name: (identifier [3, 3] - [3, 4]))\n  (qualified_name [6, 0] - [6, 1]\n    (identifier [6, 0] - [6, 1])))\n";
        assert_eq!(
            grammar_statements(tree),
            Ok(vec![((1, 0), (1, 9)), ((3, 0), (5, 1)), ((6, 0), (6, 1))])
        );
        let broken = "(script [0, 0] - [1, 0]\n  (ERROR [0, 0] - [0, 3]\n    (identifier [0, 0] - [0, 3])))\n";
        assert!(grammar_statements(broken).is_err());
        assert_eq!(
            grammar_statements("(script [0, 0] - [0, 0])\n"),
            Ok(Vec::new())
        );
    }

    #[test]
    fn the_option_words_are_written_between_their_marks() {
        let queries = "(comment) @comment\n; BEGIN WORDS\nold\n; END WORDS\n(number) @number\n";
        let section = words_section(&OptionWords {
            by_name: vec![(
                "se".to_string(),
                vec!["hc2".to_string(), "robust".to_string()],
            )],
            in_lists: vec!["mean".to_string(), "sd".to_string()],
        });
        let new = with_words(queries, &section).expect("the marks are there");
        assert!(new.starts_with(
            "(comment) @comment\n; BEGIN WORDS\n((named_argument name: (identifier) @property value:"
        ));
        assert!(new.ends_with("; END WORDS\n(number) @number\n"));
        // A word under its argument's name, in an argument and in a record's field; a word
        // in a list under no name.
        assert_eq!(
            new.matches("(#eq? @property \"se\") (#any-of? @constant.option \"hc2\" \"robust\")")
                .count(),
            2
        );
        assert_eq!(
            new.matches("(list (qualified_name . (identifier) @constant.option .)))\n  (#any-of? @constant.option \"mean\" \"sd\")")
                .count(),
            2
        );
        assert!(!new.contains("old"));
        // Written again, nothing changes.
        assert_eq!(with_words(&new, &section).as_deref(), Ok(new.as_str()));
        assert!(with_words("(comment) @comment\n", &section).is_err());
        assert!(with_words("; END WORDS\n; BEGIN WORDS\n", &section).is_err());
        // The registry's lists: the words of `se:` and of `join:` under their names, each
        // once, by the letters; no words under a name that takes none; and the words of
        // `stats:`, which are given in a list.
        let words = option_words().expect("the registry builds");
        let of = |name: &str| {
            words
                .by_name
                .iter()
                .find(|(n, _)| n == name)
                .map(|(_, words)| words.join(" "))
        };
        assert_eq!(of("se").as_deref(), Some("classical hc2 hc3 robust"));
        assert_eq!(of("join").as_deref(), Some("full inner left"));
        assert_eq!(of("cluster"), None);
        assert_eq!(of("x"), None);
        assert!(words.by_name.is_sorted() && words.by_name.len() > 20);
        for word in ["mean", "sd", "missing"] {
            assert!(words.in_lists.contains(&word.to_string()), "{word}");
        }
        assert!(!words.in_lists.contains(&"robust".to_string()));
    }

    #[test]
    fn a_difference_is_said_with_its_place() {
        let one = ((0, 0), (0, 9));
        let two = ((1, 0), (2, 5));
        assert_eq!(difference(&[one, two], &[one, two]), None);
        assert_eq!(difference(&[], &[]), None);
        // A statement that ends elsewhere: a chain the grammar ended a line early.
        assert_eq!(
            difference(&[one, ((1, 0), (1, 5))], &[one, two]).as_deref(),
            Some(
                "statement 2 (the grammar has 2 statements, the engine 2): the grammar has line 2 byte 0 to line 2 byte 5, the engine line 2 byte 0 to line 3 byte 5"
            )
        );
        // One statement more, or fewer.
        assert_eq!(
            difference(&[one], &[one, two]).as_deref(),
            Some(
                "statement 2 (the grammar has 1 statements, the engine 2): the grammar has no statement, the engine line 2 byte 0 to line 3 byte 5"
            )
        );
        assert!(difference(&[one, two], &[one]).is_some());
    }

    #[test]
    fn the_statements_of_the_engines_parser() {
        let text = "use functions::clean\n/// Twice.\n///\n/// x: The number.\nfn twice(x: Float) -> Float {\n    x * 2\n}\nlet d = wages\n    .where(a > 1)   // kept\nprint(d)";
        let parsed = zeff_syntax::parse("script.zeff", text);
        assert!(!parsed.diagnostics.has_errors());
        assert_eq!(
            engine_statements(text, &parsed.script.items),
            [
                ((0, 0), (0, 20)),
                // A function from its `fn`, not from its help lines.
                ((4, 0), (6, 1)),
                ((7, 0), (8, 17)),
                ((9, 0), (9, 8)),
            ]
        );
        assert_eq!(place("aπ\nbc", 5), (1, 1));
    }
}
