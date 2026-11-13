# anova

## Summary
Analysis of variance: one-way across groups, or factorial from a formula

## Description
Two forms under one name, chosen by the first argument. With a variable and `group:` it is the
one-way analysis: whether the population mean of a numeric variable is the same in every group.
With a formula, `anova(y ~ a * b)`, it is the factorial analysis: a table of sums of squares
with one row and one F test for each term of a linear model.

The one-way form gives Welch's F by default, as R's `oneway.test`: it does not assume that the
groups share a variance, and its second degrees of freedom are fractional. `equal_var: true`
gives the classical F on the pooled within-group variance, as R's `aov` and Stata's `oneway`,
with its table of sums of squares. Either prints the hypotheses, a table of the groups with their
counts, means and standard deviations, F with its degrees of freedom and the p-value, an effect
size, a decision at α = 0.05, and a line saying how F was computed. Two groups are allowed;
with two, Welch's F is the square of Welch's t from `t_test`, with the same p-value.

The factorial form takes the terms of section 7.2 of the Language Reference: categories, their
interactions (`a * b` is `a + b + a:b`), and numeric covariates, alone or in an interaction, so
`y ~ a + x` is the analysis of covariance. A term is a category when its variable is a Category
or is written `category(x)`; an Int, Float or Bool variable enters as a slope, and a text
variable is refused until it is written `category(x)`. `type:` says how each term is adjusted
for the others: type 2, the default, as `car::Anova` in R; type 3, as `car::Anova(type = 3)`
with sum contrasts and as Stata's `anova`; type 1, the sequential table of R's
`anova(lm(...))` and Stata's `anova, sequential`. The table has a row for the whole model, a
row for each term with its degrees of freedom, sum of squares, mean square, F, p-value and
partial effect size, the residual and the total. The variance is always pooled.

`effect:` chooses the effect size of either form. Eta squared, the default, is the share of the
sample's variation that the groups or the term account for; it overstates the share in the
population, the more so the fewer the observations. `effect: omega` gives omega squared and
`effect: epsilon` gives epsilon squared, which take the part expected by chance out of it. In
the factorial form each is the partial measure of its term. R gives the two in
`effectsize::omega_squared()` and `epsilon_squared()`, Stata in `estat esize, omega` and
`estat esize, epsilon` after `anova`. Stata's omega squared is another figure, computed from
the F of the term alone, and `effect: omega_f` gives that one, as R's
`effectsize::F_to_omega2()` does: use `omega` to agree with R's `omega_squared()` and `omega_f`
to agree with Stata's `estat esize, omega`. Details gives both formulas and says where they
part.

In both forms rows with `NA` in a variable used are dropped and counted in a note.

## Arguments
- data: The frame.
- var: The numeric variable whose means are compared; or a formula, `y ~ a * b + x`, which
  selects the factorial form.
- group: The variable whose groups are compared in the one-way form, a Bool, Int, Str or
  Category variable; a group's label is its value label where it has one and `var=value`
  otherwise. With a formula the groups are its terms, and `group:` is an error.
- equal_var: `false` for Welch's F, `true` for the classical F on the pooled variance. It
  belongs to the one-way form; the table of a formula always pools the variance.
- weights: Frequency weights, each row counted that many times, or analytic weights; sampling
  weights are refused by the shared rule (Z0411). In the one-way form analytic weights scale
  each group's variance; in the factorial form they give the weighted least-squares table, the
  weights scaled to sum to the number of rows.
- type: The sums of squares of the factorial form: 2, the default, 3 or 1. It belongs to the
  form that takes a formula.
- effect: The effect size reported: `eta`, the default, for eta squared, `omega` for omega
  squared, `omega_f` for the omega squared that Stata computes from F, or `epsilon` for epsilon
  squared; in the factorial form the partial measure of each term. It belongs to both forms.

