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

## See also
last, reversed, len

## Examples
> first([3, 1, 2])
3
> first(["b", "a"])
b
