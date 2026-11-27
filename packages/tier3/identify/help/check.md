# check

## Summary
A fitted model held against a causal graph

## In brief
- `fit.check(g, exposure: x)` says whether the regressors of the fit, beside `x`, are the right ones to hold fixed for the effect of `x` on the fit's outcome, by the graph `g`.
- A regressor that takes away part of the effect is warned of (W0510); a path that is no effect and is open is warned of and named (W0511).
- The table gives each regressor's part in the graph. The fit is never refused and never changed.

## Description
A regression gives the effect of one of its regressors only if the others are well chosen:
they must close every way by which the exposure and the outcome are related without the one
acting on the other, and they must not stand in the way of the effect itself. Which variables
do which is not in the data. It is in what is assumed of the causes, and a causal graph
writes that down.

The function takes the regressors of the fit, the exposure left out, as an adjustment set
and holds them to the generalized adjustment criterion in the graph. A regressor on a causal
path from the exposure to the outcome, or below a variable on one, removes part of the
effect, and is warned of with the path. A path between the two that is no effect and that the
regressors leave open is warned of and named: a back-door path that none of them closes, or
a path that one of them opens, as a collider on it or a descendant of one. When no set of
observed variables could do, the warning says so, with what stands in the way.

A regressor that does neither is harmless, and draws no warning: a cause of the outcome
alone, or a descendant of the exposure that is on no causal path to the outcome.

## Arguments
- fit: The fitted model, of `reg`, `ivreg` or `glm`. Its outcome and its regressors are read from its formula; the variables of `fe:` are regressors too.
- g: The causal graph, a `dag { }` block or a name it is kept under. The outcome and the exposure must be nodes of it.
- exposure: The regressor whose effect on the outcome the model is meant to estimate.

## Value
A `TestResult` that prints one line saying whether the regressors are a sufficient
adjustment set, and a table of the regressors, each with its part in the graph: the exposure,
a mediator, a collider, one that closes a back-door path, a harmless one, or one that is not
in the graph. `statistic()` is the number of warnings given. Nothing is tested here: the
result has no estimate and no p-value.

## Missing values
The function reads the formula of the fit and the graph, not the data, so it has no values that could be missing. The rows the fit dropped for `NA` are the fit's business and are counted in its own table.

## Details
The first arrow of every causal path from the exposure to the outcome is taken out of the
graph; the regressors are sufficient when they hold no variable on such a path or below one
and d-separate the exposure from the outcome in what is left. A variable the graph declares
`selected` is held fixed in every question, as the data hold it. A bidirected edge,
`a <-> b`, stands for a common cause that is not observed and is written `a ↔ b` in a path.

A regressor that is not a node of the graph is left out of the check and named under the
table: the graph says nothing of it. A regressor the graph declares `latent` is taken as
observed, since the fit holds it, and a note says so. The paths named are the shortest, and
of those the first by the names along them.

For a fit of `ivreg` whose exposure is instrumented, the paths into the exposure are for its
instruments to answer and are not held against the regressors; a regressor that takes away
part of the effect is warned of all the same.

A graph with an undirected edge, `a -- b`, is refused: such an edge is for drawing and says
nothing of cause.

## A common mistake
Adding a variable the exposure acts through as a control, because it explains the outcome
well. The coefficient of the exposure is then what is left of the effect, not the effect:

> let study = dag {
>     training -> productivity
>     training -> tenure -> productivity
> }
> let fit = dataset(staff).reg(productivity ~ training + tenure)
> let held = fit.check(study, exposure: training)
warning[W0510]: tenure lies on the path training → tenure → productivity (line 29)
  = help: adjusting for a mediator removes part of the effect of training; drop tenure to estimate the total effect

For the total effect of training, fit the model without tenure.

## Coming from R or Stata
| fit.check(g, exposure: x) | none built in | no command |

In R the regressors are typed again for dagitty, apart from the model. Stata has no command
for causal graphs.

## Related tasks
- To list the sets that would do, use `adjustment_sets`.
- To write the graph, use `dag`.
- To estimate with an instrument when no set exists, use `ivreg`.

## References
Shpitser, VanderWeele and Robins (2010), On the validity of covariate adjustment for
estimating causal effects, Proceedings of the 26th Conference on Uncertainty in Artificial
Intelligence, 527–536.
Perković, Textor, Kalisch and Maathuis (2018), Complete graphical characterization and
construction of adjustment sets in Markov equivalence classes of ancestral graphs, Journal of
Machine Learning Research 18(220), 1–62.
Cinelli, Forney and Pearl (2024), A crash course in good and bad controls, Sociological
Methods and Research 53(3), 1071–1104.

## See also
adjustment_sets, dag, implied_independencies, reg, ivreg, glm

## Examples
// The assumptions behind a study of training, and a model with tenure among its controls. Tenure is a mediator, and ability is not observed.
> let g = dag {
>     ability -> training -> productivity
>     ability -> productivity
>     training -> tenure -> productivity
>     [age, region] -> training
>     latent ability
> }
> let staff = dataset(staff)
> let m = staff.reg(productivity ~ training + tenure + age + region)
> m.check(g, exposure: training)
check · m · the effect of training on productivity
the regressors are not a sufficient adjustment set for the effect of training on productivity

  regressor   in the graph
  training    the exposure
  tenure      a mediator: removes part of the effect
  age         harmless: closes no path and opens none
  region      harmless: closes no path and opens none
warning[W0510]: tenure lies on the path training → tenure → productivity (line 10)
  = help: adjusting for a mediator removes part of the effect of training; drop tenure to estimate the total effect
warning[W0511]: no sufficient adjustment set exists: ability is latent and confounds training and productivity (line 10)
  = help: consider an instrument for training, or a front-door design

// The same graph asked about tenure: training is held fixed, and that is enough.
> staff.reg(productivity ~ tenure + training).check(g, exposure: tenure)
check · the effect of tenure on productivity
the regressors are a sufficient adjustment set for the effect of tenure on productivity

  regressor   in the graph
  tenure      the exposure
  training    closes a path that is no effect

// A common cause left out of the model: the back-door path is named, and a set that closes it.
> let simple = dag {
>     age -> training -> productivity
>     age -> productivity
> }
> staff.reg(productivity ~ training).check(simple, exposure: training)
check · the effect of training on productivity
the regressors are not a sufficient adjustment set for the effect of training on productivity

  regressor   in the graph
  training    the exposure
warning[W0511]: the regressors leave the back-door path training ← age → productivity open (line 16)
  = help: { age } is a sufficient adjustment set; `adjustment_sets` lists every one

// A collider among the controls: tenure is acted on from both sides, and holding it fixed opens a path that was closed.
> let m_graph = dag {
>     training -> productivity
>     age -> [training, tenure]
>     region -> [tenure, productivity]
> }
> staff.reg(productivity ~ training + tenure).check(m_graph, exposure: training)
check · the effect of training on productivity
the regressors are not a sufficient adjustment set for the effect of training on productivity

  regressor   in the graph
  training    the exposure
  tenure      a collider: opens a path
warning[W0511]: adjusting for tenure opens the path training ← age → tenure ← region → productivity (line 22)
  = help: tenure is a collider on the path, a common effect of the variables on either side of it: drop tenure
  = help: adjusting for nothing is sufficient: {} is an adjustment set, and `adjustment_sets` lists every one

// The number of warnings as a value: none for the model that is right for tenure.
> print(staff.reg(productivity ~ tenure + training).check(g, exposure: tenure).statistic())
0
