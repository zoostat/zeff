# merge

## Summary
Join two frames on key variables

## In brief
- `a.merge(b, on: id)`: every row of `a`, with the columns of `b` where the key matches.
- `a.merge(b, on: [firm, year], join: inner)`: the matched rows alone; `join: full` keeps every row of both.
- `a.merge(b, on: firm, relation: many_to_one)`: the key structure stated, and checked before joining.
- `a.merge(b, on: id, suffixes: ["_a", "_b"])`: for columns the two frames share beside the key.

## Description
Joins `data` (the left frame) with `frame` (the right) on the key variables in `on:`, which must
exist in both with the same type. Every merge prints how many rows matched and how many did not
on each side, as Stata's `_merge` table does. `NA` keys never match and are reported as
unmatched. The result keeps the left frame's row order, and within one left
row the right frame's order; `join: full` adds the unmatched right rows at the end, in the
right frame's order.

`relation:` states the key structure you expect and refuses the data when it differs:
`many_to_one` means each left key appears at most once on the right. A many-to-many merge is
always refused (Z0603), since it is nearly always a mistake. A non-key column present in both
frames is an error (Z0604) that lists the columns, unless `suffixes:` gives the two suffixes
that tell them apart.

## Arguments
- data: The left frame.
- frame: The right frame.
- on: The key variables, present in both frames with the same type; when the names differ, `rename` one frame first.
- join: Which rows to keep: `left` (all left rows), `inner` (matched rows only) or `full` (all rows of both frames); `left` unless given.
- relation: The expected key structure, `one_to_one`, `many_to_one` or `one_to_many`, checked before joining.
- suffixes: A list of two texts, such as `["_l", "_r"]`, added to the non-key columns that both frames share.

## Value
A frame with the left frame's columns, then the right frame's non-key columns; labels, value
labels and categories are kept on both sides, and a panel declaration on other keys is kept
unless the merge repeated or added rows.

## Missing values
A key that is `NA` matches nothing, on either side: its row is counted as unmatched, and kept or not as `join:` says. A left row with no match has `NA` in every column that comes from the right frame.

## Details
Exact: the right rows are indexed by their key values, compared by value (a category by its
label) with `NA` matching nothing, and the pairs are listed in the order stated above; nothing is
computed. `relation:` is checked on the whole of each frame before joining: `many_to_one` needs
distinct keys on the right, `one_to_many` on the left, `one_to_one` on both. Without `relation:`
the merge is refused only when a key repeats on both sides.

## A common mistake
Joining on one variable when the frames share another. A column that both have and that is not a key cannot be one column of the result:

> dataset(wages).merge(dataset(firms), on: firm)
error[Z0604]: column `industry` exists in both frames and is not a key
  = help: give suffixes: ["_l", "_r"] to keep both, or drop one of them first

Make it a key when it means the same in both, `on: [firm, industry]`, or keep both with `suffixes: ["_w", "_f"]`.

## Coming from R or Stata
| a.merge(b, on: id) | merge(a, b, by = "id", all.x = TRUE), or dplyr::left_join(a, b, by = "id") | merge 1:1 id using b, keep(master match) |
| a.merge(b, on: id, join: inner) | merge(a, b, by = "id"), or dplyr::inner_join(a, b, by = "id") | merge 1:1 id using b, keep(match) |
| a.merge(b, on: id, join: full) | merge(a, b, by = "id", all = TRUE), or dplyr::full_join(a, b, by = "id") | merge 1:1 id using b |
| a.merge(b, on: firm, relation: many_to_one) | dplyr::left_join(a, b, by = "firm", relationship = "many-to-one") | merge m:1 firm using b, keep(master match) |

R's `merge` keeps the matched rows alone unless `all.x` or `all` is given, and sorts the result
by the key; here, as in `left_join`, every left row is kept in its order unless `join:` says
otherwise. dplyr matches a key that is `NA` with another that is `NA`; here `NA` matches nothing.
Stata's `merge` asks for the relation (`1:1`, `m:1`, `1:m`) and adds the variable `_merge`;
here the relation is checked when given, and the note counts the matched and unmatched rows.
Stata's `merge m:m` pairs rows by their position within a key; here a many-to-many merge is
refused.

## Related tasks
- To put the rows of one frame under those of another, use `append`.
- To see which key values repeat before joining, use `duplicates`.
- To make the key's name the same in both frames, use `rename`.
- To bring a summary of groups back to the rows, make it with `aggregate`.

## See also
append, to_long, to_wide, aggregate, rename, duplicates

## Examples
// Each row of the shipped wages with the size and the founding date of its firm: many rows of wages to one firm.
> let firms = dataset(firms)
> let wages = dataset(wages).merge(firms, on: [firm, industry], relation: many_to_one)
> wages.select([worker, year, firm, size, founded]).head(3)
note: merge: 18,412 rows matched; 0 left rows unmatched; 3 right rows unmatched (line 2)
frame · 3 rows × 5 variables · panel worker × year

      worker   year   firm   size   founded
  1        1   2015     99      3   1993-12-14
  2        1   2016     99      3   1993-12-14
  3        1   2017     99      3   1993-12-14

// `join: full` keeps the rows of both frames; a row with no match has `NA` on the other side.
> let sales = frame(firm = [2, 1, 3, 1], revenue = [12.5, 8, 7, 9])
> let names = frame(firm = [1, 2, 4], name = ["Acme", "Bolt", "Dyne"])
> sales.merge(names, on: firm, join: full)
frame · 5 rows × 3 variables

      firm   revenue   name
  1      2      12.5   Bolt
  2      1         8   Acme
  3      3         7   NA
  4      1         9   Acme
  5      4        NA   Dyne
note: merge: 3 rows matched; 1 left row unmatched; 1 right row unmatched (line 6)

// `join: inner` keeps the matched rows alone.
> sales.merge(names, on: firm, join: inner)
frame · 3 rows × 3 variables

      firm   revenue   name
  1      2      12.5   Bolt
  2      1         8   Acme
  3      1         9   Acme
note: merge: 3 rows matched; 1 left row unmatched; 1 right row unmatched (line 7)

// Two years of wages side by side: the columns the frames share beside the key are told apart by `suffixes:`.
> let y2019 = dataset(wages).where(year == 2019).select([worker, wage, hours])
> let y2020 = dataset(wages).where(year == 2020).select([worker, wage, hours])
> y2019.merge(y2020, on: worker, join: inner, relation: one_to_one, suffixes: ["_2019", "_2020"]).head(3)
note: the panel declaration `worker × year` dropped: `select` left out `year` (line 8)
note: the panel declaration `worker × year` dropped: `select` left out `year` (line 9)
frame · 3 rows × 5 variables

      worker   wage_2019   hours_2019   wage_2020   hours_2020
  1        1       26.39         32.3       33.07         31.3
  2       15       20.14         40.3       26.73         46.2
  3       17       49.23         43.7       20.96         33.4
note: merge: 973 rows matched; 545 left rows unmatched; 129 right rows unmatched (line 10)
