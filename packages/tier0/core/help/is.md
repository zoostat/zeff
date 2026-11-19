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

## Missing values
`NA` has no type, so `is` gives `false` for it whatever type is named; `is_missing` is the test for `NA`. A column with missing values in it is still a column of its type.

## Coming from R or Stata
| x.is(Int) | is.integer(x) | confirm numeric variable x |
| x.is(Str) | is.character(x) | confirm string variable x |
| d.is(Frame) | is.data.frame(d) | no single function |

R's `is.numeric` is true of an integer and of a double alike; here `is(Float)` is false of an
Int, although an Int is taken wherever a Float is expected. R's `is.integer(3)` is false, since
`3` is a double there; here `3` is an Int.

## Related tasks
- For the name of the type, use `type_of`.
- To test for a missing value, use `is_missing`.

## See also
to, type_of, is_missing

## Examples
// A whole number is an Int.
> let x = 3
> x.is(Int)
true

// And not a Float.
> x.is(Float)
false

// A frame of the shipped data.
> let wages = dataset(wages)
> wages.is(Frame)
true

// A column, by the type of what it holds.
> wages["wage"].is(Float)
true

// A category column.
> wages["industry"].is(Category)
true

// A missing value is of no type.
> is(NA, Int)
false
