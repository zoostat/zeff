# dataset

## Summary
Load a shipped dataset by name

## Description
Loads one of the datasets that ship inside the engine as a frame, with the labels, units, value
labels, categories and panel declaration the file carries: `dataset(wages)` is the synthetic wages
panel and `dataset(sleep)` Student's data of 1908. The same files form the `zeff:` data root, so
`import("zeff:wages.csv")` reads the same values as plain text, without labels or categories.
`datasets()` lists what ships, and `zeff help wages` prints a dataset's page. Like R's `data` and
Stata's `sysuse`.

## Arguments
- name: The dataset's name, written bare: `wages`, `sleep`.

## Value
A `Frame` named after the dataset.

## See also
datasets, import, describe

## Examples
> dataset(sleep).n_rows()
20
> dataset(wages).mean(wage, by: union)
mean · wage by union

  union     mean
  no      26.463
  yes     29.314
warning: 211 rows with NA in wage were skipped (line 2)
> import("zeff:sleep.csv").n_cols()
3
note: zeff:sleep.csv read as CSV: 20 rows × 3 variables, types guessed; CSV carries no labels, value labels or categories (line 3)
