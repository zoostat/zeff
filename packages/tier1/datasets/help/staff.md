# staff

## Summary
Synthetic staff offered a training course by lot: take-up and productivity

## Description
One row per employee of a firm with twelve sites in four regions, 2,400 employees. Half of them,
chosen by lot, were offered a training course (`offer`); whether they took it (`training`) was
their own choice, and the more able took it more often. `productivity` is their output per hour
measured afterwards, `tenure` the years they have stayed with the firm since the course, and
`distance` the kilometres from their home to the training centre, which made the course easier
or harder to attend. The ability that raises both the take-up and productivity is not in the
file, as it would not be in a firm's records.

The data are generated, not collected, so the effect of the course is known: 4.5 units per hour.
A comparison of those who took the course with those who did not gives about 5.9, because the
able are over-represented among the trained. The offer was drawn by lot and moves productivity
only through the course, so it is an instrument for it, and `distance` is a second, weaker one.
The variables are also those of the causal graph of section 7.3 of the Language Reference.

## Generation
`generate/staff.py` in the datasets package, run with a fixed seed (20261116), writes the CSV;
the Arrow file with the labels, the value labels and the `region` category is built from it by
`cargo run -p xtask -- build-datasets`. Each employee works at one of twelve sites, each as
likely; sites 1 to 3 are in the North, 4 to 6 in the East, 7 to 9 in the South and 10 to 12 in
the West. Age is 22 to 60, each year as likely. Ability is standard normal and is not written.
The offer is made with probability one half. The distance is normal around 12 km with spread 6,
kept between 0.5 and 40.

The course is taken with probability 1 / (1 + exp(−u)), where u = −1.3 + 2.4 offer + 0.8 ability
− 0.03 (age − 40) − 0.05 (distance − 12) + a shift by region (North 0, East 0.3, South −0.2,
West 0.4): about a quarter of those not offered the course take it, and about three quarters of
those offered. Tenure is 3 + 1 training + a uniform draw between −2 and 2, in years. Productivity
is 50 + 4.0 training + 0.5 tenure + 3.0 ability + the site's effect + noise, normal with spread
4; the twelve site effects are drawn once from a normal with spread 2.5. Tenure and productivity
are rounded to one decimal.

The course therefore raises productivity by 4.0 itself and by 0.5 through the year of tenure it
adds, 4.5 in all, the same for every employee. A model that holds tenure fixed estimates the 4.0
and not the 4.5, and is biased besides, since tenure is itself a result of the course.

## Licence and citation
CC0-1.0: the data are synthetic and dedicated to the public domain. Cite as "Zeff synthetic
staff, version 0.8.0".

## See also
dataset, datasets, wages, reg

## Examples
> let staff = dataset(staff)
> staff.describe()
staff · 2,400 rows × 9 variables

  variable       type        non-missing   missing   example
  employee       Int               2,400         0   1
  site           Int               2,400         0   9
  region         Category          2,400         0   South (4 levels)
  age            Int               2,400         0   37
  offer          Int (lab)         2,400         0   yes
  distance       Float             2,400         0   9.5
  training       Int (lab)         2,400         0   yes
  tenure         Float             2,400         0   5.5
  productivity   Float             2,400         0   61.9
> staff.head(3)
frame · 3 rows × 9 variables

      employee   site   region   age   offer   distance   training   tenure   productivity
  1          1      9   South     37   yes          9.5   yes           5.5           61.9
  2          2      7   South     51   no          14.5   no            4.3           46.3
  3          3      1   North     22   no           9.8   yes           2.6           57.8
> staff.aggregate(by: offer, n = n(), took_course = mean(training))
frame · 2 rows × 3 variables

      offer      n   took_course
  1   no      1221      0.256347
  2   yes     1179      0.742154
> staff.reg(productivity ~ training, fe: site)
reg · productivity ~ training · fe: site
N = 2,400 (no singletons)
fixed effects: site (12)
standard errors: classical

  term       estimate   std.error   t value   p value   95% CI
  training     5.9265      0.2069     28.64    <0.001   [5.5207, 6.3323]

R² within 0.256 · R² overall 0.407
