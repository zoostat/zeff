# n_chars

## Summary
The number of characters in a text, value by value

## Description
How long a text is: the number of its Unicode scalar values (code points), the unit in which
every text function counts (section 3.8 of the Language Reference). `n_chars("café")` is 4,
though the accented letter takes two bytes, and an accent written as a combining mark is a
code point of its own, so a letter typed that way counts as two. `NA` gives `NA`, and the
empty text gives 0.

Given a column, it applies row by row and gives the length of each value: this is what
`n_chars` is for. `len` counts the same way for one text, but `len` of a column is the number
of its rows, so inside `add` it puts that one number on every row; `add` says so in a note
when that happens with a text variable.

## Arguments
- s: The text, or a column of texts.

## Value
An Int; a column of Ints when `s` is a column.

## Missing values
`NA` gives `NA`. In a column each row is taken by itself: a row with `NA` stays `NA`, no row is dropped and nothing is printed for it. The empty text is not missing: its length is 0.

## Coming from R or Stata
| n_chars(s) | nchar(s, type = "chars") | ustrlen(s) |
| n_chars(d["s"]) | nchar(d$s) | generate n = ustrlen(s) |

R's `nchar` counts characters unless asked for bytes, and gives 2 for a missing value that is
not text; here a missing value has no length. Stata's `strlen` counts bytes, so an accented
letter counts twice there; `ustrlen` counts as `n_chars` does.

## Related tasks
- For the number of rows of a frame, use `n_rows`; for the items of a list, `len`.
- To cut a text to a number of characters, use `slice`; to bring it to a width, `pad`.
- To keep the rows whose text is longer than some length, use `where`.

## See also
len, slice, pad, trim, upper

## Examples
// One text: four characters, although the accented letter takes two bytes.
> n_chars("café")
4

// The empty text has length 0.
> n_chars("")
0

// A missing text has no length.
> n_chars(NA)
NA

// A combining accent is a character of its own: the same word typed that way is longer.
> n_chars("cafe\u{301}")
5

// Row by row: the length of each name, the missing one staying missing.
> let people = frame(name = ["  Ann Lee", "bob STONE ", NA], code = ["A-12", "b-7", "C-103"])
> people.add(length = n_chars(name), trimmed = n_chars(trim(name)))
frame · 3 rows × 4 variables

      name         code    length   trimmed
  1     Ann Lee    A-12         9         7
  2   bob STONE    b-7         10         9
  3   NA           C-103       NA        NA

// `len` of a variable is the number of its rows, the same on every row, and a note says so.
> people.add(rows = len(name))
frame · 3 rows × 3 variables

      name         code    rows
  1     Ann Lee    A-12       3
  2   bob STONE    b-7        3
  3   NA           C-103      3
note: `len(name)` is the number of rows of `name`, the same on every row; `n_chars(name)` gives the length of each value (line 7)

// Rows chosen by the length of a text.
> people.where(n_chars(code) > 3)
frame · 2 rows × 2 variables

      name        code
  1     Ann Lee   A-12
  2   NA          C-103

// The longest label among the industries of the shipped firms.
> dataset(firms).add(letters = n_chars(industry)).max(letters)
21
