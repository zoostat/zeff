# join

## Summary
Join a list into one string

## Description
The texts of a list put together into one, with `sep:` between each pair; nothing between them
unless `sep:` is given. An `NA` item is written as `NA`, as R's `paste` writes it, and an empty
list gives the empty text. Together with `map` it writes a list of values into a sentence,
`join(map(xs, fn(x) format(x, digits: 1)), sep: ", ")`, and it is how a function written in Zeff
builds a message that names several things. Like R's `paste(x, collapse = sep)`.

## Arguments
- list: A list of texts.
- sep: The text put between the items; empty unless given.

## Value
A Str.

## Missing values
An `NA` item is written as the two letters `NA`, as R's `paste` writes it; the result is never `NA`.

## Coming from R or Stata
| join(xs, sep: ", ") | paste(xs, collapse = ", ") | no single function |
| join(["a", "b"]) | paste0("a", "b") | "a" + "b" |

In R's `paste` the argument `sep` goes between its arguments and `collapse` between the items of
one vector, and choosing the wrong one gives a vector back in the place of one text; `join` takes
one list and has one separator.

## Related tasks
- To cut a text into a list, use `split`.
- To write a number as text first, use `format`.

## See also
map, format, len

## Examples
// A separator between each pair.
> join(["a", "b", "c"], sep: ", ")
a, b, c

// None unless given.
> join(["caf", "é"])
café

// An empty list gives the empty text.
> len(join([], sep: "-"))
0

// Numbers written into a sentence, with `map` and `format`.
> join(map([1.5, 2], fn(x) format(x, digits: 1)), sep: " and ")
1.5 and 2.0
