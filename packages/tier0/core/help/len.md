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

## Missing values
`NA` in place of the list or the text gives `NA`. An `NA` inside a list is an item like any other and is counted: `len([1, NA])` is `2`; `n_missing` counts the missing ones. A `Failed` item is not counted, and warning W0801 says how many there were.

## A common mistake
Asking for the length of a frame, as R's `nrow` or `length` would be asked. A frame has rows and variables, and `len` counts neither:

> len(dataset(wages))
error[Z0401]: `x` of `len` must be a List, Str, Column or Mapping, but this is a Frame
  = help: usage: `len(x)`

Ask for the one meant: `dataset(wages).n_rows()` is `18412`, and `n_cols()` counts the variables.

## Coming from R or Stata
| len(xs) | length(xs) | local n : word count `xs' |
| len("café") | nchar("café") | ustrlen("café") |
| len(d["x"]) | length(d$x) | _N |

Stata has no list of values; a local macro of words is the nearest thing. Its `strlen` counts
bytes, so an accented letter counts twice there; `ustrlen` counts as `len` does.

## Related tasks
- For the rows of a frame, use `n_rows`.
- For the missing items of a list, use `n_missing`.

## See also
first, last, append, reversed, attempt

## Examples
// The items of a list.
> len([3, 1, 2])
3

// The code points of a text: four, although the accented letter takes two bytes.
> len("café")
4

// A missing item is an item.
> len([1, NA])
2

// The rows of a column of the shipped wages.
> len(dataset(wages)["wage"])
18412

// A list of fits in which one failed: the count is of those that succeeded, and a warning says so.
> let d = frame(x = [1, 2, 3, 4, 5], y = [2, 4, 7, 8, 11])
> let fits = [for cut in [0, 2, 10] { attempt(d.where(x > cut).reg(y ~ x)) }]
note: no row kept: the result is empty (line 6)

// The two that succeeded.
> len(fits)
2
warning[W0801]: 1 of 3 items failed (Z0801 at line 6) and was skipped (line 7)
  = help: the count is of the items that succeeded; `x.is(Failed)` tells which failed
