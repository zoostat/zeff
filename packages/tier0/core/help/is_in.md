# is_in

## Summary
True where x is one of the listed values

## Description
Whether `x` is identical to an item of `list`, under the identity relation of section 3.7 of the
Language Reference: `is_in(NA, [NA])` is `true`, and an Int matches the Float of the same
number. Given a column, `is_in` applies row by row and gives a `Column<Bool>`, so inside `where`
or `add` it picks rows by a list of values, as Stata's `inlist` does; a category's rows match the
list by their labels. `is_in(k, keys(m))` tests a mapping for a key before `m[k]`. Like R's `%in%`.

## Arguments
- x: A value, or a column of values.
- list: The values to look among.

## Value
A Bool; a `Column<Bool>` when `x` is a column.

## See also
unique, keys, sorted, equal

## Examples
> is_in(2, [1, 2, 3])
true
> is_in(NA, [1, NA])
true
> let d = frame(x = [1, 2, 3, 4, 5], y = [2, 4, 7, 8, 11])
> d.where(is_in(x, [1, 3, 5]))["y"]
Column<Int> · 3 values
2
7
11
