# correlation

## Summary
Correlation matrix, N per pair, optional p-values; Kendall is tau-b

## Description
The correlations between two or more numeric variables of a frame, as a square table with the
variables as rows and columns, three decimals. `method:` chooses Pearson's product-moment
correlation (the default), Spearman's rank correlation or Kendall's tau-b. `missing:` says how
rows with `NA` are dropped: `pairwise` (the default) uses, for each pair, the rows where both
variables have values; `listwise` keeps only the rows where every variable named has one. The N is
printed under the title when every pair has the same, and as a third table, `N per pair`, when
pairwise deletion leaves them different. `p_values: true` adds a second table of the same shape
with each pair's two-sided p-value under independence, `<0.001` below a thousandth. Like R's
`cor()` and `cor.test()`, and Stata's `pwcorr` (pairwise) and `correlate` (listwise).

## Arguments
- data: The frame.
- vars: Two or more variables, as a list: `[wage, educ, hours]` or a range. Each must be Bool,
  Int or Float (a Bool counts as 0 and 1); another type is refused (Z0401), as is a single
  variable.
- method: `pearson` (the default), `spearman` or `kendall`.
- missing: `pairwise` (the default) or `listwise`.
- p_values: `true` adds the table of p-values; `false` by default.

## Value
A `Table`: the correlations, then the p-values when asked, then the N per pair when it varies.

## Details
Pearson's r is Σ(x − x̄)(y − ȳ) / √(Σ(x − x̄)² Σ(y − ȳ)²) on the complete pairs, with
compensated sums in a fixed order. Spearman's ρ is Pearson's r of the ranks, ties given the
average rank. Kendall's τ-b is (C − D) / √((n₀ − n₁)(n₀ − n₂)) with C and D the concordant and
discordant pairs and n₁, n₂ the tied pairs of each variable (Kendall 1970). A variable that does
not vary has no correlation: its cells are `NA` with a warning (W0205). The p-values are those of
R's `cor.test`: for Pearson, t = r √((n − 2)/(1 − r²)) against Student's t with n − 2 degrees of
freedom; for Spearman without ties and n up to 1,290, the exact distribution of Σd² (enumerated
up to n = 9, the Edgeworth series of Best and Roberts (1975) beyond), otherwise the t
approximation; for Kendall without ties and n below 50, the exact distribution of the number of
concordant pairs, otherwise the normal approximation with the tie-corrected variance. Every
estimate and p-value agrees with R to twelve digits across these branches; the tables print three
decimals.

## See also
covariance, mean, sd, describe

## References
- Kendall, M. G. (1970). *Rank Correlation Methods*, 4th edition. Griffin.
- Best, D. J. and Roberts, D. E. (1975). Algorithm AS 89: the upper tail probabilities of
  Spearman's rho. *Applied Statistics*, 24, 377–379.
- Hollander, M., Wolfe, D. A. and Chicken, E. (2014). *Nonparametric Statistical Methods*, 3rd
  edition, chapter 8. Wiley.

## Examples
> let f = frame(x = [1, 2, 3, 4, 5, 6, 7, 8], y = [2, 1, 4, 3, 7, 5, 8, 6], z = [8, 7, 6, 5, 4, 3, 2, 1])
> f.correlation([x, y, z])
correlation · f · pearson
N = 8

           x        y        z
  x    1.000    0.833   -1.000
  y    0.833    1.000   -0.833
  z   -1.000   -0.833    1.000
> f.correlation([x, y], method: kendall, p_values: true)
correlation · f · kendall
N = 8

          x       y
  x   1.000   0.643
  y   0.643   1.000

p-values

          x       y
  x           0.031
  y   0.031
