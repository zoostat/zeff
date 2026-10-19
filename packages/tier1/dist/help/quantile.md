# quantile

## Summary
Value with cumulative probability p

## Description
The quantile of a distribution at probability `p`: the smallest x with P(X ≤ x) ≥ p, which for a
continuous distribution is the x with P(X ≤ x) = p exactly. `quantile(0.975)` of the standard
normal is the familiar 1.96. Given a column, `quantile` applies row by row and returns a column.
`NA` gives `NA`; a `p` outside [0, 1] is refused (Z0413). Like R's `q` functions and Stata's
`invnormal()` and its kin. The sample quantile of a variable is the frame method of the same name,
`wages.quantile(wage, p: 0.9)`.

## Arguments
- distribution: The distribution, a `Distribution` value such as `dist.normal()`.
- p: A probability between 0 and 1, or a column of them.

## Value
A Float; a column of Floats when `p` is a column. At `p` = 0 or 1 the end of the support, `-Inf`
or `Inf` for an unbounded one.

## Details
Each family inverts its own distribution function: in closed form where one exists (normal
through the inverse of the error function, exponential, uniform, log-normal) and by the inverse
incomplete beta and gamma functions of the kernel for the t, F, chi-square, gamma and beta
families, to full double precision on every target.

## See also
pdf, cdf, draw

## References
- The help page of each family (`zeff help dist.normal`, …) for its formula and reference.

## Examples
> dist.normal().quantile(0.975)
1.95996
> dist.normal(mean: 100, sd: 15).quantile(0.9)
119.223
> dist.exponential(rate: 2).quantile(0.5)
0.346574
> dist.uniform(min: 2, max: 5).quantile(0.25)
2.75
