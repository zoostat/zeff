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
(W0209 warns of an unguarded call). `map` and `filter` go over such a list when their function
is a test by `is`: `filter(fits, fn(f) !f.is(Failed))` keeps the fits that succeeded. `error`
may stand inside `attempt`, where it gives the failed value that holds its message:
`attempt(error("no data"))`. A bug inside the engine (an internal error, Z9xxx) is never
caught: it is not the script's failure. Like R's `tryCatch` and Stata's `capture`, with the
failure kept as a value rather than a flag.

## Arguments
- expr: Any expression, evaluated once: a model fit, a chain, a function call.

## Value
The value of `expr`, with its type, or a `Failed` value, which prints as `failed · Z0801 at line
3: ...`.

## A common mistake
Using the result as if the expression had succeeded. A failed value is refused wherever a value is needed, and the error names the failure it holds:

> attempt(d.where(x > 10).reg(y ~ x)).coef()
note: no row kept: the result is empty (line 8)
error[Z0220]: a failed value cannot be used as an argument: it failed with Z0801 at line 8: no row remains to fit the model: the frame has no rows
  = help: test it first with `x.is(Failed)`, or keep it in a list for `report` and `len` to skip with a count (section 5.4)

Test it first, `if f.is(Failed) { ... }`, or keep the results in a list, which `report` and `len` go over, skipping the failed ones with a count.

## Coming from R or Stata
| attempt(expr) | tryCatch(expr, error = function(e) e) | capture noisily command |
| x.is(Failed) | inherits(x, "error") | _rc != 0 |

R's `try` and Stata's `capture` leave a flag to be remembered and tested; here the failure is a
value that stands where the result would, and using it untested is itself an error. There is no
handler: what to do instead is written after the test.

## Related tasks
- To stop the script on purpose, use `error`.
- To put many fits side by side, the failed ones skipped, use `report`.

## See also
is, type_of, report, error, warn

## Examples
// Three fits in a loop, one of them on no rows: the loop carries on.
> let d = frame(x = [1, 2, 3, 4, 5], y = [2, 4, 7, 8, 11])
> let fits = [for cut in [0, 2, 10] {
>     attempt(d.where(x > cut).reg(y ~ x))
> }]
note: no row kept: the result is empty (line 2)

// The third is a failed value, with the error it would have stopped the script with.
> fits[3]
failed · Z0801 at line 3: no row remains to fit the model: the frame has no rows

// The test for one.
> fits[3].is(Failed)
true

// `report` uses the fits that succeeded and says how many did not.
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
warning[W0801]: 1 of 3 fits failed (Z0801 at line 3) and was skipped (line 7)
  = help: the table uses the fits that succeeded; `x.is(Failed)` tells which failed
