# draw

## Summary
n random draws; the seed is recorded

## In brief
- `d.draw(n)`: `n` random numbers from the distribution, from the run's seed.
- `d.draw(n, seed: 7)`: from a seed of their own, the same numbers wherever the call stands.

## Description
A column of `n` random draws from the distribution. Without `seed:` the draws come from the run's
stream: the seed set by `seed(n)` at the top of the script, or one chosen for the run at the first
draw and printed as a note so that `seed(n)` repeats the run exactly; each statement draws from its
own part of the stream, so two identical statements give different draws, and inserting a statement
that draws changes the draws after it, as in R and Stata, while one that draws nothing changes none.
With `seed:` the draws are a stream of their own, the same wherever the call is written.

## Arguments
- distribution: The distribution, a `Distribution` value such as `dist.normal()`.
- n: The number of draws, an Int of 0 or more.
- seed: An Int seed for these draws alone; when omitted, the run's seed is used.

## Value
A `Column<Float>`; a `Column<Int>` for a discrete distribution (binomial, Poisson, negative
binomial). `draw(1)` is a column of one value.

## Missing values
A draw is never `NA`. `n` and `seed:` are whole numbers and may not be `NA`.

## Details
Every draw is the inverse CDF of a uniform number with 53 random bits from the Philox stream
(Engine and Project Specification, section 8.3, point 3), so the draws are a pure function of the
seed, the statement's ordinal among those that draw and the draw's number, identical on every
machine and in the browser, and never depend on threads. The uniform is shifted by half a
grid step so that a draw never falls on the quantile's endpoints.

## A common mistake
Leaving out how many. A draw is a column of `n` numbers:

> dist.normal().draw()
error[Z0406]: `draw` needs `n`
  = help: usage: `distribution.draw(n, seed: Int)`

Say how many: `dist.normal().draw(100)`.

## Coming from R or Stata
| dist.normal().draw(n) | rnorm(n) | generate x = rnormal() |
| dist.normal().draw(n, seed: 7) | set.seed(7), then rnorm(n) | set seed 7, then generate x = rnormal() |

R's `set.seed` and Stata's `set seed` set the stream for everything after them; here `seed(n)`
at the top of a script does, and `seed:` on a call gives that call a stream of its own. The
numbers are not those R or Stata give for the same seed: each program has its own generator.
Stata's functions give one number for each observation of the dataset in memory; `draw` gives
a column of the length asked for.

## Related tasks
- For the density, or the probability of a count, use `pdf`.
- For a probability below a value, or a tail, use `cdf`.
- For the value at a given probability, use `quantile`.
- To set the seed of the whole run, use `seed`.
- To pick rows of a frame at random, use `sample`.

## References
- Salmon, J. K., Moraes, M. A., Dror, R. O. and Shaw, D. E. (2011). Parallel random numbers: as
  easy as 1, 2, 3. *Proceedings of SC '11*, article 16.

## See also
pdf, cdf, quantile, seed

## Examples
// Three draws from a seed of their own: the same numbers on every machine.
> dist.normal().draw(3, seed: 1)
Column<Float> · 3 values
1.22791
0.216564
0.445438

// From any distribution.
> dist.exponential(rate: 0.1).draw(4, seed: 3)
Column<Float> · 4 values
16.9639
10.6726
3.12281
2.54098

// A count distribution gives whole numbers.
> dist.poisson(lambda: 4).draw(4, seed: 3)
Column<Int> · 4 values
6
5
3
2

// As a variable of a frame: a thousand waiting times, whose mean is near the 10 of the distribution.
> frame(wait = dist.exponential(rate: 0.1).draw(1000, seed: 3)).mean(wait)
10.1922

// Under the run's seed, with no seed of its own.
> seed(2026)
> dist.uniform().draw(2)
Column<Float> · 2 values
0.705195
0.568896
