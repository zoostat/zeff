# sort

## Summary
Sort rows; stable, NA last

## Description
The rows in the order of the variables named: by the first, then by the second within equal values
of the first, and so on. The sort is stable, so rows with equal keys keep their previous order, and
`NA` comes last whether ascending or descending. Numbers sort numerically, text by code point (never
by locale, so the order is the same on every machine), dates chronologically, categories by level
position, and `false` before `true` (section 3.7 of the Language Reference, "Identity"). Nothing is
copied but the order: every variable keeps its metadata.

## Arguments
- data: The frame.
- vars: The variables to sort by, in order of precedence: `wage`, `[region, wage]` or a range.
- desc: `true` sorts every key from largest to smallest; `NA` stays last.

## Value
A `Frame` with the same variables and the rows reordered.

## Missing values
`NA` comes last, whether the order is ascending or descending. No row is dropped.

## A common mistake
Writing a minus sign for the largest first, as R's `order(-x)` and Stata's `gsort -x` do. The variables are named, and `desc:` says the direction:

> dataset(firms).sort(-size)
error[Z0401]: `vars` of `sort` takes variables of the frame, not an expression
  = help: compute it first and name the new variable: `.add(v = -size)` before `.sort(v)`

Write `dataset(firms).sort(size, desc: true)`.

## Coming from R or Stata
| d.sort(wage) | d[order(d$wage), ], or dplyr::arrange(d, wage) | sort wage |
| d.sort(wage, desc: true) | d[order(-d$wage), ], or dplyr::arrange(d, desc(wage)) | gsort -wage |
| d.sort([region, wage]) | d[order(d$region, d$wage), ] | sort region wage |

R's `order` and `arrange` put `NA` last, as here. Stata takes a missing value as the largest,
so `sort` puts it last and `gsort -wage` puts it first; here it is last both ways. Stata's
`sort` may reorder rows that are equal on the keys unless `stable` is given; this sort always
keeps them as they stood.

## Related tasks
- For the first or the last rows after a sort, use `head` or `tail`.
- For the rank of each value as a variable, use `rank`.
- To number the rows as they stand, use `row_number`.
- To keep the rows that meet a condition, use `where`.

## See also
where, head, tail

## Examples
// By one variable, the earliest first.
> let firms = dataset(firms)
> firms.sort(founded).head(3)
frame · 3 rows × 4 variables

      firm   industry   size   founded
  1     14   Mining        3   1950-01-15
  2    235   Services     39   1950-01-17
  3    346   Health       12   1950-02-11

// The largest first with `desc: true`.
> firms.sort(size, desc: true).head(3)
frame · 3 rows × 4 variables

      firm   industry        size   founded
  1     95   Manufacturing     39   1959-06-10
  2    222   Services          39   1967-05-29
  3    235   Services          39   1950-01-17

// By several: the second orders the rows that are equal on the first.
> firms.sort([industry, size]).head(4)
frame · 4 rows × 4 variables

      firm   industry      size   founded
  1      1   Agriculture      3   1990-11-06
  2      3   Agriculture      3   1998-11-07
  3     10   Agriculture      3   1992-09-02
  4      8   Agriculture      4   1986-08-19

// `NA` comes last, ascending or descending.
> dataset(wages).sort(wage).tail(2).select([worker, year, wage])
> dataset(wages).sort(wage, desc: true).tail(2).select([worker, year, wage])
frame · 2 rows × 3 variables · panel worker × year

      worker   year   wage
  1     3893   2017     NA
  2     3905   2016     NA
frame · 2 rows × 3 variables · panel worker × year

      worker   year   wage
  1     3893   2017     NA
  2     3905   2016     NA
