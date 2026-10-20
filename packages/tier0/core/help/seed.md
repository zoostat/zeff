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
R's `set.seed()` and Stata's `set seed`, with one difference: each statement draws from its own part
of the stream, so inserting a statement that does not draw never changes the draws after it.

## Arguments
- n: The seed, an Int.

## Value
Nothing.

## Details
The stream is Philox4x32-10 keyed by the seed, with each statement's position in the run and each
draw's number as its counter (Engine and Project Specification, section 8.3, point 3), so the
draws are a pure function of the seed and the script, identical on every machine and in the browser,
and never depend on threads or on what was drawn before. A `seed:` given to one `draw` is a stream
of its own, the same wherever the call stands, and leaves the run's seed untouched.

## References
- Salmon, J. K., Moraes, M. A., Dror, R. O. and Shaw, D. E. (2011). Parallel random numbers: as
  easy as 1, 2, 3. *Proceedings of SC '11*, article 16.

## See also
draw, dist.normal

## Examples
> seed(20261019)
> dist.normal().draw(3)
Column<Float> · 3 values
-0.317525
0.708477
-0.597696
