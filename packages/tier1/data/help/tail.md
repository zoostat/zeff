# tail

## Summary
Last n rows

## Description
The last `n` rows of a frame, 10 by default, as a new frame with the same variables; every row when
the frame has fewer. Nothing is copied: the new frame shares its parent's columns. Like R's `tail`.

## Arguments
- data: The frame.
- n: How many rows, an Int of 0 or more; 10 by default.

## Value
A `Frame` with the same variables and at most `n` rows; the metadata of every variable and of the
frame is kept.

## See also
head, describe, where

## Examples
> let wages = import("data/wages.csv")
note: data/wages.csv read as CSV: 3 rows × 3 variables, types guessed; CSV carries no labels, value labels or categories (line 1)
> wages.tail(1)
frame · 1 row × 3 variables

      id   wage   hours
  1    3   31.2      NA
