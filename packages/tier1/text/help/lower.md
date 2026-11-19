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

## Missing values
`NA` gives `NA`. In a column each row is taken by itself: a row with `NA` stays `NA`, no row is dropped and nothing is printed for it.

## Coming from R or Stata
| lower(s) | tolower(s) | strlower(s), or ustrlower(s) |

Stata's `strlower` changes the letters A to Z only; `ustrlower` knows the accented ones. R's
`tolower` follows the computer's language setting; here the mapping is Unicode's and the same on
every machine.

## Related tasks
- For capitals, use `upper`.
- To test a text without regard to case, use `contains`.

## See also
upper, trim, len

## Examples
// Accented capitals have their small letters.
> lower("ÉCOLE Straße")
école straße

// Digits and spaces are unchanged.
> lower("Mixed Case 1")
mixed case 1

// Row by row, to make codes typed two ways the same.
> let people = frame(name = ["  Ann Lee", "bob STONE ", NA], code = ["A-12", "b-7", "C-103"])
> people.add(code = lower(code))
frame · 3 rows × 2 variables

      name         code
  1     Ann Lee    a-12
  2   bob STONE    b-7
  3   NA           c-103
note: `code` replaced the variable of that name (line 4)