## Value
A `TestResult`, with the result interface (`summary`, `estimates`, `p_values`, `ci`, `n_obs`,
`export`, `report`). In the one-way form: the statistic, its two degrees of freedom, the
p-value, the effect size `effect:` chose, and the hypotheses; its own methods `statistic()`,
`df()`, `effect_size()`, `h0()`, `h1()` and `alternative()` give the figures as values. In the
factorial form `statistic()`, `df()` and `p_values()` give the F of the whole model, its two
degrees of freedom and its p-value, and `estimates()` gives a frame with a row for each term
(its F in `statistic`, its degrees of freedom in `df` and `df2`, its p-value) followed by a row
for each term's partial effect size, named for the measure, as `partial omega squared (dose)`.
One term is read by naming it: `p_values("dose")`, `statistic("dose")`, `df("dose")` and
`effect_size("dose")` give the p-value, the F, the two degrees of freedom and the partial
effect size of that row, a bare name reading as text does and an interaction named as the
table prints it, `"dose:diet"`; a name that is no term of the table is an error (Z0414) that
lists the terms. A table of several terms has no single hypothesis, effect size or interval, so
`h0()`, `h1()`, `effect_size()` without a name and `ci()` are `NA` and no decision line is
printed.

## Details
One-way. With k groups of sizes nᵢ, means mᵢ and variances sᵢ², Welch's F weights each group by
wᵢ = nᵢ/sᵢ²: F = Σ wᵢ(mᵢ − m̃)² / ((k − 1)(1 + 2(k − 2)Λ)), where m̃ is the weighted grand mean,
Λ = Σ (1 − wᵢ/Σw)²/(nᵢ − 1) / (k² − 1), and the degrees of freedom are k − 1 and 1/(3Λ). The
classical F is the mean square between the groups over the mean square within them, on k − 1 and
N − k degrees of freedom. The p-value is the upper tail of `dist.f`, computed directly. Welch's
test needs at least two
values with some variation in every group; the classical test needs more values than groups and
some variation within them. A grouping variable with fewer than two groups among the rows kept
is an error.

Effect sizes, one-way. Eta squared is SS between / SS total, the R² of a regression on the
groups. Omega squared (Hays 1963) is (SS between − (k − 1) MS within) / (SS total + MS within),
and epsilon squared (Kelley 1935) is (SS between − (k − 1) MS within) / SS total, the adjusted
R² of that regression. All three come from the unweighted decomposition under either F, Welch's
included. The numerator of omega and of epsilon squared is (k − 1) MS within × (F − 1) with the
classical F, so both are negative exactly when that F is below 1, and epsilon squared is never
above eta squared nor omega squared farther from zero than epsilon squared. A negative estimate
is printed as it is, with a note, and `effect_size()` returns it; it is read as no evidence of
an effect. R's `effectsize::omega_squared()` and `epsilon_squared()` use the same formulas and
print 0 in its place. No effect size has an interval, since the interval of an F effect size
needs the noncentral F distribution, which `dist` does not have yet.

Factorial. The model is fitted by least squares with an intercept, and each term's sum of
squares is the fall in the residual sum of squares when the term joins a model that already
holds the terms it is adjusted for. Under type 1 those are the terms above it in the table, so
the sums of squares add up to the model's and depend on the order of the terms. Under type 2
they are all the other terms that do not contain it: a main effect is adjusted for the other
main effects but not for its own interactions, which respects marginality. Under type 3 they are
all the other terms, interactions included: a main effect is then the effect averaged over the
levels of the other factors with equal weight, which has a meaning only when every category is
coded by deviations from the mean, so `anova` codes every category that way over the levels
present among the rows used, whatever contrast the variable carries. Types 1 and 2 do not
depend on the coding. On a balanced design the three types give the same table. The rows stand
in the order of the terms' degree, main effects before two-way interactions and so on, in the
written order within a degree, as R orders the terms of a model; under type 1 this is also the
order of adjustment. Every F is the term's mean square over the residual mean square, on the
term's and the residual degrees of freedom, and the row of the model tests all the terms
together. R's `car::Anova(type = 3)` gives the same figures only after
`options(contrasts = c("contr.sum", "contr.poly"))`, and prints a row for the intercept, which
`anova` leaves out.

