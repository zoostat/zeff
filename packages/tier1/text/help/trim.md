# trim

## Summary
Remove leading and trailing Unicode white space

## Description
The text without the white space at its start and its end: every character with the Unicode
`White_Space` property, so spaces, tabs, line breaks, the no-break space and the typographic
spaces alike (section 3.8 of the Language Reference). White space inside the text is kept.
`NA` gives `NA`. Given a column, it applies row by row, which is the usual first step after
importing text typed by hand. Like R's `trimws`, which by default removes only ASCII white
space, and Stata's `strtrim`.

## Arguments
- s: The text, or a column of texts.

## Value
A Str; a column of Strs when `s` is a column.

## See also
upper, lower, pad, len

## Examples
> trim("  padded  ")
padded
> len(trim("\u{a0}wide\u{2003}"))
4
> trim("inner  space")
inner  space
