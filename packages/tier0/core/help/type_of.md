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

## See also
is, to

## Examples
> type_of(3)
Int
> type_of("text")
Str
> type_of([1, 2.5])
List<Float>
> let wages = import("data/wages.csv")
note: data/wages.csv read as CSV: 3 rows × 3 variables, types guessed; CSV carries no labels, value labels or categories (line 4)
> type_of(wages)
Frame
> type_of(wages["hours"])
Column<Int>
