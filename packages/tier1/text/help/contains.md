# contains

## Summary
True where text contains a text or, with regex: true, a regular expression

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

## See also
starts_with, ends_with, extract, replace, is_in

## Examples
> contains("Hello, World", "World")
true
> contains("Hello, World", "world", ignore_case: true)
true
> contains("abc123", "[0-9]+", regex: true)
true
> contains("a.b", ".", regex: true)
true
