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

## Missing values
An `NA` item is given to `f` like any other; with `fn(acc, x) acc + x` the running value becomes `NA` and stays so.

## A common mistake
Starting from a whole number when the items are Floats. The running value keeps the type of `start`, and a sum of Floats is not an Int:

> reduce([1.5, 2.5], fn(acc, x) acc + x, 0)
error[Z0205]: the body of `fn(acc, x)` gives a Float, but `Fn(Int, Float) -> Int` needs an Int
  = help: a function passed as a value matches the parameter types and the return type exactly; write a body of that type

Start from a Float: `reduce([1.5, 2.5], fn(acc, x) acc + x, 0.0)` is `4`.

## Coming from R or Stata
| reduce(xs, f, start) | Reduce(f, xs, start) | no single function |

R's `Reduce` takes the function first, and without a starting value it begins from the first
item; here the start is always given, so an empty list has an answer.

## Related tasks
- For the sum of a variable, use `sum`.
- To change each item and keep a list, use `map`.

## See also
map, filter, sum

## Examples
// A sum.
> reduce([1, 2, 3], fn(acc, x) acc + x, 0)
6

// A product of Floats, from a Float.
> reduce([1.5, 2.5], fn(acc, x) acc * x, 1.0)
3.75

// The largest item.
> reduce([3, 1, 2], fn(acc, x) max(acc, x), -Inf)
3

// An empty list gives the start.
> reduce([], fn(acc, x) acc + x, 10)
10
