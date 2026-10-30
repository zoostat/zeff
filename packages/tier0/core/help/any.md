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

## See also
all, is_in, is_missing

## Examples
> any([false, true, false])
true
> any([false, NA])
NA
> any([true, NA])
true
> any([])
false
