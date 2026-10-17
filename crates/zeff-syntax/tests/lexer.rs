//! Lexer tests: literals, operators, words, the line-break rules of Language Reference section 3.1,
//! the messages (as snapshots), and the specification's own scripts.

use zeff_diagnostics::{Source, Sources};
use zeff_syntax::{Keyword, TokenKind as K, lex};

// ----- helpers -----

/// The kinds of the tokens, without the final NEWLINE and EOF.
fn kinds(text: &str) -> Vec<K> {
    let mut all = all_kinds(text);
    assert_eq!(all.pop(), Some(K::Eof));
    if all.last() == Some(&K::Newline) {
        all.pop();
    }
    all
}

/// Every token kind, including NEWLINE and the final EOF.
fn all_kinds(text: &str) -> Vec<K> {
    lex("a.zeff", text)
        .tokens
        .into_iter()
        .map(|t| t.kind)
        .collect()
}

fn codes(text: &str) -> Vec<String> {
    lex("a.zeff", text)
        .diagnostics
        .iter()
        .map(|d| d.code.clone())
        .collect()
}

fn name(s: &str) -> K {
    K::Name(s.to_string())
}

fn string(s: &str) -> K {
    K::Str(s.to_string())
}

/// Every diagnostic, printed.
fn render(text: &str) -> String {
    let lexed = lex("scripts/main.zeff", text);
    let mut sources = Sources::new();
    sources.add(Source::new("scripts/main.zeff", text));
    lexed
        .diagnostics
        .iter()
        .map(|d| d.render(&sources))
        .collect::<Vec<_>>()
        .join("\n")
}

// ----- numbers -----

#[test]
fn integers() {
    assert_eq!(kinds("42"), [K::Int(42)]);
    assert_eq!(kinds("0"), [K::Int(0)]);
    assert_eq!(kinds("1_000_000"), [K::Int(1_000_000)]);
    // 2^63 is kept as written: it is allowed only after a minus sign, which the parser checks.
    assert_eq!(kinds("9223372036854775808"), [K::Int(1 << 63)]);
    assert_eq!(kinds("-9223372036854775808"), [K::Minus, K::Int(1 << 63)]);
    assert_eq!(kinds("18446744073709551615"), [K::Int(u64::MAX)]);
    assert!(codes("18446744073709551615").is_empty());
}

#[test]
fn floats() {
    assert_eq!(kinds("0.05"), [K::Float(0.05)]);
    assert_eq!(kinds("1e-6"), [K::Float(1e-6)]);
    assert_eq!(kinds("1e6"), [K::Float(1e6)]);
    assert_eq!(kinds("1e+6"), [K::Float(1e6)]);
    assert_eq!(kinds("1E5"), [K::Float(1e5)]);
    assert_eq!(kinds("1.5e-3"), [K::Float(1.5e-3)]);
    assert_eq!(kinds("0.000_001"), [K::Float(0.000_001)]);
    assert_eq!(kinds("1e1_0"), [K::Float(1e10)]);
    assert_eq!(kinds("0e5"), [K::Float(0.0)]);
}

#[test]
fn a_float_is_the_nearest_double() {
    // 0.1 is the same number on every platform: the nearest binary64 value, ties to even.
    assert_eq!(
        kinds("0.1"),
        [K::Float(f64::from_bits(0x3fb9_9999_9999_999a))]
    );
    assert_eq!(
        kinds("2.2250738585072014e-308"),
        [K::Float(f64::MIN_POSITIVE)]
    );
    assert_eq!(kinds("4.9e-324"), [K::Float(f64::from_bits(1))]);
}

#[test]
fn ranges_keep_their_dots() {
    assert_eq!(kinds("2..5"), [K::Int(2), K::DotDot, K::Int(5)]);
    assert_eq!(kinds("x1..x9"), [name("x1"), K::DotDot, name("x9")]);
    assert_eq!(kinds("1.5..3"), [K::Float(1.5), K::DotDot, K::Int(3)]);
}

