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

## See also
map, format, len

## Examples
> join(["a", "b", "c"], sep: ", ")
a, b, c
> join(["caf", "é"])
café
> len(join([], sep: "-"))
0
> join(map([1.5, 2], fn(x) format(x, digits: 1)), sep: " and ")
1.5 and 2.0
