# extract

## Summary
First match of a regular expression (Rust regex syntax, section 3.8)

## Description
The first part of the text that matches `pattern`, a regular expression in the syntax of
section 3.8 of the Language Reference, or `NA` when nothing matches; a pattern that matches
the empty text gives the empty text where it first does. The whole match is returned, groups
included, so the pattern is written for the part wanted: `extract(s, "[0-9]+")` is the first
number. A pattern the engine cannot read is Z0419. `NA` in either argument gives `NA`. Given a
column, it applies row by row. Like R's `regmatches(s, regexpr(pattern, s, perl = TRUE))` and
Stata's `regexs` after `regexm`.

## Arguments
- s: The text, or a column of texts.
- pattern: A regular expression.

## Value
A Str, `NA` when nothing matches; a column of Strs when `s` is a column.

## Missing values
`NA` in the text or in the pattern gives `NA`, and so does a text with no match. In a column each row keeps its place, so the result lines up with the frame.

## A common mistake
Using look-ahead, as a pattern written for R with `perl = TRUE` may. The expressions here have none, so that every match runs in a time that grows only with the length of the text:

> extract("price: 12 EUR", r"\d+(?= EUR)")
error[Z0419]: `\d+(?= EUR)` is not a regular expression: it uses look-around, which Zeff's regular expressions do not have
  = help: the syntax is Rust's `regex` crate's: Unicode classes, no back-references or look-around (section 3.8); for a literal text, leave `regex:` off

Match what is wanted directly, or in two steps: `extract(extract("price: 12 EUR", r"\d+ EUR"), r"\d+")` is `12`.

## Coming from R or Stata
| extract(s, "[0-9]+") | regmatches(s, regexpr("[0-9]+", s, perl = TRUE)) | regexs(0) after regexm(s, "[0-9]+") |

R's `regmatches` leaves out the texts with no match, so its result is shorter than its input and
no longer lines up with the data; here a text with no match gives `NA` in its own row.
`stringr::str_extract` in R behaves as `extract` does.

## Related tasks
- For whether there is a match, use `contains`.
- For a part at a known position, use `slice`.
- To cut a text at its separators, use `split`.

## See also
contains, replace, split, slice

## Examples
// The first run of digits.
> extract("abc123def45", "[0-9]+")
123

// Anchored at the end: the last two digits.
> extract("2026-10-16", r"\d{2}$")
16

// No match gives `NA`.
> extract("Hello, World", "xyz")
NA

// Row by row: the number in each code, as text.
> let people = frame(name = ["  Ann Lee", "bob STONE ", NA], code = ["A-12", "b-7", "C-103"])
> people.add(number = extract(code, "[0-9]+"))
frame · 3 rows × 3 variables

      name         code    number
  1     Ann Lee    A-12    12
  2   bob STONE    b-7     7
  3   NA           C-103   103
