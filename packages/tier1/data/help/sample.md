# sample

## Summary
Random sample of rows; the seed is recorded

## Description
A random sample of the rows, `n:` of them or the share `frac:`, without replacement unless
`with_replacement: true`, the kept rows returned in their original order with every column and
the panel declaration. The draws come from the run's random stream, keyed to the statement, so
a script repeats itself under `seed(n)` at its top; `seed:` given to the call starts a stream of
its own from that seed, the same wherever the call stands. A note gives the count and the seed,
`note: 500 of 18,412 rows sampled with seed 20261015`, so that a run can be repeated. Like
dplyr's `slice_sample` and Stata's `sample`.

## Arguments
- data: The frame.
- n: The number of rows to keep; at most the frame's rows unless `with_replacement: true`.
- frac: The share of rows to keep, between 0 and 1, rounded to a whole number of rows.
- with_replacement: Whether a row may be drawn more than once; false unless given.
- seed: A seed for this call alone; the run's seed unless given.

## Value
A frame of the kept rows, in their original order.

## Details
Stochastic: without replacement the kept rows are the first `n` positions of a partial
Fisher–Yates shuffle over the row positions, each drawn uniformly from the positions not yet
chosen with one uniform number from the Philox stream; with replacement, `n` positions each
drawn uniformly from all rows. The same seed gives the same rows on every target. A sample with
replacement that repeats a row of a panel drops the declaration, with a note. `n:` and `frac:`
together are refused (Z0410).

## References
Knuth, D. E. (1997), The Art of Computer Programming, volume 2, section 3.4.2, algorithm P
(shuffling). Salmon, J. K., Moraes, M. A., Dror, R. O. and Shaw, D. E. (2011), Parallel random
numbers: as easy as 1, 2, 3 (the Philox generator).

## See also
seed, draw, distinct, head

## Examples
> let w = dataset(wages)
> w.sample(n: 5, seed: 20261015).select([worker, year, wage])
frame · 5 rows × 3 variables · panel worker × year

      worker   year    wage
  1       89   2015   24.42
  2     1141   2012    8.57
  3     1575   2017   72.92
  4     2691   2018   40.57
  5     3566   2020   33.95
note: 5 of 18,412 rows sampled with seed 20261015 (line 2)
> w.sample(frac: 0.001, seed: 7).n_rows()
18
note: 18 of 18,412 rows sampled with seed 7 (line 3)
