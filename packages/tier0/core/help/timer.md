# timer

## Summary
Run an expression and note the time it took

## Description
Evaluates `expr` and gives its value back unchanged, with a note of the time that passed while
it ran. Wrap the call whose time is wanted: `let m = timer(d.reg(y ~ x))` binds the fit as
`let m = d.reg(y ~ x)` does, and notes how long the regression took. An expression that gives
nothing, such as an `export`, is timed in the same way.

The time is the time on the clock from the moment the expression starts to the moment it ends,
so it includes whatever else the machine was doing. It is for finding which statement of a
script is the slow one, not for comparing two programs to the millisecond.

`timer` reads the clock, so it is allowed only in a script and not inside a function, whose
result must depend on its arguments alone.

## Arguments
- expr: The expression to run and time.

## Value
The value of `expr`, of its own type.

## Details
The note gives seconds to the millisecond under a minute (`elapsed time 0.412 s`), minutes
and seconds under an hour (`elapsed time 2 min 3.4 s`), and hours, minutes and seconds beyond
(`elapsed time 1 h 2 min 3 s`). If the expression stops with an error, the script stops as it
would have and no time is noted.

A script that reads the clock does not print the same thing twice. Where the output of a run
is kept to be compared with a later run, leave `timer` out.

## A common mistake
Timing inside a function:

> /// A regression of log wages on schooling.
> fn slow(d: Frame) -> RegFit {
>     timer(d.reg(log(wage) ~ educ))
> }
error[Z0110]: `timer` is allowed only in a script, not inside the function `slow`
  = help: a function's result depends on its arguments alone, so it may be cached; time the call of the function in the script: `timer(f(x))`

Time the call from the script instead: `timer(slow(wages))`.

## Coming from R or Stata
| timer(expr) | system.time(expr) | timer on 1, the command, timer off 1, timer list 1 |

R's `system.time` gives the times and throws the value away unless it is assigned inside the
call; here the value comes back and the time is a note. Stata's `timer` is switched on and off
around the commands and read afterwards.

## Related tasks
- To carry on after an expression that fails, use `attempt`.
- To read the time on the clock itself, use `now`; for the date, `today`.

## See also
attempt, now, today

## Examples
// A regression timed: the fit is bound as usual, and the note gives the time (nought here, on the fixed clock these examples run with).
> let wages = dataset(wages)
> let m = timer(wages.reg(log(wage) ~ educ + exper))
note: elapsed time 0.000 s (line 2)

// The value comes back unchanged, so the call can stand wherever the expression could.
> timer(sqrt(2))
1.41421
note: elapsed time 0.000 s (line 3)
