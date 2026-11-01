# quantile_group

## Summary
Group 1 to n by cutting at sample quantiles; tied values share a group

## Description
The group each value falls in when the column is cut at its sample quantiles: with `n: 4` the
quartile groups 1 to 4, with `n: 10` the deciles. The cut points are the quantiles at 1/n to
(n − 1)/n under the definition `type:` names (the nine of Hyndman and Fan, as `quantile`
offers them; 7 unless given, R's default), and a value equal to a cut point falls in the group
below it, so that tied values always share a group, as R's `cut(x, quantile(x, probs),
include.lowest = TRUE)` places them. `NA` stays `NA`, and the quantiles are taken over the
values present. Under `by:` the cut points are taken within each group. Like Stata's `xtile`.

## Arguments
- x: A variable of Ints or Floats.
- n: The number of groups, a whole number of at least 1.
- type: The quantile definition, 1 to 9; 7 unless given.

## Value
A column of Ints from 1 to `n`, one per row, `NA` where `x` is.

## Details
Closed form: the cut points are the kernel's sample quantiles, the same bits as `quantile`
gives, and each row's group is one more than the number of cut points below its value. When the
column has fewer distinct values than groups, two cut points coincide, the groups between them
are empty and the numbering is kept, as Stata's `xtile` keeps it; R's `cut` refuses repeated
breaks instead.

## References
Hyndman, R. J. and Fan, Y. (1996), Sample quantiles in statistical packages, The American
Statistician 50(4), 361–365. Stata, the `xtile` command.

## See also
quantile, bin, rank, winsorize

## Examples
> let d = frame(x = [5, 1, 9, 3, 7, NA, 2, 8])
> d.add(quartile = quantile_group(x, n: 4), third = quantile_group(x, n: 3, type: 1))
frame · 8 rows × 3 variables

       x   quartile   third
  1    5          2       2
  2    1          1       1
  3    9          4       3
  4    3          2       1
  5    7          3       2
  6   NA         NA      NA
  7    2          1       1
  8    8          4       3
> frame(x = [1, 1, 1, 1, 2]).add(q = quantile_group(x, n: 4))
frame · 5 rows × 2 variables

      x   q
  1   1   1
  2   1   1
  3   1   1
  4   1   1
  5   2   4
