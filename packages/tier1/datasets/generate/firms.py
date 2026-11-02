"""The synthetic `firms` dataset (Engine and Project Specification, section 8.5; release 0.6.0,
step 13; the owner's decision of 1 November 2026): one row per firm of the `wages` panel, with
three firms that employ no worker in the data, so that a merge has unmatched rows to report.

Run with `python3 -I packages/tier1/datasets/generate/firms.py` after `wages.py`; it reads
`packages/tier1/datasets/data/wages.csv` and rewrites `firms.csv`, byte for byte the same on
every machine. The firm numbers, industries and sizes are those of the wages file (the size is
the number of distinct workers of the firm there); the founding dates are drawn with a fixed seed
from Python's Mersenne Twister, uniformly over the years 1950 to 2009, and written as ISO 8601
dates, so that `founded` is a `Date` column.
"""

import calendar
import csv
import os
import random

SEED = 20261102
WITHOUT_WORKERS = 3


def main():
    rng = random.Random(SEED)
    here = os.path.dirname(os.path.abspath(__file__))
    with open(os.path.join(here, "..", "data", "wages.csv"), encoding="utf-8", newline="") as f:
        reader = csv.DictReader(f)
        workers = {}
        industry = {}
        for row in reader:
            firm = int(row["firm"])
            workers.setdefault(firm, set()).add(int(row["worker"]))
            industry[firm] = row["industry"]
    industries = sorted(set(industry.values()), key=list(industry.values()).index)
    firms = []
    for firm in sorted(workers):
        firms.append([firm, industry[firm], len(workers[firm])])
    for extra in range(WITHOUT_WORKERS):
        firms.append([max(workers) + 1 + extra, rng.choice(industries), 0])
    for row in firms:
        year = rng.randint(1950, 2009)
        month = rng.randint(1, 12)
        day = rng.randint(1, calendar.monthrange(year, month)[1])
        row.append(f"{year:04d}-{month:02d}-{day:02d}")
    path = os.path.join(here, "..", "data", "firms.csv")
    with open(path, "w", encoding="utf-8", newline="\n") as f:
        writer = csv.writer(f, lineterminator="\n")
        writer.writerow(["firm", "industry", "size", "founded"])
        for row in firms:
            writer.writerow(row)
    print("wrote", path, len(firms), "rows")


if __name__ == "__main__":
    main()
