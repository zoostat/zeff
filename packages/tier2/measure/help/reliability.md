# reliability

## Summary
Cronbach's alpha of a scale, with a look at each item

## In brief
- `d.reliability([q1, q2, q3])`: the internal consistency of the scale that is the sum of the items.
- `reverse: [q2]` scores an item worded the other way round; `standardize: true` puts the items on one scale first.
- For each item it prints the correlation with the rest of the scale and the alpha of the scale without it.

## Description
Cronbach's alpha of a set of items: how far the answers to several questions that are meant to
measure one thing go together. Alpha is near 1 when people who score high on one item score
high on the others, near 0 when the items have little to do with one another, and below 0 when
they run against one another.

The result prints the number of rows used; alpha with its confidence interval; the alpha of
the standardized items and the average correlation between items; and a table with a line for
each item: its mean and standard deviation, its correlation with the sum of the other items,
and the alpha of the scale without it. An item whose correlation with the rest is low, or
whose removal raises alpha, is one to read again.

An item worded the other way round is named in `reverse:`. An item that runs against the rest
of the scale and is not named there is warned of (W0213); it is never reversed for you. When
every item is 0 or 1, alpha is Kuder and Richardson's KR-20, and the result says so.

## Arguments
- data: The frame, one row for each respondent.
- items: The items of the scale, at least two, each a number or a Bool: a list of variables,
  `[q1, q2, q3]`, or a range, `[q1..q6]`.
- standardize: Whether the figures are taken from the correlations of the items, each item
  put on a scale of mean 0 and standard deviation 1 first; `false` by default. Use it when the
  items are answered on different scales.
- reverse: The items worded the other way round, a list of some of `items:`. They are scored
  the other way before anything is computed; their means in the table are those of the
  answers as given.
- ci: The interval of alpha: `feldt`, the default, from the F distribution, or `duhachek`,
  from the standard error of alpha, which the result then carries.
- level: The confidence level of the interval; 0.95 by default.

## Value
A `TestResult`, with the result interface (`summary`, `estimates`, `ci`, `n_obs`, `export`,
`report`): `estimates` gives alpha, the alpha of the standardized items and the average
correlation as a frame, and `statistic` one of them by its name, `statistic("standardized alpha")`,
or alpha when no name is given. The result is no test: it has no p-value and no hypotheses.

## Missing values
A row with `NA` in any item is left out, and a note gives the count: every figure is taken over the rows that have every item. To see how many answers each item lacks, use `summary`.

## Details
With k items, alpha is k / (k − 1) × (1 − the sum of the items' variances / the variance of
their sum). The alpha of the standardized items is the same formula on the correlations, which
is k r̄ / (1 + (k − 1) r̄) with r̄ the average correlation between two items. An item's
correlation with the rest is its correlation with the sum of the other items, the item itself
left out of the sum. With `standardize: true` every figure in the table and the interval are
taken from the correlations.

Every figure depends on the covariances of the items alone, so a reversed item is scored as its
negative, and neither end of its scale is needed. The score of a respondent, which does need
them, is another matter: write it yourself, as `6 - q` for an item from 1 to 5.

Feldt's interval takes (1 − alpha) / (1 − the true alpha) to have the F distribution on n − 1
and (n − 1)(k − 1) degrees of freedom, n being the number of rows. Duhachek and Iacobucci's
takes alpha to be nearly normal, with the standard error of van Zyl, Neudecker and Nel, which
does not ask the items to be equally related; it suits large samples. With `standardize: true`
either interval treats the standardized items as the items given.

A scale of two items has no alpha without one of them. An item with one value in every row has
no correlation with the rest, and is warned of; under `standardize: true` it is refused.

## A common mistake
Leaving an item that is worded the other way round as it is. In the survey `engagement` the
fourth statement is "I often think of leaving":

> let scale = dataset(engagement).reliability([eng1..eng6])
note: 214 rows with NA in eng1, eng2, eng3, eng4, eng5 or eng6 dropped (line 9)
warning[W0213]: `eng4` runs against the rest of the scale: its correlation with the sum of the other items is negative (line 9)
  = help: an item worded the other way round is named in `reverse:`, as `reverse: [eng4]`; the items are left as they are here

Alpha comes out as 0.508 where it should be 0.851. Name the item in `reverse:`.

## Coming from R or Stata
| d.reliability([q1..q6]) | psych::alpha(d[, paste0("q", 1:6)]) | alpha q1-q6, item |
| d.reliability([q1..q6], reverse: [q4]) | psych::alpha(d[, paste0("q", 1:6)], keys = "q4") | alpha q1-q6, item reverse(q4) |
| d.reliability([q1..q6], standardize: true) | the `std.alpha` of psych::alpha | alpha q1-q6, item std |

