# append

## Summary
A new list with an item added

## Description
A new list holding the items of `list` and then `x`. The list given is not changed: a binding
never changes in Zeff, so `let ys = append(xs, 4)` leaves `xs` as it was. The item joins the
type of the list's items by the rules of section 3.7 of the Language Reference ("Joining types"):
an Int added to a list of Floats gives a list of Floats, `NA` fits any list, and an item of a
type that does not join is an error naming both types (Z0214). Appending to a frame is
`data.append` in the `data` package. Like R's `c(x, value)`.

## Arguments
- list: The list to add to.
- x: The item to add after its last.

## Value
A list of the joined type, one item longer.

## Missing values
`NA` may be the item added, and fits a list of any type.

## A common mistake
Adding an item of another type, such as a text to a list of numbers. A list holds values of one type, and the checker refuses the call:

> append([1, 2], "three")
error[Z0214]: `append` adds a Str to a list of an Int, which have no one type
  = help: a list holds values of one type; Int and Float join as Float, and NA fits any list; convert with `to` where a mixture is meant

Add a value of the list's type, `append([1, 2], 3)`, or convert first with `to`.

## Coming from R or Stata
| append(xs, 4) | c(xs, 4) | local xs `xs' 4 |

R's `c` turns the whole vector into text when a text is added, without a word; here that is
refused. For the rows of one frame under another the function is the frame method `append`, as
`rbind` in R and `append` in Stata.

## Related tasks
- To put the rows of one frame under another, the data package has the frame method `append`.

## See also
len, first, last, reversed

## Examples
// A new list; the old one is as it was.
> let xs = [1, 2]
> append(xs, 3)
1
2
3

// The list given has not changed.
> xs
1
2

// An Int added to Floats becomes a Float.
> append([1.5, 2.5], 3)
1.5
2.5
3

// An empty list takes any item.
> append([], "a")
a
