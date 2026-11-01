# to

## Summary
Convert a value or a column to another type

## Description
Converts `x` to the type named, by the table of section 3.7 of the Language Reference: numbers to
text as the shortest text that reads back to the same number; text to numbers when it is a whole
decimal or a Float literal, spaces around it ignored (`"42"`, `"-7"`, `"4.2e1"`; never `"1,000"`,
`"1_000"` or `"0x10"`); text to Bool from `true` and `false` in any case; Int to Bool from 0 and 1;
Bool to Int or Float as 0 and 1; a Float to Int only when it is whole; text to Category with the
distinct values in code-point order as levels; a category to text as its labels and to Int as its
level positions, counted from 1; text to Date or DateTime from ISO 8601 (`"2026-10-16"`,
`"2026-10-16 09:30"`) or from the format written with the type, `Date("%d/%m/%Y")`, an impossible
date such as `"2026-02-30"` converting to nothing; a Date to DateTime as midnight and a DateTime
to Date by dropping the time; a date or time to text in ISO 8601. A value that cannot convert
becomes `NA` with a warning giving the count and up to three failing values (W0203); a number
with a fraction given to Int becomes `NA` with W0202; date-and-time text that carries `Z` or an
offset is converted to UTC and counted (W0702). Given a column, `to` converts row by row and
returns a column of the type; `Category` needs a column. R converts with `as.integer`,
`as.numeric`, `as.character`, `as.logical` and `as.Date`, which also read `"2.5"` as 2, `"+5"`
and `"0x10"`; Zeff refuses those.

## Arguments
- x: The value, or a column.
- type: The type to convert to: `Int`, `Float`, `Str`, `Bool`, `Date`, `DateTime` or `Category`; a
  date or time type may carry a format, `Date("%d/%m/%Y")`, written as `strftime` writes it.

## Value
A value of the type named, or a column of it when `x` is a column.

## Details
Exact: every conversion follows the rules written above, with no rounding and no guess.

## See also
is, type_of, is_missing, coalesce

## Examples
> to("42", Int)
42
> to(2.5, Int)
NA
warning: 2.5 has a fraction, so to(Int) gave NA (line 2)
  = help: `round(x)`, `floor(x)` or `truncate(x)` chooses the whole number first
> to("1,000", Float)
NA
warning: "1,000" could not be converted to Float and became NA (line 3)
  = help: the page of `to` lists what converts; `is_missing` finds the NA afterwards
> to(true, Int)
1
> to(0.1, Str)
0.1
> to("2026-10-16", Date)
2026-10-16
> to("16/10/2026", Date("%d/%m/%Y"))
2026-10-16
> to("2026-02-30", Date)
NA
warning: "2026-02-30" could not be converted to Date and became NA (line 8)
  = help: the page of `to` lists what converts; `is_missing` finds the NA afterwards
> to(to("2026-10-16", Date), DateTime)
2026-10-16T00:00:00
> let wages = import("data/wages.csv")
note: data/wages.csv read as CSV: 3 rows × 3 variables, types guessed; CSV carries no labels, value labels or categories (line 10)
> to(wages["hours"], Float)
Column<Float> · 3 values
40
35
NA
> wages["id"].to(Str)
Column<Str> · 3 values
1
2
3
