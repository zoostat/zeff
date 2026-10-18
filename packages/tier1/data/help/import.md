# import

## Summary
Read a data file into a frame

## Description
Reads a file in the project (`import("data/wages.csv")`) or in a declared data root
(`import("lfs:wages.parquet")`) and chooses the reader from the extension. This release reads
delimited text: CSV, TSV and text files, following RFC 4180, with the delimiter detected from the
first line when it is `;` or a tab. The other formats of section 6.1 arrive with release 0.11.0,
and the remaining options of this function (`format:`, `encoding:`, `decimal:`, `thousands:` and
the rest) with the releases that implement them.

The type of each column is guessed from its first 1,000 rows: Bool, Int, Float, Date (ISO 8601),
DateTime or Str. A later value that does not fit stops the run naming the row, the column and the
value (Z0606); nothing is ever coerced to `NA` silently. Empty cells, `NA` and `.` are missing, and
`na_values:` adds codes; `Inf` and `-Inf` are infinity; `NaN` becomes `NA` with a note. Dates are read
only in the ISO form, or with the format `types:` gives; a column that looks like dates in another
form is read as text, with a note that proposes the `types:` entry. Variable names are kept as the
file has them unless `names: snake` turns them into valid snake_case names. Every import prints one
note saying what was read and what the format carries; further notes report what the reader
changed or could not keep.

## Arguments
- path: The file, written with `/` from the project folder, or `root:file` for a data root. A path
  outside the project is refused (Z0701), and one that differs only in case from the file's real
  name is refused with the real spelling (Z0702).
- delim: The delimiter, one character. When omitted, `,` is used, or `;` or a tab when the first
  line has one of them and no comma.
- header: Whether the first row names the columns. With `false` the columns are `v1`, `v2`, ….
- skip: Lines to skip at the top of the file, before the header.
- na_values: Values read as missing besides the empty cell, `NA` and `.`, as numbers or text:
  `na_values: [-99, -98]`.
- types: A record declaring a column's type when the guess is wrong: `types: {hours: Str}`, or a
  date format, `types: {sold: Date("%d/%m/%Y")}`. A column may be named as the file writes it or
  by the name `names: snake` gives it.
- names: Keep the names of the file, or turn each into a snake_case name and print the changes.

## Value
A `Frame` with one column per variable of the file.

## See also
frame, describe, n_rows, variables

## Examples
> let wages = import("data/wages.csv")
note: data/wages.csv read as CSV: 3 rows × 3 variables, types guessed; CSV carries no labels, value labels or categories (line 1)
> wages
wages · 3 rows × 3 variables
> import("data/wages.csv", na_values: [40]).mean(hours)
35
note: data/wages.csv read as CSV: 3 rows × 3 variables, types guessed; CSV carries no labels, value labels or categories (line 3)
note: 1 value of hours read as NA from na_values 40 (line 3)
warning: 2 rows with NA in hours were skipped (line 3)
