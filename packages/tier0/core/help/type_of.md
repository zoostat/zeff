# type_of

## Summary
The name of a value's type

## Description
The type of `x` as section 4 of the Language Reference writes it: `Int`, `Float`, `Str`, `Bool`,
`Frame`, `Table`, `Distribution`, a column as `Column<Int>` and the like, a list as `List<Float>`
when its items share a type (Int and Float joining as Float) and `List` otherwise, a mapping as
`Mapping<Int, Str>`, a record as `Record`, and `NA` as `NA`. The name is text, so it can be
compared: `type_of(x) == "Int"`. R has `class`, whose names differ (`integer`, `numeric`,
`character`, `logical`, `data.frame`).

## Arguments
- x: Any value.

## Value
A Str.

## Missing values
`NA` has no type of its own, and its type is given as `NA`. A column with missing values in it has the type of its other values.

## Coming from R or Stata
| type_of(x) | class(x) | describe x |

R's names differ: `integer`, `numeric`, `character`, `logical`, `data.frame`, `factor`. Stata's
`describe` gives the storage type of a variable of the data (`byte`, `int`, `float`, `double`,
`str12`), which says how a value is stored and not what it is.

## Related tasks
- To test for one type, use `is`.
- For the types of every variable of a frame, use `describe`.

## See also
is, to

## Examples
// A whole number.
> type_of(3)
Int

// Text.
> type_of("text")
Str

// A list whose items join as Floats.
> type_of([1, 2.5])
List<Float>

// A frame of the shipped data.
> let wages = dataset(wages)
> type_of(wages)
Frame

// One of its columns.
> type_of(wages["industry"])
Column<Category>
