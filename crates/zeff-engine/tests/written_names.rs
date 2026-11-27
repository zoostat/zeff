//! A name as a script writes it (release 0.10.0, step 11): `core` writes a node of a causal
//! graph bare or in backticks, and keeps for that its own list of the reserved words, since it
//! does not know the lexer. The two lists are held to each other here, and what `core` writes
//! is held to what the lexer reads.

use zeff_core::dag::{reserved_words, written_name};
use zeff_syntax::Keyword;

#[test]
fn the_reserved_words_of_core_are_the_lexers() {
    let lexer: Vec<&str> = Keyword::ALL.into_iter().map(Keyword::as_str).collect();
    assert_eq!(reserved_words(), lexer.as_slice());
    assert_eq!(lexer.len(), 26);
}

#[test]
fn a_written_name_reads_back_as_the_name() {
    for name in [
        "educ",
        "_x9",
        "gross sales",
        "9lives",
        "for",
        "dag",
        "NA",
        "Inf",
        "_",
        "latent",
        "selected",
        "naïve",
        "a-b",
        "a.b",
    ] {
        let written = written_name(name);
        // A graph whose one node is the name prints it back; the block is checked and run.
        let outcome = zeff_testkit::run(&format!("dag {{\n    {written} -> other\n}}\n"));
        assert_eq!(outcome.exit_code, 0, "{name}: {}", outcome.output.stderr);
        assert_eq!(
            outcome.output.stdout,
            format!("dag {{\n    {written} -> other\n}}\n"),
            "{name}"
        );
        // Bare exactly when the name is letters, digits and `_` and no reserved word.
        let plain = name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
            && !name.starts_with(|c: char| c.is_ascii_digit())
            && name != "_"
            && Keyword::from_word(name).is_none();
        assert_eq!(written == name, plain, "{name}");
    }
}
