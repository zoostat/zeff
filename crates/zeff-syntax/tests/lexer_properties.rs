//! Property tests of the lexer: whatever the input, it answers with tokens and diagnostics and
//! never panics, every byte is accounted for, and the line-ending style makes no difference.

use proptest::prelude::*;
use zeff_syntax::{Keyword, TokenKind as K, lex};

/// Text drawn from the characters that matter to the lexer, so that strings, comments, brackets,
/// numbers and operators meet in odd ways far more often than in random Unicode.
fn code_like() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-zA-Z0-9_ \\t\\n\"'`.,:;=~+*/^!<>&|%?(){}\\[\\]#$@\\\\-]{0,160}")
        .unwrap()
}

fn kinds(text: &str) -> Vec<K> {
    lex("a.zeff", text)
        .tokens
        .into_iter()
        .map(|t| t.kind)
        .collect()
}

/// Whether the text between two tokens holds only whitespace and comments.
fn only_blank_or_comment(gap: &str) -> bool {
    gap.split('\n').all(|line| {
        let code = line.find("//").map_or(line, |at| &line[..at]);
        code.chars()
            .all(|c| c == ' ' || c == '\t' || c == '\r' || c == '\u{feff}')
    })
}

fn check_invariants(text: &str) -> Result<(), TestCaseError> {
    let lexed = lex("a.zeff", text);
    let tokens = &lexed.tokens;
    prop_assert_eq!(tokens.last().map(|t| &t.kind), Some(&K::Eof));
    let mut at = 0;
    for token in tokens {
        let (start, end) = (token.span.start, token.span.end);
        prop_assert!(
            start >= at,
            "tokens overlap or run backwards at {start} (previous end {at})"
        );
        prop_assert!(end >= start && end <= text.len());
        prop_assert!(text.is_char_boundary(start) && text.is_char_boundary(end));
        prop_assert!(
            only_blank_or_comment(&text[at..start]),
            "text outside every token that is neither blank nor a comment: {:?}",
            &text[at..start]
        );
        at = end;
    }
    prop_assert!(only_blank_or_comment(&text[at..]));
    // An error token always comes with a diagnostic.
    if tokens.iter().any(|t| t.kind == K::Error) {
        prop_assert!(!lexed.diagnostics.is_empty());
    }
    // A diagnostic is never silent about where it is.
    for diagnostic in lexed.diagnostics.iter() {
        prop_assert!(!diagnostic.labels.is_empty());
    }
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2000))]

    #[test]
    fn never_panics_and_accounts_for_every_byte_on_code_like_text(text in code_like()) {
        check_invariants(&text)?;
    }

    #[test]
    fn never_panics_and_accounts_for_every_byte_on_any_text(text in any::<String>()) {
        check_invariants(&text)?;
    }

    #[test]
    fn the_line_ending_style_changes_nothing(text in code_like()) {
        let crlf = text.replace('\n', "\r\n");
        prop_assert_eq!(kinds(&text), kinds(&crlf));
    }

    #[test]
    fn a_leading_byte_order_mark_changes_nothing(text in code_like()) {
        prop_assert_eq!(kinds(&text), kinds(&format!("\u{feff}{text}")));
    }

    #[test]
    fn valid_lexemes_joined_by_spaces_give_their_tokens_back(picks in prop::collection::vec(0..LEXEMES.len(), 1..40)) {
        let text = picks.iter().map(|&i| LEXEMES[i].0).collect::<Vec<_>>().join(" ");
        let mut expected: Vec<K> = picks.iter().map(|&i| (LEXEMES[i].1)()).collect();
        expected.push(K::Newline);
        expected.push(K::Eof);
        prop_assert_eq!(kinds(&text), expected);
    }
}

type Lexeme = (&'static str, fn() -> K);

/// Lexemes that are valid on their own anywhere in a line. Brackets, `.` and `--` are left out:
/// a bracket must be balanced, a line may not end with `.`, and `--` is an edge only in a dag.
const LEXEMES: &[Lexeme] = &[
    ("foo", || K::Name("foo".into())),
    ("x1", || K::Name("x1".into())),
    ("_tmp", || K::Name("_tmp".into())),
    ("_", || K::Underscore),
    ("`gross sales`", || K::Name("gross sales".into())),
    ("42", || K::Int(42)),
    ("1_000", || K::Int(1000)),
    ("0.25", || K::Float(0.25)),
    ("1e-3", || K::Float(1e-3)),
    ("\"text\"", || K::Str("text".into())),
    ("\"a\\nb\"", || K::Str("a\nb".into())),
    ("r\"\\d+\"", || K::RawStr("\\d+".into())),
    ("let", || K::Keyword(Keyword::Let)),
    ("NA", || K::Keyword(Keyword::Na)),
    ("true", || K::Keyword(Keyword::True)),
    ("+", || K::Plus),
    ("-", || K::Minus),
    ("*", || K::Star),
    ("/", || K::Slash),
    ("^", || K::Caret),
    ("!", || K::Bang),
    ("==", || K::EqEq),
    ("!=", || K::NotEq),
    ("<", || K::Lt),
    ("<=", || K::Le),
    (">", || K::Gt),
    (">=", || K::Ge),
    ("&&", || K::AndAnd),
    ("||", || K::OrOr),
    ("~", || K::Tilde),
    ("=~", || K::EqTilde),
    ("~~", || K::TildeTilde),
    (":=", || K::ColonEq),
    ("=>", || K::FatArrow),
    ("->", || K::Arrow),
    ("<->", || K::BiArrow),
    ("::", || K::ColonColon),
    (":", || K::Colon),
    ("=", || K::Assign),
    (",", || K::Comma),
    ("..", || K::DotDot),
    ("|", || K::Pipe),
    ("?", || K::Question),
];
