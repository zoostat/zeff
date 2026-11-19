# seq

## Summary
Numeric sequence with any step

## Description
The numbers from `from` towards `to` in steps of `step` (1 unless given). Whole-number steps are
also written `2..5`, which is `seq(2, 5)`. With Ints throughout the values are Ints and exact;
with any Float they are Floats. The count is fixed first, from the arithmetic: the number of
values is `trunc((to - from) / step + 1e-10) + 1`, and each value is `from + i × step` computed
afresh, never by adding the step to the previous value, so that `seq(0, 1, step: 0.1)` has 11
values and no error builds up along it. A value that the tolerance lets past `to` is brought
back to `to`. A step of 0, a step that points away from `to`, an argument that is `NA` or
infinite, and a sequence of more than ten million values are errors (Z0418). Unlike R's
`seq(5, 1)`, which counts down by itself, `seq(5, 1)` here has the default step 1 and is an
error: count down with `step: -1`. Like R's `seq(from, to, by)`, to the bit.

## Arguments
- from: The first value.
- to: The value to reach; the last value does not pass it.
- step: The distance between values, with the sign of `to - from`; 1 unless given.

## Value
A list of Ints when `from`, `to` and `step` are all Ints, otherwise a list of Floats.

## A common mistake
Counting down without saying so, as R's `seq(5, 1)` does by itself. The step is 1 unless given, and 1 never reaches a smaller number:

> seq(5, 1)
error[Z0418]: `seq` cannot reach `to` with this `step`
  = help: the step has the sign of `to - from`: `seq(5, 1, step: -1)` counts down, and `seq(1, 5)` counts up

Give the step its sign: `seq(5, 1, step: -1)`.

## Coming from R or Stata
| seq(1, 5) | seq(1, 5) or 1:5 | numlist "1/5" |
| seq(0, 1, step: 0.25) | seq(0, 1, by = 0.25) | numlist "0(0.25)1" |
| seq(5, 1, step: -1) | seq(5, 1) or 5:1 | numlist "5(-1)1" |

R counts down by itself when `to` is below `from`; here that is an error, since a loop over
`seq(1, n)` with `n` at 0 would otherwise run twice, over 1 and 0. The values agree with R's to
the last bit.

## Related tasks
- To go over the values, use `map`.

## See also
len, first, last, reversed

## Examples
// Whole numbers, a step of 1.
> seq(1, 5)
1
2
3
4
5

// A step with a fraction.
> seq(0, 1, step: 0.25)
0
0.25
0.5
0.75
1

// Counting down, with the step's sign given.
> seq(10, 0, step: -5)
10
5
0

// Eleven values from 0 to 1, with no error built up along the way.
> len(seq(0, 1, step: 0.1))
11
