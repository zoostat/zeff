# dag

## Summary
A causal graph, written as a block of arrows

## In brief
- `dag { a -> b }` states that `a` is a cause of `b`; each line of the block is an edge or a chain of edges.
- `a <-> b` is an unobserved common cause, `[a, b] -> c` several causes at once, and `latent u` a variable that is not observed.
- The block is a value: keep it with `let`, print it, compare two with `equal`.

## Description
A `dag { }` block states the causal assumptions behind an analysis: which variable is taken to
act on which. It is written once, as code, so that the assumptions are in the script beside the
model they belong to, and can be checked against it.

Each line of the block is an edge between two nodes, or a chain of them, `a -> b -> c`, which is
the edge from `a` to `b` and the edge from `b` to `c`. A node is a name, or several names in
square brackets, which stand for each of them: `[a, b] -> c` is the edge from `a` and the edge
from `b`. A name written alone on a line is a node with no edge. Two further lines declare
nodes: `latent a, b` names variables that are not observed, and `selected a, b` variables the
data are conditioned on by design, such as "employed" in a sample of workers.

The block is a value of type `Dag`. It prints as a block that reads back as the same graph,
one edge to a line. Two graphs are the same graph, and `equal` says so, when they have the same
nodes, edges and declarations, whatever order their lines are written in. The names in a
graph are not looked up when it is written: they are checked against a frame only when the
graph is used with one.

`dag` is a reserved word and not a function: it is followed by braces, not parentheses.

## Arguments


## Value
A `Dag`: the nodes in the order first written, the edges in the order first written, each
once, and the two declarations.

## Details
There are three links. `a -> b` is a direct causal effect of `a` on `b`. `a <-> b` says that
the two share a cause that is not in the graph; it is the same link whichever side is written
first. `a -- b` is a link without direction, for drawing only. An edge written twice is one
edge.

A graph of causes has no directed cycle: following the arrows from a node never leads back to
it. A block that has one is refused, and the message names the cycle from its first node round
to it again. An edge from a node to itself is refused, and so is a node declared both `latent`
and `selected`, since data cannot be selected on what is not observed (Z0508).

The lines of a block end at the end of the line; there is no comma between them. A name that
is not letters, digits and `_`, or that is a reserved word, is written in backticks,
`` `gross sales` -> profit ``, as anywhere else. `latent` and `selected` are ordinary names
where an edge follows them.

## A common mistake
Writing feedback between two variables as arrows both ways. That is a cycle, and no graph of
causes:

> let market = dag {
>     price -> demand
>     demand -> price
> }
error[Z0508]: the `dag { }` block has a directed cycle: price -> demand -> price
  = help: a causal graph has no cycle: a variable cannot be a cause of itself. For two variables that move together write `a <-> b`; for feedback over time write one variable for each period

If the two move together because of something not in the graph, write `price <-> demand`. If
each acts on the other later, write a node for each period, `price_1 -> demand_2`.

## Coming from R or Stata
| dag { a -> b } | dagitty::dagitty("dag { a -> b }") | no command |
| dag { a <-> b } | dagitty::dagitty("dag { a <-> b }") | no command |

In R the graph is text given to `dagitty`, read when the script runs; here it is code, checked
with the rest of the script before anything runs. dagitty marks a variable as latent or
adjusted inside square brackets after its name; here `latent` and `selected` are lines of the
block. Stata has no command for causal graphs.

## Related tasks
- To compare two graphs, use `equal`.
- To ask what a value is, use `type_of`.

## See also
equal, type_of

## Examples
// A graph of one edge.
> dag { a -> b }
dag {
    a -> b
}

// The assumptions behind a study of training: ability is not observed, and acts on both training and productivity.
> let g = dag {
>     ability -> training -> productivity
>     ability -> productivity
>     training -> tenure -> productivity
>     [age, region] -> training
>     latent ability
> }
> g
dag {
    ability -> training
    training -> productivity
    ability -> productivity
    training -> tenure
    tenure -> productivity
    age -> training
    region -> training
    latent ability
}

// The other links and lines: a common cause that is not observed, a link for drawing, a node alone, a name in backticks, and data selected on a variable.
> dag {
>     price <-> demand
>     demand -- season
>     weather
>     `ad spend` -> demand
>     selected listed
> }
dag {
    price <-> demand
    demand -- season
    `ad spend` -> demand
    weather
    selected listed
}

// The same graph with its lines in another order is the same graph.
> let h = dag {
>     [age, region] -> training
>     training -> tenure -> productivity
>     ability -> [training, productivity]
>     training -> productivity
>     latent ability
> }
> equal(g, h)
true

// What a graph is, and a list of two.
> print(type_of(g), is(g, Dag), len([g, h]))
Dag true 2
