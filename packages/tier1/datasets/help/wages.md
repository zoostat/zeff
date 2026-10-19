# wages

## Summary
Synthetic hourly wages of 3,911 workers, 2010 to 2021

## Description
An unbalanced panel of 3,911 workers observed yearly between 2010 and 2021, 18,412 worker-years:
hourly wage, schooling, potential experience, age, weekly hours, union membership, sex and industry.
The data are generated, not collected, so every parameter behind them is known and documented
below; they exist to give the help pages stable numbers and the tests a known answer. The analysis
derives what it needs from them, as the script of section 10.1 of the Language Reference does: the
log wage, `add(lwage = log(wage))`, the squared experience, and age groups.

## Generation
`generate/wages.py` in the datasets package, run with a fixed seed (20261019), writes the CSV; the
Arrow file with the labels, value labels, the `industry` category and the panel declaration is built
from it by `cargo run -p xtask -- build-datasets`. Each worker enters in a random year and stays two
to twelve years. Fixed per worker: sex (47 per cent women), schooling (8 to 20 years, centred on
13.5), a birth year giving ages 18 to 40 in 2010, one of twelve industries drawn with unequal
shares (Retail the largest), and union membership drawn with a probability rising in experience and
differing by industry. Varying by year: age, potential experience (age less schooling less six,
floored at zero) and weekly hours (around 39, spread 8, women about three fewer, union members one
more, kept between 5 and 70).

The log hourly wage is 1.70 + 0.074 schooling + 0.041 experience − 0.0007 experience² − 0.112 female
+ 0.08 union + an industry effect (Retail 0, the others within ±0.15: Agriculture −0.15, Mining
+0.15, Utilities +0.12, Construction +0.05, Manufacturing +0.04, Transport +0.02, Finance +0.14,
Services −0.03, Education +0.01, Health +0.06, Public administration +0.08) + 0.015 per year since
2010 + a worker effect, normal with spread 0.30, + noise, normal with spread 0.30; the wage is its
exponential rounded to two decimals. Missing values are drawn at random: 211 wages, 22 experience
values and 35 weekly hours, written as empty cells. The true return to schooling is therefore 0.074
log points a year.

## Licence and citation
CC0-1.0: the data are synthetic and dedicated to the public domain. Cite as "Zeff synthetic wages
panel, version 0.2.0".

## See also
dataset, datasets, sleep

## Examples
> let wages = dataset(wages)
> wages.describe()
wages · 18,412 rows × 10 variables · panel worker × year

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
> wages.summary([wage, educ, exper], by: female)
summary · by female

  female   variable       N   missing     mean       sd     min       max
  male     wage       9,375       116   29.189   14.787   4.740   152.390
  female   wage       8,826        95   25.437   12.895   3.530   140.240
  male     educ       9,491         0   13.470    2.468   8.000    20.000
  female   educ       8,921         0   13.514    2.503   8.000    20.000
  male     exper      9,477        14   15.264    7.468   0.000    36.000
  female   exper      8,913         8   14.866    7.564   0.000    36.000
warning: 211 rows with NA in wage and 22 with NA in exper were skipped (line 3)
> wages.head(3)
frame · 3 rows × 10 variables · panel worker × year

      worker   year    wage   educ   exper   age   hours   union   female   industry
  1        1   2015   18.08     17       9    32    45.1   no      male     Retail
  2        1   2016   17.28     17      10    33    41.4   no      male     Retail
  3        1   2017   20.93     17      11    34    54.3   no      male     Retail
