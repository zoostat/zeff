# shapiro_test

## Summary
Shapiro–Wilk test of normality

## Description
Whether a variable could have been drawn from a normally distributed population, judged from
how closely its ordered values follow the expected order statistics of a normal sample. It prints
the hypotheses, the number of values, the statistic W with its p-value, and a decision at
α = 0.05. Rows with `NA` are dropped and counted in a note. The test takes 3 to 5,000 values:
more are refused (Z0421), since the approximation that gives the p-value is not defined beyond
that size; in a sample that large the test rejects normality for departures too small to matter,
so `qq_plot` is the better guide, and `ks_test` tests the distribution.

## Arguments
- data: The frame.
- var: The numeric variable tested, a Bool, Int or Float variable.

## Value
A `TestResult`, with the result interface (`summary`, `p_values`, `n_obs`, `export`, `report`): the
statistic W, the p-value and the hypotheses; W has no degrees of freedom. Its own methods
`statistic()`, `df()`, `effect_size()`, `h0()`, `h1()` and `alternative()` give the figures as
values, `df()` and `effect_size()` being `NA`.

## Details
W is the squared correlation between the ordered values and the coefficients a₁ … a_n, which
approximate the normalised expected order statistics of a standard normal sample by Royston's
(1992) method: W = (Σ a_i x_(i))² / Σ (x_i − x̄)², so W lies in (0, 1] and is 1 for a sample
that is exactly a linear function of those coefficients. The p-value follows Royston's (1995)
algorithm AS R94, as R's `shapiro.test` and Stata's `swilk` compute it: exact for three values;
for 4 to 11, a transformation of log(1 − W) with constants in n; for 12 to 5,000, with constants
in log n; the transformed value is referred to the standard normal distribution. Ties are not
corrected for, as they are not in R. A variable whose values are all the same, or one with fewer
than three values after dropping `NA`, is an error.

## References
Shapiro and Wilk (1965), An analysis of variance test for normality (complete samples),
Biometrika 52, 591–611.
Royston (1992), Approximating the Shapiro–Wilk W-test for non-normality, Statistics and
Computing 2, 117–119.
Royston (1995), Remark AS R94: a remark on algorithm AS 181: the W-test for normality, Applied
Statistics 44, 547–551.

## See also
ks_test, qq_plot, t_test

## Examples
> let g = frame(s = [2.9, 3.0, 2.5, 2.6, 3.2, 3.8, 2.7, 4.0, 2.4, 2.8, 3.4, 3.7, 2.2, 2.0])
> g.shapiro_test(s)
shapiro_test · s
  H0: s was drawn from a normally distributed population
  H1: s was not drawn from a normally distributed population

N = 14
W = 0.9665   p = 0.826
decision at α = 0.05: fail to reject H0 (p = 0.826)
p-value from Royston's (1995) normal approximation of W
> let wages = dataset(wages).where(year == 2010).add(lwage = log(wage))
> wages.shapiro_test(lwage)
shapiro_test · lwage
  H0: lwage was drawn from a normally distributed population
  H1: lwage was not drawn from a normally distributed population

N = 531
W = 0.9980   p = 0.795
decision at α = 0.05: fail to reject H0 (p = 0.795)
p-value from Royston's (1995) normal approximation of W
note: 6 rows with NA in lwage dropped (line 4)
> wages.shapiro_test(educ).statistic()
0.981607
