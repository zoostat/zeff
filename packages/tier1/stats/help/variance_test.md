# variance_test

## Summary
Test of equal variances across groups

## In brief
- `pay.variance_test(wage, group: union)`: Levene's test, centred on the median.
- `method: bartlett` gives Bartlett's test; `method: f` the ratio of two variances with its interval.
- `center: mean` centres Levene's test on the mean, as Levene first wrote it.

## Description
Tests whether a numeric variable has the same variance in every group of `group:`. The null
hypothesis is that the variances are equal; a small p-value says that at least one group is
more spread out than another. It is the usual check before a t test or an analysis of variance
that pools the groups' variances.

`method:` chooses among three tests of that one hypothesis. `levene`, the default, turns each
value into its distance from the centre of its group and compares the mean distances by a
one-way F; it holds up well when the data are not normal. `bartlett` compares the logarithms of
the variances and is the more powerful test when the data are normal, but reads skewness or
heavy tails as unequal variances. `f` is for two groups only: the ratio of the two variances,
with a 95% interval, and with the same weakness as Bartlett's test.

## Arguments
- data: A frame.
- var: The variable whose spread is compared: a Bool, Int or Float variable.
- group: The variable whose values are the groups: Bool, Int, Str or Category, with two groups or more.
- method: `levene`, `bartlett` or `f`; `levene` unless given.
- center: For `method: levene`, the centre the distances are taken from: `median` or `mean`; `median` unless given.

## Value
A test result: a table of each group's size, standard deviation and variance, the statistic
with its degrees of freedom, the p-value and the decision at α = 0.05. With `method: f` it also
holds the ratio of the variances and its 95% interval. `statistic()`, `df()`, `p_values()` and
`ci()` read its parts.

## Missing values
A row with `NA` in the variable or in the grouping variable is left out, and a note gives the
count. A group left with no row is not a group of the test.

## Details
### Levene's test and its centre
Each value becomes zᵢⱼ = |xᵢⱼ − cᵢ|, its distance from the centre cᵢ of its group, and the
statistic is the F of a one-way analysis of variance of the distances, on k − 1 and N − k
degrees of freedom for k groups and N rows. With `center: median` this is the form of Brown
and Forsythe (1974), the default here, in R's `car::leveneTest` and as W₅₀ in Stata's `robvar`;
with `center: mean` it is Levene's (1960) own, Stata's W₀. The median keeps the test's level
closer to its nominal value when the data are skewed.

### Bartlett's test
K² = [(N − k) ln s²ₚ − Σ (nᵢ − 1) ln s²ᵢ] / C, where s²ₚ is the pooled variance and
C = 1 + (Σ 1/(nᵢ − 1) − 1/(N − k)) / (3(k − 1)), referred to chi-squared on k − 1 degrees of
freedom. Every group needs two values or more and a variance above zero.

### The ratio of two variances
F = s²₁ / s²₂ on n₁ − 1 and n₂ − 1 degrees of freedom, the first group over the second in the
order of the table. The p-value is two-sided, twice the smaller tail. The interval of the ratio
runs from F / F₀.₉₇₅ to F / F₀.₀₂₅, the quantiles of the same F distribution.

### Groups the tests cannot take
Levene's test takes a group of one value and a group with no spread. Bartlett's test and the
ratio do not, and say so (Z0401). `method: f` with more than two groups is refused, as is
`center:` with a method that has no centre (Z0410).

### Exactness
Closed form in double precision: the statistics agree with R to a relative 1e-10, and the
p-values through the engine's F and chi-squared distributions.

## A common mistake
Asking for the ratio of two variances where there are more than two groups:

> frame(y = [1.0, 2, 4, 3, 5, 9, 2, 8, 15], g = [1, 1, 1, 2, 2, 2, 3, 3, 3]).variance_test(y, group: g, method: f)
error[Z0401]: `method: f` of `variance_test` is the ratio of two variances, but `g` has 3 groups
  = help: `method: levene`, the default, and `method: bartlett` compare any number of groups

Leave `method:` out for Levene's test, or keep two groups with `where`.

## Coming from R or Stata
| d.variance_test(y, group: g) | car::leveneTest(y ~ factor(g), data = d) | robvar y, by(g), W50 |
| d.variance_test(y, group: g, center: mean) | car::leveneTest(y ~ factor(g), data = d, center = mean) | robvar y, by(g), W0 |
| d.variance_test(y, group: g, method: bartlett) | bartlett.test(y ~ g, data = d) | oneway y g, Bartlett's line |
| d.variance_test(y, group: g, method: f) | var.test(y ~ g, data = d) | sdtest y, by(g) |

