# firms

## Summary
Synthetic firms of the wages panel: industry, size and founding date

## Description
One row per firm of the `wages` panel: the firm's number, its industry, the number of distinct
workers it employs in `wages` and the date it was founded. Three firms employ no worker in the
data, so that a merge of `wages` with `firms` has right rows to report as unmatched, as the
page of `merge` shows. The firm numbers are those of the `firm` variable of `wages`, and the industries agree
with `wages` by construction, so the two frames merge on `[firm, industry]`.

## Generation
`generate/firms.py` in the datasets package reads the wages file written by `generate/wages.py`,
takes each firm's number, industry and count of distinct workers, adds three firms without
workers whose industries are drawn at random, and draws every firm's founding date uniformly
over the years 1950 to 2009 with a fixed seed (20261102), written as an ISO 8601 date. The Arrow
file with the labels, the `industry` category and the `founded` date is built from it by
`cargo run -p xtask -- build-datasets`. The firms themselves come from the rule of `wages.py`:
the workers of each industry, in order of their number, cut into firms of 3 to 40 workers, most
of them small.

## Licence and citation
CC0-1.0: the data are synthetic and dedicated to the public domain. Cite as "Zeff synthetic firms,
version 0.6.0".

## See also
dataset, datasets, wages, merge

## Examples
> let firms = dataset(firms)
> firms.describe()
firms · 400 rows × 4 variables

  variable   type       non-missing   missing   example
  firm       Int                400         0   1
  industry   Category           400         0   Agriculture (12 levels)
  size       Int                400         0   3
  founded    Date               400         0   1990-11-06
> firms.head(3)
frame · 3 rows × 4 variables

      firm   industry      size   founded
  1      1   Agriculture      3   1990-11-06
  2      2   Agriculture      5   1957-07-02
  3      3   Agriculture      3   1998-11-07
