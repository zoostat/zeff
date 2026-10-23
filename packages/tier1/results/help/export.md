# export

## Summary
Write a table to a Word or LaTeX file

## Description
Writes a table, as `describe`, `summary` and the other table functions return it, to a file under
the project folder, in the format the extension names: `.docx` for Word and `.tex` for LaTeX. The
file carries the same layout as the printed table: its title, its columns with numbers to the
right, and its rows. A path with no folder goes to `output/`, which is created as needed; any
other folder must exist, and a path outside the project is refused. Like Stata's `esttab using`.

## Arguments
- x: The table to write.
- path: Where to write it, from the project folder: `output/table1.docx`. The extension chooses
  the format.

## Value
Nothing; the file is written.

## Details
The Word file is a minimal document holding the title in bold, the table with a rule above and
below it and under its headings, and any notes beneath in small type; it opens in Word,
LibreOffice and Pages and takes the document's own fonts. The LaTeX file is a `tabular` with the
rules of the `booktabs` package, the title in a comment above it and the notes as full-width rows
under the bottom rule, with `&`, `%`, `_`, `^` and the other special characters escaped; put it
inside a `table` environment with `\input` in a document that loads `booktabs`. Both files depend
on the table alone, so they are the same bytes on every machine, and the conformance suite checks
them.

## See also
report, describe, summary

## Examples
> let wages = import("data/wages.csv")
note: data/wages.csv read as CSV: 3 rows × 3 variables, types guessed; CSV carries no labels, value labels or categories (line 1)
> wages.describe().export("output/wages.docx")
> wages.describe().export("wages.tex")
