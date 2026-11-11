# p_adjust

## Summary
Adjust p-values for multiple testing

## Description
The p-values of several tests adjusted so that they can be compared with one level across the
family. Eight methods are offered: Holm's step-down method (the default), Bonferroni's
correction, Hochberg's step-up method, Hommel's method and Šidák's correction, which control the
chance of any false rejection; the Benjamini–Hochberg and Benjamini–Yekutieli procedures, which
control the false discovery rate; and `none`, which returns the p-values as given. Each adjusted
p-value is at most 1 and stands where its p-value stood; an `NA` keeps its place and does not
count among the comparisons, as in R's `p.adjust`.

The p-values are given as a list or as a variable of a frame. A list gives a list. Inside `add`
or `update` a variable gives a new variable, one adjusted p-value on each row, as in
`tests.add(p_holm = p_adjust(p_value))`. With `by:` on the verb each group of rows is a family
of its own: `tests.add(p_holm = p_adjust(p_value), by: outcome)` adjusts the p-values of each
outcome among themselves, and the number of comparisons is counted within the group.

## Arguments
- p: The p-values, numbers between 0 and 1: a list, or inside `add` or `update` a variable of
  the frame; `NA` allowed. A value outside 0 to 1 is refused, and for a variable the message
  names its row.
- method: `holm` controls the family-wise error rate under any dependence between the tests and
  is never less powerful than `bonferroni`. `hochberg` is never less powerful than `holm`, and
  `hommel` never less than `hochberg`; both suppose independent or positively dependent tests.
  `sidak` is the slightly sharper form of `bonferroni` for independent tests. `bh` controls the
  false discovery rate for independent or positively dependent tests, `by` under any dependence.
  `none` makes no adjustment.

## Value
A `List<Float>` of the same length as `p` for a list, and a `Column<Float>` of the same length
for a variable or any other column, such as `tests["p"]`.

## Details
With n the number of p-values present and p₍₁₎ ≤ … ≤ p₍ₙ₎ their order statistics, the adjusted
value of p₍ᵢ₎ is as follows, each before the minimum with 1. Bonferroni: n·p₍ᵢ₎. Šidák:
1 − (1 − p₍ᵢ₎)ⁿ, the single-step form, computed so that a very small p-value keeps its digits.
Holm: the largest of (n + 1 − j)·p₍ⱼ₎ over j ≤ i. Hochberg: the smallest of (n + 1 − j)·p₍ⱼ₎
over j ≥ i, which are Holm's multipliers taken from the largest p-value downwards.
Benjamini–Hochberg: the smallest of (n/j)·p₍ⱼ₎ over j ≥ i. Benjamini–Yekutieli: the
Benjamini–Hochberg value multiplied by Σ 1/j over j = 1 … n.

Hommel's method is the closed test built on Simes's rule. The Simes p-value of a set of m
hypotheses is the smallest of (m/j)·q₍ⱼ₎ over the ordered p-values q₍₁₎ ≤ … ≤ q₍ₘ₎ of the set,
and the adjusted p-value of a hypothesis is the largest Simes p-value among the sets that hold
it. It is computed by Wright's (1992) pass over the set sizes, without visiting the sets, and for
two p-values it equals Hochberg's. Its time grows with the square of the number of p-values, as
in R: well under a second for 20,000 of them, several seconds for 100,000.

Holm and Bonferroni hold under any dependence between the tests. Hochberg and Hommel rest on
Simes's inequality, which holds for independent tests and for the usual forms of positive
dependence (Sarkar, 1998). Šidák's value is exact for independent tests and holds for two-sided
tests on jointly normal statistics (Šidák, 1967).

Which tests form a family is a choice of the study, not of the method. A study with several
outcomes often adjusts within each outcome and says so; adjusting across all of them is the
stricter choice. The grouping variable is written on the verb, `add(..., by: outcome)`, as for
`rank` and every other function that runs down a column, and not inside `p_adjust`, whose word
`by` is a method. A missing value of the grouping variable is a group of its own. After `where`
the family is the rows that were kept. A group with one p-value leaves it as it is, and a group
with none gives `NA`.

