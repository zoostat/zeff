# equal

## Summary
True when two values are identical

## Description
Whether `a` and `b` are the same value under the identity relation of section 3.7 of the
Language Reference, which `==` is not: `==` is three-valued and compares one scalar with another,
while `equal` always answers `true` or `false` and reaches inside collections. `NA` is identical
to `NA` and `-0.0` to `0.0`; an Int equals the Float of the same number; two lists are equal item
by item, two records field by field whatever order the fields are written in, two mappings entry
by entry in insertion order; two causal graphs when they have the same nodes, edges and
declarations, whatever order their lines are written in; two columns are equal when they have the same number of rows and
the same value in every row, `NA` with `NA`; values of different kinds are never equal, so
`equal(1, "1")` is `false`, and a column is not equal to a list of its values. It is the way a project tests its own functions, `assert(equal(f(x), [1, 2]), "...")`.
Like R's `identical`.

## Arguments
- a: A value: a Bool, Int, Float, Str, list, record or mapping, a column, or a causal graph.
- b: The value to compare it with.

## Value
A Bool, never `NA`.

## Missing values
`NA` is identical to `NA`, so `equal(NA, NA)` is `true`, and the answer is never `NA`; `NA == NA` is `NA`.

## Coming from R or Stata
| equal(a, b) | identical(a, b) | no single function |

R's `identical(1L, 1)` is `FALSE`, since one is an integer and the other a double; here a whole
number and the Float of the same value are equal. R's `all.equal` allows a small difference;
`equal` allows none.

## Related tasks
- For two numbers that should agree to a tolerance, compare the size of their difference, taken with `abs`.
- To check a result inside a script, use `assert`.

## See also
is_in, unique, is

## Examples
// Two lists, item by item; a missing value equals a missing value.
> equal([1, 2, NA], [1, 2, NA])
true

// Two records, whatever the order of their fields.
> equal({a: 1, b: 2}, {b: 2, a: 1})
true

// A whole number and the Float of the same value.
> equal(1, 1.0)
true

// A number and a text are never equal.
> equal(1, "1")
false

// The order of a list matters.
> equal([1, 2], [2, 1])
false
