# is

## Summary
True when a value has the type named

## Description
Tests whether `x` is of the type named, by the names of section 4 of the Language Reference: the
scalar types (`Int`, `Float`, `Str`, `Bool`), `Column`, `Frame`, `Table`, `Distribution`, `List`,
`Record` and `Mapping`. A column holds values of one type, so a scalar type name tests what the
column holds: `wages["wage"].is(Float)` is true for a Float column, as is `wages["wage"].is(Column)`;
`is(Category)` tests for a category column. An Int is not a Float here, although an Int is accepted
wherever a Float is expected. `NA` has no type, so `is` gives `false` for it whatever the name;
`is_missing` tests for `NA`. R has `is.numeric` and its kin.

## Arguments
- x: Any value.
- type: A type name.

## Value
A Bool.

## See also
to, type_of, is_missing

## Examples
> let x = 3
> x.is(Int)
true
> x.is(Float)
false
> let wages = import("data/wages.csv")
note: data/wages.csv read as CSV: 3 rows × 3 variables, types guessed; CSV carries no labels, value labels or categories (line 4)
> wages.is(Frame)
true
> wages["wage"].is(Float)
true
> wages["wage"].is(Column)
true
