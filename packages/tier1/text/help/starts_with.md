# starts_with

## Summary
True where text starts with the prefix

## Description
Whether the text begins with `prefix`, compared scalar value by scalar value and
case-sensitively, with no locale: `starts_with("Hello", "he")` is `false` (section 3.8 of the
Language Reference). Every text starts with the empty text. `NA` in either argument gives `NA`.
Given a column, it applies row by row and gives a Bool column, the usual condition for `where`.
Like R's `startsWith`.

## Arguments
- s: The text, or a column of texts.
- prefix: The text it may start with.

## Value
A Bool; a column of Bools when `s` is a column.

## Missing values
`NA` in the text or in the prefix gives `NA`. In a column each row is taken by itself, and no row is dropped.

## Coming from R or Stata
| starts_with(s, "ab") | startsWith(s, "ab") | substr(s, 1, 2) == "ab" |

R's `startsWith` is the same. Stata has no function for it and compares the first characters,
which must be counted.

## Related tasks
- For the end of a text, use `ends_with`.
- For a text anywhere inside, use `contains`.

## See also
ends_with, slice, upper

## Examples
// The prefix is there.
> starts_with("Hello, World", "Hello")
true

// Capitals matter.
> starts_with("Hello, World", "hello")
false

// Any script.
> starts_with("日本語", "日")
true

// On the labels of a category of the shipped firms.
> dataset(firms).where(starts_with(industry, "Pub")).n_rows()
34
