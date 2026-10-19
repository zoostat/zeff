# pdf

## Summary
Density, or probability for discrete distributions

## Description
The probability density of a continuous distribution at `x`, or the probability of the value `x`
for a discrete one (binomial, Poisson, negative binomial). Given a column, `pdf` applies row by row
and returns a column. `NA` gives `NA`. Like R's `d` functions (`dnorm`, `dexp`, …) and Stata's
`normalden()` and its kin, with the distribution as the receiver rather than a function per family.

## Arguments
- distribution: The distribution, a `Distribution` value such as `dist.normal()`.
- x: The point, an Int or a Float, or a column of either.

## Value
A Float; a column of Floats when `x` is a column.

## Details
Each family's density is written out in closed form on the kernel's special functions and the
engine's pure-Rust elementary functions, so the result is the same on every target; the help page of
each family gives its formula. Outside a family's support the density is 0.

## See also
cdf, quantile, draw

## References
- The help page of each family (`zeff help dist.normal`, …) for its formula and reference.

## Examples
> dist.normal().pdf(0)
0.398942
> dist.exponential(rate: 2).pdf(1)
0.270671
> let wages = import("data/wages.csv")
note: data/wages.csv read as CSV: 3 rows × 3 variables, types guessed; CSV carries no labels, value labels or categories (line 3)
> dist.normal().pdf(wages["wage"] / 25)
Column<Float> · 3 values
0.257435
0.298872
0.183106
