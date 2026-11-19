# any

## Summary
True if any item is true

## Description
`||` over the items of a list, or of a `Column<Bool>`, under the three-valued truth table of
section 3.7 of the Language Reference: `any` is `true` as soon as an item is `true`, whatever
the others are; it is `false` only when every item is `false`; otherwise, when no item is `true`
and some are `NA`, it is `NA`. So `any([false, NA])` is `NA` and `any([true, NA])` is `true`.
An empty list has no true item and gives `false`. The items must be Bools: a number is not
true or false (compare it first, `any(xs > 0)` on a column). Like R's `any`, which gives the
same answers with `NA`.

## Arguments
- list: A list of Bools, or a Bool column; `NA` items are allowed.

## Value
A Bool, or `NA` when no item is `true` and some are missing.

## Missing values
An `NA` among the items is an unknown truth. If some item is `true` the answer is `true` whatever is unknown; if none is, and some are `NA`, the answer is `NA`. Nothing is skipped: for the answer among the known items, test them first, `any(coalesce(x > 0, false))`.

## Coming from R or Stata
| any(xs) | any(xs) | no single function |
| any(d["x"] > 0) | any(d$x > 0) | count if x > 0, then r(N) > 0 |

R gives the same answers with `NA`. In Stata a comparison with a missing value is not unknown:
missing counts as larger than every number, so `x > 0` is true of a missing `x` there.

## Related tasks
- For a test of every item, use `all`.
- For whether a value is among others, use `is_in`.

## See also
all, is_in, is_missing

## Examples
// One item is true.
> any([false, true, false])
true

// None is true and one is unknown: unknown.
> any([false, NA])
NA

// One is true: the unknown one cannot change that.
> any([true, NA])
true

// On a column of the shipped wages: some wage is above 150.
> any(dataset(wages)["wage"] > 150)
true