Stata's `sdtest` reports the same F and two-sided p-value as a test of standard deviations.
`robvar` prints three statistics at once; W₁₀, centred on the trimmed mean, is not offered.

## Related tasks
- To compare the groups' means when the variances differ, use `t_test`, whose default does not pool them.
- To compare the means of several groups without assuming equal variances, use `anova`.
- To compare groups without assuming normal data at all, use `kruskal_test`.
- For each group's variance or standard deviation as a frame, use `aggregate`.
- To check normality, on which Bartlett's test leans, use `shapiro_test`.

## References
Levene, H. (1960), Robust tests for equality of variances, in Olkin, I. and others (eds.),
Contributions to Probability and Statistics, Stanford University Press, 278–292.

Brown, M. B. and Forsythe, A. B. (1974), Robust tests for the equality of variances, Journal of
the American Statistical Association 69(346), 364–367.

Bartlett, M. S. (1937), Properties of sufficiency and statistical tests, Proceedings of the
Royal Society of London A 160(901), 268–282.

## See also
t_test, anova, kruskal_test, shapiro_test, variance

## Examples
// Levene's test centred on the median, the default: do union and other wages vary alike?
> let w = dataset(wages).where(year == 2019)
> w.variance_test(wage, group: union)
variance_test · wage by union
  H0: the variance of wage is the same in both groups of union
  H1: the variance of wage differs between the two groups of union

  group       N       sd   variance
  no      1,022   14.400    207.371
  yes       470   14.693    215.889

F(1, 1,490) = 0.01   p = 0.909
decision at α = 0.05: fail to reject H0 (p = 0.909)
Levene's test over 2 groups, distances taken from each group's median (Brown–Forsythe)
note: 26 rows with NA in wage or union dropped (line 2)

// Centred on the mean, as Levene first wrote it.
> w.variance_test(wage, group: union, center: mean)
variance_test · wage by union
  H0: the variance of wage is the same in both groups of union
  H1: the variance of wage differs between the two groups of union

  group       N       sd   variance
  no      1,022   14.400    207.371
  yes       470   14.693    215.889

F(1, 1,490) = 0.04   p = 0.836
decision at α = 0.05: fail to reject H0 (p = 0.836)
Levene's test over 2 groups, distances taken from each group's mean
note: 26 rows with NA in wage or union dropped (line 3)

// Bartlett's test over the twelve industries.
> w.variance_test(wage, group: industry, method: bartlett)
variance_test · wage by industry
  H0: the variance of wage is the same in every group of industry
  H1: the variance of wage in at least one group of industry differs from that in another

  group                     N       sd   variance
  Agriculture              44   11.392    129.783
  Mining                   15   10.807    116.793
  Utilities                38   17.455    304.691
  Construction            125   13.251    175.582
  Manufacturing           204   13.896    193.109
  Retail                  231   14.913    222.405
  Transport                83   14.133    199.753
  Finance                 129   16.478    271.523
  Services                204   14.159    200.480
  Education               130   15.924    253.580
  Health                  166   14.156    200.389
  Public administration   123   12.859    165.365

K2(11) = 22.73   p = 0.019
decision at α = 0.05: reject H0 (p = 0.019)
Bartlett's test over 12 groups, chi-squared approximation; it assumes normal data and is sensitive to departures from it
note: 26 rows with NA in wage or industry dropped (line 4)

// The ratio of two variances, with its 95% interval.
> w.variance_test(wage, group: female, method: f)
variance_test · wage by female
  H0: the variance of wage is the same in both groups of female
  H1: the variance of wage differs between the two groups of female

  group      N       sd   variance
  male     777   15.101    228.054
  female   715   13.476    181.593

ratio of variances (male / female) = 1.256   95% CI [1.087, 1.450]
F(776, 714) = 1.26   p = 0.002
decision at α = 0.05: reject H0 (p = 0.002)
F test of the ratio of two variances, two-sided; it assumes normal data and is sensitive to departures from it
note: 26 rows with NA in wage or female dropped (line 5)

// Keep the result under a name to read its parts.
> let t = w.variance_test(wage, group: female, method: f)
note: 26 rows with NA in wage or female dropped (line 6)

// The statistic as a number.
> t.statistic()
1.25585

// Its degrees of freedom, of the first group and then of the second.
> t.df()
776
714

// And the p-value.
> t.p_values()
0.00196106
