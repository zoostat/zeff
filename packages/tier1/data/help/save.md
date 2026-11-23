# save

## Summary
Save as CSV or TSV, chosen by the extension

## In brief
- `d.save("wages.csv")`: a bare name goes to `output/`, as CSV by its extension.
- `d.save("output/wages.tsv")`: a path with a folder goes where it says, inside the project; `output/` is made when needed, another folder must be there.
- `d.save("wages.txt", format: "tsv")`: the format named when the extension does not say.

## Description
Writes the frame to a file under the project folder, as CSV or TSV by the path's extension or by
`format:`; a bare name goes to `output/`, which is created when it does not exist, and a path with
a folder goes where it says, inside the project (a data root or a path outside the project is
refused, Z0707). The file is UTF-8 without a byte-order mark unless `bom: true`, with LF line
endings and a header of the variable names; fields are quoted only when they hold the delimiter,
a quote, a line break or an outer blank; `NA` is an empty cell, `Inf` is `Inf`, a Float is the
shortest text that reads back to the same number (`120.0`, `0.1`, `1e300`), a date is `2026-01-31`,
a datetime `2026-01-31T09:30:00`, and a category is its label, or its level's position when
`codes: true`. Variable labels and the panel declaration are not written; Parquet, Arrow, Stata,
Excel and JSON, which keep them, arrive with release 0.11.0 and are refused by name until then
(Z0716). `save` is allowed in a script, not inside a function (Z0110).

## Arguments
- data: The frame to write.
- path: Where to write it: a bare name goes to `output/`; `.csv` or `.tsv` chooses the format.
- format: `"csv"` or `"tsv"` when the path's extension does not say.
- bom: Whether to start the file with a byte-order mark, for a spreadsheet that expects one; false unless given.
- codes: Whether to write a category as its level's position rather than its label; false unless given.
- version: The Stata file version, for `.dta` when it arrives; 118 unless given.

## Value
Nothing; a note says what was written, with the rows and variables.

## Missing values
`NA` is written as an empty cell, in a column of any type, and reads back as `NA`. A text value that is itself `NA`, `.` or empty also reads back as missing, since the reader takes those cells so.

## Details
Exact: the bytes are the same on every machine, and the conformance report carries them. A text
value that reads as missing on import (`NA`, `.`, an empty text) comes back as `NA`, since the
reader treats those cells as missing; `import` of a saved file otherwise gives the frame back,
value for value, with a category as text; an outer blank is quoted on writing and trimmed by
the reader.

## A common mistake
Giving a place on the computer outside the project. A script writes inside its project, so that running it never touches anything else:

> dataset(sleep).save("/Users/me/Desktop/sleep.csv")
error[Z0707]: cannot write `/Users/me/Desktop/sleep.csv`: it is outside the project
  = help: write under the project, as in `output/sleep.csv`

Give a name, which goes to `output/`, or a path inside the project: `dataset(sleep).save("sleep.csv")`.

## Coming from R or Stata
| d.save("wages.csv") | write.csv(d, "output/wages.csv", row.names = FALSE, na = "") | export delimited using "output/wages.csv", replace |

R's `write.csv` writes the row names as a first column unless `row.names = FALSE`, and a missing
value as the text `NA` unless `na = ""`; here a frame has no row names and a missing value is an
empty cell. Stata's `export delimited` refuses to write over a file without `replace`; `save`
writes over it, as a script that is run again must. Stata's own `save` writes its `.dta` format,
which arrives here with release 0.11.0.

## Related tasks
- To read the file back, use `import`.
- To write a table of results for a paper, use `export`.
- To keep some of the variables before saving, use `select`.
- To keep some of the rows before saving, use `where`.

## See also
import, export, select, describe

## Examples
// A bare name goes to `output/`; the note says what was written.
> let sleep = dataset(sleep)
> sleep.save("sleep.csv")
note: saved output/sleep.csv · 20 rows × 3 variables (line 2)

// A path with its folder, tab-separated by its extension.
> sleep.save("output/sleep.tsv")
note: saved output/sleep.tsv · 20 rows × 3 variables (line 3)

// `format:` when the extension does not say.
> sleep.save("sleep.txt", format: "tsv")
note: saved output/sleep.txt · 20 rows × 3 variables (line 4)

// A category written as the position of its level, and a byte-order mark for a spreadsheet that expects one.
> sleep.save("sleep_codes.csv", codes: true, bom: true)
note: saved output/sleep_codes.csv · 20 rows × 3 variables (line 5)

// `version:` is for Stata files, which arrive with release 0.11.0; on a CSV file it changes nothing.
> sleep.save("sleep_v.csv", version: 118)
note: saved output/sleep_v.csv · 20 rows × 3 variables (line 6)

// Some rows and some variables of the shipped wages.
> dataset(wages).where(year == 2019).select([worker, year, wage]).save("wages_2019.csv")
note: saved output/wages_2019.csv · 1,518 rows × 3 variables (line 7)
