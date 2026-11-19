# format

## Summary
Number or date to text

## In brief
- `format(x, digits: 2)`: a number with two decimal places, as text.
- `format(x, digits: 0, big_mark: ",")`: with a separator between every three digits.
- `format(d, pattern: "%d %B %Y")`: a date written by a pattern, `16 October 2026`.

## Description
Writes `x` as text. A number without options prints as section 3.8 of the Language Reference
prints it: an Int in full, a Float to six significant digits with trailing zeros dropped; with
`digits:` it is written to that many decimal places, a value exactly halfway between two
candidates going to the even one as `round` rounds (the tie is decided on the number as the
machine holds it, so `2.675` to two places is `2.67`), trailing zeros kept and the plain form
used whatever the size; `big_mark:` puts the separator given between every three digits of the
whole part, `1,234,567.5`. A date or a date and time without a pattern prints in ISO 8601; with
`pattern:` it prints as `strftime` writes it: `%Y` the four-digit year, `%y` its last two digits,
`%m` the month as two digits, `%b` and `%B` its English name as three letters or in full, `%d`
the day as two digits, `%a` and `%A` the weekday as three letters or in full, `%j` the day of the
year as three digits, `%H`, `%M` and `%S` the time as two digits each, `%%` a percent sign, and
anything else as it stands. `NA` gives `NA`. Given a column, `format` applies row by row and
returns a column of text. `digits:` and `big_mark:` do not apply to a date, nor `pattern:` to a
number (Z0401); `pattern:` beside either is refused (Z0410). Like R's `formatC(x, format = "f",
digits)`, `format(x, big.mark)` and `format(d, pattern)`, and Stata's `%9.2fc` formats, written
by name rather than by a format code.

## Arguments
- x: An Int, a Float, a Date or a DateTime, or a column of one of them.
- digits: Decimal places for a number, from 0 to 20.
- big_mark: The text put between every three digits of a number's whole part, `","` or `" "`.
- pattern: The pattern a date or a time is written by.

## Value
A Str; a column of Strs when `x` is a column.

## Missing values
`NA` gives `NA`. In a column each row is taken by itself: a row with `NA` stays `NA`, no row is dropped and nothing is printed for it.

## Details
Exact: the decimals are those of the correctly rounded decimal expansion of the value, so every
target writes the same text; the calendar names are English, whatever the machine's locale.

## A common mistake
Giving a number a pattern, as `sprintf` takes one in R. `pattern:` is for dates and times; a number is shaped by `digits:` and `big_mark:`:

> format(3.14159, pattern: "%.2f")
error[Z0401]: `pattern:` of `format` does not apply to a Float
  = help: `pattern:` prints a date or a time, as `pattern: "%d/%m/%Y"`; a number is shaped by `digits:` and `big_mark:`

Say the decimals by name: `format(3.14159, digits: 2)` is `3.14`.

## Coming from R or Stata
| format(x, digits: 2) | formatC(x, format = "f", digits = 2) | string(x, "%9.2f") |
| format(x, digits: 0, big_mark: ",") | format(x, big.mark = ",") | string(x, "%12.0fc") |
| format(d, pattern: "%d %B %Y") | format(d, "%d %B %Y") | string(d, "%td") |

In R's own `format`, `digits` counts significant digits, not decimal places, and `nsmall` is the
least number of decimals; here `digits:` is the number of decimal places, always. R writes the
names of months in the computer's language; here they are English everywhere. Stata writes a
date by a `%td` format with letters of its own.

## Related tasks
- To round a number and keep it a number, use `round`.
- To convert without shaping, use `to`.
- To print values on one line, use `print`.

## See also
to, round, date, print

## Examples
// Two decimal places.
> format(1234567.891, digits: 2)
1234567.89

// A separator between every three digits.
> format(1234567.891, digits: 2, big_mark: ",")
1,234,567.89

// Trailing zeros are kept.
> format(42, digits: 1)
42.0

// A date by a pattern.
> format(date("2026-10-16"), pattern: "%d %B %Y")
16 October 2026

// A date and time.
> format(datetime("2026-10-16T09:05"), pattern: "%Y-%m-%d %H:%M")
2026-10-16 09:05

// Row by row on the shipped firms: the month each was founded.
> dataset(firms).head(3).add(since = format(founded, pattern: "%B %Y"))
frame · 3 rows × 5 variables

      firm   industry      size   founded      since
  1      1   Agriculture      3   1990-11-06   November 1990
  2      2   Agriculture      5   1957-07-02   July 1957
  3      3   Agriculture      3   1998-11-07   November 1998

// Row by row on the shipped wages: each wage to one decimal; the missing one stays missing.
> let w = dataset(wages).where(worker == 19).select([worker, year, wage, hours, exper])
> w.add(shown = format(wage, digits: 1))
frame · 5 rows × 6 variables · panel worker × year

      worker   year    wage   hours   exper   shown
  1       19   2012   12.64    29.2       0   12.6
  2       19   2013   25.18    31.4       0   25.2
  3       19   2014   35.49    33.5       1   35.5
  4       19   2015   26.07    58.2       2   26.1
  5       19   2016      NA      30       3   NA
