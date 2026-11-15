# fill_missing

## Summary
Replace NA with a value, or with the last or the next value present

## In brief
- `fill_missing(hours, 0)`: 0 wherever `hours` is `NA`.
- `fill_missing(price, method: forward)`: the last price present, carried down each unit's rows.
- `fill_missing(price, method: backward)`: the next price present, carried up.

## Description
Fills the `NA` of `x` in one of two ways. Exactly one of a value and `method:` is given.

With a value, each `NA` is replaced by it, row by row; `x` and the value are joined to one type,
so an Int column filled with 0.5 becomes a Float column, and text cannot fill numbers. This
form works anywhere, on one value or on a column.

With `method:`, each `NA` is replaced by a neighbouring value of the same column: under
`forward` the last value present before it, under `backward` the next one after it. This form
stands inside `add` or `update`, where the rows have an order: each unit's rows in time order
when the frame is a declared panel, each group's rows under `by:`, and the frame's rows
otherwise.

Filling is a modelling decision, not a cleaning step: a zero in the place of a missing wage
changes every mean, and a carried price claims that nothing changed.

## Arguments
- x: A value or a variable, of any type that holds values: Bool, Int, Float, Str, Date or DateTime.
- value: What stands in for `NA`, of a type that joins with `x`; or a variable, read row by row.
- method: `forward` for the last value present, or `backward` for the next; given in the place of a value.

## Value
With a value, the joined type of `x` and the value, a column when either is one. With
`method:`, a column of the type of `x`, with its metadata.

## Missing values
With a value, no `NA` is left unless the value itself is `NA` on that row. With `method:`, an
`NA` with nothing to carry stays `NA`: before the first value of its unit or group under
`forward`, after the last under `backward`, and everywhere in a unit or group that holds no
value. Nothing is printed about the rows filled; `n_missing` counts what is left.

## Details
### The order the filling follows
On a declared panel the rows of each unit are taken in the order of the time variable, whatever
the order of the rows in the frame, and nothing is carried from one unit to the next. A gap in
a unit's time does not stop the carrying: the value carried is the last one present, however
long before. Without a declaration the filling follows the rows as they stand, so sort first
where the order matters.

### Joining types
With a value the rule is that of `coalesce`: Int and Float join as Float, `NA` fits any type,
and any other mixture is refused (Z0214).

### In a mapping or a formula
`fill_missing(x, 0)` may be written where a plot or a model takes an expression over the
frame. The form with `method:` may not: compute it with `add` first and name the new variable.

### Exactness
Exact: values are copied, never computed.

## A common mistake
Asking for both at once, to carry values down and put 0 where there is nothing to carry:

> frame(v = [NA, 3, NA]).add(f = fill_missing(v, 0, method: forward))
error[Z0410]: `fill_missing` takes a value or `method:`, not both
  = help: `fill_missing(x, 0)` puts 0 where `x` is NA; `fill_missing(x, method: forward)` carries the last value down

Write one inside the other: `fill_missing(fill_missing(v, method: forward), 0)`.

## Coming from R or Stata
| fill_missing(x, 0) | tidyr::replace_na(x, 0) | replace x = 0 if missing(x) |
| fill_missing(x, method: forward) | tidyr::fill(d, x) | bysort id (year): replace x = x[_n-1] if missing(x) |
| fill_missing(x, method: backward) | tidyr::fill(d, x, .direction = "up") | none built in |

`tidyr::fill` follows the groups of `dplyr::group_by` and the rows as they stand; here a
declared panel supplies both the groups and the order. `fill_forward(x)` of releases 0.6.0 and
0.7.0 is `fill_missing(x, method: forward)`.

## Related tasks
- To take the first of several fallbacks row by row, use `coalesce`.
- To drop the rows that hold `NA` rather than fill them, use `drop_missing`.
- To count the `NA` that are left, use `n_missing`.
- For the value of the period before, missing or not, use `lag`.
- To declare the unit and the time that the filling follows, use `panel`.

## See also
coalesce, drop_missing, n_missing, lag, panel

## Examples
// A value in the place of every `NA`.
> let d = frame(x = [1, NA, 3])
> d.add(y = fill_missing(x, 0))
frame · 3 rows × 2 variables

       x   y
  1    1   1
  2   NA   0
  3    3   3

// The value may be written by name; on one value the function gives one value.
> fill_missing(to(NA, Int), value: 0)
0

// The types join: whole numbers filled with 0.5 become Floats.
> d.add(y = fill_missing(x, 0.5))
frame · 3 rows × 2 variables

       x     y
  1    1     1
  2   NA   0.5
  3    3     3

// On a panel, the last price present is carried down each firm's years.
> let p = frame(firm = [1, 1, 1, 2, 2], year = [2020, 2021, 2022, 2020, 2021], price = [NA, 10, NA, NA, 7]).panel(unit: firm, time: year)
> p.add(carried = fill_missing(price, method: forward))
frame · 5 rows × 4 variables · panel firm × year

      firm   year   price   carried
  1      1   2020      NA        NA
  2      1   2021      10        10
  3      1   2022      NA        10
  4      2   2020      NA        NA
  5      2   2021       7         7

// Backward takes the next price present instead.
> p.add(next = fill_missing(price, method: backward))
frame · 5 rows × 4 variables · panel firm × year

      firm   year   price   next
  1      1   2020      NA     10
  2      1   2021      10     10
  3      1   2022      NA     NA
  4      2   2020      NA      7
  5      2   2021       7      7

// Without a panel, `by:` keeps the filling inside each group.
> frame(g = [1, 1, 2, 2], v = [5, NA, 7, NA]).add(f = fill_missing(v, method: forward), by: g)
frame · 4 rows × 3 variables

      g    v   f
  1   1    5   5
  2   1   NA   5
  3   2    7   7
  4   2   NA   7

// Carried down first, then 0 where there was nothing to carry.
> p.add(carried = fill_missing(fill_missing(price, method: forward), 0))
frame · 5 rows × 4 variables · panel firm × year

      firm   year   price   carried
  1      1   2020      NA         0
  2      1   2021      10        10
  3      1   2022      NA        10
  4      2   2020      NA         0
  5      2   2021       7         7
