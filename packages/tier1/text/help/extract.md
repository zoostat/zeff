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

## See also
contains, replace, split, slice

## Examples
> extract("abc123def45", "[0-9]+")
123
> extract("2026-10-16", r"\d{2}$")
16
> extract("Hello, World", "xyz")
NA
