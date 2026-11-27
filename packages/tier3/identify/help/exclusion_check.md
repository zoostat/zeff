# exclusion_check

## Summary
An instrument held against a causal graph

## In brief
- `iv.exclusion_check(g, instrument: z)` says whether `z` is a valid instrument by the graph `g`: related to the outcome of the fit through the instrumented regressor alone.
- A path that joins the instrument to the outcome some other way is warned of and named (W0512).
- The table gives the four things asked of the instrument and of the model. The fit is never refused and never changed.

## Description
Instrumental variables give the effect of an exposure when no set of controls can, on one
condition that the data cannot show: the instrument must be related to the outcome through
the exposure and in no other way. This is the exclusion restriction. A weak instrument shows
in the first stage; an instrument that breaks the exclusion restriction shows nowhere, and
the estimate is wrong all the same. Whether the restriction holds is a matter of what is
assumed of the causes, and a causal graph writes that down.

The function takes a fit of `ivreg`, whose instrumented regressor is the exposure, and one of
its instruments. In the graph it asks four things. Is the instrument a descendant of the
exposure? Then it is no instrument. Does a path join it to the outcome that does not pass
through the exposure, with the other regressors of the fit held fixed? Then the restriction
fails, and the path is named. Is one of those regressors itself brought about by the
exposure? Then no instrument is valid beside it. Does the graph relate the instrument to the
exposure at all? If not, it has nothing to work with.

## Arguments
- iv: The fitted model, of `ivreg`. Its outcome, its instrumented regressors and its other regressors are read from it.
- g: The causal graph, a `dag { }` block or a name it is kept under. The outcome, the instrumented regressors and the instrument must be nodes of it.
- instrument: The instrument to check: one of the instruments of the fit.

## Value
A `TestResult` that prints one line saying whether the instrument satisfies the exclusion
restriction in the graph, and a table of the four things asked, each with `yes` or `no`
and, for a path, the path. Under the table are the regressors held fixed.
`statistic()` is the number of warnings given. Nothing is tested here: the result has no
estimate and no p-value.

## Missing values
The function reads the fit's formula and instruments and the graph, not the data, so it has no values that could be missing. The rows the fit dropped for `NA` are counted in the fit's own table.

## Details
The arrows that leave the exposure are taken out of the graph. In what is left, the
instrument must be d-separated from the outcome given the regressors of the fit that are not
instrumented; the variables of `fe:` are among them. A variable the graph declares `selected`
is held fixed too, as the data hold it. A fit with several instrumented regressors has the
arrows out of each removed, and the instrument must be a descendant of none.

A regressor that is a descendant of the exposure is warned of by name, and the question of
the path is asked with the other regressors given. With the arrows out of the exposure gone
such a regressor looks like any other variable, but it is a consequence of the exposure: it
carries whatever confounds the exposure with the outcome, and to hold it fixed as if it were
given makes the estimate wrong whatever the instrument.

The path named is the shortest that is open and, of those, the first by the names along it.
When a regressor of the fit opens it, as a common effect on the path or a descendant of one,
the help says which. A regressor that is not a node of the graph is left out and named under
the table.

Whether the graph relates the instrument to the exposure is asked in the whole graph, the
same regressors given. A graph that does not is no evidence about the first stage, which the
data show: it says that the instrument was not written into the graph as a cause of the
exposure.

A graph with an undirected edge, `a -- b`, is refused.

## A common mistake
Taking for an instrument a variable that also acts on the outcome by another way. The
first stage may be strong all the same:

> let study = dag {
>     offer -> training -> productivity
>     offer -> tenure -> productivity
>     ability -> [training, productivity]
>     latent ability
> }
> let fit = dataset(staff).ivreg(productivity ~ training, instruments: training ~ offer)
> let held = fit.exclusion_check(study, instrument: offer)
warning[W0512]: offer does not satisfy the exclusion restriction: the path offer → tenure → productivity joins it to productivity without passing through training (line 31)
  = help: an instrument may be related to the outcome through the exposure alone: add a regressor that closes the path, or choose another instrument

