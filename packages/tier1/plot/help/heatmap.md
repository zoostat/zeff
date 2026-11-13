# heatmap

## Summary
A heat map of the correlations of several variables with one another

## Description
Computes the correlation of each pair of the variables given and draws them as a square of
tiles, each in the colour of its correlation: `w.heatmap([wage, educ, exper, hours])`. It
shows at a glance which variables move together, which move against each other, and which
are unrelated, before a model is chosen; `correlation` prints the same numbers as a table.

The variables stand down the side and along the foot in the order they are given, the first
at the top and at the left, so variables that belong together can be put side by side. The
colours run from blue at -1 through white at 0 to red at 1, whatever the correlations are,
so that two heat maps can be compared and white always means no linear relation.

It gives a `Plot`, so everything a plot takes may follow it in the chain: `labels`, `theme`,
`export`.

In R this is `corrplot::corrplot(cor(x), method = "color")`. In Stata it is `heatplot` of
the matrix of `correlate`.

## Arguments
- data: The frame whose variables are correlated.
- vars: The variables, two or more, in brackets, in the order they are to stand: numbers.

## Value
A `Plot` of one layer of tiles, with a legend of the colours from -1 to 1. It is drawn where
it is printed and may be bound, extended and exported.

## Details
The correlation is Pearson's, computed for each pair on the rows that have both variables,
as `correlation` computes it by default (`missing: pairwise`). So the pairs may rest on
different rows where values are missing; `drop_missing` before the heat map gives every pair
the same rows. The diagonal is 1, each variable with itself.

What it does not show. A correlation measures a relation along a straight line: a strong
curved relation can have a correlation near 0, and one far value can make or hide one. The
colour says nothing of how many rows a correlation rests on, nor whether it differs from 0 by
more than chance; `correlation(vars, p_values: true)` prints the numbers of rows and the
p-values, and `scatter` shows a pair as it is.

Reading the colours. The square is the same above and below its diagonal, each pair being
there twice. The eye compares colours less exactly than lengths, so a heat map shows the
pattern, and the table gives the values.

How it is built. The frame drawn has a row for each pair, with the variables `row`, `column`
and `correlation`; the plot is `plot(x: column, y: row, fill: correlation).tile()`, with the
scale of the fill fixed from -1 to 1. `scale(fill: ...)` after it changes the colours and
keeps the two ends.

A variable that does not vary, or has fewer than two values, has no correlation with
anything, not even itself: its tiles are left empty, a note names it, and the note of the
layer counts the tiles not drawn as rows of the frame of the correlations.

## References
Friendly (2002), Corrgrams: exploratory displays for correlation matrices, The American
Statistician 56, 316–324.
Anscombe (1973), Graphs in statistical analysis, The American Statistician 27, 17–21.

## See also
correlation, scatter, tile, scale, plot

## Examples
> let d = frame(a = [1, 2, 3, 4, 5], b = [2, 4, 5, 4, 5], c = [5, 3, 4, 2, 1])
> d.heatmap([a, b, c])
plot · row against column

  axis   title   scale      limits       breaks
  x              category   0.4 to 3.6   a, b, c
  y              category   0.4 to 3.6   c, b, a

  aesthetic   title         scale    values    shown as
  fill        correlation   linear   -1 to 1   a ramp through #2B5C9E, #F4F4F4, #B5352B

  layer   mark   data        marks   settings
      1   tile   the frame       9   alpha 1
