# ends_with

## Summary
True where text ends with the suffix

## Description
Whether the text ends with `suffix`, compared scalar value by scalar value and case-sensitively,
with no locale (section 3.8 of the Language Reference). Every text ends with the empty text.
`NA` in either argument gives `NA`. Given a column, it applies row by row and gives a Bool
column, the usual condition for `where`. Like R's `endsWith`.

## Arguments
- s: The text, or a column of texts.
- suffix: The text it may end with.

## Value
A Bool; a column of Bools when `s` is a column.

## See also
starts_with, slice, lower

## Examples
> ends_with("report.docx", ".docx")
true
> ends_with("café", "é")
true
> ends_with("café", "cafés")
false
