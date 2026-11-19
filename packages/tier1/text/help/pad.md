# pad

## Summary
Pad text to a width

## In brief
- `pad(s, width: 6)`: spaces on the left until the text is six characters long.
- `pad(s, width: 3, fill: "0")`: zeros in the place of the spaces, `007`.

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

## Missing values
`NA` gives `NA`. In a column each row is taken by itself: a row with `NA` stays `NA`, no row is dropped and nothing is printed for it.

## A common mistake
Giving a fill of more than one character. The width is counted in characters, and a longer fill could not land on it:

> pad("7", width: 3, fill: "00")
error[Z0401]: `fill:` of `pad` is one character, but this is 2 characters
  = help: `fill: "0"` pads a number with zeros, `fill: "."` leads the eye across a table

Give one character: `pad("7", width: 3, fill: "0")` is `007`.

## Coming from R or Stata
| pad(s, width: 6) | formatC(s, width = 6) | no single function |
| pad("7", width: 3, fill: "0") | formatC(7, width = 3, flag = "0") | string(7, "%03.0f") |

R's and Stata's forms with zeros work on numbers; `pad` works on text, so a number is written
as text first, with `to` or `format`.

## Related tasks
- To write a number with a fixed number of decimals, use `format`.
- To remove white space at the ends, use `trim`.

## See also
trim, slice, format, len

## Examples
// Spaces on the left.
> pad("abc", width: 6)
   abc

// Zeros.
> pad("7", width: 3, fill: "0")
007

// A text already longer is not cut.
> pad("abcdef", width: 3)
abcdef

// Row by row: the number of each code, to four digits.
> let people = frame(name = ["  Ann Lee", "bob STONE ", NA], code = ["A-12", "b-7", "C-103"])
> people.add(number = pad(extract(code, "[0-9]+"), width: 4, fill: "0"))
frame · 3 rows × 3 variables

      name         code    number
  1     Ann Lee    A-12    0012
  2   bob STONE    b-7     0007
  3   NA           C-103   0103
