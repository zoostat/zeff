# implied_independencies

## Summary
The independencies a causal graph implies, which data can contradict

## In brief
- `g.implied_independencies()` lists the pairs of variables the graph says are independent, each with the variables that must be held fixed.
- Every line is a prediction: if the data show the two related after all, the graph is wrong somewhere.
- With `data:`, each line is tested on a frame by a partial correlation, and the lines the data contradict are named.
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

Given a frame as `data:`, the function tests each statement on it. Two variables that are
independent given a set have no correlation left once the set is taken out of both, so each
statement is tested by a partial correlation, printed with its interval and its p value. A
statement the data contradict says that the graph is wrong somewhere. It does not say where:
which arrow is missing is not for the data to choose, and the function proposes none.

## Arguments
- g: The causal graph, a `dag { }` block or a name it is kept under.
- data: A frame to test the statements on. Every variable of a statement must be a variable of the frame; a variable that was not measured is declared `latent` in the graph.
- adjust: With `data:`, how the p values are adjusted for the number of statements tested: the methods of `p_adjust`. `none` leaves them as they are.
- level: With `data:`, the confidence level of the intervals; a statement is contradicted when its adjusted p value is below one minus the level.

## Value
A `TestResult` that prints the list as a table of three columns: a variable, the variable it
is independent of, and the variables given. `n_obs` is the number of lines. Without `data:`
nothing is tested, and the result has no estimate and no p-value.

With `data:` the table has, for each statement, the partial correlation `r`, its interval,
its p value, the p value adjusted, and the number of rows used; under it, the statements the
data contradict and those that could not be tested, each with the reason. `statistic()` is
then the number of statements contradicted, `n_obs` the number tested, and `estimates()`
gives the correlations as a table.

## Missing values
Without `data:` the function reads the graph alone. With `data:`, each statement is tested on the rows that have every one of its variables, the two and those given, so different statements may use different rows; the column `N` gives the number for each. A variable that is not observed at all is declared `latent` in the graph, and is then in no line.

## Details
Two variables are independent given a set when the set d-separates them in the graph: every
path between them is blocked, a path being blocked by a variable of the set that is not a
collider on it, or by a collider that is not in the set and has no descendant in it. A
bidirected edge, `a <-> b`, stands for a common cause that is not observed, and the two
variables it joins have an edge between them.

Where several sets of the smallest size separate a pair, one is listed: the first by the
names of its variables. Whether a pair can be separated at all is known from the ancestors of
the two variables, and a smallest set is then looked for among those ancestors.

The partial correlation of two variables given a set is the correlation of what is left of
each after a regression on the set. Its p value is that of the t test, t = r √(ν / (1 − r²))
on ν = n − 2 − k degrees of freedom, k being the number of columns given; it is the test of
the coefficient of one of the two in a regression of the other on it and the set. The
interval is Fisher's, tanh(atanh r ± z / √(n − 3 − k)). With nothing given these are the test
and the interval of an ordinary correlation. A yes or no counts as 1 and 0, and so does a
category of two levels; a category among those given is held fixed level by level. The test
is of a straight-line relation: two variables related in some other way may pass it.

A statement is not tested, and is given no number, when the variables given determine one of
the two, so that nothing of it is left; when one of the two has one value in every row; when
one of the two is a category of more than two levels, text or a date; or when the rows are too
few.

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
| g.implied_independencies(data: d) | dagitty::localTests(g, d, type = "cis") | no command |

dagitty's list of this kind is its default, one statement to a missing edge of the graph. It
has two other lists, chosen by `type =`, which are not built here. Stata has no command for
causal graphs.

## Related tasks
- To write the graph, use `dag`.
- To compare two graphs, use `equal`.
- To adjust p values of your own, use `p_adjust`.

## References
Pearl (2009), Causality: Models, Reasoning, and Inference, 2nd edition, Cambridge University
Press, sections 1.2 and 5.2.
Textor, van der Zander, Gilthorpe, Liśkiewicz and Ellison (2016), Robust causal inference
using directed acyclic graphs: the R package dagitty, International Journal of Epidemiology
45, 1887–1894.

## See also
dag, equal, p_adjust, correlation, adjustment_sets

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

// The graph met with data: the staff of the training study. Every statement holds.
> g.implied_independencies(data: dataset(staff))
implied_independencies · 3 statements tested on staff

  variable   independent of   given               r   95% CI                p   p (holm)       N
  age        region                      not tested
  age        tenure           training       -0.017   [-0.057, 0.023]   0.406      0.406   2,400
  region     tenure           training   not tested

the data do not contradict the one statement tested, at the 5% level, the p values adjusted for one test by `holm`
not tested: age and region: region is a category of 4 levels, and a correlation is of numbers
not tested: region and tenure given training: region is a category of 4 levels, and a correlation is of numbers
latent, in no statement: ability

// What is assumed of wages, tested on the workers of 2019: the data contradict two statements, and three cannot be tested, since experience was computed from age and schooling.
> let w = dataset(wages).where(year == 2019).add(lwage = log(wage))
> let earnings = dag {
>     [female, ability] -> educ
>     [female, educ, exper, union, ability] -> lwage
>     female -> union
>     [age, educ] -> exper
>     latent ability
> }
> earnings.implied_independencies(data: w)
implied_independencies · 7 statements tested on w

  variable   independent of   given                  r   95% CI                p   p (holm)       N
  age        educ                               -0.005   [-0.056, 0.045]   0.837      1.000   1,518
  age        female                             -0.014   [-0.065, 0.036]   0.575      1.000   1,518
  age        union                               0.085   [0.035, 0.135]    0.001      0.006   1,518
  educ       union            female            -0.038   [-0.088, 0.012]   0.139      0.558   1,518
  exper      female           educ              -0.014   [-0.065, 0.036]   0.578      1.000   1,516
  exper      union            educ               0.085   [0.035, 0.135]    0.001      0.006   1,516
  age        lwage            educ, exper   not tested

the data contradict 2 of the 6 statements tested, at the 5% level, the p values adjusted for 6 tests by `holm`: age and union; exper and union given educ
not tested: age and lwage given educ, exper: educ and exper determine age in these rows, so nothing of it is left to test
latent, in no statement: ability

// The p values as they are, and intervals at 99%.
> earnings.implied_independencies(data: w, adjust: none, level: 0.99)
implied_independencies · 7 statements tested on w

  variable   independent of   given                  r   99% CI                p       N
  age        educ                               -0.005   [-0.071, 0.061]   0.837   1,518
  age        female                             -0.014   [-0.080, 0.052]   0.575   1,518
  age        union                               0.085   [0.019, 0.150]    0.001   1,518
  educ       union            female            -0.038   [-0.104, 0.028]   0.139   1,518
  exper      female           educ              -0.014   [-0.080, 0.052]   0.578   1,516
  exper      union            educ               0.085   [0.019, 0.150]    0.001   1,516
  age        lwage            educ, exper   not tested

the data contradict 2 of the 6 statements tested, at the 1% level, the p values not adjusted: age and union; exper and union given educ
not tested: age and lwage given educ, exper: educ and exper determine age in these rows, so nothing of it is left to test
latent, in no statement: ability
