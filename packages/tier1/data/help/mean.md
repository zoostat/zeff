# mean

## Summary
Mean of a variable

## Description
The arithmetic mean of the non-missing values of `var`. Missing values are skipped with a warning
that gives their count; a column with no value left gives `NA` with the same warning. On a Bool
column the mean is the share of `true` values, the commonest summary of a treatment indicator. An
infinite value gives `Inf` or `-Inf`; `Inf` and `-Inf` together give `NA` with a warning.

Unlike R's `mean`, which needs `na.rm = TRUE`, missing values never make the result `NA` silently:
they are skipped and counted.

## Arguments
- data: The frame.
- var: The variable, which must be Bool, Int or Float.

## Value
A `Float`.

## Details
The sum is a compensated (Neumaier) sum taken in row order, divided by the count and corrected once
by the mean of the deviations, so the result is within a rounding of the exact mean whatever the
magnitude of the values, and the same bits on every target.

## References
Neumaier (1974), Rundungsfehleranalyse einiger Verfahren zur Summation endlicher Summen, ZAMM 54, 39–51.
Higham (2002), Accuracy and Stability of Numerical Algorithms, second edition, chapter 4.

## See also
sd, summary, describe

## Examples
> let wages = import("data/wages.csv")
note: data/wages.csv read as CSV: 3 rows × 3 variables, types guessed; CSV carries no labels, value labels or categories (line 1)
> wages.mean(wage)
24.5333
> wages.mean(hours)
37.5
warning: 1 row with NA in hours was skipped (line 3)
