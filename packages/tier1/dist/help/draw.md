# draw

## Summary
n random draws; the seed is recorded

## Description
A column of `n` random draws from the distribution. Without `seed:` the draws come from the run's
stream: the seed set by `seed(n)` at the top of the script, or one chosen for the run at the first
draw and printed as a note so that `seed(n)` repeats the run exactly; each statement draws from its
own part of the stream, so two identical statements give different draws, and inserting a statement
changes the draws after it, as in R and Stata. With `seed:` the draws are a stream of their own,
the same wherever the call is written. Like R's `r` functions (`rnorm(n)`, …) and Stata's
`rnormal()` and its kin.

## Arguments
- distribution: The distribution, a `Distribution` value such as `dist.normal()`.
- n: The number of draws, an Int of 0 or more.
- seed: An Int seed for these draws alone; when omitted, the run's seed is used.

## Value
A `Column<Float>`; a `Column<Int>` for a discrete distribution (binomial, Poisson, negative
binomial). `draw(1)` is a column of one value.

## Details
Every draw is the inverse CDF of a uniform number with 53 random bits from the Philox stream
(Engine and Project Specification, section 8.3, point 3), so the draws are a pure function of the
seed, the statement's position and the draw's number, identical on every machine and in the
browser, and never depend on threads or on what was drawn before. The uniform is shifted by half a
grid step so that a draw never falls on the quantile's endpoints.

## See also
pdf, cdf, quantile, seed

## References
- Salmon, J. K., Moraes, M. A., Dror, R. O. and Shaw, D. E. (2011). Parallel random numbers: as
  easy as 1, 2, 3. *Proceedings of SC '11*, article 16.

## Examples
> dist.normal().draw(3, seed: 1)
Column<Float> · 3 values
1.22791
0.216564
0.445438
> dist.exponential(rate: 0.1).draw(4, seed: 3)
Column<Float> · 4 values
16.9639
10.6726
3.12281
2.54098
