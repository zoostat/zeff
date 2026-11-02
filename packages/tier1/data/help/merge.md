# merge

## Summary
Join two frames on key variables

## Description
Joins `data` (the left frame) with `frame` (the right) on the key variables in `on:`, which must
exist in both with the same type. Every merge prints how many rows matched and how many did not
on each side, as Stata's `_merge` table does. `NA` keys never match and are reported as
unmatched (dplyr matches them). The result keeps the left frame's row order, and within one left
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

## Details
Exact: the right rows are indexed by their key values, compared by value (a category by its
label) with `NA` matching nothing, and the pairs are listed in the order stated above; nothing is
computed. `relation:` is checked on the whole of each frame before joining: `many_to_one` needs
distinct keys on the right, `one_to_many` on the left, `one_to_one` on both. Without `relation:`
the merge is refused only when a key repeats on both sides.

## See also
append, to_long, to_wide, aggregate, rename, duplicates

## Examples
> let firms = dataset(firms)
> let wages = dataset(wages).merge(firms, on: [firm, industry], relation: many_to_one)
note: merge: 18,412 rows matched; 0 left rows unmatched; 3 right rows unmatched (line 2)
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
note: merge: 3 rows matched; 1 left row unmatched; 1 right row unmatched (line 5)
> wages.merge(firms, on: firm)
error[Z0604]: columns `industry`, `size`, `founded` exist in both frames and are not a key
 --> examples.zeff:6:13
  |
6 | wages.merge(firms, on: firm)
  |             ^^^^^ shares a column with the left frame
  |
  = help: give suffixes: ["_l", "_r"] to keep both, or drop one of them first
