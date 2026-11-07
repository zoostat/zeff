# lower

## Summary
Lower case by Unicode simple case mapping, one scalar value to one

## Description
The text in lower case: each Unicode scalar value is replaced by its simple lower-case mapping,
one value to one, so that the text's length never changes (section 3.8 of the Language
Reference). A Greek capital sigma becomes `σ` wherever it stands, since the final form is a
rule of words that the mapping does not apply; accented capitals, Greek and Cyrillic map as
expected; digits, punctuation, CJK and emoji are unchanged. No locale is consulted, so the
result is the same on every machine. `NA` gives `NA`. Given a column, it applies row by row.
Like R's `tolower`.

## Arguments
- s: The text, or a column of texts.

## Value
A Str of the same length; a column of Strs when `s` is a column.

## See also
upper, trim, len

## Examples
> lower("ÉCOLE")
école
> lower("MiXeD 123")
mixed 123
> lower("Привет")
привет
