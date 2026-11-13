// The scanner of the Zeff grammar for the ends of lines (Language Reference, section 3.1,
// "Statements").
//
// A line break ends a statement where the grammar asks for one: at the top level and in a
// block, after a statement that could end there. It does not where the next line that is
// neither blank nor a comment begins with `.` and a name, which continues a chain:
//
//     wages
//         .where(year == 2015)
//         .mean(wage)
//
// A line that begins with `..` is a spread and continues nothing. Where the grammar asks for
// no line break (inside round or square brackets, a record or `case { }`, after an operator
// or a comma), the grammar's own rule for blank space passes over it. The end of the file
// ends the last statement as a line break does.

#include "tree_sitter/parser.h"

enum TokenType {
  NEWLINE,
};

void *tree_sitter_zeff_external_scanner_create(void) { return NULL; }

void tree_sitter_zeff_external_scanner_destroy(void *payload) { (void)payload; }

unsigned tree_sitter_zeff_external_scanner_serialize(void *payload, char *buffer) {
  (void)payload;
  (void)buffer;
  return 0;
}

void tree_sitter_zeff_external_scanner_deserialize(void *payload, const char *buffer,
                                                   unsigned length) {
  (void)payload;
  (void)buffer;
  (void)length;
}

static bool begins_a_name(int32_t c) {
  return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_' || c == '`';
}

bool tree_sitter_zeff_external_scanner_scan(void *payload, TSLexer *lexer,
                                            const bool *valid_symbols) {
  (void)payload;
  if (!valid_symbols[NEWLINE]) {
    return false;
  }
  // The blank space before the end of the line. A comment there is left to the grammar,
  // which keeps it as a comment; the scanner is asked again after it.
  while (lexer->lookahead == ' ' || lexer->lookahead == '\t' || lexer->lookahead == '\r') {
    lexer->advance(lexer, true);
  }
  if (lexer->eof(lexer)) {
    // The end of the file ends the last statement. The token is empty, and is asked for
    // only once, after a statement.
    lexer->mark_end(lexer);
    lexer->result_symbol = NEWLINE;
    return true;
  }
  if (lexer->lookahead != '\n') {
    return false;
  }
  lexer->advance(lexer, false);
  lexer->mark_end(lexer);
  // Past blank lines and lines that hold only a comment, to what the next line begins with.
  for (;;) {
    while (lexer->lookahead == ' ' || lexer->lookahead == '\t' || lexer->lookahead == '\r' ||
           lexer->lookahead == '\n') {
      lexer->advance(lexer, false);
    }
    if (lexer->lookahead != '/') {
      break;
    }
    lexer->advance(lexer, false);
    if (lexer->lookahead != '/') {
      // A line that begins with `/` and no comment: no continuation.
      lexer->result_symbol = NEWLINE;
      return true;
    }
    while (lexer->lookahead != '\n' && !lexer->eof(lexer)) {
      lexer->advance(lexer, false);
    }
  }
  if (lexer->lookahead == '.') {
    lexer->advance(lexer, false);
    if (begins_a_name(lexer->lookahead)) {
      // `.name` continues the statement: the line break means nothing.
      return false;
    }
  }
  lexer->result_symbol = NEWLINE;
  return true;
}
