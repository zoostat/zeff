# export

## Summary
Write a table, a result or a plot to a file

## Description
Writes a table, a result or a plot to a file under the project folder, in the format the
extension of the path names. A path with no folder goes to `output/`, which is created as
needed; any other folder must exist, and a path outside the project is refused.

A table, as `describe`, `summary`, `report` and the other table functions return it, is written
as `.docx` for Word or `.tex` for LaTeX. A regression exports its summary table, with the lines
above and below the grid as notes, and a test's result its title, its statistic and, for
`compare`, the models it sets side by side. The file carries the same layout as the printed
table: its title, its columns with numbers to the right, and its rows. Like Stata's
`esttab using`.

A plot is written as a figure on a page of the size `size:` gives: one column of a journal
unless told otherwise. `.pdf` and `.svg` are drawn with lines and stay sharp at any
enlargement; `.png` is drawn in dots, 300 to the inch. The figure is drawn by Zeff itself with
its own fonts, so each file is the same, byte for byte, on every computer. Like R's `ggsave`.

## Arguments
- x: The table, result or plot to write.
- path: Where to write it, from the project folder: `output/table1.docx`. The extension chooses
  the format, whether written in small letters or in capitals: `.docx` or `.tex` for a table
  or a result, `.pdf`, `.svg` or `.png` for a plot.
- size: For a plot, the size of the page: `one_column` (89 mm wide and 66 mm high),
  `two_column` (183 by 110 mm), `slide` (254 by 143 mm), or a width and a height in
  millimetres as a list, `[120, 80]`. Without it the page is `one_column`. The text of a
  figure is never set below 8 points, so a page too small for the plot's labels is refused
  (Z0401) and never drawn with smaller type.
- notes: For a plot, `true` writes its notes, the rows that could not be drawn and why, under
  the figure. Without it a figure carries no note: the notes are printed with the plot.

## Value
Nothing; the file is written.

## Details
The Word file is a minimal document holding the title in bold, the table with a rule above and
below it and under its headings, and any notes beneath in small type; it opens in Word,
LibreOffice and Pages and takes the document's own fonts. The LaTeX file is a `tabular` with the
rules of the `booktabs` package, the title in a comment above it and the notes as full-width rows
under the bottom rule, with `&`, `%`, `_`, `^` and the other special characters escaped; put it
inside a `table` environment with `\input` in a document that loads `booktabs`. A rule inside
the table, as `report` draws one above its statistics, is a `\midrule` in LaTeX and a border in
Word. Both files depend on the table alone, so they are the same bytes on every machine, and the
conformance suite checks them.

A figure. The plot is laid out for the page it is written on, so a wider page has a wider
panel and the same size of type, 8 points for labels and 9 for titles, in IBM Plex Sans; give
the size the figure will be printed at, and do not enlarge it afterwards. Nothing in a file
says when or where it was made, and it holds no row of the data and not the name of the frame:
what it shows is what it holds.

Which format. A journal usually asks for a PDF, and LaTeX places one with `\includegraphics`.
The fonts are inside the file, so it prints the same anywhere, and its words are text, which
can be searched and copied. An SVG opens in every browser, and Word and PowerPoint place it as
a picture that can be resized. Its letters are drawn as their outlines and not as text in a
font, because a program that opens an SVG may not have the font and would set the labels in
another, of other widths; the words are kept in the file, so a search in a browser finds them,
but a label cannot be retyped in a drawing program. A PNG is a picture in dots, for where
neither of the two is taken: a web page, or an older word processor. It has 300 dots to the
inch and says so, so a page of 89 mm is 1051 dots wide and is placed 89 mm wide. Its page is
white, without transparency. Enlarged, a PNG shows its dots; the other two do not.

The largest page. A page of a PDF is at most 5080 mm each way, and a PNG holds at most 40
million dots, a page of about 535 mm each way; a larger page is refused (Z0401). An SVG has no
largest page.

In R the same is `ggsave("wage.pdf", width = 89, height = 66, units = "mm")`. R's PDF names its
fonts without carrying them, unless the `cairo_pdf` device is asked for, and writes the day
into the file, so two runs do not give the same bytes. In Stata it is `graph export wage.pdf`.

## See also
report, describe, summary, plot, points

## Examples
> let wages = import("data/wages.csv")
note: data/wages.csv read as CSV: 3 rows × 3 variables, types guessed; CSV carries no labels, value labels or categories (line 1)
> wages.describe().export("output/wages.docx")
> wages.describe().export("wages.tex")
> let m = dataset(wages).reg(log(wage) ~ educ + exper + female)
> m.export("output/model.docx")
> m.test(educ == exper).export("output/test.tex")
> let p = dataset(wages).plot(x: educ, y: wage, color: union).points(alpha: 0.3)
note: 211 rows with NA in educ or wage dropped (line 7)
> p.export("output/wage.pdf")
> p.export("output/wage.svg")
> p.export("output/wage.png")
> p.export("wage_wide.pdf", size: two_column, notes: true)
> p.export("output/wage_own.png", size: [120, 80])