Effect sizes, factorial. With SS and df the sum of squares and the degrees of freedom of a
term, and N the number of observations, partial eta squared is SS / (SS + SS residual), partial
omega squared is (SS − df × MS residual) / (SS + (N − df) × MS residual), and partial epsilon
squared is (SS − df × MS residual) / (SS + SS residual) (Olejnik and Algina 2003). On the row
of the model they are R², (SS model − df × MS residual) / (SS total + MS residual) and the
adjusted R². With one category and no other term they are the three measures of the classical
one-way form. A row whose F is below 1 has a negative partial omega and epsilon squared, printed
as it is; one note names every such row. None has an interval, for the reason given above. In R,
`effectsize::eta_squared()`, `omega_squared()` and `epsilon_squared()` on the same table give
these figures, a negative one printed as 0. In Stata, after `anova` or `regress`, `estat esize`
gives partial eta squared and `estat esize, epsilon` partial epsilon squared by the same
formulas. `estat esize, omega`, like R's `effectsize::F_to_omega2()`, computes partial omega
squared from the F of the term alone, (F − 1) / (F + (df residual + 1) / df), which is
(SS − df × MS residual) / (SS + (df residual + 1) × MS residual): the same figure on the row of
the model and where the model has one term, since N − df is then df residual + 1, and a figure
farther from zero for a term of a model with several terms, where df residual + 1 is the
smaller number. `effect: omega_f` reports it, under the name "partial omega squared from F";
in the one-way form it is omega squared itself and the line under the table says so. Which to
report is a matter of which program a reader will compare with: `omega` is the definition of
Olejnik and Algina and of R's `omega_squared()`, `omega_f` the figure of Stata.

When a combination of levels has no observations, or a term repeats the others, some columns of
the model are determined by the rest. Types 1 and 2 still give the table: those columns carry
no degree of freedom, a note says how many, and a term left with none keeps its row without a
test. Type 3 is refused (Z0401), as car refuses it, since no term can then be adjusted for every
other. Also refused: a formula without an intercept, since every type adjusts for the grand
mean; a category with one level among the rows used; an outcome that does not vary; a model
with as many estimated columns as observations, or one that reproduces the outcome exactly
(Z0802); a value too large to square (Z0803). A model with no category is allowed, each row
then testing a slope, with a note that `reg` gives the estimates.

## References
Welch (1951), On the comparison of several mean values: an alternative approach, Biometrika 38,
330–336.
Fisher (1925), Statistical Methods for Research Workers, Oliver and Boyd, chapter 7.
Yates (1934), The analysis of multiple classifications with unequal numbers in the different
classes, Journal of the American Statistical Association 29, 51–66.
Nelder (1977), A reformulation of linear models, Journal of the Royal Statistical Society A
140, 48–77.
Fox and Weisberg (2019), An R Companion to Applied Regression, third edition, Sage.
Hays (1963), Statistics for Psychologists, Holt, Rinehart and Winston.
Kelley (1935), An unbiased correlation ratio measure, Proceedings of the National Academy of
Sciences 21, 554–559.
Olejnik and Algina (2003), Generalized eta and omega squared statistics: measures of effect
size for some common research designs, Psychological Methods 8, 434–447.

## See also
t_test, kruskal_test, reg, tabulate, posthoc, p_adjust

## Examples
> let g = frame(s = [2.9, 3.0, 2.5, 2.6, 3.2, 3.8, 2.7, 4.0, 2.4, 2.8, 3.4, 3.7, 2.2, 2.0], site = [1, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 3])
> g.anova(s, group: site)
anova · Welch one-way · s by site
  H0: the population means of s are equal in all 3 groups of site          μ₁ = μ₂ = μ₃
  H1: at least one group of site has a different population mean of s      not all equal

  group    N    mean      sd
  site=1   5   2.840   0.288
  site=2   4   3.225   0.793
  site=3   5   2.820   0.736

F(2, 5.5) = 0.39   p = 0.693
eta squared = 0.092
decision at α = 0.05: fail to reject H0 (p = 0.693)
Welch's F over 3 groups, the variances not assumed equal; eta squared = SS between / SS total
> g.anova(s, group: site, equal_var: true)
anova · classical one-way · s by site
  H0: the population means of s are equal in all 3 groups of site          μ₁ = μ₂ = μ₃
  H1: at least one group of site has a different population mean of s      not all equal

  group    N    mean      sd
  site=1   5   2.840   0.288
  site=2   4   3.225   0.793
  site=3   5   2.820   0.736

  source           df      SS      MS
  between groups    2   0.447   0.223
  within groups    11   4.388   0.399
  ───────────────────────────────────
  total            13   4.834

