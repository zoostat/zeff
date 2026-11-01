# format

## Summary
Number or date to text

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

## Details
Exact: the decimals are those of the correctly rounded decimal expansion of the value, so every
target writes the same text; the calendar names are English, whatever the machine's locale.

## See also
to, round, date, print

## Examples
> format(1234567.891, digits: 2)
1234567.89
> format(1234567.891, digits: 2, big_mark: ",")
1,234,567.89
> format(42, digits: 1)
42.0
> format(2.675, digits: 2)
2.67
> format(1 / 3)
0.333333
> format(date("2026-10-16"), pattern: "%d %B %Y")
16 October 2026
> format(date("2026-10-16"), pattern: "%a %b %y")
Fri Oct 26
> format(datetime("2026-10-16T09:05"), pattern: "%Y-%m-%d %H:%M")
2026-10-16 09:05
> let sales = frame(sold = [date("2026-01-05"), date("2024-02-29")], amount = [1250.5, 980])
> sales.add(month = format(sold, pattern: "%b %Y"), text = format(amount, digits: 2, big_mark: ","))
frame · 2 rows × 4 variables

      sold         amount   month      text
  1   2026-01-05   1250.5   Jan 2026   1,250.50
  2   2024-02-29      980   Feb 2024   980.00
