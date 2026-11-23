# last

## Summary
Last item of a list, or the value of the last row of a column

## Description
The last item of a list, in the list's current order (section 6 of the Language Reference, "Row
order"). An empty list has no last item and is an error (Z0207), never `NA`. The last state of
a path from `iterate(keep: true)` is `last(path)`. Like R's `tail(x, 1)`.

Given a column, it is the value of the column's last row, as the rows stand: `last(d["x"])`, or
`last(x)` inside `add` or `aggregate`, where it is the last value of the frame or, under `by:`, of
each group. A column with no row has no last value and is an error (Z0207), as an empty list is.
Like R's `x[length(x)]`.

## Arguments
- list: The list, or a column; it needs at least one item, or one row.

## Value
The last item, with the type of the list's items; of a column, the value of its last row, with the type of the column's values (a category gives its label, as text).

## Missing values
An `NA` that stands last is the last item: `last([1, NA])` is `NA`. Nothing is skipped. The same holds for a column: where its last row is missing the result is `NA`, and no warning is given, since one value was asked for and it is that one.

## A common mistake
Taking the last item of a list that turned out empty. There is no item to give, and the statement stops rather than hand back `NA`:

> last([])
error[Z0207]: `last` of an empty list
  = help: a list must have an item for `last` to give one; test `len(xs) > 0` first, or build the list so that it is never empty

Test the list first, `len(xs) > 0`, or build it so that it cannot be empty.

## Coming from R or Stata
| last(xs) | tail(xs, 1) | no single function |
| last(d["x"]) | d$x[nrow(d)], or tail(d$x, 1) | x[_N] |

R's `tail(xs, 1)` on an empty vector gives an empty vector without a word; Zeff stops.

## Related tasks
- For the first item, use `first`.
- For the largest item, sort first with `sorted`.

## See also
first, reversed, len

## Examples
// The last item as the list stands, not the largest.
> last([3, 1, 2])
2

// The largest, after sorting.
> last(sorted([3, 1, 2]))
3

// Text.
> last(["b", "a"])
a

// The value of the last row of a column.
> let prices = frame(firm = ["a", "a", "b", "b", "b"], year = [2020, 2021, 2020, 2021, 2022], price = [10.0, 12.5, 7.0, 8.0, 9.0])
> last(prices["price"])
9

// Inside `aggregate`, the last value of each group, in the order the rows stand.
> prices.aggregate(close = last(price), by: firm)
frame · 2 rows × 2 variables

      firm   close
  1   a       12.5
  2   b          9
