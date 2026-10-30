# append

## Summary
A new list with an item added

## Description
A new list holding the items of `list` and then `x`. The list given is not changed: a binding
never changes in Zeff, so `let ys = append(xs, 4)` leaves `xs` as it was. The item joins the
type of the list's items by the rules of section 3.7 of the Language Reference ("Joining types"):
an Int added to a list of Floats gives a list of Floats, `NA` fits any list, and an item of a
type that does not join is an error naming both types (Z0214). Appending to a frame is
`data.append` in the `data` package, which arrives with release 0.6.0. Like R's `c(x, value)`.

## Arguments
- list: The list to add to.
- x: The item to add after its last.

## Value
A list of the joined type, one item longer.

## See also
len, first, last, reversed

## Examples
> let xs = [1, 2]
> append(xs, 3)
1
2
3
> xs
1
2
> append([1.5, 2.5], 3)
1.5
2.5
3
> append([], "a")
a
