# variables

## Summary
List of variable names

## Description
The names of a frame's variables, in the frame's order, as a list of `Str`. They are the names as
the frame has them: the header of the file for an imported frame (made snake_case when `import` was
given `names: snake`), the names written for a frame built with `frame`. The list prints one name
per line, up to 20, then the count of the rest.

Like R's `names`. The result is text, not a `Vars`: to name a variable for a frame method, write
it directly, as in `wages.mean(wage)`.

## Arguments
- data: The frame.

## Value
A `List<Str>`, empty for a frame with no variables.

## See also
n_cols, n_rows, describe

## Examples
> let wages = import("data/wages.csv")
note: data/wages.csv read as CSV: 3 rows × 3 variables, types guessed; CSV carries no labels, value labels or categories (line 1)
> wages.variables()
id
wage
hours
