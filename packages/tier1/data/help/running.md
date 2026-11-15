# running

## Summary
Running sum or product down the rows

## In brief
- `running(sales, stat: sum)`: the total so far, row by row.
- `running(growth, stat: product)`: the product so far, so growth factors become an index.
- Under `by:` the running value starts again in each group.

## Description
The sum or the product of `x` over the rows so far, in the frame's current row order: inside
`add` or `update`, each row receives its own value combined with every value above it. `stat:`
says which, and has no default. Under `by:` the running value restarts in each group.

Sort first when the order matters: `data.sort(year).add(to_date = running(sales, stat: sum))`.

## Arguments
- x: A variable of Ints or Floats.
- stat: `sum` for the running total or `product` for the running product; always given.

## Value
With `stat: sum`, a column of the type of `x`: Int stays Int and Float stays Float. With
`stat: product`, a column of Floats, whatever the type of `x`. One value per row.

## Missing values
An `NA` makes that row and every row after it `NA`, in its group under `by:`, as R's `cumsum`
and `cumprod` do. Stata's `sum()` treats a missing value as zero instead and carries on. To
carry on past a gap, fill it first: `running(fill_missing(sales, 0), stat: sum)`.

## Details
### The order of the rows
The running value follows the rows as they stand, not a declared panel's time order, so a
frame is sorted first. Under `by:` each group is taken in its own row order.

### Whole numbers and overflow
A running sum of Ints is kept exact. One that no longer fits in 64 bits is an error naming the
row (Z0211); `running(1.0 * x, stat: sum)` sums as Float instead. A product is always a
Float, as R's `cumprod`, so a long product of large Ints rounds.

### Infinite values
`Inf` and `-Inf` together in a sum, or `0` and `Inf` in a product, have no value: the row is
`NA` with a warning (W0205), and the rest of the column after it.

### Exactness
Closed form, in row order: each value is the previous one plus, or times, the row's value,
with the rounding of a plain double operation.

## A common mistake
Leaving `stat:` out. There is no default, so that a script says which running value it means:

> frame(sales = [120, 80, 150]).add(to_date = running(sales))
error[Z0406]: `running` needs `stat:`
  = help: usage: `running(x, stat: sum)` for the running total, or `running(x, stat: product)` for the running product

Write `running(sales, stat: sum)`.

## Coming from R or Stata
| running(x, stat: sum) | cumsum(x) | gen s = sum(x) |
| running(x, stat: product) | cumprod(x) | gen p = exp(sum(ln(x))) |
| d.add(s = running(x, stat: sum), by: g) | ave(x, g, FUN = cumsum) | bysort g: gen s = sum(x) |

Stata's `sum()` skips missing values; here, as in R, a missing value ends the running value.
`running_sum(x)` and `running_product(x)` of releases 0.6.0 and 0.7.0 are
`running(x, stat: sum)` and `running(x, stat: product)`.

## Related tasks
- For the total of a whole column or group, use `sum`.
- For the value of the row before, use `lag`.
- To put the rows in the order the running value should follow, use `sort`.
- To put a value where one is missing before summing, use `fill_missing`.
- For the position of each value among the others, use `rank`.

## References
R Core Team (2026), R: A Language and Environment for Statistical Computing, `cumsum` and
`cumprod`.

## See also
sum, lag, sort, fill_missing, rank

## Examples
// The running total; the missing month ends it.
> let s = frame(month = [1, 2, 3, 4, 5], sales = [120, 80, NA, 150, 90])
> s.add(to_date = running(sales, stat: sum))
frame · 5 rows × 3 variables

      month   sales   to_date
  1       1     120       120
  2       2      80       200
  3       3      NA        NA
  4       4     150        NA
  5       5      90        NA

// Under `by:` the total starts again in each shop.
> let t = frame(shop = ["a", "a", "b", "b", "b"], sales = [120, 80, 150, 90, 60])
> t.add(so_far = running(sales, stat: sum), by: shop)
frame · 5 rows × 3 variables

      shop   sales   so_far
  1   a        120      120
  2   a         80      200
  3   b        150      150
  4   b         90      240
  5   b         60      300

// The running product turns yearly growth factors into an index.
> let g = frame(year = [2021, 2022, 2023, 2024], growth = [1.1, 0.9, 1.25, 1.0])
> g.add(index = running(growth, stat: product))
frame · 4 rows × 3 variables

      year   growth    index
  1   2021      1.1      1.1
  2   2022      0.9     0.99
  3   2023     1.25   1.2375
  4   2024        1   1.2375

// A missing value filled with 0 first, so that the total carries on.
> s.add(to_date = running(fill_missing(sales, 0), stat: sum))
frame · 5 rows × 3 variables

      month   sales   to_date
  1       1     120       120
  2       2      80       200
  3       3      NA       200
  4       4     150       350
  5       5      90       440
