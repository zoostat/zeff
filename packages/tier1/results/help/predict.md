# predict

## Summary
Predictions of a model on new data

## Description
Gives the model's prediction for each row of `newdata:` as a column of its length. The model
matrix is rebuilt on the new frame from the fit's formula with the fit's own category levels and
contrasts, so a category coded the same way and a text variable holding the fit's labels both
work; the outcome need not be present. A variable of the model that the frame lacks is an error
(Z0101); a row with `NA` in a model variable predicts `NA`; a row whose level the fit never saw
predicts `NA` with a warning (W0207); a term omitted as collinear contributes nothing. `fitted()`
gives the predictions for the rows the fit used. A test's result gives `NA`. Like `predict` on
new data in R and Stata's `predict`. For a `glm` fit the prediction is a probability, as Stata's
`predict` gives after `logit`; R's `predict` gives the linear predictor unless told `type =
"response"`, which here is `type: link`.

## Arguments
- fit: The result.
- newdata: The frame to predict for, holding the model's variables.
- type: The scale of the prediction: `response`, the mean of the outcome, or `link`, the linear predictor; `response` unless given. For a `glm` fit with a binary outcome, `response` is the probability and `link` its log-odds (or, under the probit link, its normal score). A regression has one scale, and both give the same.

## Value
A `Column<Float>` of the frame's length.

## Details
Each prediction is the row of the new design times the coefficients, x′β, with the design built
by the rules of section 7.2 of the Language Reference on the new frame and the fit's category
levels in the fit's order, so an indicator or contrast column means the same as in the fit. For
a model fitted by maximum likelihood that is the linear predictor, `type: link`; the response is
the link's mean of it, 1/(1 + e^−η) under the logit link and Φ(η) under the probit. No standard
error of prediction is given at this release.

## References
Davidson, R. and MacKinnon, J. G. (2004). *Econometric Theory and Methods*. Oxford University
Press, chapter 3.

## See also
fitted, residuals, coef

## Examples
> let m = dataset(wages).reg(log(wage) ~ educ + exper + female)
> m.predict(newdata: frame(educ = [12, 16], exper = [5, 10], female = [0, 1]))
Column<Float> · 2 values
2.92202
3.20323
> let g = dataset(wages).glm(union ~ educ + exper + female, family: binomial)
> g.predict(newdata: frame(educ = [12, 16], exper = [5, 20], female = [0, 1]))
Column<Float> · 2 values
0.253911
0.353629
> g.predict(newdata: frame(educ = [12, 16], exper = [5, 20], female = [0, 1]), type: link)
Column<Float> · 2 values
-1.07786
-0.603124
