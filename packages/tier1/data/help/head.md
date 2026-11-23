# head

## Summary
First n rows

## Description
The first `n` rows of a frame, 10 by default, as a new frame with the same variables; every row
when the frame has fewer. Nothing is copied: the new frame shares its parent's columns. It is the
quick look at a file after `import`, beside `describe`.

## Arguments
- data: The frame.
- n: How many rows, an Int of 0 or more; 10 by default.

## Value
A `Frame` with the same variables and at most `n` rows; the metadata of every variable and of the
frame is kept.

## Missing values
Rows are taken by their position, whatever they hold.

## A common mistake
Asking for all but the last rows with a negative count, as R's `head(d, -1)` allows. A count of rows is 0 or more:

> dataset(sleep).head(n: -1)
error[Z0401]: `n:` of `head` cannot be -1
  = help: write how many rows to show, 0 or more: `head(n: 5)`

Count the rows instead: `d.head(n: d.n_rows() - 1)` is all but the last.

## Coming from R or Stata
| d.head() | head(d, 10) | list in 1/10 |
| d.head(n: 5) | head(d, 5) | list in 1/5 |

R's `head` shows 6 rows unless told; here 10, the rows a frame prints. Stata's `list in` prints
the rows; `head` gives them as a frame, which prints and can be used further.

## Related tasks
- For the last rows, use `tail`.
- For the rows that meet a condition, use `where`.
- For the rows with the largest or the smallest values, sort first with `sort`.
- For rows drawn at random, use `sample`.

## See also
tail, describe, where

## Examples
// The first three rows.
> dataset(firms).head(3)
frame · 3 rows × 4 variables

      firm   industry      size   founded
  1      1   Agriculture      3   1990-11-06
  2      2   Agriculture      5   1957-07-02
  3      3   Agriculture      3   1998-11-07

// Ten rows unless told.
> dataset(sleep).head()
frame · 10 rows × 3 variables

       extra   group                id
   1     0.7   dextro-hyoscyamine    1
   2    -1.6   dextro-hyoscyamine    2
   3    -0.2   dextro-hyoscyamine    3
   4    -1.2   dextro-hyoscyamine    4
   5    -0.1   dextro-hyoscyamine    5
   6     3.4   dextro-hyoscyamine    6
   7     3.7   dextro-hyoscyamine    7
   8     0.8   dextro-hyoscyamine    8
   9       0   dextro-hyoscyamine    9
  10       2   dextro-hyoscyamine   10

// After a sort: the three largest firms.
> dataset(firms).sort(size, desc: true).head(n: 3)
frame · 3 rows × 4 variables

      firm   industry        size   founded
  1     95   Manufacturing     39   1959-06-10
  2    222   Services          39   1967-05-29
  3    235   Services          39   1950-01-17

// No rows, with the variables kept.
> dataset(firms).head(n: 0).n_rows()
0
