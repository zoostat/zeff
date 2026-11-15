# running_sum

## Summary
Cumulative sum down the rows; an NA makes the rest NA

## Description
The sum of `x` over the rows so far, in the frame's current row order: inside `add` or `update`,
each row receives the total of its own value and every value above it. Under `by:` the sum
restarts in each group. An `NA` makes that row and every row after it `NA`, as R's `cumsum`
does, and unlike Stata's `sum()`, which treats missing as zero. Sort first when the running
order matters: `data.sort(year).add(to_date = running_sum(sales))`.

## Arguments
- x: A variable of Ints or Floats.

## Value
A column of the type of `x`: Int stays Int and Float stays Float, one value per row.

## Details
Closed form, in row order: each value is the previous total plus the row's value, with the
rounding of a plain double sum. An Int sum that no longer fits in 64 bits is an error naming the
row (Z0211); `running_sum(1.0 * x)` sums as Float instead. `Inf` and `-Inf` together give `NA`
with a warning, and the rest of the column is then `NA`.

## References
R Core Team (2026), R: A Language and Environment for Statistical Computing, `cumsum`.

## See also
running_product, sum, lag, sort

## Examples
> let s = frame(month = [1, 2, 3, 4, 5], sales = [120, 80, NA, 150, 90])
> s.add(to_date = running_sum(sales))
frame · 5 rows × 3 variables

      month   sales   to_date
  1       1     120       120
  2       2      80       200
  3       3      NA        NA
  4       4     150        NA
  5       5      90        NA
> let t = frame(shop = ["a", "a", "b", "b", "b"], sales = [120, 80, 150, 90, 60])
> t.add(so_far = running_sum(sales), by: shop)
frame · 5 rows × 3 variables

      shop   sales   so_far
  1   a        120      120
  2   a         80      200
  3   b        150      150
  4   b         90      240
  5   b         60      300