Holding tenure fixed in the model would close the path, if the graph is right.

## Coming from R or Stata
| iv.exclusion_check(g, instrument: z) | none built in | no command |

dagitty lists the instruments a graph allows and does not check the one a model uses. Stata
has no command for causal graphs.

## Related tasks
- To check the other regressors of the model, use `check`.
- To see the strength of the instrument in the data, use `first_stage`.
- To write the graph, use `dag`.

## References
Angrist, Imbens and Rubin (1996), Identification of causal effects using instrumental
variables, Journal of the American Statistical Association 91, 444–455.
Brito and Pearl (2002), Generalized instrumental variables, Proceedings of the 18th
Conference on Uncertainty in Artificial Intelligence, 85–93.
Pearl (2009), Causality: Models, Reasoning, and Inference, 2nd edition, Cambridge University
Press, section 7.4.5.

## See also
ivreg, check, first_stage, dag, adjustment_sets

## Examples
// An offer of a place on the course moves training and reaches productivity no other way: ability, which is not observed, is why an instrument is needed.
> let g = dag {
>     offer -> training -> productivity
>     ability -> [training, productivity]
>     latent ability
> }
> let staff = dataset(staff)
> let m = staff.ivreg(productivity ~ training, instruments: training ~ offer)
> m.exclusion_check(g, instrument: offer)
exclusion_check · m · offer as an instrument for training
offer satisfies the exclusion restriction in the graph

  the instrument                                in the graph
  no descendant of training                     yes
  reaches productivity through training alone   yes
  no regressor below training                   yes
  related to training                           yes

no regressor is held fixed: the fit has none but the instrumented

// An instrument that is valid only with a regressor held fixed: region moves both the offers and productivity.
> let by_region = dag {
>     offer -> training -> productivity
>     region -> [offer, productivity]
>     ability -> [training, productivity]
>     latent ability
> }
> m.exclusion_check(by_region, instrument: offer)
exclusion_check · m · offer as an instrument for training
offer does not satisfy the exclusion restriction in the graph

  the instrument                                in the graph
  no descendant of training                     yes
  reaches productivity through training alone   no: offer ← region → productivity
  no regressor below training                   yes
  related to training                           yes

no regressor is held fixed: the fit has none but the instrumented
warning[W0512]: offer does not satisfy the exclusion restriction: the path offer ← region → productivity joins it to productivity without passing through training (line 15)
  = help: an instrument may be related to the outcome through the exposure alone: add a regressor that closes the path, or choose another instrument

// The same graph and the model with region in it.
> staff.ivreg(productivity ~ training + region, instruments: training ~ offer).exclusion_check(by_region, instrument: offer)
exclusion_check · offer as an instrument for training
offer satisfies the exclusion restriction in the graph

  the instrument                                in the graph
  no descendant of training                     yes
  reaches productivity through training alone   yes
  no regressor below training                   yes
  related to training                           yes

held fixed, as the fit holds them: region

// A control measured after training and shaped by it: no instrument is valid beside it.
> let later = dag {
>     offer -> training -> tenure -> productivity
>     ability -> [training, productivity]
>     latent ability
> }
> staff.ivreg(productivity ~ training + tenure, instruments: training ~ offer).exclusion_check(later, instrument: offer)
exclusion_check · offer as an instrument for training
offer does not satisfy the exclusion restriction in the graph

  the instrument                                in the graph
  no descendant of training                     yes
  reaches productivity through training alone   yes
  no regressor below training                   no: tenure
  related to training                           yes

held fixed, as the fit holds them: tenure
warning[W0512]: offer is no valid instrument with tenure held fixed: tenure is a descendant of training (line 22)
  = help: what the exposure brings about carries whatever confounds the exposure with productivity, and cannot be held fixed as if it were given: drop tenure from the model

// The number of warnings as a value.
> print(m.exclusion_check(g, instrument: offer).statistic())
0