F(2, 11) = 0.56   p = 0.587
eta squared = 0.092
decision at α = 0.05: fail to reject H0 (p = 0.587)
classical F over 3 groups on the pooled within-group variance; eta squared = SS between / SS total
> let wages = dataset(wages).add(lwage = log(wage))
> wages.anova(lwage, group: industry)
anova · Welch one-way · lwage by industry
  H0: the population means of lwage are equal in all 12 groups of industry         μ₁ = μ₂ = … = μ₁₂
  H1: at least one group of industry has a different population mean of lwage      not all equal

  group                       N    mean      sd
  Agriculture               528   3.004   0.460
  Mining                    199   3.190   0.410
  Utilities                 366   3.229   0.459
  Construction            1,457   3.244   0.478
  Manufacturing           2,543   3.238   0.475
  Retail                  2,836   3.151   0.479
  Transport                 962   3.175   0.462
  Finance                 1,544   3.279   0.503
  Services                2,612   3.097   0.492
  Education               1,669   3.198   0.482
  Health                  2,077   3.197   0.497
  Public administration   1,408   3.278   0.456

F(11, 3,489.4) = 31.60   p < 0.001
eta squared = 0.019
decision at α = 0.05: reject H0 (p < 0.001)
Welch's F over 12 groups, the variances not assumed equal; eta squared = SS between / SS total
note: 211 rows with NA in lwage or industry dropped (line 5)
> wages.anova(lwage, group: industry).effect_size()
0.0185603
note: 211 rows with NA in lwage or industry dropped (line 6)
> let trial = frame(gain = [4.1, 5.3, 4.8, 6.9, 7.4, 6.1, 5.2, 4.4, 8.3, 9.1, 7.7, 5.9, 6.4, 7.0, 9.8, 10.4], dose = ["low", "low", "low", "high", "high", "low", "low", "low", "high", "high", "high", "low", "low", "high", "high", "high"], diet = ["a", "a", "a", "a", "a", "b", "b", "b", "b", "b", "b", "c", "c", "c", "c", "c"], age = [31, 45, 38, 52, 29, 41, 36, 48, 33, 27, 44, 39, 50, 35, 30, 42]).add(dose = category(dose), diet = category(diet))
note: `dose` replaced the variable of that name (line 7)
note: `diet` replaced the variable of that name (line 7)
> trial.anova(gain ~ dose * diet)
anova · factorial, type 2 · gain ~ dose * diet
each row tests H0: the population effect of the term is zero, against H1: it is not

  source      df       SS       MS       F   p value   partial eta squared
  model        5   44.043    8.809    8.81     0.002                 0.815
  dose         1   31.592   31.592   31.60    <0.001                 0.760
  diet         2    6.482    3.241    3.24     0.082                 0.393
  dose:diet    2    0.351    0.176    0.18     0.842                 0.034
  residual    10    9.997    1.000
  ────────────────────────────────────────────────────────────────────────
  total       15   54.040

type 2 sums of squares: each term adjusted for every other term that does not contain it
partial eta squared = SS / (SS + SS residual); N = 16
> trial.anova(gain ~ dose * diet, type: 3)
anova · factorial, type 3 · gain ~ dose * diet
each row tests H0: the population effect of the term is zero, against H1: it is not

  source      df       SS       MS       F   p value   partial eta squared
  model        5   44.043    8.809    8.81     0.002                 0.815
  dose         1   30.722   30.722   30.73    <0.001                 0.754
  diet         2    6.669    3.334    3.34     0.078                 0.400
  dose:diet    2    0.351    0.176    0.18     0.842                 0.034
  residual    10    9.997    1.000
  ────────────────────────────────────────────────────────────────────────
  total       15   54.040

type 3 sums of squares: each term adjusted for every other term, the categories coded by deviations from the mean
partial eta squared = SS / (SS + SS residual); N = 16
> trial.anova(gain ~ dose + diet + age, type: 1)
anova · factorial, type 1 · gain ~ dose + diet + age
each row tests H0: the population effect of the term is zero, against H1: it is not

  source     df       SS       MS       F   p value   partial eta squared
  model       4   43.853   10.963   11.84     0.001                 0.811
  dose        1   37.210   37.210   40.18    <0.001                 0.785
  diet        2    6.482    3.241    3.50     0.067                 0.389
  age         1    0.160    0.160    0.17     0.685                 0.016
  residual   11   10.187    0.926
  ───────────────────────────────────────────────────────────────────────
  total      15   54.040

