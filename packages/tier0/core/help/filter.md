# filter

## Summary
Keep the items of a list a function answers true for

## Description
A new list with the items of `list` for which `f` is `true`, in their order, so that choosing
from a list is written once: `filter(xs, fn(x) x > 0)` keeps the positive values. The function
is a name or a function literal written where it is used (section 3.7 of the Language Reference,
"Functions as values"), and it must answer `true` or `false` for every item: `NA` is an error
(Z0203), as it is in `if`, so a list with missing values is filtered by a function that decides
the missing case, `fn(x) !is_missing(x) && x > 0`. A list that holds a failed value of `attempt`
is filtered when `f` is a test of its item by `is`: `filter(fits, fn(f) !f.is(Failed))` keeps the
values that succeeded. Like R's `Filter` or `purrr::keep`.

## Arguments
- list: The list to choose from.
- f: A function of one item giving a Bool, a named function or a literal `fn(x) ...`.

## Value
A list of the same type, holding the items `f` answered `true` for, in order.

## Missing values
`f` must answer `true` or `false` for every item. If it answers `NA`, as `x > 1` does for a missing `x`, the statement stops (Z0203): the function decides the missing case, `fn(x) !is_missing(x) && x > 1`.

## A common mistake
Filtering a list that has a missing value with a plain comparison. The comparison is unknown for the missing item, and `filter` does not guess whether to keep it:

> filter([1, NA, 3], fn(x) x > 1)
error[Z0203]: `f` of `filter` gave NA where true or false was needed
  = help: decide the missing case in the function: `!is_missing(x) && x > 0`, or `coalesce(x > 0, false)`

Decide the missing case in the function: `filter([1, NA, 3], fn(x) !is_missing(x) && x > 1)`.

## Coming from R or Stata
| filter(xs, f) | Filter(f, xs) | no single function |

R's `Filter` takes the function first. `dplyr::filter` there chooses rows of a data frame, which
is `where` here, as `keep if` in Stata.

## Related tasks
- To keep rows of a frame, use `where`.
- To change each item, use `map`.

## See also
map, reduce, is_in, any

## Examples
// The items above 1, in their order.
> filter([3, 1, 2], fn(x) x > 1)
3
2

// A list with a missing value: the function says what to do with it.
> filter([1, NA, 3], fn(x) !is_missing(x) && x > 1)
3
