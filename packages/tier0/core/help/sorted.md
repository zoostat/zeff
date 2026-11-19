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

## Missing values
`NA` comes last, ascending and descending alike, and none is dropped.

## Coming from R or Stata
| sorted(xs) | sort(xs, na.last = TRUE) | local s : list sort xs |
| sorted(xs, desc: true) | sort(xs, decreasing = TRUE, na.last = TRUE) | no single function |

R's `sort` drops missing values unless `na.last` is given; here they are kept, last. R sorts text
by the computer's language setting, so its order differs between machines; here text is sorted
by code point, capitals before small letters, everywhere. For the rows of a frame the function
is `sort`.

## Related tasks
- To sort the rows of a frame, use `sort`.

## See also
unique, reversed, first, last

## Examples
// Ascending.
> sorted([3, 1, 2])
1
2
3

// Descending; the missing value stays last.
> sorted([3, NA, 1], desc: true)
3
1
NA

// Text by code point: capitals before small letters.
> sorted(["b", "a", "B"])
B
a
b
