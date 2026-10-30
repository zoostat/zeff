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

## See also
len, first, last, reversed

## Examples
> seq(1, 5)
1
2
3
4
5
> seq(0, 1, step: 0.25)
0
0.25
0.5
0.75
1
> seq(10, 0, step: -5)
10
5
0
> len(seq(0, 1, step: 0.1))
11
