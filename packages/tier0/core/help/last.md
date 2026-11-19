# last

## Summary
Last item of a list

## Description
The last item of a list, in the list's current order (section 6 of the Language Reference, "Row
order"). An empty list has no last item and is an error (Z0207), never `NA`. The last state of
a path from `iterate(keep: true)` is `last(path)`. Like R's `tail(x, 1)`.

## Arguments
- list: The list; it needs at least one item.

## Value
The last item, with the type of the list's items.

## Missing values
An `NA` that stands last is the last item: `last([1, NA])` is `NA`. Nothing is skipped.

## A common mistake
Taking the last item of a list that turned out empty. There is no item to give, and the statement stops rather than hand back `NA`:

> last([])
error[Z0207]: `last` of an empty list
  = help: a list must have an item for `last` to give one; test `len(xs) > 0` first, or build the list so that it is never empty

Test the list first, `len(xs) > 0`, or build it so that it cannot be empty.

## Coming from R or Stata
| last(xs) | tail(xs, 1) | no single function |

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
