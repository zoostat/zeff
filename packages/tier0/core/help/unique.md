# unique

## Summary
Items with duplicates removed

## Description
A new list holding each distinct item once, in the order of first occurrence (section 6 of the
Language Reference, "Row order"). Two items are the same under the identity relation of section
3.7, not under `==`: `NA` is identical to `NA`, so `unique([NA, NA])` is `[NA]`, and `-0.0` is
identical to `0.0`, so one of them is kept. The list given is not changed. Like R's `unique`.

## Arguments
- list: The list.

## Value
A list of the same type, no longer than the one given.

## Missing values
`NA` is identical to `NA`, so several of them leave one, where it first stood. Nothing is dropped and nothing is printed.

## Coming from R or Stata
| unique(xs) | unique(xs) | local u : list uniq xs |

R's `unique` keeps the first of each, as here, and keeps one `NA`. For the distinct rows of a
frame the function is `distinct`, as `dplyr::distinct` in R and `duplicates drop` in Stata.

## Related tasks
- For the distinct rows of a frame, use `distinct`.
- To count the distinct values of a variable, use `n_distinct`.

## See also
sorted, is_in, len, equal

## Examples
// Each value once, in the order first met.
> unique([3, 1, 3, 2, 1])
3
1
2

// One missing value is kept.
> unique([NA, 2, NA])
NA
2

// Text.
> unique(["b", "a", "b"])
b
a
