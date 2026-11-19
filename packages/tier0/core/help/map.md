# map

## Summary
Apply a function to each item of a list

## Description
A new list with `f` applied to each item of `list`, in order, so that a computation over a list
is written once and read as the mathematics: `map(seq(1, n - 1), fn(i) i / n)` is the
probabilities i/n, each one computed exactly as i divided by n. The function is a name, as a
function value is anywhere (section 3.7 of the Language Reference, "Functions as values"), or a
function literal written where it is used, `fn(i) i / n`, whose parameter takes the type of the
list's items and whose body is one expression over it and the bindings in sight. The result is a
list of what `f` gives; an empty list gives an empty list. Like R's `lapply` or `purrr::map`
without the choice of output type: a list always.

## Arguments
- list: The list to go over.
- f: A function of one item, a named function or a literal `fn(x) ...`.

## Value
A list of the same length, holding what `f` gave for each item, in order.

## Missing values
An `NA` item is given to `f` like any other; what comes back is what `f` makes of it.

## A common mistake
Giving a column of a frame, as `lapply` would be given one in R. A column is gone over by `add`, row by row, without `map`:

> map(dataset(wages)["wage"], fn(v) v * 2)
error[Z0401]: `list` of `map` must be a List, but this is a Column
  = help: `map` goes over the items of a list; `[for ...]` and `seq` make one, and `wages["wage"]` is a column, which `add` goes over

Inside a frame write the expression itself: `dataset(wages).add(double = wage * 2)`.

## Coming from R or Stata
| map(xs, f) | lapply(xs, f) | foreach x of local xs { ... } |
| map(xs, fn(x) x * 2) | lapply(xs, function(x) x * 2) | foreach x of local xs { ... } |

R's `sapply` and `purrr::map_dbl` choose the kind of result; here the result is always a list.
A variable of a frame needs no `map`: arithmetic and every function of `core` go row by row.

## Related tasks
- To keep some items, use `filter`.
- To fold the items into one value, use `reduce`.

## See also
filter, reduce, seq, len

## Examples
// A function written where it is used.
> map([1, 2, 3], fn(i) i * 2)
2
4
6

// Over a sequence, with a binding in sight.
> let k = 4
> map(seq(1, k - 1), fn(i) i / k)
0.25
0.5
0.75

// A function by its name.
> map(["b", "a"], len)
1
1
