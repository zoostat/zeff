//! Tokens (Language Reference, section 3.4, "Tokens").

use zeff_diagnostics::Span;

/// The 26 reserved words of section 3.1: the 18 in use and the 8 reserved for later.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Keyword {
    Let,
    Fn,
    If,
    Else,
    Case,
    For,
    In,
    Break,
    Continue,
    Return,
    Use,
    Model,
    Dag,
    As,
    True,
    False,
    Na,
    Inf,
    // Reserved for later use, so that no edition is needed to add them.
    Type,
    While,
    Match,
    Mut,
    Pub,
    Loop,
    Trait,
    Impl,
}

impl Keyword {
    /// All 26, the 18 in use first.
    pub const ALL: [Keyword; 26] = [
        Keyword::Let,
        Keyword::Fn,
        Keyword::If,
        Keyword::Else,
        Keyword::Case,
        Keyword::For,
        Keyword::In,
        Keyword::Break,
        Keyword::Continue,
        Keyword::Return,
        Keyword::Use,
        Keyword::Model,
        Keyword::Dag,
        Keyword::As,
        Keyword::True,
        Keyword::False,
        Keyword::Na,
        Keyword::Inf,
        Keyword::Type,
        Keyword::While,
        Keyword::Match,
        Keyword::Mut,
        Keyword::Pub,
        Keyword::Loop,
        Keyword::Trait,
        Keyword::Impl,
    ];

    /// The word as written (`NA` and `Inf` keep their capitals).
    pub fn as_str(self) -> &'static str {
        match self {
            Keyword::Let => "let",
            Keyword::Fn => "fn",
            Keyword::If => "if",
            Keyword::Else => "else",
            Keyword::Case => "case",
            Keyword::For => "for",
            Keyword::In => "in",
            Keyword::Break => "break",
            Keyword::Continue => "continue",
            Keyword::Return => "return",
            Keyword::Use => "use",
            Keyword::Model => "model",
            Keyword::Dag => "dag",
            Keyword::As => "as",
            Keyword::True => "true",
            Keyword::False => "false",
            Keyword::Na => "NA",
            Keyword::Inf => "Inf",
            Keyword::Type => "type",
            Keyword::While => "while",
            Keyword::Match => "match",
            Keyword::Mut => "mut",
            Keyword::Pub => "pub",
            Keyword::Loop => "loop",
            Keyword::Trait => "trait",
            Keyword::Impl => "impl",
        }
    }

    /// The keyword a word spells, if any. Case matters: `Let` and `na` are ordinary names.
    pub fn from_word(word: &str) -> Option<Keyword> {
        Keyword::ALL.into_iter().find(|k| k.as_str() == word)
    }

    /// One of the eight reserved so that a later release can give them a meaning.
    pub fn is_reserved_for_later(self) -> bool {
        matches!(
            self,
            Keyword::Type
                | Keyword::While
                | Keyword::Match
                | Keyword::Mut
                | Keyword::Pub
                | Keyword::Loop
                | Keyword::Trait
                | Keyword::Impl
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    /// A name: ASCII letters, digits and `_`, or any text in backticks (the backticks are not part
    /// of the value). A reserved word in backticks is a name.
    Name(String),
    Keyword(Keyword),
    /// `_` alone, the catch-all of `case`.
    Underscore,
    /// An integer literal. The value is the magnitude as written: it may be 2^63, which is allowed
    /// only directly after a minus sign (section 3.2); the parser checks that.
    Int(u64),
    Float(f64),
    /// A string with its escapes decoded.
    Str(String),
    /// A raw string, `r"..."`, without escapes.
    RawStr(String),
    /// A `///` line: the text after the slashes, without one leading space.
    Doc(String),

    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Comma,
    /// `.`
    Dot,
    /// `..`: range and spread.
    DotDot,
    Colon,
    /// `::`
    ColonColon,
    /// `:=`
    ColonEq,
    /// `=`
    Assign,
    EqEq,
    NotEq,
    Lt,
    Le,
    Gt,
    Ge,
    /// `=>`
    FatArrow,
    /// `->`
    Arrow,
    /// `<->`
    BiArrow,
    /// `--`, only inside a `dag { }` block; elsewhere two minus signs.
    DashDash,
    /// `~`
    Tilde,
    /// `=~`
    EqTilde,
    /// `~~`
    TildeTilde,
    Plus,
    Minus,
    Star,
    Slash,
    Caret,
    Bang,
    AndAnd,
    OrOr,
    /// A single `|`, which appears only between the words of a type (`classical | robust`); the
    /// parser refuses it anywhere else (Z0315).
    Pipe,
    /// `?`, after a type.
    Question,

    /// A line break that ends a statement or a line of a `model { }` or `dag { }` block.
    Newline,
    /// The end of the text.
    Eof,
    /// Something the lexer could not read. The lexer has already reported it; later stages skip it
    /// without a second message.
    Error,
}

impl TokenKind {
    /// Whether a line ending with this token carries on to the next line (section 3.1: a comma, a
    /// binary operator, `=`, `=>`, `->`, `::`).
    pub fn continues_line(&self) -> bool {
        matches!(
            self,
            TokenKind::Comma
                | TokenKind::Plus
                | TokenKind::Minus
                | TokenKind::Star
                | TokenKind::Slash
                | TokenKind::Caret
                | TokenKind::EqEq
                | TokenKind::NotEq
                | TokenKind::Lt
                | TokenKind::Le
                | TokenKind::Gt
                | TokenKind::Ge
                | TokenKind::AndAnd
                | TokenKind::OrOr
                | TokenKind::Tilde
                | TokenKind::EqTilde
                | TokenKind::TildeTilde
                | TokenKind::Colon
                | TokenKind::ColonEq
                | TokenKind::ColonColon
                | TokenKind::DotDot
                | TokenKind::Assign
                | TokenKind::FatArrow
                | TokenKind::Arrow
                | TokenKind::BiArrow
                | TokenKind::DashDash
                | TokenKind::Keyword(Keyword::As)
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}
