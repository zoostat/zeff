//! Lexer and parser for the Zeff grammar (Language Reference, section 3).

mod lexer;
mod token;

pub use lexer::{Lexed, Pragma, lex};
pub use token::{Keyword, Token, TokenKind};
