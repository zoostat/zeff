# dataset

## Summary
Load a shipped dataset by name

## Description
Loads one of the datasets that ship inside the engine as a frame, with the labels, units, value
labels, categories and panel declaration the file carries: `dataset(wages)` is the synthetic wages
panel and `dataset(sleep)` Student's data of 1908. The same files form the `zeff:` data root, so
`import("zeff:wages.csv")` reads the same values as plain text, without labels or categories.
`datasets()` lists what ships, and `zeff help wages` prints a dataset's page.

## Arguments
- name: The dataset's name, written bare: `wages`, `sleep`.

## Value
A `Frame` named after the dataset.

## Missing values
A dataset comes with its missing values as they are: the shipped wages have 211 wages missing. Nothing is filled or dropped on loading.

## A common mistake
Misspelling the name. Nothing is loaded in its place, and the message offers the nearest name:

> dataset(wage)
error[Z0101]: no dataset is called `wage`
  = help: did you mean `wages`?

Write `dataset(wages)`; `datasets()` lists the names.

## Coming from R or Stata
| dataset(wages) | data(mtcars), which makes the object `mtcars` | sysuse auto |

R's `data(name)` puts the dataset into the workspace under its own name; here `dataset` gives a
frame, which a script binds with `let` or uses at once. Stata's `sysuse` replaces the dataset
in memory.

## Related tasks
- For the list of the datasets that ship, use `datasets`.
- To read the same values as a plain file, or a file of your own, use `import`.
- To see the variables of a dataset with their types, use `describe`.

## See also
datasets, import, describe

## Examples
// A dataset by its name.
> dataset(sleep).n_rows()
20

// It carries its labels, categories and panel declaration: the header and the labelled values show them.
> dataset(wages).head(2)
frame · 2 rows × 11 variables · panel worker × year

      worker   year    wage   educ   exper   age   hours   union   female   industry   firm
  1        1   2015   18.08     17       9    32    45.1   no      male     Retail       99
  2        1   2016   17.28     17      10    33    41.4   no      male     Retail       99

// Used at once, with no binding.
> dataset(wages).mean(wage, by: union)
mean · wage by union

  union     mean
  no      26.463
  yes     29.314
warning[W0201]: 211 rows with NA in wage were skipped (line 3)

// The same values read as a plain file have no labels: the group is a code.
> import("zeff:sleep.csv").head(2)
frame · 2 rows × 3 variables

      extra   group   id
  1     0.7       1    1
  2    -1.6       1    2
note: zeff:sleep.csv read as CSV: 20 rows × 3 variables, types guessed; CSV carries no labels, value labels or categories (line 4)
