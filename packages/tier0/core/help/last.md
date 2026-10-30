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

## See also
first, reversed, len

## Examples
> last([3, 1, 2])
2
> last(["b", "a"])
a
