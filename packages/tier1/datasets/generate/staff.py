"""The synthetic `staff` dataset (Engine and Project Specification, section 8.5; release 0.8.0,
step 13a; question 2 of the release, taken as recommended on 15 November 2026): 2,400 employees
of a firm with twelve sites in four regions, in an encouragement design. A training course is
offered to half of them by lot; taking it up is the employee's choice, and the more able take it
up more; productivity is measured afterwards. Run with
`python3 -I packages/tier1/datasets/generate/staff.py`; it rewrites
`packages/tier1/datasets/data/staff.csv` from draws of Python's Mersenne Twister with a fixed
seed.

The generating model, which the dataset's help page documents and the tests use. For each
employee, in this order of draws:

    site         1 to 12, each as likely; region is North for sites 1 to 3, East for 4 to 6,
                 South for 7 to 9, West for 10 to 12
    age          22 to 60 whole years, each as likely
    ability      N(0, 1); not written to the file
    offer        1 with probability 1/2, by lot
    distance     N(12, 6^2) kilometres from home to the training centre, kept between 0.5 and
                 40, one decimal
    training     1 with probability 1 / (1 + exp(-u)),
                 u = -1.3 + 2.4 offer + 0.8 ability - 0.03 (age - 40) - 0.05 (distance - 12)
                     + the region's shift (North 0, East 0.3, South -0.2, West 0.4)
    tenure       3.0 + 1.0 training + U(-2, 2) years with the firm since the course, one decimal
    productivity 50 + 4.0 training + 0.5 tenure + 3.0 ability + the site's effect + N(0, 4^2),
                 one decimal

The twelve site effects are drawn once, before any employee, from N(0, 2.5^2) and rounded to two
decimals. The course therefore raises productivity by 4.0 itself and by 0.5 through the year of
tenure it adds: 4.5 in all. Ability raises both the take-up and productivity, so the difference
in productivity between those who took the course and those who did not overstates the effect;
the offer and the distance move the take-up and nothing else.
"""

import csv
import math
import os
import random

SEED = 20261116
EMPLOYEES = 2400
REGIONS = ["North", "East", "South", "West"]
SITES_PER_REGION = 3
REGION_SHIFT = [0.0, 0.3, -0.2, 0.4]
SITE_SD = 2.5

COURSE = 4.0
TENURE_FROM_COURSE = 1.0
PRODUCTIVITY_PER_YEAR = 0.5
ABILITY = 3.0
NOISE_SD = 4.0


def main():
    rng = random.Random(SEED)
    here = os.path.dirname(os.path.abspath(__file__))
    site_effect = [round(rng.gauss(0.0, SITE_SD), 2) for _ in range(len(REGIONS) * SITES_PER_REGION)]
    rows = []
    for employee in range(1, EMPLOYEES + 1):
        site = rng.randint(1, len(site_effect))
        region = (site - 1) // SITES_PER_REGION
        age = rng.randint(22, 60)
        ability = rng.gauss(0.0, 1.0)
        offer = 1 if rng.random() < 0.5 else 0
        distance = round(min(max(rng.gauss(12.0, 6.0), 0.5), 40.0), 1)
        u = (-1.3 + 2.4 * offer + 0.8 * ability - 0.03 * (age - 40) - 0.05 * (distance - 12.0)
             + REGION_SHIFT[region])
        training = 1 if rng.random() < 1.0 / (1.0 + math.exp(-u)) else 0
        tenure = round(3.0 + TENURE_FROM_COURSE * training + rng.uniform(-2.0, 2.0), 1)
        productivity = round(
            50.0 + COURSE * training + PRODUCTIVITY_PER_YEAR * tenure + ABILITY * ability
            + site_effect[site - 1] + rng.gauss(0.0, NOISE_SD),
            1,
        )
        rows.append([employee, site, REGIONS[region], age, offer, distance, training, tenure, productivity])
    header = ["employee", "site", "region", "age", "offer", "distance", "training", "tenure", "productivity"]
    path = os.path.join(here, "..", "data", "staff.csv")
    with open(path, "w", encoding="utf-8", newline="\n") as f:
        writer = csv.writer(f, lineterminator="\n")
        writer.writerow(header)
        for row in rows:
            writer.writerow([repr(v) if isinstance(v, float) else v for v in row])
    print("wrote", path, len(rows), "rows; site effects", site_effect)


if __name__ == "__main__":
    main()
