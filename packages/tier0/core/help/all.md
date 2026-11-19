# all

## Summary
True if every item is true

## Description
`&&` over the items of a list, or of a `Column<Bool>`, under the three-valued truth table of
section 3.7 of the Language Reference: `all` is `false` as soon as an item is `false`, whatever
the others are; it is `true` only when every item is `true`; otherwise, when no item is `false`
and some are `NA`, it is `NA`. So `all([true, NA])` is `NA` and `all([false, NA])` is `false`.
An empty list has no false item and gives `true`. The items must be Bools: compare a number
first, `all(xs >= 0)` on a column. Like R's `all`, which gives the same answers with `NA`.

## Arguments
- list: A list of Bools, or a Bool column; `NA` items are allowed.

## Value
A Bool, or `NA` when no item is `false` and some are missing.

## Missing values
An `NA` among the items is an unknown truth. If some item is `false` the answer is `false` whatever is unknown; if none is, and some are `NA`, the answer is `NA`. Nothing is skipped: for the answer among the known items, test them first, `all(coalesce(x > 0, true))`.

## Coming from R or Stata
| all(xs) | all(xs) | no single function |
| all(d["x"] > 0) | all(d$x > 0) | count if !(x > 0), then r(N) == 0 |

R gives the same answers with `NA`. In Stata a comparison with a missing value is not unknown:
missing counts as larger than every number, so `x > 0` is true of a missing `x` there.

## Related tasks
- For a test of at least one item, use `any`.

## See also
any, is_in, is_missing

## Examples
// Every item is true.
> all([true, true])
true

// None is false and one is unknown: unknown.
> all([true, NA])
NA

// One is false: the unknown one cannot change that.
> all([false, NA])
false

// On a column of the shipped wages: the hours have missing values, so whether all are positive is unknown.
> all(dataset(wages)["hours"] > 0)
NA
