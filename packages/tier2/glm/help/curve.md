# curve

## Summary
The ROC curve of a binary model and the area under it

## In brief
- `g.curve(type: roc)`: the ROC curve of the fit on the rows it used, with the area under it.
- `g.curve(newdata: test, type: roc)`: the same on other rows, such as rows kept back from the fit.
- `g.curve(type: roc).plot()` draws the curve; `estimates()` gives its points and `statistic()` the area.

## Description
Says how well the fitted probabilities of a binary model tell the rows with a 1 from the rows
with a 0. Call a row a predicted 1 when its fitted probability is at least some threshold. A low
threshold finds nearly all the ones and wrongly takes many noughts with them; a high one does the
reverse. The ROC curve shows the whole trade: for every threshold, the sensitivity, the share of
the ones found, against one less the specificity, the share of the noughts wrongly taken.

The area under the curve sums it up in one number between 0 and 1: the probability that a row
with a 1, taken at random, has a higher fitted probability than a row with a 0, a tie counting
as half. One half is a model that tells nothing; 1 is a model that puts every one above every
nought.

`type:` names the curve and is required: `roc` at this release, with other curves of a
classifier to come as further words.

## Arguments
- fit: The fit of a binary model, from `glm` with `family: binomial`.
- newdata: A frame with the model's variables and its outcome, to draw the curve on in the place of the rows the fit used.
- type: The curve: `roc`. Required.

## Value
A `TestResult` that prints the rows the curve rests on, how many of them are ones and noughts,
and the area under the curve. `statistic` gives the area as a number. `estimates` gives the
points as a frame, one row for each distinct fitted probability, from the highest threshold to
the lowest: `threshold`, `sensitivity` and `specificity`. `plot` draws the curve over the
diagonal of a model that tells nothing. The result has no p-value and no interval.

## Missing values
On the rows the fit used there is nothing missing. A row of `newdata` with `NA` in the outcome
or in a variable of the model, or with a level of a category the fit never saw, is left out,
and the line of the rows says how many.

## Details
### The points
Rows with the same fitted probability cannot be told apart by any threshold, so the curve has
one point for each distinct fitted probability and no more. At a threshold, the sensitivity
counts the ones at or above it and the specificity the noughts below it. The curve starts at
the corner where nothing is taken for a 1 and ends where everything is.

### The area
The area is taken under the straight lines that join the points, which is the same number as
the Mann–Whitney statistic of the fitted probabilities of the two groups divided by the number
of pairs. Without weights it is a ratio of whole numbers and is exact. It depends on the order
of the fitted probabilities alone, so the logit and the probit of one model give nearly the
same area, and so does any rising change of the scores.

### Weights
After frequency weights each row counts as the number of observations it stands for, as if it
were written that many times. After sampling weights each row counts as its weight, so the
curve describes the population the weights stand for. The rows of `newdata` count once each.

### On the rows of the fit, or on new rows
A model is fitted to do well on its own rows, so the curve on those rows flatters it a little.
The honest measure is on rows the fit did not see: fit the model to part of the data and give
the rest as `newdata:`.

## A common mistake
Leaving out the type:

> dataset(wages).glm(union ~ educ, family: binomial).curve()
error[Z0406]: `curve` needs `type:`
  = help: the ROC curve is `type: roc`

`type:` has no default, so that a script always says which curve it draws.

## Coming from R or Stata
| g.curve(type: roc) | pROC::roc(d$y, fitted(g)) | lroc |
| g.curve(type: roc).statistic() | pROC::auc(pROC::roc(d$y, fitted(g))) | lroc, then r(area) |
| g.curve(newdata: test, type: roc) | pROC::roc(test$y, predict(g, test, type = "response")) | predict p, then roctab y p |
| g.curve(type: roc).plot() | plot(pROC::roc(d$y, fitted(g))) | lroc |

pROC draws the specificity along a reversed axis; here, as in Stata, the horizontal axis is one
less the specificity, and the curve is the same. pROC places its thresholds midway between
neighbouring scores; here a threshold is a fitted probability itself, and a row at the
threshold is taken for a 1, so the points are the same and only their labels differ. Neither
weights the rows unless asked; Stata's `lroc` takes frequency weights as here. An interval for
the area and a test between two curves, which pROC and Stata's `roccomp` give, are not in this
release.

## Related tasks
- To fit the model whose curve this is, use `glm`.
- To see the effect of each regressor on the probability, use `margins`.
- To get each row's fitted probability, use `fitted`; for new rows, `predict`.
- To draw the curve, use `plot` on the result; to write the figure to a file, `export`.

## References
Hanley, J. A. and McNeil, B. J. (1982), The meaning and use of the area under a receiver
operating characteristic (ROC) curve, Radiology, 143, 29–36.

Fawcett, T. (2006), An introduction to ROC analysis, Pattern Recognition Letters, 27, 861–874.

## See also
glm, margins, fitted, predict, plot, estimates, statistic

## Examples
// The ROC curve of a logit of union membership, on the rows the fit used.
> let wages = dataset(wages)
> let g = wages.glm(union ~ educ + exper + exper^2 + female + industry, family: binomial)
> let c = g.curve(type: roc)
> c
curve · g · ROC
glm · union ~ educ + exper + exper^2 + female + industry · binomial, logit
N = 18,390: 5,849 with `union` 1 and 12,541 with 0
area under the curve = 0.6523
the curve has 5,721 points, one for each distinct fitted probability; `estimates()` gives them and `plot()` draws the curve

// The area as a number.
> c.statistic()
0.652306

// The first points of the curve: the highest thresholds, where almost nothing is taken for a 1.
> c.estimates().head(4)
frame · 4 rows × 3 variables

      threshold   sensitivity   specificity
  1    0.737079   0.000170969             1
  2    0.723583   0.000170969       0.99992
  3    0.723265   0.000341939       0.99992
  4    0.709444   0.000512908       0.99992

// On rows the fit did not see: fitted to the years up to 2017, the curve on the later years.
> let later = wages.where(year > 2017)
> wages.where(year <= 2017).glm(union ~ educ + exper + exper^2 + female + industry, family: binomial).curve(newdata: later, type: roc)
curve · union ~ educ + exper + exper^2 + female + industry · ROC
N = 4,995: 1,562 with `union` 1 and 3,433 with 0 among the rows of `newdata` (5 left out for NA in a variable of the model)
area under the curve = 0.6586
the curve has 3,046 points, one for each distinct fitted probability; `estimates()` gives them and `plot()` draws the curve

// The curve as a figure: sensitivity against one less the specificity, over the diagonal.
> c.plot()
plot · sensitivity against false_positive_rate

  axis   title             scale    limits          breaks
  x      1 − specificity   linear   -0.05 to 1.05   0, 0.2, 0.4, 0.6, 0.8, 1
  y      sensitivity       linear   -0.05 to 1.05   0, 0.2, 0.4, 0.6, 0.8, 1

  layer   mark    data            marks
      1   lines   its own frame       2
      2   lines   the frame       5,722

  settings
  x false_positive_rate, y sensitivity, color grey, width 0.5, linetype dashed, 1 line
  width 1, 1 line

caption: area under the curve 0.6523
coordinates: fixed, one unit of `y` as long as one of `x`
