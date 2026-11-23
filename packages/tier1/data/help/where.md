# where

## Summary
Keep the rows where a condition is true

## Description
Returns the rows for which `condition` is `true`. Rows where it is `NA` are dropped and counted, so
`where(age > 30)` and `where(!(age > 30))` both drop rows with missing age, and the note of each says
so; the two never partition a frame silently.

Inside the condition a bare name is a variable of the frame; a script binding of the same name is
used only when the frame has no such variable, and both at once is a warning (W0102). Row order is
kept, and every variable's metadata and the frame's are carried.

## Arguments
- data: The frame to filter.
- condition: An expression over the frame's variables that is true or false in every row, such as
  `hours > 36 && !is_missing(wage)`. A single Bool, `NA`, or a column of another type is an
  error that names the problem (Z0203).

## Value
A `Frame` with the same variables and a subset of the rows.

## Missing values
A row where the condition is `NA` is not kept, and a note counts those rows and says how to list them. So `where(age > 30)` and `where(!(age > 30))` together are not the whole frame when age has missing values. To keep the rows with a missing value on purpose, say so: `where(is_missing(age) || age > 30)`.

## A common mistake
Joining two conditions with `&`, as R and Stata write it. In Zeff and is `&&` and or is `||`:

> dataset(wages).where(wage > 20 & hours > 40)
error[Z0315]: `&` is not used in Zeff
  = help: write `&&` for and

Write `dataset(wages).where(wage > 20 && hours > 40)`.

## Coming from R or Stata
| d.where(wage > 20) | subset(d, wage > 20), or dplyr::filter(d, wage > 20) | keep if wage > 20 & !missing(wage) |
| d.where(wage > 20 && union == 1) | subset(d, wage > 20 & union == 1) | keep if wage > 20 & union == 1 & !missing(wage) |
| d.where(is_missing(wage)) | subset(d, is.na(wage)) | keep if missing(wage) |

In R, `d[d$wage > 20, ]` gives a row of `NA`s for every row with a missing wage; `subset` and
`filter` leave those rows out, as `where` does, and say nothing, where the note here counts
them. In Stata a missing value is larger than any number, so `keep if wage > 20` keeps the rows
with no wage. Stata's `keep if` changes the dataset in memory; `where` gives a new frame, and
the frame it was given is as it was.

## Related tasks
- To keep some of the variables, use `select`.
- To remove the rows with a missing value in given variables, use `drop_missing`.
- For the first rows, use `head`.
- To change values in the rows that meet a condition, use `update`.
- To test for a missing value, use `is_missing`.

## See also
select, drop, is_missing, head

## Examples
// The rows that meet a condition. The note counts the rows where it could not be decided.
> let wages = dataset(wages)
> wages.where(wage > 100).select([worker, year, wage]).head(3)
frame · 3 rows × 3 variables · panel worker × year

      worker   year     wage
  1       56   2013   105.87
  2       56   2015   140.24
  3      311   2020   119.48
note: 211 rows dropped where `wage > 100` was NA; `wages.where(is_missing(wage))` lists them (line 2)

// Two conditions that must both hold, and either of two.
> wages.where(year == 2019 && educ >= 18).n_rows()
> wages.where(year == 2015 || year == 2020).n_rows()
99
3281

// The rows where a value is missing.
> wages.where(is_missing(wage)).n_rows()
211

// A condition and its negation leave out the rows where it is `NA`: together they are fewer than the 18,412 rows.
> wages.where(wage > 20).n_rows()
> wages.where(!(wage > 20)).n_rows()
11961
note: 211 rows dropped where `wage > 20` was NA; `wages.where(is_missing(wage))` lists them (line 6)
6240
note: 211 rows dropped where `!(wage > 20)` was NA; `wages.where(is_missing(wage))` lists them (line 7)

// To keep the rows with a missing value on purpose, say so.
> wages.where(is_missing(wage) || wage > 100).n_rows()
237

// No row may meet the condition; the note says so.
> wages.where(year == 1900).n_rows()
0
note: no row kept: the result is empty (line 9)
