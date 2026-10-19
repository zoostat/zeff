# cdf

## Summary
P(X ≤ x); upper: true gives P(X > x) accurately in the tail

## Description
The cumulative probability P(X ≤ x) of a distribution at `x`, or with `upper: true` the upper tail
P(X > x), computed on its own side so that a tail probability of 1e-300 keeps its digits (Engine
and Project Specification, section 13: a tail is never one minus the other). Given a column, `cdf`
applies row by row and returns a column. `NA` gives `NA`. Like R's `p` functions with `lower.tail`
and Stata's `normal()`, `ttail()` and their kin.

## Arguments
- distribution: The distribution, a `Distribution` value such as `dist.t(df: 24)`.
- x: The point, an Int or a Float, or a column of either.
- upper: `true` for P(X > x), the upper tail; `false` (the default) for P(X ≤ x).

## Value
A Float between 0 and 1; a column of Floats when `x` is a column.

## Details
For a discrete distribution P(X ≤ x) sums the probabilities up to the largest whole number not
above `x`, and `upper: true` is the complementary sum from the next whole number. A p-value is a
tail: `dist.normal().cdf(2.5, upper: true)` is the one-sided p-value of z = 2.5, and twice it the
two-sided one.

## See also
pdf, quantile, draw

## References
- The help page of each family (`zeff help dist.normal`, …) for its formula and reference.

## Examples
> dist.normal().cdf(1.96)
0.975002
> dist.normal().cdf(1.96, upper: true)
0.0249979
> dist.normal(mean: 100, sd: 15).cdf(130)
0.97725
> let wages = import("data/wages.csv")
note: data/wages.csv read as CSV: 3 rows × 3 variables, types guessed; CSV carries no labels, value labels or categories (line 4)
> dist.normal().cdf(wages["hours"] / 40 - 1)
Column<Float> · 3 values
0.5
0.450262
NA
