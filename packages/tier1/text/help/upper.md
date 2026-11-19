# upper

## Summary
Upper case by Unicode simple case mapping, one scalar value to one

## Description
The text in upper case: each Unicode scalar value is replaced by its simple upper-case mapping,
one value to one, so that the text's length never changes (section 3.8 of the Language
Reference). A character whose full upper-case form is longer, `ß` or the ligature `ﬁ`, stays as
it is; accented letters, Greek and Cyrillic map as expected; digits, punctuation, CJK and emoji
are unchanged, and a combining mark stays after the letter it follows. No locale is consulted,
so the result is the same on every machine. `NA` gives `NA`. Given a column, it applies row by
row. Like R's `toupper`, which on most platforms gives these same mappings.

## Arguments
- s: The text, or a column of texts.

## Value
A Str of the same length; a column of Strs when `s` is a column.

## Missing values
`NA` gives `NA`. In a column each row is taken by itself: a row with `NA` stays `NA`, no row is dropped and nothing is printed for it.

## Coming from R or Stata
| upper(s) | toupper(s) | strupper(s), or ustrupper(s) |

Stata's `strupper` changes the letters a to z only; `ustrupper` knows the accented ones. R's
`toupper` follows the computer's language setting; here the mapping is Unicode's and the same on
every machine, and a letter whose capital is two letters, `ß`, is left as it is, so the length
never changes.

## Related tasks
- For small letters, use `lower`.
- To compare texts without regard to case, put both through `lower`.

## See also
lower, trim, format, len

## Examples
// Accented letters have their capitals.
> upper("café")
CAFÉ

// A letter whose capital would be two letters stays as it is.
> upper("Straße")
STRAßE

// Digits and spaces are unchanged.
> upper("mixed Case 1")
MIXED CASE 1

// Row by row; the missing name stays missing.
> let people = frame(name = ["  Ann Lee", "bob STONE ", NA], code = ["A-12", "b-7", "C-103"])
> people.add(big = upper(name))
frame · 3 rows × 3 variables

      name         code    big
  1     Ann Lee    A-12      ANN LEE
  2   bob STONE    b-7     BOB STONE
  3   NA           C-103   NA

// A category of the shipped firms gives the text of its labels.
> upper(dataset(firms).head(2)["industry"])
Column<Str> · 2 values
AGRICULTURE
AGRICULTURE
