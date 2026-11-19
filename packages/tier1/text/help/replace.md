# replace

## Summary
Replace every occurrence of text or a pattern (Rust regex syntax, section 3.8)

## Description
The text with every occurrence of `from` replaced by `to`: `from` is the text itself unless
`regex: true`, when it is a regular expression in the syntax of section 3.8 of the Language
Reference. The replacement is plain text in both modes, so a `$` or a backslash in `to` stands
for itself and no group is referred to. An empty literal `from` is refused (Z0401), as R refuses
it; under `regex: true` an expression that matches the empty text inserts `to` between the
characters and at both ends. A pattern the engine cannot read is Z0419. `NA` in any argument
gives `NA`. Given a column, it applies row by row. Like R's `gsub(fixed = TRUE)` and
`gsub(perl = TRUE)` with a plain replacement, and Stata's `subinstr`.

## Arguments
- s: The text, or a column of texts.
- from: The text to replace, or a regular expression under `regex: true`.
- to: The text put in its place.
- regex: Read `from` as a regular expression.

## Value
A Str; a column of Strs when `s` is a column.

## Missing values
`NA` in any of the three gives `NA`. In a column each row is taken by itself, and no row is dropped.

## A common mistake
Replacing a character that means something in a regular expression, with `regex: true` on. A plus sign repeats what stands before it there, and alone it cannot be read:

> replace("1+1", "+", " plus ", regex: true)
error[Z0419]: `+` is not a regular expression: repetition operator missing expression
  = help: the syntax is Rust's `regex` crate's: Unicode classes, no back-references or look-around (section 3.8); for a literal text, leave `regex:` off

Leave `regex:` off for a text that is meant as it stands: `replace("1+1", "+", " plus ")` is `1 plus 1`.

## Coming from R or Stata
| replace(s, "a", "b") | gsub("a", "b", s, fixed = TRUE) | subinstr(s, "a", "b", .) |
| replace(s, "[0-9]+", "#", regex: true) | gsub("[0-9]+", "#", s, perl = TRUE) | ustrregexra(s, "[0-9]+", "#") |

Every occurrence is replaced, as R's `gsub` does; R's `sub` and Stata's `regexr` replace the
first only, and Stata's `subinstr` needs the full stop as its last argument to replace them all.
In R the replacement may refer to a group, `\\1`; here the replacement is plain text. Stata's
`replace` is another thing, the command that changes a variable: that is `update` here.

## Related tasks
- To take the matching part out, use `extract`.
- To remove white space at the ends, use `trim`.
- To change the values of a variable where a condition holds, use `update`.

## See also
contains, extract, split, trim

## Examples
// Every occurrence of a text.
> replace("Hello, World", "o", "0")
Hell0, W0rld

// A full stop is a full stop, unless `regex: true`.
> replace("a.b.c", ".", "-")
a-b-c

// A regular expression: every run of digits.
> replace("abc123def45", "[0-9]+", "#", regex: true)
abc#def#

// Row by row.
> let people = frame(name = ["  Ann Lee", "bob STONE ", NA], code = ["A-12", "b-7", "C-103"])
> people.add(code = replace(code, "-", "/"))
frame · 3 rows × 2 variables

      name         code
  1     Ann Lee    A/12
  2   bob STONE    b/7
  3   NA           C/103
note: `code` replaced the variable of that name (line 5)
