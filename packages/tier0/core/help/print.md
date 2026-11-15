# print

## Summary
Print values

## In brief
- `print("Mean wage:", 24.35)`: one line, `Mean wage: 24.35`.
- `print(w.head(3))`: a frame, a table or a result prints as its own block.
- `sep: "\t"` puts a tab between the values of a line in the place of the space.

## Description
Prints each value through the display interface that `core` declares and every type
implements, so a value prints the way it prints when a statement that is not a `let` stands
on its own: an Int in full, a Float to six significant digits, text and Bool as they are, `NA`
as written.

Plain values given side by side, a number, a text, a Bool, a date, a time or `NA`, share one
line, with `sep:` between them, a space unless given. Any other value, a frame, a list, a
record, a column, a result, a table or a plot, ends the line and prints as its own block, as it
does alone, and the plain values after it begin a new line. Every line ends, so a script never
writes `"\n"`.

`print` returns nothing: it is a statement, and its result cannot be bound or passed on
(Z0206).

## Arguments
- x: The value to print; any type.
- ...: Further values, printed after it: on the same line when both are plain values.
- sep: The text between the plain values of one line; a space unless given.

## Value
Nothing.

## Missing values
`NA` prints as `NA`, on the line with the plain values around it.

## Details
### What shares a line
An Int, a Float, a Str, a Bool, a Date, a DateTime and `NA`. A column is a block even when
it holds one value, and so is a list of one item: the line is for values that are single by
their type.

### Numbers on a line
Numbers keep the printing rules of section 3.8 of the Language Reference, six significant
digits for a Float. `format` shapes a number when more is wanted:
`print("p =", format(p, digits: 3))`.

### One value
A single value prints exactly as it did before values shared a line. Until release 0.7.0
every value printed on lines of its own, so `print(1, 2)` gave two lines where it now gives
`1 2`.

## A common mistake
Naming the separator `separator:` or giving it as the last value. The option is `sep:`, and a
text given by place is printed like any other value:

> print("n", "mean", separator: "\t")
error[Z0404]: `print` has no argument called `separator`
  = help: usage: `print(x, ..., sep: Str = " ")`

Write `print("n", "mean", sep: "\t")`.

## Coming from R or Stata
| print("Mean:", m) | cat("Mean:", m, "\n") | display "Mean: " m |
| print("a", "b", sep: "\t") | cat("a", "b", sep = "\t"); cat("\n") | display "a" _tab "b" |
| print(d) | print(d) | list |

R's `print` takes one value and `cat` several; here one function does both, and every line
ends without `"\n"`. R's `paste` builds the text without printing it, as `format` and `+` on
texts do here.

## Related tasks
- To shape a number before printing it, use `format`.
- To see a frame's variables and their types, use `describe`.
- To print the first rows of a frame, use `head`.
- To write a table or a figure to a file, use `export`.

## See also
format, describe, head, export

## Examples
// One value, as a statement that is not a `let` prints it.
> print(42)
42

// Plain values side by side share one line.
> print("Extra sleep, in hours:", 1.58)
Extra sleep, in hours: 1.58

// A Bool, `NA`, numbers and a date on one line, each printed as it prints alone.
> print(true, NA, 0.1, 1e15, date("2026-11-16"))
true NA 0.1 1e+15 2026-11-16

// A tab between the values in the place of the space.
> print("n", "mean", "sd", sep: "\t")
n	mean	sd

// A frame ends the line and prints as its own block.
> let d = frame(x = [1, 2], y = ["a", "b"])
> print("Rows:", d.n_rows(), d)
Rows: 2
d · 2 rows × 2 variables

      x   y
  1   1   a
  2   2   b

// A list is a block too, and the plain value after it begins a new line.
> print("Scores", [1, 2.5], "done")
Scores
1
2.5
done
