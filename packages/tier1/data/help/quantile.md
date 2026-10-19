# quantile

## Summary
Sample quantiles

## Description
The quantile of `var` at probability `p:`, or one per probability for a list, under one of the nine
definitions of Hyndman and Fan numbered as R's `quantile(type =)`; type 7, R's default, interpolates
between order statistics. Missing values are skipped with a warning giving their count. With `by:`
the result is a table with one row per group and one column per probability; with `weights:` the
quantile is the smallest value whose cumulative weight reaches `p` of the total, whatever the type.
Stata's `centile` uses another definition (type 6 for its default).

## Arguments
- data: The frame.
- var: The variable, which must be Bool, Int or Float.
- p: A probability between 0 and 1, or a list of them: `p: 0.9`, `p: [0.25, 0.5, 0.75]`.
- type: The definition, 1 to 9; 7 by default.

## Value
A `Float`, or a `List<Float>` for a list of probabilities; a `Table` with `by:`.

## Details
R's own arithmetic for each type, including its guard against rounding in the index, so the values
agree with R to the last bit; compared with R on every type at nine probabilities in the test suite.

## References
The weighted formulas: frequency weights replicate rows; analytic and sampling weights give the mean Σwx / Σw and a variance with the weights scaled to sum to n and divisor n − 1, as Stata's `summarize [aweight]` (the owner's decision of 19 October 2026, docs/decisions.md). Hyndman, R. J. and Fan, Y. (1996), Sample quantiles in statistical packages, The American Statistician 50, 361–365.

## See also
median, summary, min, max

## Examples
> let wages = import("data/wages.csv")
note: data/wages.csv read as CSV: 3 rows × 3 variables, types guessed; CSV carries no labels, value labels or categories (line 1)
> wages.quantile(wage, p: 0.5)
23.4
> wages.quantile(wage, p: [0.25, 0.75])
21.2
27.3
> wages.quantile(wage, p: 0.25, type: 1)
19
