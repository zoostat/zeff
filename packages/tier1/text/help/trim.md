# trim

## Summary
Remove leading and trailing Unicode white space

## Description
The text without the white space at its start and its end: every character with the Unicode
`White_Space` property, so spaces, tabs, line breaks, the no-break space and the typographic
spaces alike (section 3.8 of the Language Reference). White space inside the text is kept.
`NA` gives `NA`. Given a column, it applies row by row, which is the usual first step after
importing text typed by hand. Like R's `trimws`, which by default removes only ASCII white
space, and Stata's `strtrim`.

## Arguments
- s: The text, or a column of texts.

## Value
A Str; a column of Strs when `s` is a column.

## Missing values
`NA` gives `NA`. In a column each row is taken by itself: a row with `NA` stays `NA`, no row is dropped and nothing is printed for it.

## Coming from R or Stata
| trim(s) | trimws(s) | strtrim(s), or ustrtrim(s) |

R's `trimws` removes spaces, tabs and line breaks unless told more; Stata's `strtrim` removes
blanks only. `trim` removes every kind of white space Unicode knows, the no-break space that a
spreadsheet or a web page leaves behind among them.

## Related tasks
- To make a text longer, to a width, use `pad`.
- To take white space out of the inside of a text, use `replace`.

## See also
upper, lower, pad, len

## Examples
// Both ends.
> trim("  padded  ")
padded

// A no-break space and a typographic space go too: four letters are left.
> len(trim("\u{a0}wide\u{2003}"))
4

// The inside is kept.
> trim("inner  space")
inner  space

// Row by row, the first step after reading names typed by hand.
> let people = frame(name = ["  Ann Lee", "bob STONE ", NA], code = ["A-12", "b-7", "C-103"])
> people.add(clean = trim(name))
frame · 3 rows × 3 variables

      name         code    clean
  1     Ann Lee    A-12    Ann Lee
  2   bob STONE    b-7     bob STONE
  3   NA           C-103   NA
