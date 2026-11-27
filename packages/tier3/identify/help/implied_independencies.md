# implied_independencies

## Summary
The independencies a causal graph implies, which data can contradict

## In brief
- `g.implied_independencies()` lists the pairs of variables the graph says are independent, each with the variables that must be held fixed.
- Every line is a prediction: if the data show the two related after all, the graph is wrong somewhere.
- Latent variables are in no line.

## Description
A causal graph says more than which variable acts on which. Wherever two variables have no
edge between them, it says that they are independent once the right other variables are held
fixed. These statements are what the graph risks: each can be compared with data, and a graph
that survives none of them is not a description of those data.

The function lists one statement for each pair of observed variables that have no edge between
them and that some set of observed variables separates. Each line names the two variables and
a smallest set of variables given which they are independent; an empty set means that they are
independent as they stand. The lines are in one order whatever order the graph was written in:
those with nothing given first, then by the names.

A variable declared `latent` is not observed, so it is in no line, neither as one of the two
nor among those given; a pair that only a latent variable could separate is left out. A
variable the data are `selected` on is held fixed in every line, since the data hold it fixed,
and is named once under the list.

## Arguments
- g: The causal graph, a `dag { }` block or a name it is kept under.

## Value
A `TestResult` that prints the list as a table of three columns: a variable, the variable it
is independent of, and the variables given. `n_obs` is the number of lines. Nothing is tested
here: the result has no estimate and no p-value.

## Missing values
The function reads the graph alone, and a graph holds no values that could be missing. A variable that is not observed at all is declared `latent` in the graph, and is then in no line.

## Details
Two variables are independent given a set when the set d-separates them in the graph: every
path between them is blocked, a path being blocked by a variable of the set that is not a
collider on it, or by a collider that is not in the set and has no descendant in it. A
bidirected edge, `a <-> b`, stands for a common cause that is not observed, and the two
variables it joins have an edge between them.

Where several sets of the smallest size separate a pair, one is listed: the first by the
names of its variables. Whether a pair can be separated at all is known from the ancestors of
the two variables, and a smallest set is then looked for among those ancestors.

A graph with an undirected edge, `a -- b`, is refused: such an edge is for drawing and says
nothing of cause, so the graph does not say what it implies.

## A common mistake
Asking a graph that still has an undirected edge, drawn before the direction was decided:

> let sketch = dag {
>     price -- demand
>     season -> demand
> }
> sketch.implied_independencies()
error[Z0401]: the graph has an undirected edge, `--`, so `implied_independencies` cannot say what it implies
  = help: an undirected edge is for drawing and says nothing of cause: write `a -> b` for a cause, or `a <-> b` for a common cause that is not observed

Decide the edge: `price -> demand` for a cause, or `price <-> demand` for a common cause that
is not observed.

## Coming from R or Stata
| g.implied_independencies() | dagitty::impliedConditionalIndependencies(g) | no command |

dagitty's list of this kind is its default, one statement to a missing edge of the graph. It
has two other lists, chosen by `type =`, which are not built here. Stata has no command for
causal graphs.

## Related tasks
- To write the graph, use `dag`.
- To compare two graphs, use `equal`.

## References
Pearl (2009), Causality: Models, Reasoning, and Inference, 2nd edition, Cambridge University
Press, sections 1.2 and 5.2.
Textor, van der Zander, Gilthorpe, Liśkiewicz and Ellison (2016), Robust causal inference
using directed acyclic graphs: the R package dagitty, International Journal of Epidemiology
45, 1887–1894.

## See also
dag, equal

## Examples
// A chain: the two ends are independent once the middle is held fixed.
> dag { x -> m -> y }.implied_independencies()
implied_independencies · 1 among 3 observed variables

  variable   independent of   given
  x          y                m

// The assumptions behind a study of training. Ability is latent, so nothing observed separates age or region from productivity, and three statements are left.
> let g = dag {
>     ability -> training -> productivity
>     ability -> productivity
>     training -> tenure -> productivity
>     [age, region] -> training
>     latent ability
> }
> g.implied_independencies()
implied_independencies · 3 among 5 observed variables

  variable   independent of   given
  age        region
  age        tenure           training
  region     tenure           training

latent, in no statement: ability

// A sample of the employed only: the data are selected on a variable, and it is in no line.
> dag {
>     educ -> skill -> wage
>     skill -> employed
>     union -> wage
>     selected employed
> }.implied_independencies()
implied_independencies · 3 among 4 observed variables

  variable   independent of   given
  educ       union
  skill      union
  educ       wage             skill

the data are selected on employed: every statement is of data so selected

// The number of statements as a value.
> print(g.implied_independencies().statistic())
3
