# plot

## Summary
A causal graph drawn as a figure

## In brief
- `g.plot()` draws the graph `g`: its variables in boxes, its arrows between them, laid out in layers from causes to effects.
- `exposure:` and `outcome:` mark the two variables the question is about.
- `layout:` or `at:` puts the variables where the script says; without them the same graph is always drawn the same way.

## Description
A causal graph is easier to read as a picture than as its lines, and a picture is what a
paper prints. The function gives a plot that draws the graph: `export` writes it as a PDF,
an SVG or a PNG, and `labels` and `theme` give it a title and a look, as for any plot.

A variable that is measured is a box. A variable declared `latent` is an ellipse with a
dashed outline. A variable the data are `selected` on has a second outline inside the
first. An arrow is a cause; a dashed arrow with a head at each end is a common cause that is
not observed, `a <-> b`; a plain line is a link for drawing, `a -- b`. The exposure and the
outcome, where the call names them, are filled. A line at the foot of the figure says what
the marks of this figure mean.

The variables are placed in layers, each cause on a layer before its effects, so that the
arrows are short and cross as little as they can. The same graph is placed the same way in
every run and on every machine, whatever order its lines were written in. A figure that is
to look a certain way is placed by the script, with `layout:` or `at:`.

## Arguments
- g: The causal graph, a `dag { }` block or a name it is kept under.
- exposure: The cause the question is about; a node of `g`. It is filled in colour.
- outcome: The effect the question is about; a node of `g`. It is filled dark.
- direction: The way the layers follow: `down` the page, or to the `right`.
- layout: Places for the nodes as a grid, one text to a row: the names of the nodes of the row, with `.` for an empty place. Every node must be on it.
- at: Places for the nodes by name, each a column and a row counted from 0; a part of a step is allowed. Every node must have one.
- same_rank: Sets of nodes to keep on one layer of the automatic layout.
- spacing: The room between two layers and between two neighbours on a layer, in millimetres.

## Value
A `Plot` that draws the graph. It prints as a table of the nodes, each with what it is and
the part it plays, and a line saying how they are placed. `export` writes the figure;
`labels` gives it a title, a subtitle and a caption; `theme` sets its type. The methods that
add marks, scales or panels have no meaning for it and are refused.

## Missing values
The function reads the graph alone, and a graph holds no values that could be missing. A variable that is not observed at all is declared `latent` in the graph, and is drawn dashed.

## Details
The layers are those that make the arrows shortest, and within a layer the variables are
ordered so that the arrows cross as little as they can; a small graph has no crossing it
does not need. A variable stands under the middle of its causes where that can be, and a
long arrow runs as straight as its neighbours allow. Ties are settled by the names of the
variables, never by the order of the lines.

The type is that of the labels of the theme. A graph too large for the page is drawn in
smaller type so that it fits; one that leaves room is not made larger, so that its type
matches the other figures of a paper.

With `layout:` or `at:` the script places every node, and the arrows run straight between
them. A step of the grid is the widest node and the room between neighbours one way, the
tallest node and the room between layers the other. `same_rank:` belongs to the automatic
layout and is refused beside them.

## A common mistake
Placing some of the nodes and leaving the rest to chance. Where the script places the
nodes it places them all:

> let study = dag {
>     age -> training -> productivity
>     age -> productivity
> }
> study.plot(layout: ["age .", "training productivity"], exposure: tenure)
error[Z0101]: `exposure:` names `tenure`, which is no node of the graph
  = help: the nodes of the graph are: age, training, productivity

Every name the call gives is a node of the graph, and every node of the graph has its place.

## Coming from R or Stata
| g.plot() | plot(g) with dagitty, or ggdag::ggdag(g) | no command |
| g.plot(at: {x: [0, 0], y: [1, 0]}) | dagitty::coordinates(g) <- list(x = c(x = 0, y = 1), y = c(x = 0, y = 0)) | no command |

dagitty places the nodes at random unless coordinates are given, so two runs differ; here
one graph has one picture. Stata has no command for causal graphs.

## Related tasks
- To write the graph, use `dag`.
- To write the figure to a file, use `export`.
- To give the figure a title, use `labels`.

## References
Gansner, Koutsofios, North and Vo (1993), A technique for drawing directed graphs, IEEE
Transactions on Software Engineering 19, 214–230.
Brandes and Köpf (2001), Fast and simple horizontal coordinate assignment, Graph Drawing,
Lecture Notes in Computer Science 2265, 31–44.

## See also
dag, export, labels, theme, adjustment_sets, check

## Examples
// The assumptions behind a study of training, with the cause and the effect in question marked.
> let g = dag {
>     ability -> training -> productivity
>     ability -> productivity
>     training -> tenure -> productivity
>     [age, region] -> training
>     latent ability
> }
> g.plot(exposure: training, outcome: productivity)
plot · causal graph · 6 nodes, 7 edges

  node           kind       part
  ability        latent
  training       observed   exposure
  productivity   observed   outcome
  tenure         observed
  age            observed
  region         observed

placed in layers, down the page

// The layers from left to right, with more room between them.
> g.plot(direction: right, spacing: [14, 5])
plot · causal graph · 6 nodes, 7 edges

  node           kind
  ability        latent
  training       observed
  productivity   observed
  tenure         observed
  age            observed
  region         observed

placed in layers, to the right
14 mm between layers, 5 mm between neighbours

// Age and region kept on the layer of ability.
> g.plot(same_rank: [[ability, age, region]])
plot · causal graph · 6 nodes, 7 edges

  node           kind
  ability        latent
  training       observed
  productivity   observed
  tenure         observed
  age            observed
  region         observed

placed in layers, down the page
on one layer: ability, age, region

// The nodes placed by the script on a grid: a text to a row, a dot for an empty place.
> let f = dag {
>     x -> m -> y
>     u -> [x, y]
>     latent u
> }
> f.plot(layout: [". u .", "x m y"])
plot · causal graph · 4 nodes, 4 edges

  node   kind
  x      observed
  m      observed
  y      observed
  u      latent

placed by the script, each node on its place of a grid

// The same places by name, as a column and a row.
> f.plot(at: {u: [1, 0], x: [0, 1], m: [1, 1], y: [2, 1]})
plot · causal graph · 4 nodes, 4 edges

  node   kind
  x      observed
  m      observed
  y      observed
  u      latent

placed by the script, each node on its place of a grid
