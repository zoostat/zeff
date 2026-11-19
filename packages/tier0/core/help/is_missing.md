# is_missing

## Summary
True where x is NA

## Description
Whether a value is missing: `true` for `NA` and `false` for anything else, including `Inf`, zero and
the empty text. It is the test to write where `x == NA` would not do: a comparison with `NA` is
always `NA` (section 4.1 of the Language Reference), so it never selects anything. Given a column,
`is_missing` applies row by row and returns a column of Bools with no `NA` in it, which is what
`where` needs to keep or drop the rows with gaps. R has `is.na`; Stata has `missing`.

## Arguments
- x: A value of any computable type (Bool, Int, Float or Str), or a column of one.

## Value
A Bool; a column of Bools when `x` is a column.

## Missing values
It is the test for them: `true` for `NA` and `false` for everything else, so the result has no `NA` in it.

## Details
Exact.

## A common mistake
Comparing with `NA` to find the missing values. A comparison with a value that is not known is not known either, so it selects nothing, and the checker says so:

> dataset(wages).where(wage == NA)
error[Z0226]: `== NA` is never true: a comparison with `NA` gives `NA`, not true or false
  = help: write `is_missing(wage)`; `NA` stands for a value that is not known, so nothing is known to equal it

Ask the question the function answers: `dataset(wages).where(is_missing(wage))` keeps the 211 rows with no wage.

## Coming from R or Stata
| is_missing(x) | is.na(x) | missing(x) |

Stata's `missing` is also true of the empty text; here `""` is a text of no characters and not a
missing value. In Stata a missing number counts as larger than every number, so `x > 100` is
true of it; here and in R the comparison is `NA`.

## Related tasks
- To put a value in place of the missing ones, use `coalesce`.
- To count them, use `n_missing`.
- To drop the rows that have them, use `drop_missing`.

## References
Language Reference, section 4.1 (the rules of `NA`).

## See also
coalesce, is_finite

## Examples
// A missing value.
> is_missing(NA)
true

// Zero is a value.
> is_missing(0)
false

// So is the empty text.
> is_missing("")
false

// Row by row on the shipped wages: the year with no wage.
> let w = dataset(wages).where(worker == 19).select([worker, year, wage, hours, exper])
> w.add(no_wage = is_missing(wage))
frame · 5 rows × 6 variables · panel worker × year

      worker   year    wage   hours   exper   no_wage
  1       19   2012   12.64    29.2       0   false
  2       19   2013   25.18    31.4       0   false
  3       19   2014   35.49    33.5       1   false
  4       19   2015   26.07    58.2       2   false
  5       19   2016      NA      30       3   true
