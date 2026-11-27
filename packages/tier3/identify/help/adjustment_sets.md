# adjustment_sets

## Summary
Sufficient adjustment sets from a causal graph

## In brief
- `g.adjustment_sets(exposure: x, outcome: y)` lists the sets of observed variables to hold fixed so that the relation of `x` to `y` is the effect of `x` on `y`.
- `{}` is a set: adjust for nothing. An empty list, with its note, means that no set of observed variables will do.
- `type: all` lists every set that will do, not the smallest ones alone.

## Description
The relation seen in data between an exposure and an outcome mixes the effect of the one on
the other with everything else that ties the two together: a common cause, or a selection of
the sample. A causal graph says which is which. A set of variables is sufficient when, with
its variables held fixed (as regressors, or by matching or weighting), what remains of the
relation is the total effect of the exposure, the effect through every path that leads from it
to the outcome.

The function lists the sufficient sets among the observed variables, by the generalized
adjustment criterion: a set must close every path between the two that is not a causal path,
and must hold no variable that lies on a causal path from the exposure to the outcome or
comes after one. A variable declared `latent` is in no set, since it cannot be held fixed. A
variable the data are `selected` on is held fixed already, in every set, and is not listed.

When no set of observed variables is sufficient the list is empty, and a note says what
stands in the way and what kind of design may still give the effect: an instrument (a cause
of the exposure with no other way to the outcome) or a front-door variable (one through which
the whole effect passes).

## Arguments
- g: The causal graph, a `dag { }` block or a name it is kept under.
- exposure: The cause whose effect is wanted; a node of `g`.
- outcome: The effect; a node of `g`.
- type: `minimal` lists the sufficient sets that have no sufficient set inside them; `all` lists every sufficient set.

## Value
A `List<Vars>`, one set of variables to an item, printed one set to a line in braces; `{}` is
the set of no variables, "adjust for nothing". The list is empty when no set of observed
variables is sufficient, and a note then says why. The sets are in one order whatever order
the graph was written in: by size, then by the names.

## Missing values
The function reads the graph alone, and a graph holds no values that could be missing. A variable that is not observed at all is declared `latent` in the graph, and is then in no set.

## Details
The exposure's arrows that begin a causal path to the outcome are taken out of the graph; a
set is then sufficient when it d-separates the exposure from the outcome in what is left and
holds none of the variables that are on those causal paths or below them. A bidirected edge,
`a <-> b`, stands for a common cause that is not observed.

Whether any set exists is known at once: if one does, the observed ancestors of the exposure,
of the outcome and of the selected variables form one. The minimal sets all lie among those
ancestors, and are found by trying their sets from the smallest. A variable the data are
selected on that lies on a causal path, or below one, leaves no sufficient set at all.

A graph with an undirected edge, `a -- b`, is refused: such an edge is for drawing and says
nothing of cause.

## A common mistake
Asking about a variable that is in the data but was never written in the graph. The graph is
the whole of what the function knows:

> let study = dag {
>     training -> productivity
>     age -> training
>     age -> productivity
> }
> study.adjustment_sets(exposure: experience, outcome: productivity)
error[Z0101]: no node `experience` in the graph
  = help: the nodes of the graph are: training, productivity, age

Write the variable into the graph with its arrows, or name a node the graph has. An empty
list is another thing to read with care: it is not the set `{}`, "adjust for nothing", but
the answer that no set of observed variables will do, and it comes with a note.

## Coming from R or Stata
| g.adjustment_sets(exposure: x, outcome: y) | dagitty::adjustmentSets(g, exposure = "x", outcome = "y") | no command |
| g.adjustment_sets(exposure: x, outcome: y, type: all) | dagitty::adjustmentSets(g, "x", "y", type = "all") | no command |

dagitty has also `type = "canonical"` and `effect = "direct"`, which are not built here. Stata
has no command for causal graphs.

## Related tasks
- To write the graph, use `dag`.
- To list what the graph says the data must show, use `implied_independencies`.
- To estimate with an instrument when no set exists, use `ivreg`.

## References
Shpitser, VanderWeele and Robins (2010), On the validity of covariate adjustment for
estimating causal effects, Proceedings of the 26th Conference on Uncertainty in Artificial
Intelligence, 527–536.
Perković, Textor, Kalisch and Maathuis (2018), Complete graphical characterization and
construction of adjustment sets in Markov equivalence classes of ancestral graphs, Journal of
Machine Learning Research 18(220), 1–62.
van der Zander, Liśkiewicz and Textor (2014), Constructing separators and adjustment sets in
ancestral graphs, Proceedings of the 30th Conference on Uncertainty in Artificial
Intelligence, 907–916.
Textor, van der Zander, Gilthorpe, Liśkiewicz and Ellison (2016), Robust causal inference
using directed acyclic graphs: the R package dagitty, International Journal of Epidemiology
45, 1887–1894.

## See also
dag, implied_independencies, ivreg

## Examples
// A common cause: hold it fixed.
> dag {
>     season -> price -> sales
>     season -> sales
> }.adjustment_sets(exposure: price, outcome: sales)
{ season }

// The assumptions behind a study of training. For the effect of tenure, training is enough.
> let g = dag {
>     ability -> training -> productivity
>     ability -> productivity
>     training -> tenure -> productivity
>     [age, region] -> training
>     latent ability
> }
> g.adjustment_sets(exposure: tenure, outcome: productivity)
{ training }

// For the effect of training nothing observed will do: the list is empty, and the note says why.
> g.adjustment_sets(exposure: training, outcome: productivity)
[]
note: no adjustment set among observed variables: ability is latent and confounds training -> productivity (line 13)
  = help: consider an instrument for training, or a front-door variable

// A collider: adjust for nothing, and `motive` alone would open a path that was closed.
> let m = dag {
>     dose -> recovery
>     [income, health] -> motive
>     income -> dose
>     health -> recovery
> }
> m.adjustment_sets(exposure: dose, outcome: recovery)
{}

// Every sufficient set: `motive` is in one only with income or health beside it.
> m.adjustment_sets(exposure: dose, outcome: recovery, type: all)
{}
{ health }
{ income }
{ health, income }
{ health, motive }
{ income, motive }
{ health, income, motive }

// The number of sets as a value.
> print(len(m.adjustment_sets(exposure: dose, outcome: recovery, type: all)))
7
