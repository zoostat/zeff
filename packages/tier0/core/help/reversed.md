# reversed

## Summary
Reversed copy of a list

## Description
A new list with the items of `list` in the opposite order; the list given is not changed.
`first(reversed(xs))` is `last(xs)`, and reversing twice gives the list back. Like R's `rev`.

## Arguments
- list: The list to reverse.

## Value
A list of the same type and length.

## Coming from R or Stata
| reversed(xs) | rev(xs) | no single function |

R's `rev` is the same.

## Related tasks
- For a list in descending order, use `sorted`.

## See also
first, last, len, append

## Examples
// The opposite order.
> reversed([3, 1, 2])
2
1
3

// One item is its own reverse.
> reversed(["a"])
a

// An empty list stays empty.
> reversed([])
[]
