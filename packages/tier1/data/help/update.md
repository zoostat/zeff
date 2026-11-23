# update

## Summary
Change the values of a variable, everywhere or where a condition holds

## Description
A frame with the values of existing variables changed as the definitions say, in every row or,
with `where:`, only in the rows where the condition is true: `update(hours = 38, where:
is_missing(hours))` fills the gaps and leaves the other rows as they were. A variable keeps its type
and its metadata, so new values must be of that type (an Int may go into a Float variable); to
change a variable's type, `add` replaces it. Rows where the condition is `NA` are left unchanged
and counted in a note. A name the frame does not have is an error that points to `add`.

## Arguments
- data: The frame.
- where: The rows to change: an expression over the frame's variables that is true or false in
  every row. Every row when omitted.
- ...: Definitions `name = expr`, each naming a variable of the frame.

## Value
A `Frame` with the same rows and variables, the values changed where asked.

## Missing values
A row where the `where:` condition is `NA` is left as it was, and a note counts those rows. `NA` alone has no type and is refused; a value is made missing with an `NA` that has the variable's type, as `wage * NA` has: `update(wage = wage * NA, where: wage > 500)`.

## A common mistake
Naming a variable the frame does not have. `update` changes a variable that is there:

> dataset(firms).update(staff = size * 2)
error[Z0101]: no variable `staff` in `firms`
  = help: the variables of `firms` are: firm, industry, size, founded
  = help: `update` changes a variable the frame has; `add` creates a new one

For a new variable write `dataset(firms).add(staff = size * 2)`.

## Coming from R or Stata
| d.update(hours = 38, where: is_missing(hours)) | d$hours[is.na(d$hours)] <- 38 | replace hours = 38 if missing(hours) |
| d.update(wage = wage * 2) | d$wage <- d$wage * 2 | replace wage = wage * 2 |

In Stata a missing value is larger than any number, so `replace wage = 0 if hours > 36` also
changes the rows with no hours; here a row where the condition cannot be decided is left as it
was and counted. Stata's `replace` and R's assignment change the data they are given; `update`
gives a new frame.

## Related tasks
- To make a new variable, or to change the type of one, use `add`.
- To fill missing values from a rule, use `fill_missing`.
- To give codes other values, use `recode`.
- To keep the rows that meet a condition, use `where`.

## See also
add, where, is_missing

## Examples
// A value changed in the rows where a condition holds: the firms recorded with no staff counted as one.
> let firms = dataset(firms)
> firms.update(size = 1, where: size == 0).sort(size).head(3)
frame · 3 rows × 4 variables

      firm   industry        size   founded
  1    398   Construction       1   1965-11-27
  2    399   Manufacturing      1   1952-08-10
  3    400   Education          1   1979-12-12

// Every row, from the variable's own values.
> firms.update(size = size * 2).head(3)
frame · 3 rows × 4 variables

      firm   industry      size   founded
  1      1   Agriculture      6   1990-11-06
  2      2   Agriculture     10   1957-07-02
  3      3   Agriculture      6   1998-11-07

// Rows where the condition is `NA` are left as they were, and counted.
> dataset(wages).update(hours = 40, where: wage > 100).n_rows()
18412
note: 211 rows where `wage > 100` was NA were left unchanged (line 4)

// A value made missing: `wage * NA` is an `NA` of the type of `wage`. The 211 wages missing before are among those counted.
> dataset(wages).update(wage = wage * NA, where: wage > 150).where(is_missing(wage)).n_rows()
212
note: 211 rows where `wage > 150` was NA were left unchanged (line 5)
