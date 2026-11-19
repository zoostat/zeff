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

## Missing values
`NA` is identical to `NA`: `is_in(NA, [1, NA])` is `true`, and `is_in(NA, [1, 2])` is `false`, never `NA`. In a column each row is taken by itself.

## Coming from R or Stata
| is_in(x, [1, 3, 5]) | x %in% c(1, 3, 5) | inlist(x, 1, 3, 5) |

R's `%in%` agrees, missing values included. Stata's `inlist` takes at most ten values when they
are text; a list here has no such limit.

## Related tasks
- To keep the rows whose value is in a list, use it inside `where`.

## See also
unique, keys, sorted, equal

## Examples
// A value among others.
> is_in(2, [1, 2, 3])
true

// A missing value is found among missing values.
> is_in(NA, [1, NA])
true

// Row by row inside `where`, on the shipped wages: the rows of two industries.
> dataset(wages).where(is_in(industry, ["Mining", "Utilities"])).n_rows()
570
