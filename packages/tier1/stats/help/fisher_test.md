# fisher_test

## Summary
Fisher's exact test of independence in a cross-table

## In brief
- `d.fisher_test(a, b)`: the exact test of independence of two variables.
- On a 2 × 2 table it gives the odds ratio with its interval, and `alternative: less` or `greater` tests one side.
- On a larger table it gives the p-value alone.

## Description
Fisher's exact test of independence between two variables, from their cross-table. On a 2 × 2
table it gives the exact p-value for the alternative asked, the conditional maximum-likelihood
estimate of the odds ratio and its 95% interval, as R's `fisher.test` defines them. On a larger
table it gives the exact p-value alone, summed over every table with the same margins that is no
more probable than the one observed; such a table takes no direction, and one with more than a
million tables sharing its margins is refused with `chi2_test` suggested. Each result prints the
hypotheses, H0 and H1; the table of counts; the odds ratio with its interval on a 2 × 2 table; the
p-value; and a decision at α = 0.05.

Values are listed in the order of section 3.7 of the Language Reference; rows with `NA` in a
tested variable are dropped and counted in a note, and a value left with no rows (possible only
through weights) is left out of the table and named below it.

## Arguments
- data: The frame.
- a: The first variable, down the side of the table.
- b: The second variable, along the top.
- alternative: `two_sided`, the default, or on a 2 × 2 table `less` or `greater`: the side of 1
  the odds ratio is tested against, where the odds ratio is the odds of the first value of `b`
  among rows with the first value of `a` relative to rows with the second. On a larger table
  `less` or `greater` is refused (Z0420).
- weights: Frequency weights, each row standing for that many rows; the counts must come to
  whole numbers. Analytic weights are refused, since the test counts rows, and sampling weights
  by the shared rule (Z0411).

## Value
A `TestResult`, with the result interface (`summary`, `estimates`, `p_values`, `ci`, `n_obs`,
`export`, `report`): the p-value and the hypotheses; on a 2 × 2 table the odds ratio with its
interval, and no statistic of its own. Its own methods `statistic()`, `df()`, `effect_size()`,
`h0()`, `h1()` and `alternative()` give the figures as values.

## Missing values
A row with `NA` in either variable is left out, and a note gives the count; `NA` is never a category of the table. With `weights:`, a row whose weight is `NA` is dropped and counted with them, and the note names the weight; a row whose weight is zero is left out, and the note says how many.

## Details
Given the margins of a 2 × 2 table, the first cell has the hypergeometric distribution under
independence. The two-sided p-value is the sum of the probabilities of the outcomes no more
probable than the one observed (within a relative 1e-7, as R counts them); `less` is the
probability of a first cell no larger, `greater` of one no smaller. The odds ratio is the
conditional maximum-likelihood estimate: the value ψ of Fisher's noncentral hypergeometric
distribution whose mean is the observed cell, 0 or `Inf` when the cell sits at an end of its
range. The interval inverts the tails of that distribution: the upper bound is the ψ at which
P(X ≤ x) = α/2 and the lower the ψ at which P(X ≥ x) = α/2, with α = 0.05; a one-sided
interval uses α on one side and 0 or `Inf` on the other. The distribution is formed by the
recurrence of its probability ratios from its mode, never from factorials, and the bounds are
solved to the last digit, where R's `fisher.test` stops near 1e-4 of the root.

On a larger table the probability of each table with the same margins is
Π rᵢ! Π cⱼ! / (N! Π nᵢⱼ!), and the p-value sums those no more probable than the observed one.

## A common mistake
Asking for one side on a table larger than 2 × 2. Only an odds ratio has a side:

> dataset(staff).fisher_test(region, offer, alternative: less)
error[Z0420]: `alternative:` of `fisher_test` applies to a 2 × 2 table, but `region` × `offer` is 4 × 2
  = help: the exact test of a larger table has no direction: drop `alternative:`, or keep two values of each variable with `where`

Leave `alternative:` out, or keep two values of each variable with `where`.

