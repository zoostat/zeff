# row_number

## Summary
The position of each row, from 1

## In brief
- `add(id = row_number())`: 1, 2, 3, … down the rows as they stand.
- `add(visit = row_number(), by: patient)`: the count starts again in each group.
- Sort first when the order matters: the numbers follow the rows, not a variable.

## Description
A whole number for each row, counting from 1 in the frame's current row order: the first row
gets 1, the next 2, and so on to the number of rows. It takes no argument. Inside `add`,
`update` or `where` it numbers the whole frame; under `by:` it numbers each group on its own,
so the first row of every group gets 1. It is dplyr's `row_number()` and Stata's `_n`.

It is used to give each observation an identifier, to number the visits of a person or the
years of a firm, and, with `where`, to keep the first row, or the first few, of the frame or
of each group. It can stand in an expression: `row_number() / n()` is how far down the frame
a row is.

## Arguments

## Value
A column of Ints, one for each row: 1 to the number of rows, or 1 to the size of each group
under `by:`. No value is missing.

## Missing values
There is nothing to be missing: every row has a position, whatever its values. A row whose `by:` variable is `NA` belongs to the group of the rows with `NA` there, as in every grouped verb, and is numbered within it.

## Details
### The order of the rows
The numbers follow the rows as they stand, not a declared panel's time order and not the
value of any variable. To number by a variable, sort by it first:
`data.sort(year).add(k = row_number(), by: firm)` numbers each firm's years from the earliest.
Under `by:` the rows of a group keep the order they have in the frame.

### Exactness
Exact: a count.

## A common mistake
Calling it where there is no frame. The rows it numbers are those of the frame a definition is evaluated for:

> row_number()
error[Z0401]: `row_number()` numbers the rows of a frame and stands inside `add`, `update` or `where`
  = help: write `wages.add(id = row_number())` for a row number, or `wages.add(k = row_number(), by: worker)` to number the rows of each worker; `seq(1, 5)` is a list of numbers

## Coming from R or Stata
| d.add(id = row_number()) | mutate(d, id = row_number()), or d$id <- seq_len(nrow(d)) | generate id = _n |
| d.add(k = row_number(), by: g) | mutate(group_by(d, g), k = row_number()), or ave(seq_along(g), g, FUN = seq_along) | bysort g: generate k = _n |

dplyr's `row_number(x)` with an argument is a rank of `x`; here that is `rank` with
`ties: first`. Stata's `bysort g (year): generate k = _n` sorts within the group as it numbers;
here the frame is sorted first, in a step of its own.

## Related tasks
- For the number of rows, the same on every row, use `n`.
- For the rank of each value of a variable, use `rank`.
- For a total so far down the rows, use `running`.
- To keep the first rows of a frame without numbering them, use `head`.

## See also
n, rank, running, sort, head, add

## Examples
// An identifier for each row.
> let visits = frame(patient = ["a", "a", "b", "a", "b"], day = [3, 1, 2, 9, 5], score = [4.0, 5.5, 3.0, 6.0, 2.5])
> visits.add(id = row_number())
frame · 5 rows × 4 variables

      patient   day   score   id
  1   a           3       4    1
  2   a           1     5.5    2
  3   b           2       3    3
  4   a           9       6    4
  5   b           5     2.5    5

// Within each patient the count starts again; the rows stay where they are.
> visits.add(visit = row_number(), by: patient)
frame · 5 rows × 4 variables

      patient   day   score   visit
  1   a           3       4       1
  2   a           1     5.5       2
  3   b           2       3       1
  4   a           9       6       3
  5   b           5     2.5       2

// Sorted by day first, the number is the visit in order of time.
> let numbered = visits.sort(day).add(visit = row_number(), by: patient)
> numbered
numbered · 5 rows × 4 variables

      patient   day   score   visit
  1   a           1     5.5       1
  2   b           2       3       1
  3   a           3       4       2
  4   b           5     2.5       2
  5   a           9       6       3

// The first visit of each patient.
> numbered.where(visit == 1)
frame · 2 rows × 4 variables

      patient   day   score   visit
  1   a           1     5.5       1
  2   b           2       3       1

// The first two rows of the frame as it stands.
> visits.where(row_number() <= 2)
frame · 2 rows × 3 variables

      patient   day   score
  1   a           3       4
  2   a           1     5.5

// Beside the size of the group: which visit of how many.
> numbered.add(of = n(), by: patient).where(visit == of)
frame · 2 rows × 5 variables

      patient   day   score   visit   of
  1   b           5     2.5       2    2
  2   a           9       6       3    3
