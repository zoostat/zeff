# describe

## Summary
Type, non-missing and missing count of each variable

## Description
One row per variable of the frame, in the frame's order: the variable's name, its type, how many
of its values are present, how many are missing, and an example. The example is the first value
that is not missing, printed as the value prints anywhere in Zeff (a Float to six significant
digits, a date in ISO form), so that the same file always shows the same example; a variable whose
values are all missing shows `NA`. A category's example adds its number of levels. The title line
is the frame's header, the same line that prints before a frame's first ten rows.

With `by:` the table has one block of rows per group, the group's keys in its first columns, so that
the variables of each group can be read side by side.

## Arguments
- data: The frame.

## Value
A `Table`, which prints as shown and can be passed to `print`.

## Missing values
Missing values are what the table counts: for each variable, the values present and the values that are `NA`. A variable of nothing but `NA` has `NA` as its example.

## Coming from R or Stata
| d.describe() | str(d), with colSums(is.na(d)) for the missing values | describe, with misstable summarize for the missing values |

R's `str` and Stata's `describe` say how each variable is stored; this table says what a reader
of the data asks first: the type, how many values are there, and what one looks like.

## Related tasks
- For the mean, the spread and the quantiles of each variable, use `summary`.
- For the first rows themselves, use `head`.
- For the names alone, use `variables`.
- For the count of each value of a variable, use `tabulate`.

## See also
summary, n_rows, n_cols, variables

## Examples
// The shipped firms: a date and a category beside numbers.
> dataset(firms).describe()
firms · 400 rows × 4 variables

  variable   type       non-missing   missing   example
  firm       Int                400         0   1
  industry   Category           400         0   Agriculture (12 levels)
  size       Int                400         0   3
  founded    Date               400         0   1990-11-06

// The shipped wages, where some values are missing.
> dataset(wages).describe()
wages · 18,412 rows × 11 variables · panel worker × year

  variable   type        non-missing   missing   example
  worker     Int              18,412         0   1
  year       Int              18,412         0   2015
  wage       Float            18,201       211   18.08
  educ       Int              18,412         0   17
  exper      Int              18,390        22   9
  age        Int              18,412         0   32
  hours      Float            18,377        35   45.1
  union      Int (lab)        18,412         0   no
  female     Int (lab)        18,412         0   male
  industry   Category         18,412         0   Retail (12 levels)
  firm       Int              18,412         0   99

// Group by group with `by:`.
> dataset(sleep).describe(by: group)
sleep · 20 rows × 3 variables · by group

  group                variable   type        non-missing   missing   example
  dextro-hyoscyamine   extra      Float                10         0   0.7
  dextro-hyoscyamine   group      Int (lab)            10         0   dextro-hyoscyamine
  dextro-hyoscyamine   id         Int                  10         0   1
  laevo-hyoscyamine    extra      Float                10         0   1.9
  laevo-hyoscyamine    group      Int (lab)            10         0   laevo-hyoscyamine
  laevo-hyoscyamine    id         Int                  10         0   1
