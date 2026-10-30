# equal

## Summary
True when two values are identical

## Description
Whether `a` and `b` are the same value under the identity relation of section 3.7 of the
Language Reference, which `==` is not: `==` is three-valued and compares one scalar with another,
while `equal` always answers `true` or `false` and reaches inside collections. `NA` is identical
to `NA` and `-0.0` to `0.0`; an Int equals the Float of the same number; two lists are equal item
by item, two records field by field whatever order the fields are written in, two mappings entry
by entry in insertion order; values of different kinds are never equal, so `equal(1, "1")` is
`false`. It is the way a project tests its own functions, `assert(equal(f(x), [1, 2]), "...")`.
Like R's `identical`.

## Arguments
- a: A value: a Bool, Int, Float, Str, list, record or mapping.
- b: The value to compare it with.

## Value
A Bool, never `NA`.

## See also
is_in, unique, is

## Examples
> equal([1, 2, NA], [1, 2, NA])
true
> equal({a: 1, b: 2}, {b: 2, a: 1})
true
> equal(1, 1.0)
true
> equal(1, "1")
false
> equal([1, 2], [2, 1])
false
