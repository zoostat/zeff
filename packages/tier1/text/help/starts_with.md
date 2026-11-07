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

## See also
ends_with, slice, upper

## Examples
> starts_with("Hello, World", "Hello")
true
> starts_with("Hello, World", "hello")
false
> starts_with("日本語", "日")
true
