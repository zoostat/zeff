# filter

## Summary
Keep the items of a list a function answers true for

## Description
A new list with the items of `list` for which `f` is `true`, in their order, so that choosing
from a list is written once: `filter(xs, fn(x) x > 0)` keeps the positive values. The function
is a name or a function literal written where it is used (section 3.7 of the Language Reference,
"Functions as values"), and it must answer `true` or `false` for every item: `NA` is an error
(Z0203), as it is in `if`, so a list with missing values is filtered by a function that decides
the missing case, `fn(x) !is_missing(x) && x > 0`. Like R's `Filter` or `purrr::keep`.

## Arguments
- list: The list to choose from.
- f: A function of one item giving a Bool, a named function or a literal `fn(x) ...`.

## Value
A list of the same type, holding the items `f` answered `true` for, in order.

## See also
map, reduce, is_in, any

## Examples
> filter([3, 1, 2], fn(x) x > 1)
3
2
> filter([1, NA, 3], fn(x) !is_missing(x) && x > 1)
3
