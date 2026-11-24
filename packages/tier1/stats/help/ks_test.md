# ks_test

## Summary
Kolmogorov–Smirnov test of a variable against a distribution

## In brief
- `d.ks_test(x, dist.normal(mean: 3, sd: 0.5))`: a variable against a distribution whose parameters are given.
- `d.ks_test(x, dist.normal(), estimated: true)`: against the normal family, the mean and standard deviation taken from the data (Lilliefors' test).

## Description
Whether a variable could have been drawn from a given distribution, judged from the largest
distance D between the variable's empirical distribution function and the distribution's. The
distribution is a `dist` value with its parameters, such as `dist.normal(mean: 3, sd: 0.5)` or
`dist.exponential(rate: 2)`; a truncated or mixed distribution is accepted too. `estimated: true`
is Lilliefors' form of the test for the normal family alone: the mean and standard deviation are
taken from the data, which the ordinary p-value does not allow for, and the p-value follows
Dallal and Wilkinson's approximation; any other family with `estimated: true` is an error
(Z0422). The test prints the hypotheses, the number of values (with the estimates where they are
taken from the data), D with its p-value, a decision at α = 0.05 and a line saying how the
p-value was computed. Rows with `NA` are dropped and counted in a note. Tied values are warned
about (W0212), since the test's distribution theory assumes a continuous distribution.

## Arguments
- data: The frame.
- var: The numeric variable tested, a Bool, Int or Float variable.
- distribution: The distribution compared with, a `dist` value with its parameters.
- estimated: `true` takes the mean and standard deviation of the normal family from the data and
  applies Lilliefors' correction; the parameters written in the distribution are then ignored.
  Default `false`.

## Value
A `TestResult`, with the result interface (`summary`, `p_values`, `n_obs`, `export`, `report`): the
statistic D, the p-value and the hypotheses; D has no degrees of freedom. Its own methods
`statistic()`, `df()`, `effect_size()`, `h0()`, `h1()` and `alternative()` give the figures as
values, `df()` and `effect_size()` being `NA`.

## Missing values
A row with `NA` in the variable is left out, and a note gives the count.

## Details
With the values sorted, D = max over i of the larger of F(x₍ᵢ₎) − (i − 1)/n and i/n − F(x₍ᵢ₎),
where F is the distribution function given, as R's `ks.test` and Stata's `ksmirnov` compute it.
Below 100 values without ties the p-value is exact, P(D ≥ d) from the distribution of D by
Marsaglia, Tsang and Wang's (2003) method; otherwise it comes from the limiting Kolmogorov
distribution of √n D, 2 Σ (−1)^(k−1) exp(−2k²x²), summed to convergence (R stops the sum at a
tolerance of 1e-6, so its p-value can differ in the fourth decimal). With ties the limiting
distribution is used whatever the size, as R does. With `estimated: true` the distribution is
the normal with the sample mean and standard deviation (divisor n − 1), and the p-value is Dallal
and Wilkinson's (1986) approximation, with Stephens' (1974) form where it exceeds 0.1, as
`nortest`'s `lillie.test` computes it; at least five values are needed. For a discrete
distribution the test is conservative, since F jumps; `chi2_test` with `p:` compares frequencies
exactly.

## A common mistake
Asking for estimated parameters with a family other than the normal. Lilliefors' correction is worked out for the normal alone:

> dataset(staff).ks_test(tenure, dist.exponential(rate: 0.2), estimated: true)
error[Z0422]: `estimated: true` of `ks_test` is defined for the normal distribution alone, but the distribution is the exponential distribution · rate: 0.2
  = help: give the distribution with its parameters and drop `estimated:`; a test of normality with the parameters estimated is also `shapiro_test`

Give the distribution with its parameters and leave `estimated:` out. The p-value then supposes that the parameters were not chosen from the same data.

## Coming from R or Stata
| d.ks_test(y, dist.normal(mean: 3, sd: 0.5)) | ks.test(d$y, "pnorm", 3, 0.5) | ksmirnov y = normal((y - 3) / 0.5) |
| d.ks_test(y, dist.exponential(rate: 2)) | ks.test(d$y, "pexp", 2) | ksmirnov y = 1 - exp(-2 * y) |
| d.ks_test(y, dist.normal(), estimated: true) | nortest::lillie.test(d$y) | (none of its own) |

In R and Stata the distribution is named by its distribution function; here it is a `dist`
value, so any family, truncated or mixed, is written in one way. Stata's `ksmirnov` prints the
two one-sided distances with the combined one, which is D. The comparison of two samples with
each other, `ks.test(x, y)` in R and `ksmirnov y, by(g)` in Stata, is not given.

## Related tasks
- For a test of normality in a small sample, use `shapiro_test`.
- To see where a variable departs from a distribution, use `qq_plot`.
- For the fit of counts in categories to given proportions, use `chi2_test`.
- To write the distribution compared with, use `dist.normal`.

## References
Kolmogorov (1933), Sulla determinazione empirica di una legge di distribuzione, Giornale
dell'Istituto Italiano degli Attuari 4, 83–91.
Marsaglia, Tsang and Wang (2003), Evaluating Kolmogorov's distribution, Journal of Statistical
Software 8(18), 1–4.
Lilliefors (1967), On the Kolmogorov–Smirnov test for normality with mean and variance unknown,
Journal of the American Statistical Association 62, 399–402.
Dallal and Wilkinson (1986), An analytic approximation to the distribution of Lilliefors's test
statistic for normality, The American Statistician 40, 294–296.

## See also
shapiro_test, qq_plot, chi2_test, dist.normal

## Examples
// Eight values against the uniform distribution on 0 to 1.
> let g = frame(u = [0.11, 0.32, 0.45, 0.58, 0.63, 0.77, 0.84, 0.92])
> g.ks_test(u, dist.uniform())
ks_test · u · uniform distribution · min: 0, max: 1
  H0: u was drawn from the uniform distribution with min 0 and max 1
  H1: u was not drawn from the uniform distribution with min 0 and max 1

N = 8
D = 0.2050   p = 0.827
decision at α = 0.05: fail to reject H0 (p = 0.827)
exact p-value from the distribution of D

// The log wages of one year against a normal distribution with its parameters given.
> let wages = dataset(wages).where(year == 2010).add(lwage = log(wage))
> wages.ks_test(lwage, dist.normal(mean: 3, sd: 0.5))
ks_test · lwage · normal distribution · mean: 3, sd: 0.5
  H0: lwage was drawn from the normal distribution with mean 3 and sd 0.5
  H1: lwage was not drawn from the normal distribution with mean 3 and sd 0.5

N = 531
D = 0.0296   p = 0.742
decision at α = 0.05: fail to reject H0 (p = 0.742)
asymptotic p-value from the limiting Kolmogorov distribution, with ties present
note: 6 rows with NA in lwage dropped (line 4)
warning[W0212]: 64 of the 531 values of `lwage` tie with another; the Kolmogorov–Smirnov test assumes a continuous distribution, so its p-value is approximate (line 4)
  = help: the asymptotic distribution is used; for a discrete variable, `chi2_test` with `p:` compares the frequencies instead

// The same with the mean and standard deviation taken from the data: Lilliefors' test.
> wages.ks_test(lwage, dist.normal(), estimated: true)
ks_test · lwage · normal family, parameters estimated
  H0: lwage was drawn from a normal distribution with unspecified parameters
  H1: lwage was not drawn from any normal distribution

N = 531   mean = 2.987   sd = 0.498   (estimated from the data)
D = 0.0259   p = 0.523
decision at α = 0.05: fail to reject H0 (p = 0.523)
p-value by Dallal and Wilkinson's (1986) approximation of Lilliefors' distribution
note: 6 rows with NA in lwage dropped (line 5)
warning[W0212]: 64 of the 531 values of `lwage` tie with another; the Kolmogorov–Smirnov test assumes a continuous distribution, so its p-value is approximate (line 5)
  = help: Lilliefors' approximation assumes distinct values, so read the p-value as a guide; for a discrete variable, `chi2_test` with `p:` compares the frequencies instead

// Against another family, with the default of `estimated:` written out: ten waiting times and an exponential distribution.
> let q = frame(wait = [0.3, 1.2, 0.7, 2.9, 0.1, 1.8, 4.4, 0.5, 2.2, 0.9])
> q.ks_test(wait, dist.exponential(rate: 0.7), estimated: false)
ks_test · wait · exponential distribution · rate: 0.7
  H0: wait was drawn from the exponential distribution with rate 0.7
  H1: wait was not drawn from the exponential distribution with rate 0.7

N = 10
D = 0.1163   p = 0.996
decision at α = 0.05: fail to reject H0 (p = 0.996)
exact p-value from the distribution of D
