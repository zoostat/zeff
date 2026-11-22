# confusion

## Summary
The classification table of a binary model at a cutoff, with its rates

## In brief
- `g.confusion()`: observed against predicted outcomes, a row predicted 1 at a fitted probability of 0.5 or more.
- `g.confusion(cutoff: 0.3)`: the same at another cutoff.
- `g.confusion(newdata: test)`: on other rows, such as rows kept back from the fit.

## Description
Turns the fitted probabilities of a binary model into predictions and counts how they fare. A
row is predicted 1 when its fitted probability is at least the cutoff, and 0 otherwise. The
table sets what was observed against what was predicted: the ones found, the ones missed, the
noughts wrongly taken for ones, and the noughts rightly left.

Five shares are made of the four counts. The sensitivity is the share of the ones that are
predicted 1, and the specificity the share of the noughts that are predicted 0. The accuracy
is the share of all rows predicted as observed. The positive predictive value is the share of
the rows predicted 1 that are ones, and the negative predictive value the share of the rows
predicted 0 that are noughts.

A cutoff of one half is the usual one and seldom the best: where ones are rare, hardly a row
reaches it and the model predicts 0 almost everywhere. A lower cutoff finds more of the ones at
the price of more false alarms.

## Arguments
- fit: The fit of a binary model, from `glm` with `family: binomial`.
- newdata: A frame with the model's variables and its outcome, to classify in the place of the rows the fit used.
- cutoff: The fitted probability at or above which a row is predicted 1; 0.5 unless given.

## Value
A `TestResult` that prints the table of counts with its totals and, under it, the five shares.
`estimates` gives the shares as a frame, and `statistic` one of them by its name,
`statistic("sensitivity")`, or the accuracy when no name is given. The result has no p-value
and no interval.

## Missing values
On the rows the fit used there is nothing missing. A row of `newdata` with `NA` in the outcome
or in a variable of the model, or with a level of a category the fit never saw, is left out,
and the line of the rows says how many.

## Details
### The cutoff
A row whose fitted probability equals the cutoff is predicted 1. The cutoff lies strictly
between 0 and 1. The table at every cutoff at once is the ROC curve, which `curve` gives: each
of its points is the sensitivity and the specificity of one table.

### A share of nothing
A share whose denominator is nought has no value and is `NA`: the positive predictive value
when no row is predicted 1, the sensitivity when the rows hold no one.

### Weights
After frequency weights each row counts as the number of observations it stands for. After
sampling weights each row counts as its weight, the weights scaled so that they sum to the
rows used; the cells are then not whole numbers and are printed to one decimal. The rows of
`newdata` count once each.

### On the rows of the fit, or on new rows
A model is fitted to do well on its own rows, so the table on those rows flatters it a little.
The honest table is on rows the fit did not see, given as `newdata:`.

## A common mistake
Writing the cutoff as a percentage:

> dataset(wages).glm(union ~ educ, family: binomial).confusion(cutoff: 50)
error[Z0401]: `cutoff:` is a probability strictly between 0 and 1, but this is 50
  = help: a row is predicted 1 when its fitted probability is at least the cutoff; 0.5 unless given

The cutoff is on the scale of the fitted probabilities: one half is `0.5`.

## Coming from R or Stata
| g.confusion() | table(d$y, fitted(g) >= 0.5) | estat classification |
| g.confusion(cutoff: 0.3) | table(d$y, fitted(g) >= 0.3) | estat classification, cutoff(0.3) |
| g.confusion(newdata: test) | table(test$y, predict(g, test, type = "response") >= 0.5) | predict p, then tabulate y (p >= 0.5) |

R has no one function for it in base; `caret::confusionMatrix` takes the two columns of
predictions and outcomes and gives the same counts and shares. Stata's `estat classification`
prints the same table with the observed outcome in the columns, and the same shares under it
as percentages; here the observed outcome is in the rows and the shares are proportions.
Stata, like this function, predicts 1 at a fitted probability of the cutoff or more.

## Related tasks
- To fit the model whose predictions these are, use `glm`.
- To see the sensitivity and the specificity at every cutoff, use `curve` for the ROC curve.
- To get each row's fitted probability, use `fitted`; for new rows, `predict`.
- To see the effect of each regressor on the probability, use `margins`.

## References
Hosmer, D. W., Lemeshow, S. and Sturdivant, R. X. (2013), Applied Logistic Regression, 3rd
edition, Wiley, chapter 5.

Fawcett, T. (2006), An introduction to ROC analysis, Pattern Recognition Letters, 27, 861–874.

## See also
glm, curve, margins, fitted, predict, estimates, statistic

## Examples
// The table at a cutoff of one half: few rows reach it, so few of the ones are found.
> let wages = dataset(wages)
> let g = wages.glm(union ~ educ + exper + exper^2 + female + industry, family: binomial)
> g.confusion()
confusion · g · cutoff 0.5
glm · union ~ educ + exper + exper^2 + female + industry · binomial, logit
N = 18,390: 5,849 with `union` 1 and 12,541 with 0
a row is predicted 1 when its fitted probability is at least 0.5

            predicted 1   predicted 0    total
  union 1           669         5,180    5,849
  union 0           480        12,061   12,541
  ────────────────────────────────────────────
  total           1,149        17,241   18,390

sensitivity 0.1144 · specificity 0.9617 · accuracy 0.6922
positive predictive value 0.5822 · negative predictive value 0.6996

// A lower cutoff finds more of the ones, and takes more noughts with them.
> let c = g.confusion(cutoff: 0.3)
> c
confusion · g · cutoff 0.3
glm · union ~ educ + exper + exper^2 + female + industry · binomial, logit
N = 18,390: 5,849 with `union` 1 and 12,541 with 0
a row is predicted 1 when its fitted probability is at least 0.3

            predicted 1   predicted 0    total
  union 1         3,984         1,865    5,849
  union 0         5,790         6,751   12,541
  ────────────────────────────────────────────
  total           9,774         8,616   18,390

sensitivity 0.6811 · specificity 0.5383 · accuracy 0.5837
positive predictive value 0.4076 · negative predictive value 0.7835

// One share as a number.
> c.statistic("sensitivity")
0.681142

// On rows the fit did not see: fitted to the years up to 2017, the later years classified.
> wages.where(year <= 2017).glm(union ~ educ + exper + exper^2 + female + industry, family: binomial).confusion(newdata: wages.where(year > 2017), cutoff: 0.3)
confusion · union ~ educ + exper + exper^2 + female + industry · cutoff 0.3
N = 4,995: 1,562 with `union` 1 and 3,433 with 0 among the rows of `newdata` (5 left out for NA in a variable of the model)
a row is predicted 1 when its fitted probability is at least 0.3

            predicted 1   predicted 0   total
  union 1         1,216           346   1,562
  union 0         1,971         1,462   3,433
  ───────────────────────────────────────────
  total           3,187         1,808   4,995

sensitivity 0.7785 · specificity 0.4259 · accuracy 0.5361
positive predictive value 0.3816 · negative predictive value 0.8086
