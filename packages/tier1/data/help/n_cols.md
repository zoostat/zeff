# n_cols

## Summary
Number of variables

## Description
The number of variables (columns) of a frame, as an `Int`. It counts every variable whatever its
type, and it is the length of the list that `variables` gives.

Like R's `ncol`.

## Arguments
- data: The frame.

## Value
An `Int`.

## See also
n_rows, variables, describe

## Examples
> let wages = import("data/wages.csv")
note: data/wages.csv read as CSV: 3 rows × 3 variables, types guessed; CSV carries no labels, value labels or categories (line 1)
> wages.n_cols()
3
> frame(x = [1, 2], y = [true, false]).n_cols()
2