`psych::alpha` prints both intervals; here `ci:` chooses one. Stata's `alpha` gives no
interval, and reverses an item that runs against the scale by itself unless told `asis`;
`psych::alpha` does so when asked with `check.keys = TRUE`. Here an item is reversed only when
you name it.

## Related tasks
- For the correlations between the items, use `correlation`.
- For a respondent's score, the mean or the sum of the items, use `row`.
- For the answers each item lacks and its distribution, use `summary` or `tabulate`.

## References
Cronbach (1951), Coefficient alpha and the internal structure of tests, Psychometrika 16,
297–334.
Feldt (1965), The approximate sampling distribution of Kuder–Richardson reliability coefficient
twenty, Psychometrika 30, 357–370.
Duhachek and Iacobucci (2004), Alpha's standard error (ASE): an accurate and precise confidence
interval estimate, Journal of Applied Psychology 89, 792–808.

## See also
correlation, row, summary

## Examples
// The six statements of the survey, the fourth scored the other way round.
> let survey = dataset(engagement)
> survey.reliability([eng1..eng6], reverse: [eng4])
reliability · Cronbach's alpha · 6 items
N = 986 rows with every item
alpha = 0.851   95% CI [0.836, 0.865] (Feldt)
alpha of the standardized items = 0.851   average correlation between items = 0.487

  item               mean      sd   item–rest correlation   alpha without the item
  eng1              3.042   1.007                   0.691                    0.815
  eng2              3.060   0.980                   0.652                    0.823
  eng3              3.067   1.005                   0.628                    0.827
  eng4 (reversed)   2.966   1.027                   0.656                    0.822
  eng5              3.024   0.976                   0.575                    0.837
  eng6              3.055   1.032                   0.607                    0.831
note: 214 rows with NA in eng1, eng2, eng3, eng4, eng5 or eng6 dropped (line 2)

// Items on one scale first, with the interval from the standard error of alpha at 90%.
> survey.reliability([eng1..eng6], reverse: [eng4], standardize: true, ci: duhachek, level: 0.9)
reliability · Cronbach's alpha · 6 items · standardized
N = 986 rows with every item
alpha of the standardized items = 0.851   90% CI [0.839, 0.863] (Duhachek)
alpha of the items as scored = 0.851   average correlation between items = 0.487

  item               mean      sd   item–rest correlation   alpha without the item
  eng1              3.042   1.007                   0.691                    0.815
  eng2              3.060   0.980                   0.651                    0.823
  eng3              3.067   1.005                   0.628                    0.827
  eng4 (reversed)   2.966   1.027                   0.656                    0.822
  eng5              3.024   0.976                   0.575                    0.837
  eng6              3.055   1.032                   0.607                    0.831
note: 214 rows with NA in eng1, eng2, eng3, eng4, eng5 or eng6 dropped (line 3)

// Questions answered right or wrong: alpha is KR-20.
> let quiz = frame(q1 = [1, 0, 1, 1, 0, 1, 0, 1, 1, 0, 1, 1, 0, 0, 1, 1], q2 = [1, 0, 1, 0, 0, 1, 0, 1, 1, 0, 1, 1, 0, 1, 1, 0], q3 = [1, 1, 1, 1, 0, 1, 0, 0, 1, 0, 1, 0, 0, 0, 1, 1], q4 = [0, 0, 1, 1, 0, 1, 1, 1, 1, 0, 0, 1, 0, 0, 1, 1])
> quiz.reliability([q1, q2, q3, q4], standardize: false, ci: feldt)
reliability · Cronbach's alpha · 4 items
N = 16 rows with every item
alpha = 0.748   95% CI [0.460, 0.902] (Feldt)
alpha of the standardized items = 0.750   average correlation between items = 0.428

  item    mean      sd   item–rest correlation   alpha without the item
  q1     0.625   0.500                   0.881                    0.484
  q2     0.562   0.512                   0.447                    0.742
  q3     0.562   0.512                   0.447                    0.742
  q4     0.562   0.512                   0.447                    0.742

every item is 0 or 1: this alpha is Kuder and Richardson's coefficient KR-20

// The figures as values: alpha, and the three coefficients as a frame.
> let r = quiz.reliability([q1, q2, q3, q4])
> print(r.statistic(), r.n_obs())
> r.estimates()
0.747795 16
frame · 3 rows × 9 variables

      term                  estimate   std_error   statistic   df   df2   p_value   ci_lower   ci_upper
  1   alpha                 0.747795          NA    0.747795   NA    NA        NA   0.459978   0.901674
  2   standardized alpha    0.749626          NA    0.749626   NA    NA        NA         NA         NA
  3   average correlation   0.428083          NA    0.428083   NA    NA        NA         NA         NA
