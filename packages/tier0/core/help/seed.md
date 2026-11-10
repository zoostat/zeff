# seed

## Summary
Script-wide seed; per-method seeds derive from it

## Description
Sets the seed of the run's random stream, so that every draw in the script (`draw`, and later the
bootstrap, `sample` and the simulations) comes out the same each time the script runs. It is set
once, at the top, before any statement draws: a second `seed`, or one after a draw, is refused
(Z0111), because the draws before it would not be reproduced by the seed printed. Without `seed`
the engine chooses one at the first draw and prints it as a note, so that `seed(N)` repeats the run.
`seed` returns nothing: it is a statement, and its result cannot be bound or passed on (Z0206). Like
R's `set.seed()` and Stata's `set seed`: inserting a statement that draws changes the draws of the
statements after it, and one that draws nothing changes none. Each statement that draws has its own
part of the stream, so two identical statements draw different numbers.

## Arguments
- n: The seed, an Int.

## Value
Nothing.

## Details
The stream is Philox4x32-10 keyed by the seed, with the ordinal of the statement among those that
draw, the pass of a loop and each draw's number as its counter (Engine and Project Specification,
section 8.3, point 3), so the draws are a pure function of the seed and the script, identical on
every machine and in the browser, and never depend on threads. The first statement that draws from
the run's stream has ordinal 1, the next 2, and so on: `seed(n)` itself and every statement that
draws nothing are not counted, which is why the seed noted for a run, written at the top of the
script, repeats it. A loop is one statement whose passes draw apart. A `seed:` given to one `draw`
is a stream of its own, the same wherever the call stands; it leaves the run's seed untouched and is
not counted either.

## References
- Salmon, J. K., Moraes, M. A., Dror, R. O. and Shaw, D. E. (2011). Parallel random numbers: as
  easy as 1, 2, 3. *Proceedings of SC '11*, article 16.

## See also
draw, dist.normal

## Examples
> seed(20261019)
> dist.normal().draw(5)
Column<Float> · 5 values
1.28259
1.27044
1.50841
-0.713328
-0.277729
