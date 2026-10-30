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

## See also
sorted, is_in, len, equal

## Examples
> unique([3, 1, 3, 2, 1])
3
1
2
> unique([NA, 2, NA])
NA
2
> unique(["b", "a", "b"])
b
a
