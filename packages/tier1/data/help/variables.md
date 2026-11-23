# variables

## Summary
List of variable names

## Description
The names of a frame's variables, in the frame's order, as a list of `Str`. They are the names as
the frame has them: the header of the file for an imported frame (made snake_case when `import` was
given `names: snake`), the names written for a frame built with `frame`. The list prints one name
per line, up to 20, then the count of the rest.

The result is text, not a `Vars`: to name a variable for a frame method, write
it directly, as in `wages.mean(wage)`.

## Arguments
- data: The frame.

## Value
A `List<Str>`, empty for a frame with no variables.

## Missing values
The names are given whatever the variables hold; a name is never missing.

## Coming from R or Stata
| d.variables() | names(d) | ds |

In R the names can be assigned to, `names(d)[2] <- "pay"`; here a frame is not changed in place,
and `rename` gives a frame with the new name.

## Related tasks
- For the number of variables, use `n_cols`.
- For the type and the missing values of each variable, use `describe`.
- To give a variable another name, use `rename`.
- To keep some of the variables, use `select`.

## See also
n_cols, n_rows, describe

## Examples
// The names of the shipped firms, in the frame's order.
> dataset(firms).variables()
firm
industry
size
founded

// One name, by its position.
> dataset(firms).variables()[2]
industry

// How many there are.
> len(dataset(wages).variables())
11
