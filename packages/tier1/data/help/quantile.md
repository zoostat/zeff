# quantile

## Summary
Sample quantiles

## In brief
- `d.quantile(wage, p: 0.9)`: the value below which nine tenths of the wages lie.
- `d.quantile(wage, p: [0.25, 0.5, 0.75])`: several at once, as a list.
- `d.quantile(wage, p: 0.9, by: year)`: a table with the quantile of each group.
- `d.quantile(wage, p: 0.5, type: 1)`: another of the nine definitions; 7, R's default, unless given.
- `d.aggregate(p90 = quantile(wage, p: 0.9), by: year)`: inside a definition.

## Description
The quantile of `var` at probability `p:`, or one per probability for a list, under one of the nine
definitions of Hyndman and Fan numbered as R's `quantile(type =)`; type 7, R's default, interpolates
between order statistics. Missing values are skipped with a warning giving their count. With `by:`
the result is a table with one row per group and one column per probability; with `weights:` the
quantile is the smallest value whose cumulative weight reaches `p` of the total, whatever the type.

## Arguments
- data: The frame.
- var: The variable, which must be Bool, Int or Float.
- p: A probability between 0 and 1, or a list of them: `p: 0.9`, `p: [0.25, 0.5, 0.75]`.
- type: The definition, 1 to 9; 7 by default.

## Value
A `Float`, or a `List<Float>` for a list of probabilities; a `Table` with `by:`.

## Missing values
Missing values are skipped, and a warning counts the rows skipped. A variable with no value left gives `NA`, with a warning.

## Details
R's own arithmetic for each type, including its guard against rounding in the index, so the values
agree with R to the last bit; compared with R on every type at nine probabilities in the test suite.

## A common mistake
Writing the probability as a percentage. `p:` is a share between 0 and 1:

> dataset(wages).quantile(wage, p: 90)
error[Z0401]: `p:` of `quantile` must lie between 0 and 1, but this is 90
  = help: a quantile's probability is a share of the values: `p: 0.9` is the ninetieth percentile

The ninetieth percentile is `dataset(wages).quantile(wage, p: 0.9)`.

## Coming from R or Stata
| d.quantile(wage, p: 0.9) | quantile(d$wage, 0.9, na.rm = TRUE) | centile wage, centile(90) |
| d.quantile(wage, p: [0.25, 0.75]) | quantile(d$wage, c(0.25, 0.75), na.rm = TRUE) | centile wage, centile(25 75) |
| d.quantile(wage, p: 0.5, type: 6) | quantile(d$wage, 0.5, type = 6, na.rm = TRUE) | centile wage |

The nine definitions are numbered as R numbers them, and type 7 is the default of both. Stata
writes the probability as a percentage, and its `centile` uses the definition numbered 6 here,
so its figures differ a little from R's and from these unless `type: 6` is given.

## Related tasks
- For the middle value alone, use `median`.
- For the distance between the quartiles, use `iqr`.
- To cut a variable into groups at its quantiles, use `bin`.
- For the quantile of a distribution and not of data, as in `dist.normal().quantile(0.9)`, use the distribution's own `quantile`.

## References
The weighted formulas: frequency weights replicate rows; analytic and sampling weights give the mean Σwx / Σw and a variance with the weights scaled to sum to n and divisor n − 1, as Stata's `summarize [aweight]` (the owner's decision of 19 October 2026, docs/decisions.md). Hyndman, R. J. and Fan, Y. (1996), Sample quantiles in statistical packages, The American Statistician 50, 361–365.

## See also
median, summary, min, max

## Examples
// The ninth decile of the wages. The warning counts the rows with no wage.
> let wages = dataset(wages)
> wages.quantile(wage, p: 0.9)
45.39
warning[W0201]: 211 rows with NA in wage were skipped (line 2)

// The quartiles, as a list.
> wages.quantile(wage, p: [0.25, 0.5, 0.75])
17.56
24.35
33.73
warning[W0201]: 211 rows with NA in wage were skipped (line 3)

// One row for each group with `by:`, one column for each probability.
> wages.quantile(wage, p: [0.1, 0.9], by: female)
quantiles · wage by female

  female     p0.1     p0.9
  male     14.094   48.116
  female   12.290   41.935
warning[W0201]: 211 rows with NA in wage were skipped (line 4)

// Another definition: type 1 gives a value that is in the data.
> wages.quantile(wage, p: 0.9, type: 1)
45.39
warning[W0201]: 211 rows with NA in wage were skipped (line 5)

// Inside `aggregate`: the gap between the ninth and the first decile of each year.
> wages.aggregate(p90 = quantile(wage, p: 0.9), p10 = quantile(wage, p: 0.1), gap = p90 - p10, by: year).head(3)
frame · 3 rows × 4 variables

      year      p90      p10     gap
  1   2010    36.68    10.21   26.47
  2   2011   39.634   11.194   28.44
  3   2012    39.61    12.06   27.55
warning[W0201]: 211 rows with NA in wage were skipped (line 6)
note: the panel declaration `worker × year` dropped: `aggregate` summarised the rows (line 6)
