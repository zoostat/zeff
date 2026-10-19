# min

## Summary
Smallest non-missing value

## Description
The min of the non-missing values of a variable, with the variable's type: an Int for an Int
variable, a Float otherwise. Missing values are skipped with a warning giving their count; nothing
left gives `NA`; an infinite value is a value. With `by:` the result is a table with one row per
group; with `weights:` the rows with a zero weight are left out, as section 4.1 of the Language
Reference says, and the weights change nothing else. The min of two or more values row by row is
the `core` function of the same name.

## Arguments
- data: The frame.
- var: The variable, which must be Bool, Int or Float.

## Value
An `Int` for an Int variable, a `Float` otherwise; a `Table` with `by:`.

## Details
Exact.

## References
The weighted formulas: frequency weights replicate rows; analytic and sampling weights give the mean Σwx / Σw and a variance with the weights scaled to sum to n and divisor n − 1, as Stata's `summarize [aweight]` (the owner's decision of 19 October 2026, docs/decisions.md). Hyndman, R. J. and Fan, Y. (1996), Sample quantiles in statistical packages, The American Statistician 50, 361–365.

## See also
summary, median, quantile

## Examples
> let wages = import("data/wages.csv")
note: data/wages.csv read as CSV: 3 rows × 3 variables, types guessed; CSV carries no labels, value labels or categories (line 1)
> wages.min(wage)
19
> wages.min(hours)
35
warning: 1 row with NA in hours was skipped (line 3)
