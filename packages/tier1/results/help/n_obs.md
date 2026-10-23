# n_obs

## Summary
The observations a result rests on

## Description
Gives the number of observations a result was computed from, after the rows with `NA` in a model
variable were dropped: the rows used, or the sum of the frequency weights under them. Like `nobs`
in R and `e(N)` in Stata.

## Arguments
- fit: The result.

## Value
An `Int`.

## See also
summary, estimates, describe

## Examples
> let m = dataset(wages).reg(log(wage) ~ educ + exper + female)
> m.n_obs()
18179
