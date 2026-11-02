# save

## Summary
Save as CSV or TSV, chosen by the extension

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
(Z0716). `save` is allowed in a script, not inside a function (Z0110). Like R's `write.csv` and
Stata's `export delimited`.

## Arguments
- data: The frame to write.
- path: Where to write it: a bare name goes to `output/`; `.csv` or `.tsv` chooses the format.
- format: `"csv"` or `"tsv"` when the path's extension does not say.
- bom: Whether to start the file with a byte-order mark, for a spreadsheet that expects one; false unless given.
- codes: Whether to write a category as its level's position rather than its label; false unless given.
- version: The Stata file version, for `.dta` when it arrives; 118 unless given.

## Value
Nothing; a note says what was written, with the rows and variables.

## Details
Exact: the bytes are the same on every machine, and the conformance report carries them. A text
value that reads as missing on import (`NA`, `.`, an empty text) comes back as `NA`, since the
reader treats those cells as missing; `import` of a saved file otherwise gives the frame back,
value for value, with a category as text; an outer blank is quoted on writing and trimmed by
the reader.

## See also
import, export, select, describe

## Examples
> let by_year = frame(year = [2010, 2011, 2012], sales = [120.5, 130, NA], region = ["North", "South, East", "West"])
> by_year.save("by_year.csv")
note: saved output/by_year.csv · 3 rows × 3 variables (line 2)
> by_year.save("output/by_year.tsv")
note: saved output/by_year.tsv · 3 rows × 3 variables (line 3)
