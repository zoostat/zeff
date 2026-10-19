# head

## Summary
First n rows

## Description
The first `n` rows of a frame, 10 by default, as a new frame with the same variables; every row
when the frame has fewer. Nothing is copied: the new frame shares its parent's columns. It is the
quick look at a file after `import`, beside `describe`. Like R's `head`.

## Arguments
- data: The frame.
- n: How many rows, an Int of 0 or more; 10 by default.

## Value
A `Frame` with the same variables and at most `n` rows; the metadata of every variable and of the
frame is kept.

## See also
tail, describe, where

## Examples
> let wages = import("data/wages.csv")
note: data/wages.csv read as CSV: 3 rows × 3 variables, types guessed; CSV carries no labels, value labels or categories (line 1)
> wages.head(2)
frame · 2 rows × 3 variables

      id   wage   hours
  1    1   23.4      40
  2    2     19      35
> wages.head(n: 0).n_rows()
0
