# correlation

## Summary
Correlation matrix, N per pair, optional p-values; Kendall is tau-b

## In brief
- `d.correlation([wage, educ, hours])`: Pearson's correlation of every pair, as a square table.
- `method: spearman` and `method: kendall`: the rank correlations.
- `p_values: true`: a second table, of the two-sided p-value of each pair.
- `missing: listwise`: only the rows that are complete in every variable named.

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

## Missing values
With `missing: pairwise`, the default, each pair is computed on the rows where both of its variables have a value, so the pairs may rest on different rows; the table `N per pair` then shows how many. With `missing: listwise` a row with `NA` in any variable named is left out of every pair. No note is printed: the N under the title, or the table of N, says what was used.

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

Pearson's coefficient multiplies deviations, so a variable with a value above 1e150 in size is
refused under `method: pearson` (Z0803), where R gives a coefficient of 0; divide the variable
by a power of ten first, which changes no coefficient. The rank methods read the order of the
values and take any size.

## A common mistake
Naming two variables one after the other, as `cor(x, y)` in R. The variables go in one list, and the place after it is `method:`, so the second name is read as a method:

> dataset(wages).correlation(wage, educ)
error[Z0402]: `educ` is not one of the allowed words

Write the variables as a list: `correlation([wage, educ])`.

## Coming from R or Stata
| d.correlation([x, y, z]) | cor(d[, c("x", "y", "z")], use = "pairwise.complete.obs") | pwcorr x y z |
| d.correlation([x, y, z], missing: listwise) | cor(d[, c("x", "y", "z")], use = "complete.obs") | correlate x y z |
| d.correlation([x, y, z], method: spearman) | cor(d[, c("x", "y", "z")], method = "spearman", use = "pairwise.complete.obs") | spearman x y z, pw |
| d.correlation([x, y], method: kendall) | cor(d$x, d$y, method = "kendall", use = "complete.obs") | ktau x y |
| d.correlation([x, y], p_values: true) | cor.test(d$x, d$y) | pwcorr x y, sig |

R's `cor` gives `NA` for a pair as soon as one of its variables has a missing value, unless
`use =` is written; here the complete pairs are used without asking. `cor.test` takes one pair
at a time. Stata's `ktau` prints tau-a beside tau-b; the coefficient here is tau-b.

## Related tasks
- For the covariances, in the units of the variables, use `covariance`.
- For the line fitted through two variables, with its slope, use `reg`.
- To see two variables against each other, use `scatter`.
- For the association of two categories, use `chi2_test`.
- For the count of missing values in each variable, use `describe`.

## References
- Kendall, M. G. (1970). *Rank Correlation Methods*, 4th edition. Griffin.
- Best, D. J. and Roberts, D. E. (1975). Algorithm AS 89: the upper tail probabilities of
  Spearman's rho. *Applied Statistics*, 24, 377–379.
- Hollander, M., Wolfe, D. A. and Chicken, E. (2014). *Nonparametric Statistical Methods*, 3rd
  edition, chapter 8. Wiley.

## See also
covariance, mean, sd, describe

## Examples
// Pearson's correlations of wage, schooling and hours in one year. The variables have different rows missing, so the table of N follows.
> let w = dataset(wages).where(year == 2019)
> w.correlation([wage, educ, hours])
correlation · w · pearson

           wage     educ    hours
  wage    1.000    0.304    0.006
  educ    0.304    1.000   -0.027
  hours   0.006   -0.027    1.000

N per pair

           wage    educ   hours
  wage    1,492   1,492   1,489
  educ    1,492   1,518   1,515
  hours   1,489   1,515   1,515

// The same on the rows complete in all three, with `missing: listwise`: one N for the table.
> w.correlation([wage, educ, hours], missing: listwise)
correlation · w · pearson
N = 1,489

           wage     educ    hours
  wage    1.000    0.303    0.006
  educ    0.303    1.000   -0.027
  hours   0.006   -0.027    1.000

// Spearman's rank correlations, with the p-value of each pair.
> w.correlation([wage, educ, exper], method: spearman, p_values: true)
correlation · w · spearman

           wage     educ    exper
  wage    1.000    0.298    0.075
  educ    0.298    1.000   -0.341
  exper   0.075   -0.341    1.000

p-values

            wage     educ    exper
  wage             <0.001    0.004
  educ    <0.001            <0.001
  exper    0.004   <0.001

N per pair

           wage    educ   exper
  wage    1,492   1,492   1,490
  educ    1,492   1,518   1,516
  exper   1,490   1,516   1,516

// Kendall's tau-b of two short variables, where the p-value is exact.
> let f = frame(x = [1, 2, 3, 4, 5, 6, 7, 8], y = [2, 1, 4, 3, 7, 5, 8, 6], z = [8, 7, 6, 5, 4, 3, 2, 1])
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

// Pearson's coefficient written out, with the two defaults named.
> f.correlation([x, y, z], method: pearson, missing: pairwise, p_values: false)
correlation · f · pearson
N = 8

           x        y        z
  x    1.000    0.833   -1.000
  y    0.833    1.000   -0.833
  z   -1.000   -0.833    1.000
