# upper

## Summary
Upper case by Unicode simple case mapping, one scalar value to one

## Description
The text in upper case: each Unicode scalar value is replaced by its simple upper-case mapping,
one value to one, so that the text's length never changes (section 3.8 of the Language
Reference). A character whose full upper-case form is longer, `ß` or the ligature `ﬁ`, stays as
it is; accented letters, Greek and Cyrillic map as expected; digits, punctuation, CJK and emoji
are unchanged, and a combining mark stays after the letter it follows. No locale is consulted,
so the result is the same on every machine. `NA` gives `NA`. Given a column, it applies row by
row. Like R's `toupper`, which on most platforms gives these same mappings.

## Arguments
- s: The text, or a column of texts.

## Value
A Str of the same length; a column of Strs when `s` is a column.

## See also
lower, trim, format, len

## Examples
> upper("café")
CAFÉ
> upper("Straße")
STRAßE
> len(upper("ﬁsh"))
3
> upper("mixed Case 1")
MIXED CASE 1