In R the form on a frame is `mutate(p_holm = p.adjust(p))` of dplyr, and the form within
families is `group_by(outcome) |> mutate(p_holm = p.adjust(p))`, or `ave(p, outcome, FUN =
p.adjust)` in base R. Stata adjusts within one `test` or `pwcompare` command and has no
command of its own that adjusts a variable of p-values.

R's `p.adjust` writes the methods `"holm"`, `"hochberg"`, `"hommel"`, `"bonferroni"`, `"BH"` (or
`"fdr"`), `"BY"` and `"none"`, and has no Šidák. Stata writes `bonferroni`, `holm`, `sidak` and
`noadjust` in the `mtest()` option of `test`, and `bonferroni`, `sidak` and `noadjust` in the
`mcompare()` option of `pwcompare`; its `sidak` is the single-step form given here.

## References
Holm (1979), A simple sequentially rejective multiple test procedure, Scandinavian Journal of
Statistics 6, 65–70.
Hochberg (1988), A sharper Bonferroni procedure for multiple tests of significance, Biometrika
75, 800–802.
Hommel (1988), A stagewise rejective multiple test procedure based on a modified Bonferroni
test, Biometrika 75, 383–386.
Simes (1986), An improved Bonferroni procedure for multiple tests of significance, Biometrika
73, 751–754.
Wright (1992), Adjusted p-values for simultaneous inference, Biometrics 48, 1005–1013.
Sarkar (1998), Some probability inequalities for ordered MTP2 random variables: a proof of the
Simes conjecture, Annals of Statistics 26, 494–504.
Šidák (1967), Rectangular confidence regions for the means of multivariate normal
distributions, Journal of the American Statistical Association 62, 626–633.
Benjamini and Hochberg (1995), Controlling the false discovery rate: a practical and powerful
approach to multiple testing, Journal of the Royal Statistical Society B 57, 289–300.
Benjamini and Yekutieli (2001), The control of the false discovery rate in multiple testing
under dependency, Annals of Statistics 29, 1165–1188.

## See also
kruskal_test, t_test, anova, posthoc, add, rank

## Examples
> let p = [0.01, 0.04, 0.03, 0.005, 0.2]
> p_adjust(p)
0.04
0.09
0.09
0.025
0.2
> p_adjust(p, method: hochberg)
0.04
0.08
0.08
0.025
0.2
> p_adjust(p, method: hommel)
0.04
0.08
0.06
0.025
0.2
> p_adjust(p, method: sidak)
0.04901
0.184627
0.141266
0.0247512
0.67232
> p_adjust(p, method: bonferroni)
0.05
0.2
0.15
0.025
1
> p_adjust([0.01, NA, 0.03, 0.005], method: bh)
0.015
NA
0.03
0.015
> let tests = frame(outcome = ["wage", "wage", "wage", "hours", "hours", "hours"], arm = [1, 2, 3, 1, 2, 3], p_value = [0.01, 0.04, 0.03, 0.2, 0.001, 0.02])
> tests.add(p_holm = p_adjust(p_value))
frame · 6 rows × 4 variables

      outcome   arm   p_value   p_holm
  1   wage        1      0.01     0.05
  2   wage        2      0.04     0.09
  3   wage        3      0.03     0.09
  4   hours       1       0.2      0.2
  5   hours       2     0.001    0.006
  6   hours       3      0.02     0.08
> tests.add(p_holm = p_adjust(p_value), by: outcome)
frame · 6 rows × 4 variables

      outcome   arm   p_value   p_holm
  1   wage        1      0.01     0.03
  2   wage        2      0.04     0.06
  3   wage        3      0.03     0.06
  4   hours       1       0.2      0.2
  5   hours       2     0.001    0.003
  6   hours       3      0.02     0.04
> tests.add(p_bh = p_adjust(p_value, method: bh), by: outcome).where(p_bh < 0.05)
frame · 5 rows × 4 variables

      outcome   arm   p_value    p_bh
  1   wage        1      0.01    0.03
  2   wage        2      0.04    0.04
  3   wage        3      0.03    0.04
  4   hours       2     0.001   0.003
  5   hours       3      0.02    0.03
