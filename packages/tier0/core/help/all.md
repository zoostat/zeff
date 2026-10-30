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

## See also
any, is_in, is_missing

## Examples
> all([true, true])
true
> all([true, NA])
NA
> all([false, NA])
false
> all([])
true
