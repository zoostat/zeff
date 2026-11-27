# instruments

## Summary
The instruments a causal graph allows for an effect

## In brief
- `g.instruments(exposure: x, outcome: y)` lists the observed variables that the graph allows as instruments for the effect of `x` on `y`.
- A variable with others under `given` is a conditional instrument: valid only with those held fixed in the model.
- A graph that allows none says so.

## Description
When something that is not observed acts on both the exposure and the outcome, no choice of
controls gives the effect, and `adjustment_sets` returns an empty list. An instrument may
still give it: a variable that moves the exposure and is related to the outcome in no other
way. Whether a variable is one cannot be read from the data. It is a matter of what is
assumed of the causes, which the graph writes down.

The function goes through the observed variables of the graph and lists each one that
would do, with the smallest set of other observed variables that must be held fixed for it to
do. A variable listed with nothing given is an instrument as it stands. One listed with a set
is a conditional instrument: it is related to the outcome by some other way too, and the set
closes that way.

Each line is a candidate for `instruments:` of `ivreg`, with the variables given as
regressors of the model. How strong the instrument is in the data is another question, which
`first_stage` answers.

## Arguments
- g: The causal graph, a `dag { }` block or a name it is kept under.
- exposure: The cause whose effect is wanted; a node of `g`.
- outcome: The effect; a node of `g`.

## Value
A `TestResult` that prints the list as a table of two columns: the instrument, and the
variables given, empty for an instrument that needs none. The lines are in one order whatever
order the graph was written in: those with nothing given first, then by the names.
`statistic()` and `n_obs` are the number of instruments. Nothing is tested here: the result
has no estimate and no p-value.

## Missing values
The function reads the graph alone, and a graph holds no values that could be missing. A variable that is not observed at all is declared `latent` in the graph, and is then no instrument and in no set.

## Details
A variable is an instrument given a set when four things hold. It is no descendant of the
exposure. The set holds no descendant of the exposure or of the outcome. In the graph without
the arrows that leave the exposure, the set d-separates the variable from the outcome. And in
the whole graph the set does not d-separate it from the exposure. These are the conditions
`exclusion_check` asks of an instrument in a fitted model, with the last made a condition.

The sets are tried from the smallest, and of one size in the order of the names, so each
instrument is listed with one set, the first of the smallest; other sets may do as well. A
variable the data are `selected` on is held fixed in every question; when the data are
selected on something the exposure or the outcome brings about, nothing is an instrument. A
bidirected edge, `a <-> b`, stands for a common cause that is not observed.

A graph with an undirected edge, `a -- b`, is refused.

## A common mistake
Asking for instruments for a variable that is in the data but was never written in the
graph. The graph is the whole of what the function knows:

> let study = dag {
>     offer -> training -> productivity
>     ability -> [training, productivity]
>     latent ability
> }
> study.instruments(exposure: training, outcome: wage)
error[Z0101]: no node `wage` in the graph
  = help: the nodes of the graph are: offer, training, productivity, ability

Write the variable into the graph with its arrows, or name a node the graph has.

## Coming from R or Stata
| g.instruments(exposure: x, outcome: y) | dagitty::instrumentalVariables(g, exposure = "x", outcome = "y") | no command |

dagitty removes the arrow from the exposure to the outcome where Zeff removes every arrow
that leaves the exposure, so the two can differ on a graph with a mediator; they have not been
run side by side. Stata has no command for causal graphs.

## Related tasks
- To estimate with an instrument, use `ivreg`.
- To check the instrument of a fitted model against the graph, use `exclusion_check`.
- To see whether controls alone would do, use `adjustment_sets`.

## References
Brito and Pearl (2002), Generalized instrumental variables, Proceedings of the 18th
Conference on Uncertainty in Artificial Intelligence, 85–93.
van der Zander, Textor and Liśkiewicz (2015), Efficiently finding conditional instruments
for causal inference, Proceedings of the 24th International Joint Conference on Artificial
Intelligence, 3243–3249.
Pearl (2009), Causality: Models, Reasoning, and Inference, 2nd edition, Cambridge University
Press, section 7.4.5.

## See also
ivreg, exclusion_check, adjustment_sets, dag, first_stage

## Examples
// Ability is not observed, so no controls give the effect of training; the offer of a place and the distance to the course are instruments.
> let g = dag {
>     [offer, distance] -> training -> productivity
>     ability -> [training, productivity]
>     latent ability
> }
> g.instruments(exposure: training, outcome: productivity)
instruments · 2 for the effect of training on productivity

  instrument   given
  distance
  offer

latent, so no instrument and in no set: ability

// An instrument that needs a variable held fixed: region moves both the offers and productivity.
> dag {
>     offer -> training -> productivity
>     region -> [offer, productivity]
>     ability -> [training, productivity]
>     latent ability
> }.instruments(exposure: training, outcome: productivity)
instruments · 1 for the effect of training on productivity

  instrument   given
  offer        region

an instrument with variables under `given` is valid only with those held fixed in the model
latent, so no instrument and in no set: ability

// A graph that allows none.
> dag {
>     age -> training -> productivity
>     age -> productivity
> }.instruments(exposure: training, outcome: productivity)
instruments · 0 for the effect of training on productivity
the graph allows no instrument among its observed variables for the effect of training on productivity

// The number of instruments as a value.
> print(g.instruments(exposure: training, outcome: productivity).statistic())
2
