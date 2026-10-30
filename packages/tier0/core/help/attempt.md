# attempt

## Summary
Run an expression and carry on if it fails

## Description
Evaluates `expr`. If it succeeds, its value is returned unchanged. If it stops with an error,
`attempt` returns a `Failed` value holding the error's code, message and line instead of ending
the script. It is the one way to continue after an error: Zeff has no try/catch, because an
analysis that fails at the top level should stop where it fails, and it is repeated work that
needs protection: one fit out of hundreds that cannot be estimated should not throw the rest
away.

A `Failed` value may sit in any list, so `[for ...]` with `attempt` gives a list in which some
entries failed; `report` and `len` skip them and say how many (W0801). Using a `Failed` value
anywhere else, as an argument, in arithmetic, in a record or as a model input, is an error that
reports the original failure (Z0220). `x.is(Failed)` tells which values failed, and after
`if x.is(Failed) { continue }` the checker knows that `x` succeeded for the rest of the block
(W0209 warns of an unguarded call). A bug inside the engine (an internal error, Z9xxx) is never
caught: it is not the script's failure. Like R's `tryCatch` and Stata's `capture`, with the
failure kept as a value rather than a flag.

## Arguments
- expr: Any expression, evaluated once: a model fit, a chain, a function call.

## Value
The value of `expr`, with its type, or a `Failed` value, which prints as `failed · Z0801 at line
3: ...`.

## See also
is, type_of, report, error, warn

## Examples
> let d = frame(x = [1, 2, 3, 4, 5], y = [2, 4, 7, 8, 11])
> let fits = [for cut in [0, 2, 10] {
>     attempt(d.where(x > cut).reg(y ~ x))
> }]
note: no row kept: the result is empty (line 2)
> fits[3]
failed · Z0801 at line 3: no row remains to fit the model: the frame has no rows
> fits[3].is(Failed)
true
> report(fits)
                    (1)         (2)
  (intercept)   -0.200       0.667
                (0.542)     (2.357)
  x              2.200***    2.000
                (0.163)     (0.577)
  ───────────────────────────────────
  SEs           classical   classical
  N                     5           3
  R²                0.984       0.923

* p < 0.05, ** p < 0.01, *** p < 0.001
warning: 1 of 3 fits failed (Z0801 at line 3) and was skipped (line 7)
  = help: the table uses the fits that succeeded; `x.is(Failed)` tells which failed
