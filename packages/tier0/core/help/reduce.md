# reduce

## Summary
Fold a list into one value with a function

## Description
One value built from the items of `list` in order: `f` is called with the running value and the
first item, then with what it gave and the second item, and so on, starting from `start`;
`reduce(xs, fn(acc, x) acc + x, 0)` is the sum, and `reduce(xs, fn(acc, x) max(acc, x), -Inf)`
the largest. The running value keeps the type of `start`, and `f` gives that type. The function
is a name or a function literal written where it is used (section 3.7 of the Language Reference,
"Functions as values"); an empty list gives `start`. Like R's `Reduce` with `init`, or
`purrr::reduce`.

## Arguments
- list: The list to fold.
- f: A function of the running value and one item, giving the next running value; a named function or a literal `fn(acc, x) ...`.
- start: The running value before the first item, whose type the result has.

## Value
The running value after the last item, of the type of `start`.

## See also
map, filter, sum

## Examples
> reduce([1, 2, 3], fn(acc, x) acc + x, 0)
6
> reduce([1.5, 2.5], fn(acc, x) acc * x, 1.0)
3.75
> reduce([], fn(acc, x) acc + x, 10)
10
