# p_adjust

## Summary
Adjust p-values for multiple testing

## Description
The p-values of several tests adjusted so that they can be compared with one level across the
family, by Holm's step-down method (the default), Bonferroni's correction, or the
Benjamini–Hochberg and Benjamini–Yekutieli procedures that control the false discovery rate.
The result is a list in the order given, each adjusted p-value at most 1; an `NA` keeps its
place and does not count among the comparisons, as in R's `p.adjust`.

## Arguments
- p: The p-values, a list of numbers between 0 and 1, or a numeric column; `NA` allowed.
- method: `holm` controls the family-wise error rate and is never less powerful than
  `bonferroni`; `bh` controls the false discovery rate for independent or positively dependent
  tests, `by` under any dependence.

## Value
A `List<Float>` of the same length as `p`.

## Details
With n the number of p-values present and p₍₁₎ ≤ … ≤ p₍ₙ₎ their order statistics: Bonferroni
gives min(1, n·pᵢ); Holm gives min(1, max over j ≤ i of (n + 1 − j)·p₍ⱼ₎); Benjamini–Hochberg
gives min(1, min over j ≥ i of (n/j)·p₍ⱼ₎); Benjamini–Yekutieli multiplies the
Benjamini–Hochberg value by Σ 1/j over j = 1 … n before taking the minimum with 1. Holm and
Bonferroni hold under any dependence between the tests.

## References
Holm (1979), A simple sequentially rejective multiple test procedure, Scandinavian Journal of
Statistics 6, 65–70.
Benjamini and Hochberg (1995), Controlling the false discovery rate: a practical and powerful
approach to multiple testing, Journal of the Royal Statistical Society B 57, 289–300.
Benjamini and Yekutieli (2001), The control of the false discovery rate in multiple testing
under dependency, Annals of Statistics 29, 1165–1188.

## See also
kruskal_test, t_test, anova

## Examples
> p_adjust([0.01, 0.04, 0.03, 0.005, 0.2])
0.04
0.09
0.09
0.025
0.2
> p_adjust([0.01, 0.04, 0.03, 0.005, 0.2], method: bonferroni)
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
