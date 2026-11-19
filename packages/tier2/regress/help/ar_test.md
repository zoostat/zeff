# ar_test

## Summary
The Anderson–Rubin test and confidence set, valid with weak instruments

## In brief
- `iv.ar_test()`: tests that the coefficient of the instrumented regressor is zero, and gives its 95% confidence set.
- The test and the set hold however weak the instruments are, where the usual interval does not.
- `level: 0.99` asks for another level; the set may be an interval, two rays, every value or empty.

## Description
Tests the coefficient of the instrumented regressor of a model fitted by `ivreg`, in a way that
does not lean on the strength of the instruments. The usual t test and confidence interval of a
two-stage fit are too optimistic when the instruments are weak; this test keeps its level
whatever their strength, at the price of a wider answer when they are weak.

It also gives the confidence set: every value of the coefficient that the test would not
reject. With strong instruments the set is an interval close to the usual one. With weak
instruments it can be unbounded, two rays or every value, which is the honest answer when the
data cannot bound the effect. With more instruments than needed it can be empty, when the
instruments disagree with one another.

With several instrumented regressors the test is of all their coefficients at once, and no
set is given.

## Arguments
- iv: A fit by `ivreg`.
- level: The confidence level of the set, and one less the level of the decision; 0.95 unless given.

## Value
A test result: the hypotheses, the F statistic with its degrees of freedom, the p-value, the
decision, and the confidence set in words. `statistic()`, `df()` and `p_values()` read its
parts.

## Missing values
The test uses the rows the fit used: a row with `NA` in any variable of the model, the
instruments among them, was left out when the model was fitted.

## Details
### The statistic
If the coefficient is b, the outcome less b times the regressor is unrelated to the
instruments. The statistic at b is therefore the test of the instruments outside the equation
in the regression of that difference on all the instruments: an F under classical errors, the
Wald statistic over the number of those instruments under robust or clustered errors. It is
referred to F on the number of instruments outside the equation and N less all the
instruments, or the number of clusters less one. The test printed is the statistic at zero.

### The confidence set
The set holds the values b at which the statistic is not above its critical value. Under
classical errors, and with one instrument under any errors, its ends are the roots of a
quadratic, found exactly; otherwise they are found by a search over the statistic.

### Reading the shapes
An interval is the usual case. Two rays, every value below one number and every value above
another, or every value at all, arise exactly when the first-stage F is under the critical
value: the instruments are then too weak to rule out large effects of either sign. An empty
set arises when no value agrees with all the instruments at once, so it is evidence against
the instruments, not for a precise estimate. Under robust or clustered errors with several
instruments the set can have more than two ends, and each piece is printed.

### Fixed effects, weights and clusters
They are the fit's: the effects are taken out of the outcome, the regressor and the
instruments, and the variance is of the kind the fit was given. With clustered errors the test
needs more clusters than instruments outside the equation; with as many or fewer its variance
cannot be inverted and the test is refused (Z0802).

### Exactness
Closed form in double precision where the set is a quadratic; otherwise the ends are found to
a relative 1e-13 of their size. The sets agree with R to a relative 1e-6 on the reference
models.

## A common mistake
Asking for the test of a model fitted by `reg`, which has no instruments:

> dataset(staff).reg(productivity ~ training).ar_test()
error[Z0401]: `ar_test` cannot be called on a RegFit
  = help: `ar_test` takes an IvFit first; `x.ar_test(...)` is `ar_test(x, ...)`

Fit the model with `ivreg` and its `instruments:` first.

## Coming from R or Stata
| iv.ar_test() | ivmodel::AR.test(ivmodel(Y, D, Z, X)) | weakiv after ivregress or ivreg2 |
| iv.ar_test(level: 0.99) | AR.test(m, alpha = 0.01) | weakiv, level(99) |

`ivmodel` and `weakiv` print the same test and set under classical errors; under robust or
clustered errors their statistics use their own small-sample factors and can differ in the
third digit. Both also offer the conditional likelihood ratio test, which is not offered here
before release 1.18.

## Related tasks
- To fit the model this test belongs to, use `ivreg`.
- To see how strong the instruments are and whether they agree, use `first_stage`.
- To test a hypothesis about any coefficient by the usual Wald test, use `test`.
- To read the usual confidence interval of the coefficient, use `ci`.

## References
Anderson, T. W. and Rubin, H. (1949), Estimation of the parameters of a single equation in a
complete system of stochastic equations, Annals of Mathematical Statistics 20(1), 46–63.

Dufour, J.-M. and Taamouti, M. (2005), Projection-based statistical inference in linear
structural models with possibly weak instruments, Econometrica 73(4), 1351–1365.

Andrews, I., Stock, J. H. and Sun, L. (2019), Weak instruments in instrumental variables
regression: theory and practice, Annual Review of Economics 11, 727–753.

## See also
ivreg, first_stage, test, ci

## Examples
// A strong instrument: the set is an interval close to the usual one.
> let staff = dataset(staff)
> let iv = staff.ivreg(productivity ~ training, fe: site, instruments: training ~ offer)
> iv.ar_test()
ar_test · iv · training
  H0: the coefficient of training is zero          β = 0
  H1: the coefficient of training is not zero      β ≠ 0
F(1, 2,387) = 90.24   p < 0.001
decision at α = 0.05: reject H0 (p < 0.001)
95% confidence set for the coefficient of training: from 3.7492 to 5.4380
Anderson–Rubin test on the instrument offer; valid however weak the instruments

// A weak instrument, the distance alone on the first 300 employees, at the 99% level: the set is every value.
> staff.where(employee <= 300).ivreg(productivity ~ training, instruments: training ~ distance).ar_test(level: 0.99)
ar_test · productivity ~ training · training
  H0: the coefficient of training is zero          β = 0
  H1: the coefficient of training is not zero      β ≠ 0
F(1, 298) = 2.65   p = 0.105
decision at α = 0.01: fail to reject H0 (p = 0.105)
99% confidence set for the coefficient of training: every value; the instruments are too weak to say anything about it
Anderson–Rubin test on the instrument distance; valid however weak the instruments
warning[W1007]: weak instruments: the Cragg–Donald F is 0.25, under 10 (line 4)
  = help: with weak instruments the estimate leans towards the least-squares one and its confidence interval is too narrow; `ar_test()` gives a test and a confidence set that hold however weak the instruments are
