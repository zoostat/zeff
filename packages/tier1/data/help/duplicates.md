# duplicates

## Summary
Table of repeated key values

## Description
The values of the named variables that more than one row holds, with the number of copies of each: a table titled `duplicates · firm_id` with the key
columns and `copies`, one row per repeated value in key order (`NA` last), in the shape of
Stata's `duplicates report`. A note says when no value is repeated. Values are compared by
identity, so rows with `NA` keys count together. Before a `merge` that expects one row per
key, `duplicates` shows which keys would break it; `distinct` then keeps the first of each.

## Arguments
- data: The frame.
- vars: The variable, or list of variables, whose repeated values are counted.

## Value
A table with the key columns and `copies`; empty, with a note, when nothing is repeated.

## Details
Exact: the rows are grouped by the kernel's identity on the key values and the groups of more
than one row are listed with their sizes.

## See also
distinct, n_distinct, merge

## Examples
> let sales = frame(firm = [2, 1, 2, 1, 3], year = [2021, 2020, 2020, 2020, 2021], revenue = [12.5, 8, 11, 9, 7])
> sales.duplicates(firm)
duplicates · firm

  firm   copies
  1           2
  2           2
> sales.duplicates([firm, year])
duplicates · firm, year

  firm   year   copies
  1      2020        2
