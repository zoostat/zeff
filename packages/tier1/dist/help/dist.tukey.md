# dist.tukey

## Summary
Studentized range distribution, for Tukey's honest significant difference

## Description
A `Distribution` value for the studentized range of `groups:` means on `df:` degrees of freedom:
the largest difference among `groups:` independent standard normal values, divided by an
independent estimate of their standard deviation that has `df:` degrees of freedom. It is the
reference distribution of Tukey's honest significant difference. When each of k group means is the
mean of n observations and s² is the within-group mean square on ν degrees of freedom, the largest
of the differences |ȳᵢ − ȳⱼ| / (s/√n) follows `dist.tukey(groups: k, df: ν)` if the population
means are equal. Like R's `ptukey` and `qtukey` and Stata's `tukeyprob()` and `invtukeyprob()`.

## Arguments
- groups: The number of means compared, a whole number of 2 or more. Anything else is refused
  (Z0413).
- df: The degrees of freedom of the estimate of the standard deviation, a number of 1 or more (not
  necessarily whole). A number below 1, `Inf` or `NA` is refused (Z0413).

## Value
A `Distribution`. Its methods: `pdf(x)`, `cdf(x, upper:)`, `quantile(p)` and `draw(n, seed:)`.

## Details
The range W of k standard normal values has P(W ≤ w) = k ∫ φ(z) [Φ(z) − Φ(z − w)]^(k−1) dz. The
studentized range is Q = W/S, where νS² is an independent chi-squared variable on ν degrees of
freedom, so that P(Q ≤ q) = ∫ P(W ≤ qs) f(s) ds over the density f of S. Both integrals are
computed in logarithms by the trapezoidal rule over the whole line, the outer one in the variable
ln s; for integrands of this kind the rule gains digits geometrically as its step shrinks, and the
step is divided by three until three interleaved sums agree. The upper tail has its own integral,
k ∫ φ(z) Φ(z)^(k−1) {1 − [1 − Φ(z − w)/Φ(z)]^(k−1)} dz, and so has the density, so that a far tail
such as `cdf(50, upper: true)` keeps its relative accuracy and `pdf` is not a difference of two
probabilities. The quantile solves the distribution function on its smaller tail.

The values agree with an exact decimal reference to a relative 1e-12 in both tails and in the
density, from 2 to 200 groups, from 1 degree of freedom to a million million, and down to tails of
1e-98. R's `ptukey` sums fixed Gauss–Legendre rules: against the same reference it is out by up
to 5e-3 on 2 degrees of freedom, 3e-5 on 5 and 1e-7 on 20 and on 120 (the worst cases have 100
groups), and by 2e-6 on a million, where it uses the range of normal values instead; far out it
keeps few digits (the tail above 50 for 10 groups on 5 degrees of freedom comes out at twice its
size). `qtukey` stops its search at about four digits. Zeff's values therefore differ from R's
beyond those digits. R and Stata refuse fewer than 2 degrees of freedom; Zeff accepts 1 or more.
A `df:` of 1e30 or more gives the range of the normal values itself, the case of a known standard
deviation. R's `nranges` (the largest of several independent ranges) is not offered (open
decision #102).

The p-value of Tukey's test is the upper tail, `dist.tukey(groups: 3, df: 20).cdf(3.2, upper:
true)`, and the 5% critical value is `quantile(0.95)`. Every value costs a double integral, so
`draw`, which inverts the distribution function once for each value drawn, is slow for a large `n`.

## See also
dist.t, dist.f, dist.normal, posthoc

## References
- Tukey, J. W. (1949). Comparing individual means in the analysis of variance. *Biometrics*, 5,
  99–114.
- Harter, H. L. (1960). Tables of range and studentized range. *Annals of Mathematical
  Statistics*, 31, 1122–1147.
- Copenhaver, M. D. and Holland, B. (1988). Computation of the distribution of the maximum
  studentized range statistic with application to multiple significance testing of simple effects.
  *Journal of Statistical Computation and Simulation*, 30, 1–15.
- Trefethen, L. N. and Weideman, J. A. C. (2014). The exponentially convergent trapezoidal rule.
  *SIAM Review*, 56, 385–458.

## Examples
> let q = dist.tukey(groups: 3, df: 20)
> q
tukey distribution · groups: 3, df: 20
> q.quantile(0.95)
3.57793
> q.cdf(3.2, upper: true)
0.0848265
> dist.tukey(groups: 10, df: 5).cdf(50, upper: true)
4.71352e-06
