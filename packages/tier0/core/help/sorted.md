# sorted

## Summary
Sorted copy of a list

## Description
A new list with the items in order, ascending unless `desc: true`. The order is that of section
3.7 of the Language Reference ("Identity"): `false` before `true`, numbers numerically with Int
and Float compared as numbers, text by Unicode scalar value (so `"B"` comes before `"a"`, and
never by locale, section 3.8), and `NA` last in both directions, as `sort` on a frame does
(section 4.1). The sort is stable: items that are identical, such as `-0.0` and `0.0`, keep
their order. The list given is not changed. Like R's `sort(x, na.last = TRUE)`.

## Arguments
- list: A list of Bools, Ints, Floats or Strs.
- desc: Sort from largest to smallest; `NA` stays last.

## Value
A list of the same type and length.

## See also
unique, reversed, first, last

## Examples
> sorted([3, 1, 2])
1
2
3
> sorted([3, NA, 1], desc: true)
3
1
NA
> sorted(["b", "a", "B"])
B
a
b
