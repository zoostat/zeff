# add

## Summary
Add variables computed from the frame

## In brief
- `wages.add(lwage = log(wage))`: a new variable from an expression over the frame's variables.
- `wages.add(dev = wage - mean(wage), by: industry)`: a definition with a reducer in it, computed
  within each group.
- `wages.add(source = "survey")`: one value written into every row.

## Description
A frame with new variables, each defined as `name = expr`, where `expr` is an expression over the
frame's variables (`pay = wage * hours`), a reducer over the frame or over each group of `by:`
(`dev = wage - mean(wage)`), or a single value written into every row (`year = 2026`).
Definitions are evaluated left to right, and a later one may use an earlier one in the same call:
`add(lwage = log(wage), high = lwage > 3)`. A definition whose name the frame already has replaces
that variable, and a note says so; `update` changes an existing variable's values without changing
its type. Unchanged columns are shared with the parent, not copied, and every variable keeps its
metadata.

## Arguments
- data: The frame.
- ...: Definitions `name = expr`: an expression over the frame's variables, a reducer over them,
  or a Bool, Int, Float, Str, Date or DateTime value for every row.

## Value
A `Frame` with the same rows and the variables added or replaced.

## Missing values
An expression gives `NA` in a row where a value it reads is `NA`, by the rules of section 4.1 of
the Language Reference, and a reducer skips `NA` with the reducers' own warning; no row is
dropped. A definition that is `NA` alone is refused (Z0219), since the type of the new variable
would be unknown: write `to(NA, Float)`.

## Details
### What a definition may contain
Arithmetic, comparisons and the functions of `core` over the frame's variables, row by row; a
reducer of `data` (`mean`, `sd`, `sum`, `n`, `min`, `max` and the rest), which gives one value
for the frame, or one for each group under `by:`, written into every row of it; a column of the
same length, such as `lag(wage)` on a panel; a single value. A list, a record or a frame is not a
variable and is refused (Z0401). `len` of a text variable is one number too, the count of its
rows, and not the length of each value: a note says so and names `n_chars`, which gives that.

### Replacing a variable
A definition named after an existing variable replaces it, whatever its type, and a note says
so; `update` is the verb for changing values without changing the type, and it refuses a value
of another type. Replacing a variable of a panel declaration drops the declaration, with a note.

## A common mistake
Misspelling a variable of the frame inside a definition. The checker refuses the statement
before anything runs, names the frame's variables that are close, and reminds that a category
level is written in quotes:

> wages.add(pay = wge * 40)
error[Z0101]: no variable `wge` in `wages`
  = help: did you mean `wage`? If you meant a category level, write it in quotes: `"wage"`

Write the variable as the frame has it: `wages.add(pay = wage * 40)`.

## Coming from R or Stata
| wages.add(pay = wage * hours) | mutate(d, pay = wage * hours) | generate pay = wage * hours |
| wages.add(dev = wage - mean(wage), by: industry) | d %>% group_by(industry) %>% mutate(dev = wage - mean(wage)) | bysort industry: egen m = mean(wage), then generate dev = wage - m |
| wages.add(wage = wage / 10) | mutate(d, wage = wage / 10) | replace wage = wage / 10 |

Zeff never changes a frame in place: `add` returns a new frame, and the old binding keeps the
old columns. A definition that reuses a name replaces the variable with a note, where Stata's
`generate` refuses an existing name and R's `mutate` replaces silently. A reducer inside a
definition needs no `egen`.

## Related tasks
- To change values without changing a variable's type, use `update`.
- To keep some variables and drop the rest, use `select`.
- To give a variable another name, use `rename`.
- For one row per group with its statistics, use `aggregate`.

## See also
update, select, rename, aggregate

## Examples
// New variables from an expression over the frame, and a later definition using an earlier one.
> let wages = dataset(wages)
> wages.select([worker, year, wage]).add(pay = wage * 40, high = pay > 1000).head(3)
frame · 3 rows × 5 variables · panel worker × year

      worker   year    wage     pay   high
  1        1   2015   18.08   723.2   false
  2        1   2016   17.28   691.2   false
  3        1   2017   20.93   837.2   false

// A reducer inside a definition, computed within each group of `by:`.
> wages.select([worker, year, industry, wage]).add(dev = wage - mean(wage), by: industry).head(3)
frame · 3 rows × 5 variables · panel worker × year

      worker   year   industry    wage        dev
  1        1   2015   Retail     18.08   -8.12836
  2        1   2016   Retail     17.28   -8.92836
  3        1   2017   Retail     20.93   -5.27836
warning[W0201]: 211 rows with NA in wage were skipped (line 3)

// One value written into every row, and a variable replaced with a note.
> wages.select([worker, year, wage]).add(wage = wage / 10, source = "survey").head(3)
frame · 3 rows × 4 variables · panel worker × year

      worker   year    wage   source
  1        1   2015   1.808   survey
  2        1   2016   1.728   survey
  3        1   2017   2.093   survey
note: `wage` replaced the variable of that name (line 4)
