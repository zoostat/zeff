# tail

## Summary
Last n rows

## Description
The last `n` rows of a frame, 10 by default, as a new frame with the same variables; every row when
the frame has fewer. Nothing is copied: the new frame shares its parent's columns.

## Arguments
- data: The frame.
- n: How many rows, an Int of 0 or more; 10 by default.

## Value
A `Frame` with the same variables and at most `n` rows; the metadata of every variable and of the
frame is kept.

## Missing values
Rows are taken by their position, whatever they hold.

## A common mistake
Asking for all but the first rows with a negative count, as R's `tail(d, -1)` allows. A count of rows is 0 or more:

> dataset(sleep).tail(n: -1)
error[Z0401]: `n:` of `tail` cannot be -1
  = help: write how many rows to show, 0 or more: `tail(n: 5)`

Count the rows instead: `d.tail(n: d.n_rows() - 1)` is all but the first.

## Coming from R or Stata
| d.tail() | tail(d, 10) | list in -10/l |
| d.tail(n: 5) | tail(d, 5) | list in -5/l |

R's `tail` shows 6 rows unless told; here 10. In Stata's `list in -5/l` the `l` stands for the
last row.

## Related tasks
- For the first rows, use `head`.
- For the rows that meet a condition, use `where`.
- For the rows with the largest or the smallest values, sort first with `sort`.

## See also
head, describe, where

## Examples
// The last three rows; the rows of the frame given back are numbered from 1.
> dataset(firms).tail(3)
frame · 3 rows × 4 variables

      firm   industry        size   founded
  1    398   Construction       0   1965-11-27
  2    399   Manufacturing      0   1952-08-10
  3    400   Education          0   1979-12-12

// `n:` by its name.
> dataset(sleep).tail(n: 2)
frame · 2 rows × 3 variables

      extra   group               id
  1     4.6   laevo-hyoscyamine    9
  2     3.4   laevo-hyoscyamine   10

// After a sort: the three longest gains in sleep.
> dataset(sleep).sort(extra).tail(n: 3)
frame · 3 rows × 3 variables

      extra   group               id
  1     4.4   laevo-hyoscyamine    6
  2     4.6   laevo-hyoscyamine    9
  3     5.5   laevo-hyoscyamine    7
