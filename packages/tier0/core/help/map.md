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

## See also
filter, reduce, seq, len

## Examples
> map([1, 2, 3], fn(i) i * 2)
2
4
6
> let k = 4
> map(seq(1, k - 1), fn(i) i / k)
0.25
0.5
0.75
> map(["b", "a"], len)
1
1
