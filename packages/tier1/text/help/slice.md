# slice

## Summary
Part of a string; positions start at 1 and count Unicode scalar values

## In brief
- `slice(s, start: 1, len: 3)`: the first three characters.
- `slice(s, start: 8, len: 5)`: five characters from the eighth; positions count from 1, in
  characters, not bytes.

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

## Missing values
`NA` in any of the three gives `NA`. In a column each row is taken by itself, and no row is dropped.

## A common mistake
Counting positions from 0, as Python and C do. The first character is at position 1:

> slice("abc", start: 0, len: 2)
error[Z0401]: `start:` of `slice` counts positions from 1, but this is 0
  = help: the first character is at position 1, as in R's `substr` and Stata's `substr`

Count from 1: `slice("abc", start: 1, len: 2)` is `ab`.

## Coming from R or Stata
| slice(s, start: 8, len: 5) | substr(s, 8, 12) | usubstr(s, 8, 5) |

R's `substr` takes the position of the last character, not a length. Stata's `substr` counts
bytes, so an accented letter counts twice and may be cut in half; `usubstr` counts characters,
as `slice` does.

## Related tasks
- For a part found by a pattern, use `extract`.
- For the length of a text, use `len`.

## See also
len, pad, starts_with, ends_with

## Examples
// Five characters from the eighth.
> slice("Hello, World", start: 8, len: 5)
World

// Positions count characters: the fourth of four.
> slice("café", start: 4, len: 1)
é

// Past the end, what is there.
> slice("abc", start: 2, len: 10)
bc

// Row by row: the letter each code begins with.
> let people = frame(name = ["  Ann Lee", "bob STONE ", NA], code = ["A-12", "b-7", "C-103"])
> people.add(letter = slice(code, start: 1, len: 1))
frame · 3 rows × 3 variables

      name         code    letter
  1     Ann Lee    A-12    A
  2   bob STONE    b-7     b
  3   NA           C-103   C
