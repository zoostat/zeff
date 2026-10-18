# print

## Summary
Print values

## Description
Prints each value through the display interface that `core` declares and every type implements, so
a frame, a table, a number or a text prints the way it prints when a statement that is not a `let`
stands on its own. Several values print one after another, each on its own lines.

`print` returns nothing: it is a statement, and its result cannot be bound or passed on (Z0206).

## Arguments
- x: The value to print; any type.

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
