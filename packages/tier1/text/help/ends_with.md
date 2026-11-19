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

## Missing values
`NA` in the text or in the suffix gives `NA`. In a column each row is taken by itself, and no row is dropped.

## Coming from R or Stata
| ends_with(s, ".csv") | endsWith(s, ".csv") | substr(s, -4, 4) == ".csv" |

R's `endsWith` is the same. Stata has no function for it and compares the last characters, which
must be counted.

## Related tasks
- For the start of a text, use `starts_with`.
- For a text anywhere inside, use `contains`.

## See also
starts_with, slice, lower

## Examples
// The suffix is there.
> ends_with("report.csv", ".csv")
true

// Capitals matter; put the text through `lower` first when they should not.
> ends_with("report.CSV", ".csv")
false

// Row by row.
> let people = frame(name = ["  Ann Lee", "bob STONE ", NA], code = ["A-12", "b-7", "C-103"])
> people.add(three = ends_with(code, "3"))
frame · 3 rows × 3 variables

      name         code    three
  1     Ann Lee    A-12    false
  2   bob STONE    b-7     false
  3   NA           C-103   true
