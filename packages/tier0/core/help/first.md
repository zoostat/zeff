# first

## Summary
First item of a list

## Description
The first item of a list, in the list's current order (section 6 of the Language Reference,
"Row order"): a list is kept in the order it was built or sorted in, so `first(sorted(xs))` is
the smallest item. An empty list has no first item and is an error (Z0207), never `NA`, since a
missing value would hide the mistake. Like R's `x[[1]]`.

## Arguments
- list: The list; it needs at least one item.

## Value
The first item, with the type of the list's items.

## Missing values
An `NA` that stands first is the first item: `first([NA, 1])` is `NA`. Nothing is skipped.

## A common mistake
Taking the first item of a list that turned out empty, such as the rows a condition kept. There is no item to give, and the statement stops rather than hand back `NA`:

> first([])
error[Z0207]: `first` of an empty list
  = help: a list must have an item for `first` to give one; test `len(xs) > 0` first, or build the list so that it is never empty

Test the list first, `len(xs) > 0`, or build it so that it cannot be empty.

## Coming from R or Stata
| first(xs) | xs[[1]] | local a : word 1 of `xs' |

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