type 1 sums of squares: each term adjusted for the terms above it, so the order of the terms matters
partial eta squared = SS / (SS + SS residual); N = 16
> trial.anova(gain ~ dose * diet).estimates().select([term, estimate, statistic, df, df2, p_value])
frame · 6 rows × 6 variables

      term                               estimate   statistic   df   df2       p_value
  1   dose                                     NA     31.6028    1    10   0.000220956
  2   diet                                     NA     3.24223    2    10     0.0821533
  3   dose:diet                                NA    0.175571    2    10      0.841509
  4   partial eta squared (dose)         0.759632          NA   NA    NA            NA
  5   partial eta squared (diet)         0.393368          NA   NA    NA            NA
  6   partial eta squared (dose:diet)   0.0339231          NA   NA    NA            NA
> wages.anova(lwage ~ industry * category(female) + educ)
anova · factorial, type 2 · lwage ~ industry * category(female) + educ
each row tests H0: the population effect of the term is zero, against H1: it is not

  source                df         SS        MS         F   p value   partial eta squared
  model                 24    505.438    21.060    101.47    <0.001                 0.118
  industry              11     77.856     7.078     34.10    <0.001                 0.020
  female                 1     88.792    88.792    427.81    <0.001                 0.023
  educ                   1    329.462   329.462   1587.37    <0.001                 0.080
  industry:female       11      7.137     0.649      3.13    <0.001                 0.002
  residual          18,176   3772.468     0.208
  ───────────────────────────────────────────────────────────────────────────────────────
  total             18,200   4277.906

type 2 sums of squares: each term adjusted for every other term that does not contain it
partial eta squared = SS / (SS + SS residual); N = 18,201
note: 211 rows with NA in a model variable dropped (line 12)
> g.anova(s, group: site, effect: omega)
anova · Welch one-way · s by site
  H0: the population means of s are equal in all 3 groups of site          μ₁ = μ₂ = μ₃
  H1: at least one group of site has a different population mean of s      not all equal

  group    N    mean      sd
  site=1   5   2.840   0.288
  site=2   4   3.225   0.793
  site=3   5   2.820   0.736

F(2, 5.5) = 0.39   p = 0.693
omega squared = -0.067
decision at α = 0.05: fail to reject H0 (p = 0.693)
Welch's F over 3 groups, the variances not assumed equal; omega squared = (SS between − (k − 1) MS within) / (SS total + MS within)
note: the classical F of the same sums of squares is 0.560, below 1, so omega squared is negative; read it as no evidence of an effect (line 13)
> wages.anova(lwage, group: industry, effect: epsilon).effect_size()
0.0179668
note: 211 rows with NA in lwage or industry dropped (line 14)
> trial.anova(gain ~ dose * diet, effect: omega)
anova · factorial, type 2 · gain ~ dose * diet
each row tests H0: the population effect of the term is zero, against H1: it is not

  source      df       SS       MS       F   p value   partial omega squared
  model        5   44.043    8.809    8.81     0.002                   0.709
  dose         1   31.592   31.592   31.60    <0.001                   0.657
  diet         2    6.482    3.241    3.24     0.082                   0.219
  dose:diet    2    0.351    0.176    0.18     0.842                  -0.115
  residual    10    9.997    1.000
  ──────────────────────────────────────────────────────────────────────────
  total       15   54.040

type 2 sums of squares: each term adjusted for every other term that does not contain it
partial omega squared = (SS − df × MS residual) / (SS + (N − df) × MS residual); N = 16
note: F is below 1 for `dose:diet`, so its partial omega squared is negative; read it as no evidence of an effect (line 15)
> trial.anova(gain ~ dose * diet, effect: omega_f)
anova · factorial, type 2 · gain ~ dose * diet
each row tests H0: the population effect of the term is zero, against H1: it is not

  source      df       SS       MS       F   p value   partial omega squared from F
  model        5   44.043    8.809    8.81     0.002                          0.709
  dose         1   31.592   31.592   31.60    <0.001                          0.718
  diet         2    6.482    3.241    3.24     0.082                          0.256
  dose:diet    2    0.351    0.176    0.18     0.842                         -0.145
  residual    10    9.997    1.000
  ─────────────────────────────────────────────────────────────────────────────────
  total       15   54.040

type 2 sums of squares: each term adjusted for every other term that does not contain it
partial omega squared from F = (F − 1) / (F + (df residual + 1) / df); N = 16
note: F is below 1 for `dose:diet`, so its partial omega squared from F is negative; read it as no evidence of an effect (line 16)
> let a = trial.anova(gain ~ dose * diet)
> a.p_values("diet")
0.0821533
> a.statistic(dose)
31.6028
> a.df("dose:diet")
2
10
> a.effect_size("diet")
0.393368