#[test]
fn malformed_numbers_are_z0310() {
    for text in [
        ".5",
        "5.",
        "007",
        "00.5",
        "1_",
        "1__0",
        "0._5",
        "1e",
        "1e+",
        "5abc",
        "0x10",
        "10L",
        "18446744073709551616",
        "1e999",
        "1.5.3",
    ] {
        assert!(
            codes(text).contains(&"Z0310".to_string()),
            "{text} should be Z0310: {:?}",
            codes(text)
        );
        assert!(
            all_kinds(text).contains(&K::Error),
            "{text} should leave an error token"
        );
    }
    // The valid part of `1.5.3` is kept; only `.3` is wrong.
    assert_eq!(kinds("1.5.3"), [K::Float(1.5), K::Error]);
}

// ----- strings -----

#[test]
fn strings_and_their_escapes() {
    assert_eq!(kinds(r#""text""#), [string("text")]);
    assert_eq!(kinds(r#""a\nb\tc\rd\"e\\f""#), [string("a\nb\tc\rd\"e\\f")]);
    assert_eq!(kinds(r#""\u{e9}\u{1F600}\u{000041}""#), [string("é😀A")]);
    assert_eq!(kinds("\"日本語 café\""), [string("日本語 café")]);
    assert_eq!(kinds(r#""""#), [string("")]);
}

#[test]
fn raw_strings_have_no_escapes() {
    assert_eq!(kinds(r#"r"\d+""#), [K::RawStr(r"\d+".to_string())]);
    assert_eq!(
        kinds(r#"r"D:\surveys""#),
        [K::RawStr(r"D:\surveys".to_string())]
    );
    // `r` followed by a quote is a raw string; any longer name ending in r is not.
    assert_eq!(kinds(r#"bar"x""#), [name("bar"), string("x")]);
}

#[test]
fn string_errors_are_z0311() {
    for text in [
        r#""\d""#,
        r#""abc"#,
        r#""\u{110000}""#,
        r#""\u{}""#,
        r#""\u{D800}""#,
        r#""\u00e9""#,
        "r\"abc",
        "\"a\nb\"",
    ] {
        assert!(
            codes(text).contains(&"Z0311".to_string()),
            "{text} should be Z0311: {:?}",
            codes(text)
        );
    }
}

#[test]
fn single_quotes_are_an_error_that_suggests_double() {
    assert_eq!(codes("'text'"), ["Z0315"]);
    assert!(render("'text'").contains("write text in double quotes: \"text\""));
}

// ----- names and words -----

#[test]
fn names() {
    assert_eq!(kinds("abc_1"), [name("abc_1")]);
    assert_eq!(kinds("_helper"), [name("_helper")]);
    assert_eq!(kinds("_"), [K::Underscore]);
    assert_eq!(
        kinds("`gross sales (total)`"),
        [name("gross sales (total)")]
    );
    assert_eq!(kinds("`case`"), [name("case")]);
    assert_eq!(kinds("`café`"), [name("café")]);
}

#[test]
fn every_reserved_word_is_a_keyword_and_case_matters() {
    assert_eq!(Keyword::ALL.len(), 26);
    for keyword in Keyword::ALL {
        assert_eq!(
            kinds(keyword.as_str()),
            [K::Keyword(keyword)],
            "{}",
            keyword.as_str()
        );
    }
    assert_eq!(
        Keyword::ALL
            .iter()
            .filter(|k| k.is_reserved_for_later())
            .count(),
        8
    );
    for word in ["Let", "LET", "na", "inf", "True", "NaN", "Nothing"] {
        assert_eq!(kinds(word), [name(word)], "{word}");
    }
}

#[test]
fn words_from_other_languages_are_z0315_with_the_zeff_form() {
    for (word, help) in [
        ("TRUE", "write `true`"),
        ("FALSE", "write `false`"),
        ("NULL", "a missing value is `NA`"),
        ("None", "a missing value is `NA`"),
    ] {
        assert_eq!(codes(word), ["Z0315"], "{word}");
        assert!(render(word).contains(help), "{word}");
    }
    // In backticks they are columns.
    assert_eq!(kinds("`TRUE`"), [name("TRUE")]);
}

#[test]
fn tokens_from_other_languages_are_z0315_with_the_zeff_form() {
    for (token, help) in [
        (";", "no semicolons"),
        ("&", "`&&`"),
        ("%", "`mod(a, b)`"),
        ("**", "`^`"),
        ("<-", "`let name = value`"),
    ] {
        assert_eq!(codes(token), ["Z0315"], "{token}");
        assert!(render(token).contains(help), "{token}");
    }
}

#[test]
fn characters_outside_the_grammar_are_z0301() {
    for text in ["café", "#", "$x", "a @ b", "x\u{a0}y", "日本", "\u{0}"] {
        assert!(
            codes(text).contains(&"Z0301".to_string()),
            "{text:?}: {:?}",
            codes(text)
        );
    }
    assert!(render("# comment").contains("comments start with `//`"));
    assert_eq!(codes("``"), ["Z0301"]);
    assert_eq!(codes("`abc"), ["Z0301"]);
}

// ----- operators -----

#[test]
fn every_operator_of_section_3_3() {
    let text = "== != <= >= < > && || => =~ ~~ :: := -> <-> ~ + - * / ^ ! = : , . .. | ?";
    let expected = [
        K::EqEq,
        K::NotEq,
        K::Le,
        K::Ge,
        K::Lt,
        K::Gt,
        K::AndAnd,
        K::OrOr,
        K::FatArrow,
        K::EqTilde,
        K::TildeTilde,
        K::ColonColon,
        K::ColonEq,
        K::Arrow,
        K::BiArrow,
        K::Tilde,
        K::Plus,
        K::Minus,
        K::Star,
        K::Slash,
        K::Caret,
        K::Bang,
        K::Assign,
        K::Colon,
        K::Comma,
        K::Dot,
        K::DotDot,
        K::Pipe,
        K::Question,
    ];
    // The dot before `..` is followed by more text, so it is not the end of the line.
    assert_eq!(kinds(text), expected);
}

#[test]
fn longest_operator_wins() {
    assert_eq!(kinds("a<=b"), [name("a"), K::Le, name("b")]);
    assert_eq!(kinds("a==b"), [name("a"), K::EqEq, name("b")]);
    assert_eq!(kinds("a=>b"), [name("a"), K::FatArrow, name("b")]);
    assert_eq!(kinds("y~x"), [name("y"), K::Tilde, name("x")]);
    assert_eq!(kinds("a::b"), [name("a"), K::ColonColon, name("b")]);
    assert_eq!(kinds("m:=a"), [name("m"), K::ColonEq, name("a")]);
    assert_eq!(
        kinds("a&&b||c"),
        [name("a"), K::AndAnd, name("b"), K::OrOr, name("c")]
    );
}

#[test]
fn two_dashes_are_an_edge_only_inside_a_dag() {
    assert_eq!(kinds("a--b"), [name("a"), K::Minus, K::Minus, name("b")]);
    let inside = all_kinds("dag {\n a -- b\n}\n");
    assert!(inside.contains(&K::DashDash));
    assert!(!all_kinds("model {\n a ~ b\n}\n").contains(&K::DashDash));
}

#[test]
fn brackets_and_calls() {
    assert_eq!(
        kinds("f(a, [b], {c: 1})"),
        [
            name("f"),
            K::LParen,
            name("a"),
            K::Comma,
            K::LBracket,
            name("b"),
            K::RBracket,
            K::Comma,
            K::LBrace,
            name("c"),
            K::Colon,
            K::Int(1),
            K::RBrace,
            K::RParen
        ]
    );
}

// ----- comments, doc lines, pragmas, file oddities -----

#[test]
fn comments_are_dropped() {
    assert_eq!(kinds("a // note\nb"), [name("a"), K::Newline, name("b")]);
    assert_eq!(kinds("// only a comment"), []);
    assert_eq!(kinds("//// four slashes are an ordinary comment"), []);
    assert_eq!(kinds("a // \"quoted\" ' ` stuff"), [name("a")]);
}

#[test]
fn a_doc_line_stands_next_to_the_function_it_describes() {
    let tokens = all_kinds("/// Share of rows.\n///\n/// More.\nfn f() -> Float {\n 1.0\n}\n");
    assert_eq!(
        tokens[..4],
        [
            K::Doc("Share of rows.".to_string()),
            K::Doc(String::new()),
            K::Doc("More.".to_string()),
            K::Keyword(Keyword::Fn)
        ],
        "no NEWLINE between the doc lines and `fn`"
    );
}

#[test]
fn a_doc_line_after_code_still_ends_the_statement() {
    let tokens = all_kinds("let a = 1 /// note\nlet b = 2\n");
    assert!(tokens.contains(&K::Doc("note".to_string())));
    let doc_at = tokens.iter().position(|k| matches!(k, K::Doc(_))).unwrap();
    assert_eq!(tokens[doc_at + 1], K::Newline);
}

#[test]
fn pragmas_are_kept_out_of_the_token_stream() {
    let lexed = lex(
        "a.zeff",
        "// zeff: allow(W0203) \"varies\" coded as NA\nlet a = 1 // zeff: allow(W0203) \"x\"\n",
    );
    assert_eq!(lexed.pragmas.len(), 2);
    assert_eq!(lexed.pragmas[0].text, "allow(W0203) \"varies\" coded as NA");
    assert!(lexed.tokens.iter().all(|t| !matches!(t.kind, K::Doc(_))));
    assert_eq!(
        kinds("// zeff: allow(W0203) \"x\"\nlet a = 1")[0],
        K::Keyword(Keyword::Let)
    );
}

#[test]
fn crlf_bom_tabs_and_missing_final_line_break() {
    assert_eq!(
        kinds("let a = 1\r\nlet b = 2\r\n"),
        kinds("let a = 1\nlet b = 2\n")
    );
    assert_eq!(kinds("\u{feff}let a = 1"), kinds("let a = 1"));
    assert_eq!(kinds("\tlet\ta\t=\t1"), kinds("let a = 1"));
    assert_eq!(all_kinds("a"), [name("a"), K::Newline, K::Eof]);
    assert_eq!(all_kinds(""), [K::Eof]);
    assert_eq!(all_kinds("\n\n// c\n\n"), [K::Eof]);
}

#[test]
fn unicode_in_strings_and_comments_is_fine() {
    assert!(codes("let label = \"naïve — 日本語 😀\" // コメント 😀\n").is_empty());
}

// ----- line breaks (section 3.1) -----

#[test]
fn a_line_break_ends_a_statement_and_blank_lines_collapse() {
    use K::*;
    assert_eq!(
        all_kinds("a\n\n// c\n\nb\n"),
        [name("a"), Newline, name("b"), Newline, Eof]
    );
    assert_eq!(all_kinds("\n\na"), [name("a"), Newline, Eof]);
}

#[test]
fn a_line_break_inside_brackets_means_nothing() {
    use K::*;
    assert_eq!(
        all_kinds("f(1,\n  2\n)\n"),
        [
            name("f"),
            LParen,
            Int(1),
            Comma,
            Int(2),
            RParen,
            Newline,
            Eof
        ]
    );
    assert_eq!(
        all_kinds("[1\n 2]\n")
            .iter()
            .filter(|k| **k == Newline)
            .count(),
        1
    );
    // A record literal and the body of a case are brackets too.
    assert_eq!(
        all_kinds("let r = {\n a: 1,\n b: 2\n}\n")
            .iter()
            .filter(|k| **k == Newline)
            .count(),
        1
    );
    assert_eq!(
        all_kinds("let r = case {\n a => 1,\n _ => 2\n}\n")
            .iter()
            .filter(|k| **k == Newline)
            .count(),
        1
    );
}

#[test]
fn a_line_ending_with_an_operator_or_comma_carries_on() {
    for text in [
        "let x = 1 +\n 2\n",
        "let x = a ~\n b\n",
        "let x = a =~\n b\n",
        "let x = a ~~\n b\n",
        "let x =\n 1\n",
        "let x = a:\n b\n",
        "let x = a as\n b\n",
        "let x = 1 ..\n 5\n",
        "let x = a,\n b\n",
        "let x = a &&\n b\n",
        "let x = a::\n b\n",
        "let x = a ->\n b\n",
        "let x = a =>\n b\n",
    ] {
        let newlines = all_kinds(text).iter().filter(|k| **k == K::Newline).count();
        assert_eq!(
            newlines, 1,
            "only the final line break ends the statement in {text:?}"
        );
    }
}

#[test]
fn a_line_starting_with_dot_name_continues_the_chain() {
    use K::*;
    let chain = all_kinds("wages\n    .where(a)\n    .add(b)\n");
    assert_eq!(chain.iter().filter(|k| **k == Newline).count(), 1);
    // Blank lines and comments between the lines do not break the chain.
    let spaced = all_kinds("wages\n\n    // keep\n    .where(a)\n");
    assert_eq!(spaced.iter().filter(|k| **k == Newline).count(), 1);
    // A line beginning with `..` is a spread, not a continuation.
    let spread = all_kinds("f(a)\n..opts\n");
    assert_eq!(
        spread,
        [
            name("f"),
            LParen,
            name("a"),
            RParen,
            Newline,
            DotDot,
            name("opts"),
            Newline,
            Eof
        ]
    );
    // `.5` is not a chain either.
    assert!(codes("a\n.5\n").contains(&"Z0310".to_string()));
}

#[test]
fn else_after_the_closing_brace_on_one_line() {
    let tokens = all_kinds("if x {\n a\n} else {\n b\n}\n");
    let else_at = tokens
        .iter()
        .position(|k| *k == K::Keyword(Keyword::Else))
        .unwrap();
    assert_eq!(tokens[else_at - 1], K::RBrace);
    assert_eq!(tokens[else_at + 1], K::LBlock);
    // Statements inside the blocks are separated by line breaks.
    assert_eq!(tokens.iter().filter(|k| **k == K::Newline).count(), 5);
}

#[test]
fn a_block_inside_square_brackets_still_ends_statements() {
    // `[for ... { a NEWLINE b }]` (section 3.1).
    let tokens = all_kinds("let r = [for i in 1..3 {\n a\n b\n}]\n");
    assert_eq!(tokens.iter().filter(|k| **k == K::Newline).count(), 4);
}

#[test]
fn an_arm_may_hold_a_block_or_a_record() {
    let block = all_kinds("let r = case {\n x => {\n y\n z\n },\n _ => 0\n}\n");
    assert_eq!(
        block.iter().filter(|k| **k == K::Newline).count(),
        4,
        "{block:?}"
    );
    let record = all_kinds("let r = case {\n x => {a: 1,\n b: 2},\n _ => {}\n}\n");
    assert_eq!(
        record.iter().filter(|k| **k == K::Newline).count(),
        1,
        "{record:?}"
    );
}

#[test]
fn model_and_dag_blocks_end_a_line_at_each_line_break() {
    let model = all_kinds("let m = model {\n dem60 =~ y1 + y2\n dem60 ~ ind60\n}\n");
    assert_eq!(
        model.iter().filter(|k| **k == K::Newline).count(),
        4,
        "{model:?}"
    );
    let dag = all_kinds("let g = dag {\n a -> b -> c\n latent u\n}\n");
    assert_eq!(
        dag.iter().filter(|k| **k == K::Newline).count(),
        4,
        "{dag:?}"
    );
}

#[test]
fn a_line_may_not_end_with_a_dot() {
    assert_eq!(codes("x.\ny\n"), ["Z0312"]);
    assert_eq!(codes("x. // comment\ny\n"), ["Z0312"]);
    assert_eq!(codes("x.\n"), ["Z0312"]);
    assert!(codes("x.f()\n").is_empty());
}

// ----- brackets that do not match -----

#[test]
fn an_unclosed_bracket_is_z0302_and_a_stray_closer_is_z0303() {
    assert_eq!(codes("f(a, b"), ["Z0302"]);
    assert_eq!(codes("let a = [1, 2"), ["Z0302"]);
    assert_eq!(codes("a)"), ["Z0303"]);
    // The stray `]` is met first; the `(` is found unclosed at the end.
    assert_eq!(codes("f(a]"), ["Z0303", "Z0302"]);
    // `f({a: 1)`: the brace was never closed.
    assert_eq!(codes("f({a: 1)"), ["Z0302"]);
}

/// The Z0302 example of the specification, rebuilt from its own two source lines.
#[test]
fn the_specification_example_of_an_unclosed_bracket_is_reproduced() {
    const SPEC: &str = include_str!("../../../docs/spec/zeff_language_reference.md");
    let block: Vec<&str> = SPEC
        .split("```")
        .find(|b| b.trim_start().starts_with("error[Z0302]"))
        .expect("the example is in the specification")
        .trim_start()
        .lines()
        .collect();
    let expected = block.join("\n") + "\n";
    let text = format!(
        "{}let m = wages.reg(lwage ~ educ + exper, se: robust\nreport([m])\n",
        "\n".repeat(6)
    );
    assert_eq!(render(&text), expected);
}

// ----- the specification's scripts -----

#[test]
fn the_sample_script_lexes_without_a_diagnostic() {
    let text = include_str!("../../../docs/spec/zeff_samples.zeff");
    let lexed = lex("zeff_samples.zeff", text);
    let sources = {
        let mut s = Sources::new();
        s.add(Source::new("zeff_samples.zeff", text));
        s
    };
    let shown: Vec<String> = lexed
        .diagnostics
        .iter()
        .map(|d| d.render(&sources))
        .collect();
    assert!(shown.is_empty(), "{}", shown.join("\n"));
    assert!(lexed.tokens.len() > 1000);
}

#[test]
fn the_complete_analysis_of_section_10_lexes_without_a_diagnostic() {
    const SPEC: &str = include_str!("../../../docs/spec/zeff_language_reference.md");
    let after = SPEC
        .split("### 10.1 The script")
        .nth(1)
        .expect("section 10.1");
    let script = after.split("```").nth(1).expect("the script block");
    let lexed = lex("scripts/main.zeff", script);
    let mut sources = Sources::new();
    sources.add(Source::new("scripts/main.zeff", script));
    let shown: Vec<String> = lexed
        .diagnostics
        .iter()
        .map(|d| d.render(&sources))
        .collect();
    assert!(shown.is_empty(), "{}", shown.join("\n"));
}

// ----- the messages, as snapshots -----

#[test]
fn messages() {
    let cases: &[(&str, &str)] = &[
        ("number_leading_dot", "let a = .5\n"),
        ("number_trailing_dot", "let a = 5.\n"),
        ("number_leading_zero", "let a = 007\n"),
        ("number_leading_zero_float", "let a = 00.5\n"),
        ("number_separator", "let a = 1__000\n"),
        ("number_missing_exponent", "let a = 1e+\n"),
        ("number_runs_into_name", "let a = 2x\n"),
        ("number_hexadecimal", "let a = 0x10\n"),
        ("number_too_big", "let a = 18446744073709551616\n"),
        ("number_float_too_big", "let a = 1e999\n"),
        ("string_unknown_escape", "let p = \"\\d+\"\n"),
        ("string_bad_unicode_escape", "let p = \"\\u{110000}\"\n"),
        ("string_unterminated", "let p = \"abc\nlet q = 1\n"),
        ("raw_string_unterminated", "let p = r\"abc\n"),
        ("single_quotes", "let p = 'male'\n"),
        ("semicolon", "let a = 1; let b = 2\n"),
        ("ampersand", "let a = x & y\n"),
        ("percent", "let a = x % 2\n"),
        ("power_operator", "let a = x ** 2\n"),
        ("assignment_arrow", "x <- 1\n"),
        ("upper_case_true", "let a = TRUE\n"),
        ("null", "let a = NULL\n"),
        ("hash_comment", "# a comment\nlet a = 1\n"),
        ("non_ascii_name", "let café = 1\n"),
        ("no_break_space", "let a =\u{a0}1\n"),
        ("empty_backticks", "let a = ``\n"),
        ("unclosed_backticks", "wages.mean(`gross sales)\n"),
        ("trailing_dot", "wages.\n    mean(x)\n"),
        ("unclosed_paren", "let m = f(a, b\n"),
        (
            "unclosed_paren_before_next_statement",
            "let m = wages.reg(a ~ b, se: robust\nlet n = 1\n",
        ),
        ("stray_closer", "let a = 1)\n"),
        ("unclosed_inside", "let a = f([1, 2)\n"),
    ];
    for (name, text) in cases {
        insta::assert_snapshot!(*name, render(text));
    }
}
