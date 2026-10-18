# print

## Summary
Print values

## Description
Prints each value through the display interface that `core` declares and every type implements, so
a frame, a table, a number or a text prints the way it prints when a statement that is not a `let`
stands on its own: an Int in full, a Float to six significant digits, text and Bool as they are,
`NA` as written, a list one item per line, a frame as its header and first ten rows, a table as its
title and rows. Several values print one after another, each on its own lines.

`print` returns nothing: it is a statement, and its result cannot be bound or passed on (Z0206).

## Arguments
- x: The value to print; any type.
- ...: Further values, each printed after the one before.

## Value
Nothing.

## See also
describe, summary

## Examples
> print(42)
42
> print("hello", 0.1)
hello
0.1
> print(true, NA, 0.1, 1e15, [1, 2.5])
true
NA
0.1
1e+15
1
2.5
> let d = frame(x = [1, 2], y = ["a", "b"])
> print(d.n_rows(), d)
2
d · 2 rows × 2 variables

      x   y
  1   1   a
  2   2   b
