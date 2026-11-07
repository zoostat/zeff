# split

## Summary
Split text into a list

## Description
The pieces of the text between the matches of `sep`, a regular expression in the syntax of
section 3.8 of the Language Reference, in order, as a list of texts: `split("a,b,c", ",")` is
`["a", "b", "c"]`. Every piece is kept, the empty one before a leading separator or after a
trailing one included, so that `join(split(s, ","), sep: ",")` gives `s` back; R's `strsplit`
drops a trailing empty piece. A pattern that matches the empty text splits between the scalar
values, `split("abc", "")` giving the three characters. A literal separator that is special in
a pattern is escaped, `split("a.b", r"\.")` or `split("a|b", r"\|")`; a raw string, `r"..."`, has no escapes of its own and is the form for patterns (section 3.1). A pattern the engine cannot
read is Z0419. `NA` in either argument gives `NA`. `split` works on one text, not on a column
(Z0401), since a column holds one value per row and never a list. Like R's
`strsplit(s, sep, perl = TRUE)`.

## Arguments
- s: The text.
- sep: A regular expression matching the separators.

## Value
A list of Strs.

## See also
join, extract, replace, contains

## Examples
> split("a,b,c", ",")
a
b
c
> split("one  two three", r"\s+")
one
two
three
> len(split("a,b,", ","))
3
