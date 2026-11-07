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

## See also
contains, extract, split, trim

## Examples
> replace("Hello, World", "o", "0")
Hell0, W0rld
> replace("a.b.c", ".", "-")
a-b-c
> replace("abc123def45", "[0-9]+", "#", regex: true)
abc#def#
> replace("  two  words ", r"\s+", " ", regex: true)
 two words 
