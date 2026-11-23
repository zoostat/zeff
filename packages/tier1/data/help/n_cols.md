# n_cols

## Summary
Number of variables

## Description
The number of variables (columns) of a frame, as an `Int`. It counts every variable whatever its
type, and it is the length of the list that `variables` gives.

## Arguments
- data: The frame.

## Value
An `Int`.

## Missing values
A variable is counted whatever it holds, one of nothing but `NA` included.

## Coming from R or Stata
| d.n_cols() | ncol(d) | display c(k) |

Stata keeps the number of variables in memory as `c(k)`, and `describe` prints it.

## Related tasks
- For the number of rows, use `n_rows`.
- For the names of the variables, use `variables`.
- For the type of each variable, use `describe`.

## See also
n_rows, variables, describe

## Examples
// The variables of the shipped wages.
> let wages = dataset(wages)
> wages.n_cols()
11

// After keeping some.
> wages.select([worker, year, wage]).n_cols()
3

// It is the length of the list of names.
> len(wages.variables())
11
