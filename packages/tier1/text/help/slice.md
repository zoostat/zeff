# slice

## Summary
Part of a string; positions start at 1 and count Unicode scalar values

## Description
The `len:` scalar values of the text from position `start:`, counted from 1 (section 3.8 of the
Language Reference), so `slice("Hello, World", start: 8, len: 5)` is `World`. A slice that runs
past the end gives what is there, and one that starts past the end gives the empty text; so does
`len: 0`. A start below 1 or a negative length is an error (Z0401). `NA` in any argument gives
`NA`. Given a column, it applies row by row. Like R's `substr(s, start, start + len - 1)` and
Stata's `substr(s, start, len)`.

## Arguments
- s: The text, or a column of texts.
- start: The position of the first character taken, from 1.
- len: How many characters to take.

## Value
A Str; a column of Strs when `s` is a column.

## See also
len, pad, starts_with, ends_with

## Examples
> slice("Hello, World", start: 8, len: 5)
World
> slice("café", start: 4, len: 1)
é
> slice("abc", start: 2, len: 10)
bc
> len(slice("abc", start: 5, len: 2))
0
