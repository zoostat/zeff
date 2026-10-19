"""The synthetic `wages` dataset (Engine and Project Specification, section 8.5; release 0.2.0,
step 18; the generating model approved by the owner on 19 October 2026).

An unbalanced panel of 3,911 workers observed yearly from 2010 to 2021, 18,412 worker-years in
all. Run with `python3 -I packages/tier1/datasets/generate/wages.py`; it rewrites
`packages/tier1/datasets/data/wages.csv`, byte for byte the same on every machine: the draws
come from Python's Mersenne Twister with a fixed seed, and the exponential and the logarithm are
computed with the `decimal` module and rounded once, never with the platform's maths library.

The true parameters, which the dataset's help page documents and the truth-recovery tests use:

    log wage = 1.70 + 0.074 educ + 0.041 exper - 0.0007 exper^2 - 0.112 female + 0.08 union
               + industry effect + 0.015 (year - 2010) + worker effect + noise

with the worker effect N(0, 0.30^2), the noise N(0, 0.30^2) and the industry effects below
(Retail is the base). `wage` is exp(log wage) rounded to two decimals. The file holds the ten
variables a researcher receives (worker, year, wage, educ, exper, age, hours, union, female,
industry); log wage, squared experience and the age group are derived in the analysis, as the
script of section 10.1 of the Language Reference does, which also keeps the file under the
megabyte that ships inside the engine (section 8.5). Missing values are written as empty cells:
211 wages, 22 experience values, 35 hours, all at random.
"""

import csv
import decimal
import math
import os
import random
from decimal import Decimal

SEED = 20261019
WORKERS = 3911
ROWS = 18412
YEARS = range(2010, 2022)

RETURN_TO_EDUCATION = 0.074
EXPERIENCE = 0.041
EXPERIENCE_SQUARED = -0.0007
FEMALE = -0.112
UNION = 0.08
YEAR_TREND = 0.015
INTERCEPT = 1.70
WORKER_SD = 0.30
NOISE_SD = 0.30

# Industry: share of workers, wage effect (log points, Retail the base), union propensity shift.
INDUSTRIES = [
    ("Agriculture", 0.03, -0.15, 0.0),
    ("Mining", 0.01, 0.15, 0.5),
    ("Utilities", 0.02, 0.12, 0.6),
    ("Construction", 0.08, 0.05, 0.4),
    ("Manufacturing", 0.14, 0.04, 0.6),
    ("Retail", 0.16, 0.0, -0.3),
    ("Transport", 0.06, 0.02, 0.3),
    ("Finance", 0.08, 0.14, -0.5),
    ("Services", 0.14, -0.03, -0.3),
    ("Education", 0.09, 0.01, 0.5),
    ("Health", 0.12, 0.06, 0.0),
    ("Public administration", 0.07, 0.08, 0.8),
]
MISSING_WAGE, MISSING_EXPER, MISSING_HOURS = 211, 22, 35

decimal.getcontext().prec = 40


def exp_exact(x):
    """exp(x) correctly rounded to a double, independent of the platform."""
    return float(Decimal(repr(x)).exp())


def main():
    rng = random.Random(SEED)
    here = os.path.dirname(os.path.abspath(__file__))

    # Panel shape: each worker enters in a random year and stays 2 to 12 years; the durations
    # are then nudged so that the total is exactly ROWS.
    durations = [rng.randint(2, 12) for _ in range(WORKERS)]
    while sum(durations) != ROWS:
        i = rng.randrange(WORKERS)
        if sum(durations) < ROWS and durations[i] < 12:
            durations[i] += 1
        elif sum(durations) > ROWS and durations[i] > 2:
            durations[i] -= 1
    educ_levels = list(range(8, 21))
    educ_weights = [math.exp(-0.5 * ((e - 13.5) / 2.6) ** 2) for e in educ_levels]
    industry_names = [i[0] for i in INDUSTRIES]
    industry_weights = [i[1] for i in INDUSTRIES]
    effect = {i[0]: i[2] for i in INDUSTRIES}
    union_shift = {i[0]: i[3] for i in INDUSTRIES}

    rows = []
    for worker in range(1, WORKERS + 1):
        duration = durations[worker - 1]
        start = rng.randint(2010, 2021 - duration + 1)
        female = 1 if rng.random() < 0.47 else 0
        educ = rng.choices(educ_levels, educ_weights)[0]
        age_2010 = rng.randint(18, 40)
        industry = rng.choices(industry_names, industry_weights)[0]
        exper_2010 = max(age_2010 - educ - 6, 0)
        union_logit = -1.2 + 0.03 * exper_2010 + union_shift[industry]
        union = 1 if rng.random() < 1.0 / (1.0 + math.exp(-union_logit)) else 0
        worker_effect = rng.gauss(0.0, WORKER_SD)
        for year in range(start, start + duration):
            age = age_2010 + (year - 2010)
            exper = max(age - educ - 6, 0)
            lwage_latent = (
                INTERCEPT
                + RETURN_TO_EDUCATION * educ
                + EXPERIENCE * exper
                + EXPERIENCE_SQUARED * exper * exper
                + FEMALE * female
                + UNION * union
                + effect[industry]
                + YEAR_TREND * (year - 2010)
                + worker_effect
                + rng.gauss(0.0, NOISE_SD)
            )
            wage = round(exp_exact(lwage_latent), 2)
            hours = round(min(max(rng.gauss(39.0, 8.0) - 3.0 * female + 1.0 * union, 5.0), 70.0), 1)
            rows.append([worker, year, wage, educ, exper, age, hours, union, female, industry])
    assert len(rows) == ROWS

    # Missing values at random, in distinct rows per variable.
    for count, column in ((MISSING_WAGE, 2), (MISSING_EXPER, 4), (MISSING_HOURS, 6)):
        for i in rng.sample(range(ROWS), count):
            rows[i][column] = None

    header = ["worker", "year", "wage", "educ", "exper", "age", "hours", "union", "female", "industry"]
    path = os.path.join(here, "..", "data", "wages.csv")
    with open(path, "w", encoding="utf-8", newline="\n") as f:
        writer = csv.writer(f, lineterminator="\n")
        writer.writerow(header)
        for row in rows:
            writer.writerow(["" if v is None else (repr(v) if isinstance(v, float) else v) for v in row])
    print("wrote", path, ROWS, "rows")


if __name__ == "__main__":
    main()
