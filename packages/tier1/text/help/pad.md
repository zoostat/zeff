# pad

## Summary
Pad text to a width

## Description
The text with `fill:` repeated on its left until it is `width:` scalar values long (section 3.8
of the Language Reference), so that numbers written as text line up on the right, as `pad("7",
width: 3, fill: "0")` gives `007`. A text already that long or longer is returned as it is,
never cut. `fill:` is one character; a longer or empty fill is an error (Z0401), as is a
negative width. `NA` gives `NA`. Given a column, it applies row by row. Like R's `formatC(s,
width)` and Stata's right-justified string formats.

## Arguments
- s: The text, or a column of texts.
- width: The length to pad to, in characters.
- fill: The character put on the left; a space unless given.

## Value
A Str of at least `width:` characters; a column of Strs when `s` is a column.

## See also
trim, slice, format, len

## Examples
> pad("abc", width: 6)
   abc
> pad("7", width: 3, fill: "0")
007
> pad("abcdef", width: 3)
abcdef
