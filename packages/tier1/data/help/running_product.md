# running_product

## Summary
Cumulative product down the rows; an NA makes the rest NA

## Description
The product of `x` over the rows so far, in the frame's current row order: inside `add` or
`update`, each row receives its own value multiplied by every value above it, so a column of
growth factors becomes an index. Under `by:` the product restarts in each group. An `NA` makes
that row and every row after it `NA`, as R's `cumprod` does.

## Arguments
- x: A variable of Ints or Floats.

## Value
A column of Floats, one value per row, whatever the type of `x`.

## Details
Closed form, in row order: each value is the previous product times the row's value, as a
double, so an Int column is multiplied as Floats and a long product may round. `0` and `Inf`
together give `NA` with a warning, and the rest of the column is then `NA`.

## References
R Core Team (2026), R: A Language and Environment for Statistical Computing, `cumprod`.

## See also
running_sum, lag, sort

## Examples
> let g = frame(year = [2020, 2021, 2022, 2023], growth = [1.1, 0.9, 1.25, 1.0])
> g.add(index = running_product(growth))
frame · 4 rows × 3 variables

      year   growth    index
  1   2020      1.1      1.1
  2   2021      0.9     0.99
  3   2022     1.25   1.2375
  4   2023        1   1.2375
