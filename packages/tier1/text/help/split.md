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

## Missing values
`NA` in the text or in the separator gives `NA`.

## A common mistake
Splitting at a character that means something in a regular expression. A square bracket opens a class of characters there, and alone it cannot be read:

> split("a[1]", "[")
error[Z0419]: `[` is not a regular expression: unclosed character class
  = help: the syntax is Rust's `regex` crate's: Unicode classes, no back-references or look-around (section 3.8); for a literal text, leave `regex:` off

Escape the character in a raw string: `split("a[1]", r"\[")` gives `a` and `1]`.

## Coming from R or Stata
| split(s, ",") | strsplit(s, ",", perl = TRUE)[[1]] | tokenize "`s'", parse(",") |

R's `strsplit` drops an empty piece after a trailing separator; here every piece is kept, so the
pieces joined again give the text back. Stata's `split` is a command that makes new variables
from one variable of the data; `split` here works on one text and gives a list.

## Related tasks
- To put pieces together, use `join`.
- For one part of every row of a variable, use `extract`.

## See also
join, extract, replace, contains

## Examples
// Pieces between commas.
> split("a,b,c", ",")
a
b
c

// The separator is a regular expression: any run of white space.
> split("one  two three", r"\s+")
one
two
three

// The empty piece after a trailing separator is kept: three pieces.
> len(split("a,b,", ","))
3

// Split and joined again with another separator.
> join(split("2026-10-16", "-"), sep: "/")
2026/10/16
