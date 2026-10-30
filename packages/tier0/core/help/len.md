# len

## Summary
Items in a list, or code points in a string

## Description
The number of items in a list, of Unicode scalar values (code points) in a string, of rows in a
column, or of entries in a mapping. A string is counted in code points, never in bytes or in
grapheme clusters (section 3.8 of the Language Reference): `len("café")` is 4, and an accent
written as a combining mark is its own code point, so `len("e\u{301}")` is 2. `NA` gives `NA`.

A list may hold `Failed` values from `attempt` in a loop. `len` counts the items that succeeded
and says how many failed (W0801), so that `len(fits)` is the number of fits a `report` would
use. Like R's `length` and `nchar(type = "chars")`.

## Arguments
- x: A list, a string, a column or a mapping.

## Value
An Int.

## See also
first, last, append, reversed, attempt

## Examples
> len([3, 1, 2])
3
> len("café")
4
> len([])
0
> let d = frame(x = [1, 2, 3, 4, 5], y = [2, 4, 7, 8, 11])
> len(d["x"])
5
> let fits = [for cut in [0, 2, 10] { attempt(d.where(x > cut).reg(y ~ x)) }]
note: no row kept: the result is empty (line 6)
> len(fits)
2
warning: 1 of 3 items failed (Z0801 at line 6) and was skipped (line 7)
  = help: the count is of the items that succeeded; `x.is(Failed)` tells which failed
