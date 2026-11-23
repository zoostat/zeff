# first

## Summary
First item of a list, or the value of the first row of a column

## Description
The first item of a list, in the list's current order (section 6 of the Language Reference,
"Row order"): a list is kept in the order it was built or sorted in, so `first(sorted(xs))` is
the smallest item. An empty list has no first item and is an error (Z0207), never `NA`, since a
missing value would hide the mistake. Like R's `x[[1]]`.

Given a column, it is the value of the column's first row, as the rows stand: `first(d["x"])`, or
`first(x)` inside `add` or `aggregate`, where it is the first value of the frame or, under `by:`, of
each group. A column with no row has no first value and is an error (Z0207), as an empty list is.
Like R's `x[1]`.

## Arguments
- list: The list, or a column; it needs at least one item, or one row.

## Value
The first item, with the type of the list's items; of a column, the value of its first row, with the type of the column's values (a category gives its label, as text).

## Missing values
An `NA` that stands first is the first item: `first([NA, 1])` is `NA`. Nothing is skipped. The same holds for a column: where its first row is missing the result is `NA`, and no warning is given, since one value was asked for and it is that one.

## A common mistake
Taking the first item of a list that turned out empty, such as the rows a condition kept. There is no item to give, and the statement stops rather than hand back `NA`:

> first([])
error[Z0207]: `first` of an empty list
  = help: a list must have an item for `first` to give one; test `len(xs) > 0` first, or build the list so that it is never empty

Test the list first, `len(xs) > 0`, or build it so that it cannot be empty.

## Coming from R or Stata
| first(xs) | xs[[1]] | local a : word 1 of `xs' |
| first(d["x"]) | d$x[1] | x[1] |

R's `xs[[1]]` on an empty list is an error too; `xs[1]` there gives an empty list without a word,
and `head(xs, 1)` likewise.

## Related tasks
- For the last item, use `last`.
- For the smallest item, sort first with `sorted`.

## See also
last, reversed, len

## Examples
// The first item as the list stands, not the smallest.
> first([3, 1, 2])
3

// The smallest, after sorting.
> first(sorted([3, 1, 2]))
1

// Text.
> first(["b", "a"])
b

// The value of the first row of a column.
> let prices = frame(firm = ["a", "a", "b", "b", "b"], year = [2020, 2021, 2020, 2021, 2022], price = [10.0, 12.5, 7.0, 8.0, 9.0])
> first(prices["price"])
10

// Inside `aggregate`, the first value of each group, in the order the rows stand.
> prices.aggregate(open = first(price), by: firm)
frame · 2 rows × 2 variables

      firm   open
  1   a        10
  2   b         7
