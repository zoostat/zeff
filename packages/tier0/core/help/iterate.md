# iterate

## Summary
Iterate to convergence without mutation

## In brief
- `iterate(start: {x: 1.0}, step: f, done: g)`: calls `f` on the state until `g(old, new)` is
  true, and gives the last state.
- `args: {a: 2.0}` passes further arguments to `step` by name.
- `keep: true` gives every state the passes produced, and `max: 50` stops after fifty passes
  with a warning.

## Description
Calls `step` on a state until `done` says it has converged, and returns the final state. Custom EM
algorithms, fixed-point solvers and iterative reweighting need a value that changes from one pass
to the next, which immutable bindings rule out; `iterate` supplies it without mutation: each pass
calls `step(state, ..args)` and takes the record it returns as the next state, then calls
`done(old, new)` and stops when it is `true`. The passes stop at `max:` in any case, with a warning
(W0805), so every script finishes.

The state is a record, and every pass keeps its fields: a `step` that returns a record with a field
the start lacks, or without one it has, is an error at the call (Z0401). The fields `step` and
`done` read from the state are checked where `iterate` is called against the record given as
`start:`, and the record given as `args:` is matched by name against the parameters of `step`
after the state, so a missing or misspelt argument is found before anything runs. The number of
passes is printed as a note under the result. With `keep: true` the result is the list of every
state the passes produced, for a simulated path or a sampler's chain.

Each pass is an interpreted call, which suits algorithms of hundreds of passes such as EM;
samplers of many thousands of draws are the native Bayesian release (1.17), not user code. Like
R's `repeat` with a counter and Stata's `while`, without a variable that is reassigned.

## Arguments
- start: The first state, a record.
- step: A function of the state and the fields of `args:`, returning the next state with the same fields.
- done: A function of the old and the new state, `true` when the passes may stop.
- args: A record whose fields are passed to `step` by name after the state; omitted when `step` takes the state alone.
- max: The most passes to make; stopping here without `done` warns (W0805). At least 1.
- keep: Return every state the passes produced, in order, instead of the last one.

## Value
The final state, a record with the fields of `start:`; with `keep: true`, a list of those records,
one per pass.

## A common mistake
Leaving out `args:` when `step` takes more than the state. The call is checked before anything runs, and the message names what is missing:

> iterate(start: {x: 1.0}, step: newton, done: close)
error[Z0406]: `newton` needs `a`, which `args:` does not supply
  = help: write `args: {a: ...}`; `iterate` passes the fields of `args:` to `newton` by name after the state

Give the further arguments by name: `iterate(start: {x: 1.0}, step: newton, args: {a: 2.0}, done: close)`.

## Coming from R or Stata
| iterate(start: s, step: f, done: g) | repeat { new <- f(s); if (g(s, new)) break; s <- new } | while (!done) { ... } |

R and Stata loop with a variable that is given a new value in each pass. A binding here never
changes, so the state is handed from pass to pass by `iterate`, and the passes always end, at
`done` or at `max:`.

## Related tasks
- To go over the items of a list, use `map`.
- To fold a list into one value, use `reduce`.

## See also
attempt, is, error

## Examples
// Newton's steps to the square root of 2, until two states agree; the passes are counted in a note.
> /// One step of Newton's method for the square root of a.
> fn newton(s: Record, a: Float) -> Record { {x: (s.x + a / s.x) / 2} }
> /// Whether two states agree.
> fn close(old: Record, new: Record) -> Bool { abs(new.x - old.x) < 1e-12 }
> iterate(start: {x: 1.0}, step: newton, args: {a: 2.0}, done: close)
x: 1.41421
note: 6 passes until done (line 5)

// Every state on the way, with `keep: true`.
> iterate(start: {x: 1.0}, step: newton, args: {a: 2.0}, done: close, keep: true)
{x: 1.5}
{x: 1.41667}
{x: 1.41422}
{x: 1.41421}
{x: 1.41421}
{x: 1.41421}
note: 6 passes until done (line 6)

// Stopped at `max:` before `done` was true: the last state, and a warning.
> iterate(start: {x: 1.0}, step: newton, args: {a: 2.0}, done: close, max: 2)
x: 1.41667
warning[W0805]: iterate stopped at max: 2 passes without done becoming true (line 7)
  = help: the state returned is the last one; raise `max:` or loosen `done:`, and `keep: true` shows the path