## Coming from R or Stata
| d.fisher_test(a, b) | fisher.test(table(d$a, d$b)) | tabulate a b, exact |
| d.fisher_test(a, b, alternative: greater) | fisher.test(table(d$a, d$b), alternative = "greater") | tabulate a b, exact |

The odds ratio is R's, the conditional maximum-likelihood estimate; it is not the ratio of
cross-products of the four counts, though it is close to it. Stata's `tabulate, exact` prints
the p-value, with a one-sided p-value under it on a 2 × 2 table, and no odds ratio.

## Related tasks
- For a table too large to enumerate, use `chi2_test`.
- For the difference of two proportions with its interval, use `proportion_test`.
- For the table itself, with percentages, use `tabulate`.
- For an odds ratio adjusted for other variables, use `glm`.

## References
Fisher (1935), The logic of inductive inference, Journal of the Royal Statistical Society 98,
39–82.
Agresti (2002), Categorical Data Analysis, 2nd edition, Wiley, section 3.5.

## See also
chi2_test, proportion_test, tabulate

## Examples
// A small 2 × 2 table: the p-value, the odds ratio and its interval.
> let t = frame(a = ["x", "x", "x", "x", "y", "y", "y", "y"], b = [1, 1, 1, 2, 1, 2, 2, 2])
> t.fisher_test(a, b)
fisher_test · exact · a × b
  H0: a and b are independent in the population      OR = 1
  H1: a and b are associated in the population       OR ≠ 1

  a       1   2   Total
  x       3   1       4
  y       1   3       4
  ─────────────────────
  Total   4   4       8

odds ratio (conditional MLE) = 6.408   95% CI [0.212, 626.244]
p = 0.486
decision at α = 0.05: fail to reject H0 (p = 0.486)
exact p-value from the distribution of the first cell given the margins, 5 outcomes

// One side on the shipped wages: are the odds greater than 1?
> let wages = dataset(wages)
> wages.fisher_test(union, female, alternative: greater)
fisher_test · exact · union × female
  H0: union and female are independent in the population                                OR = 1
  H1: the odds of female = male are higher among union = no than among union = yes      OR > 1

  union    male   female    Total
  no      6,516    6,040   12,556
  yes     2,975    2,881    5,856
  ───────────────────────────────
  Total   9,491    8,921   18,412

odds ratio (conditional MLE) = 1.045   95% CI [0.991, Inf)
p = 0.086
decision at α = 0.05: fail to reject H0 (p = 0.086)
exact p-value from the distribution of the first cell given the margins, 5,857 outcomes

// A 3 × 2 table: the p-value alone.
> let m = frame(r = ["a", "a", "a", "a", "a", "a", "b", "b", "b", "b", "c", "c", "c", "c", "c", "c"], c = [1, 1, 2, 2, 2, 2, 1, 1, 1, 2, 1, 2, 2, 2, 2, 2])
> m.fisher_test(r, c)
fisher_test · exact · r × c
  H0: r and c are independent in the population
  H1: r and c are associated in the population

  r       1    2   Total
  a       2    4       6
  b       3    1       4
  c       1    5       6
  ──────────────────────
  Total   6   10      16

p = 0.302
decision at α = 0.05: fail to reject H0 (p = 0.302)
exact p-value over the 25 tables with these margins

// A table given as counts, one row for each cell, with the two-sided alternative named.
> let tab = frame(treated = ["yes", "yes", "no", "no"], improved = ["yes", "no", "yes", "no"], count = [18, 7, 9, 16])
> tab.fisher_test(treated, improved, alternative: two_sided, weights: count, weight_type: frequency)
fisher_test · exact · treated × improved
  H0: treated and improved are independent in the population      OR = 1
  H1: treated and improved are associated in the population       OR ≠ 1

  treated   no   yes   Total
  no        16     9      25
  yes        7    18      25
  ──────────────────────────
  Total     23    27      50

odds ratio (conditional MLE) = 4.422   95% CI [1.200, 18.027]
p = 0.022
decision at α = 0.05: reject H0 (p = 0.022)
exact p-value from the distribution of the first cell given the margins, 24 outcomes
