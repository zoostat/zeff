# sample

## Summary
Random sample of rows; the seed is recorded

## In brief
- `d.sample(n: 500)`: 500 of the rows, none twice.
- `d.sample(frac: 0.1)`: a tenth of the rows.
- `d.sample(n: d.n_rows(), with_replacement: true)`: a bootstrap sample, the rows drawn with replacement.
- `seed:` gives the call a seed of its own; without it the run's `seed` decides.

## Description
A random sample of the rows, `n:` of them or the share `frac:`, without replacement unless
`with_replacement: true`, the kept rows returned in their original order with every column and
the panel declaration. The draws come from the run's random stream, keyed to the statement, so
a script repeats itself under `seed(n)` at its top; `seed:` given to the call starts a stream of
its own from that seed, the same wherever the call stands. A note gives the count and the seed,
`note: 500 of 18,412 rows sampled with seed 20261015`, so that a run can be repeated.

## Arguments
- data: The frame.
- n: The number of rows to keep; at most the frame's rows unless `with_replacement: true`.
- frac: The share of rows to keep, between 0 and 1, rounded to a whole number of rows.
- with_replacement: Whether a row may be drawn more than once; false unless given.
- seed: A seed for this call alone; the run's seed unless given.

## Value
A frame of the kept rows, in their original order.

## Missing values
Rows are drawn whatever they hold; a row with missing values is as likely as any other.

## Details
Stochastic: without replacement the kept rows are the first `n` positions of a partial
Fisher–Yates shuffle over the row positions, each drawn uniformly from the positions not yet
chosen with one uniform number from the Philox stream; with replacement, `n` positions each
drawn uniformly from all rows. The same seed gives the same rows on every target. A sample with
replacement that repeats a row of a panel drops the declaration, with a note. `n:` and `frac:`
together are refused (Z0410).

## A common mistake
Asking for more rows than the frame has. Without replacement a sample is at most the frame:

> dataset(firms).sample(n: 500)
error[Z0401]: `n:` of `sample` is 500, but firms has 400 rows
  = help: a sample without replacement is at most the frame; `with_replacement: true` allows repeats

Ask for at most the 400 rows, or allow repeats: `dataset(firms).sample(n: 500, with_replacement: true)`.

## Coming from R or Stata
| d.sample(n: 500) | d[sample(nrow(d), 500), ], or dplyr::slice_sample(d, n = 500) | sample 500, count |
| d.sample(frac: 0.1) | dplyr::slice_sample(d, prop = 0.1) | sample 10 |
| d.sample(n: d.n_rows(), with_replacement: true) | d[sample(nrow(d), replace = TRUE), ] | bsample |

R gives the rows in the order drawn; here the kept rows stay in the order they had. Stata's
`sample` takes a percentage unless `count` is given, and changes the dataset in memory. In R
and Stata the rows drawn depend on a seed set somewhere earlier, `set.seed` or `set seed`; here
the note of every sample prints its seed, so the same rows can be drawn again.

## Related tasks
- To set the seed of the whole run, use `seed`.
- To draw numbers from a distribution, use `draw`.
- For the first rows instead of random ones, use `head`.
- To keep one row of each value, use `distinct`.

## References
Knuth, D. E. (1997), The Art of Computer Programming, volume 2, section 3.4.2, algorithm P
(shuffling). Salmon, J. K., Moraes, M. A., Dror, R. O. and Shaw, D. E. (2011), Parallel random
numbers: as easy as 1, 2, 3 (the Philox generator).

## See also
seed, draw, distinct, head

## Examples
// Five rows; the note gives the count and the seed.
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

// A share of the rows.
> w.sample(frac: 0.001, seed: 7).n_rows()
18
note: 18 of 18,412 rows sampled with seed 7 (line 3)

// With replacement a row may come more than once: a bootstrap sample of the 400 firms, and how many different firms are in it.
> let firms = dataset(firms)
> firms.sample(n: 400, with_replacement: true, seed: 1).distinct(firm).n_rows()
255
note: 400 of 400 rows sampled with replacement with seed 1 (line 5)
note: 145 rows dropped as repeats of firm; 255 rows distinct (line 5)

// Without a seed of its own, the run's seed decides.
> seed(2026)
> firms.sample(n: 3)
frame · 3 rows × 4 variables

      firm   industry    size   founded
  1    228   Services       5   1954-04-14
  2    283   Education     36   2006-09-13
  3    351   Health        15   1961-01-13
note: 3 of 400 rows sampled with seed 2026 (line 7)
