# contains

## Summary
True where text contains a text or, with regex: true, a regular expression

## In brief
- `contains(s, "abc")`: whether the text holds `abc`, read as it stands.
- `contains(s, "[0-9]+", regex: true)`: whether it holds a match of the regular expression.
- `contains(s, "abc", ignore_case: true)`: either of them, capitals and small letters alike.

## Description
Whether the text holds `pattern`: the text itself unless `regex: true`, when the pattern is a
regular expression in the syntax of section 3.8 of the Language Reference (Unicode classes,
anchors, alternation, groups and repetition; no back-references or look-around, so every match
runs in linear time and reads the same on every machine). `ignore_case: true` folds case by
Unicode simple case folding in either mode. Every text contains the empty text. A pattern the
engine cannot read is an error (Z0419) that names look-around and back-references when they are
the cause. `NA` in the text or the pattern gives `NA`. Given a column, it applies row by row and
gives a Bool column, the usual condition for `where`. For whether a value is among a list's
items, see `is_in`. Like R's `grepl(fixed = TRUE)` and `grepl(perl = TRUE)`.

## Arguments
- s: The text, or a column of texts.
- pattern: The text to look for, or a regular expression under `regex: true`.
- regex: Read `pattern` as a regular expression.
- ignore_case: Fold case before matching.

## Value
A Bool; a column of Bools when `s` is a column.

## Missing values
`NA` in the text or in the pattern gives `NA`. In a column each row is taken by itself, and no row is dropped.

## A common mistake
Looking for a character that means something in a regular expression, with `regex: true` on. A bracket opens a group there, and the pattern cannot be read:

> contains("f(x)", "(", regex: true)
error[Z0419]: `(` is not a regular expression: unclosed group
  = help: the syntax is Rust's `regex` crate's: Unicode classes, no back-references or look-around (section 3.8); for a literal text, leave `regex:` off

Leave `regex:` off for a text that is meant as it stands, `contains("f(x)", "(")`, or escape the character, `r"\("`.

## Coming from R or Stata
| contains(s, "abc") | grepl("abc", s, fixed = TRUE) | strpos(s, "abc") > 0 |
| contains(s, "[0-9]+", regex: true) | grepl("[0-9]+", s, perl = TRUE) | regexm(s, "[0-9]+") |
| contains(s, "abc", ignore_case: true) | grepl("abc", s, ignore.case = TRUE) | strpos(strlower(s), "abc") > 0 |

R's `grepl` reads its pattern as a regular expression unless `fixed = TRUE` is given, so a full
stop there matches any character; here the pattern is the text itself unless `regex: true`. For
a missing text R's `grepl` gives `FALSE`; here the answer is `NA`.

## Related tasks
- For the start or the end of a text only, use `starts_with` or `ends_with`.
- To take the matching part out, use `extract`.
- For whether a value is one of a list, use `is_in`.

## See also
starts_with, ends_with, extract, replace, is_in

## Examples
// The text as it stands.
> contains("Hello, World", "World")
true

// Capitals and small letters alike.
> contains("Hello, World", "world", ignore_case: true)
true

// A regular expression: one digit or more.
> contains("abc123", "[0-9]+", regex: true)
true

// Row by row, as a condition: the codes with two digits together.
> let people = frame(name = ["  Ann Lee", "bob STONE ", NA], code = ["A-12", "b-7", "C-103"])
> people.where(contains(code, "[0-9]{2}", regex: true))
frame · 2 rows × 2 variables

      name        code
  1     Ann Lee   A-12
  2   NA          C-103

// On the labels of a category of the shipped firms.
> dataset(firms).where(contains(industry, "ing")).n_rows()
47
