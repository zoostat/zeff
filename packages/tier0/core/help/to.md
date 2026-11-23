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
returns a column of the type; `Category` needs a column. `to(x, List)` is of another kind: it
takes a column and gives the list of its values, each as a value of the column's type, so that
a column can be handed to a function of lists or walked in a loop; one value out of a column is
`x[i]`, `first(x)` or `last(x)`, and needs no list. `to(t, Frame)` gives the frame behind a
printed table that has one: the table of `duplicates`, as its key variables and `copies`. R converts with `as.integer`,
`as.numeric`, `as.character`, `as.logical` and `as.Date`, which also read `"2.5"` as 2, `"+5"`
and `"0x10"`; Zeff refuses those.

## Arguments
- x: The value, or a column.
- type: The type to convert to: `Int`, `Float`, `Str`, `Bool`, `Date`, `DateTime` or `Category`; a
  date or time type may carry a format, `Date("%d/%m/%Y")`, written as `strftime` writes it.
  `List`, for a column, gives the list of its values, and `Frame`, for the table of
  `duplicates`, its frame.

## Value
A value of the type named, or a column of it when `x` is a column. With `List`, a list with one
item for each row of the column, a missing value as `NA` and a category as its label.

## Missing values
`NA` converts to `NA` of the new type. A value that cannot convert becomes `NA`, and warning W0203 gives the count and up to three of the values; a number with a fraction given to `Int` becomes `NA` with warning W0202. No row is dropped.

## Details
Exact: every conversion follows the rules written above, with no rounding and no guess.

## A common mistake
Converting a number written with a thousands separator. `to` reads the text of a number as a script writes one, so the comma stops it; the value becomes `NA`, and the warning shows it:

> to("1,000", Float)
NA
warning[W0203]: "1,000" could not be converted to Float and became NA (line 11)
  = help: the page of `to` lists what converts; `is_missing` finds the NA afterwards

Take the separators out first: `to(replace("1,000", ",", ""), Float)` is `1000`.

## Coming from R or Stata
| to(x, Int) | as.integer(x) | real(x), or destring x, replace |
| to(x, Float) | as.numeric(x) | real(x), or destring x, replace |
| to(x, Str) | as.character(x) | string(x), or tostring x, replace |
| to(x, Date) | as.Date(x) | date(x, "YMD") |
| to(x, Category) | factor(x) | encode x, generate(x2) |

R's `as.integer("2.5")` is 2: the fraction is cut off without a word; here it is `NA` with a
warning, and `round`, `floor` or `truncate` says which whole number is meant. For text that is not
a number R warns without saying which values or how many, and Stata's `real` returns missing
without a word; here the warning counts them and shows up to three.

## Related tasks
- To test a value's type before converting, use `is`.
- To choose the whole number of a fraction, use `round`.
- To make a category with its levels in a chosen order, use `category`.

## See also
is, type_of, is_missing, coalesce

## Examples
// Text to a whole number.
> to("42", Int)
42

// A fraction is not cut off: the result is missing, and the warning says how to choose.
> to(2.5, Int)
NA
warning[W0202]: 2.5 has a fraction, so to(Int) gave NA (line 2)
  = help: `round(x)`, `floor(x)` or `truncate(x)` chooses the whole number first

// A Bool to a number.
> to(true, Int)
1

// A number to text, the shortest that reads back the same.
> to(0.1, Str)
0.1

// Text to a date, written the ISO way.
> to("2026-10-16", Date)
2026-10-16

// A date written another way, with its format.
> to("16/10/2026", Date("%d/%m/%Y"))
2026-10-16

// A day that does not exist converts to nothing.
> to("2026-02-30", Date)
NA
warning[W0203]: "2026-02-30" could not be converted to Date and became NA (line 7)
  = help: the page of `to` lists what converts; `is_missing` finds the NA afterwards

// Row by row on the shipped wages, in the method form: wages as text and experience as Floats; the missing wage stays missing.
> let w = dataset(wages).where(worker == 19).select([worker, year, wage, hours, exper])
> w.add(wage_text = wage.to(Str), years = exper.to(Float))
frame · 5 rows × 7 variables · panel worker × year

      worker   year    wage   hours   exper   wage_text   years
  1       19   2012   12.64    29.2       0   12.64           0
  2       19   2013   25.18    31.4       0   25.18           0
  3       19   2014   35.49    33.5       1   35.49           1
  4       19   2015   26.07    58.2       2   26.07           2
  5       19   2016      NA      30       3   NA              3

// The values of a column as a list, the missing one among them.
> to(w["wage"], List)
12.64
25.18
35.49
26.07
NA
