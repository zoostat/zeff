"""Write the three data files of the sample project of release 0.6.0, each the same bytes on
every run (Python's standard library only, a fixed seed): `data/sales_wide.csv`, the yearly
sales of the 400 firms of the shipped `firms` dataset in one row per firm with a column per
year, as a spreadsheet keeps them, with some years missing; `data/policy.csv`, a staggered
policy over 60 units and ten years, adopted in 2013, 2015 or 2017 or never, whose effect grows
with the years since adoption; and `data/counties.csv`, a two-period panel of 80 counties with
a treatment that raises employment after 2010.
"""

import random
from pathlib import Path


def sales_wide(rng):
    lines = ["firm,sales_2018,sales_2019,sales_2020,sales_2021,sales_2022"]
    for firm in range(1, 401):
        level = rng.lognormvariate(3.0, 0.8)
        cells = []
        for year in range(2018, 2023):
            if rng.random() < 0.06:
                cells.append("")
            else:
                growth = 1.0 + 0.03 * (year - 2018) - (0.12 if year == 2020 else 0.0)
                cells.append(f"{level * growth * rng.lognormvariate(0.0, 0.15):.1f}")
        lines.append(f"{firm}," + ",".join(cells))
    return "\n".join(lines) + "\n"


def policy(rng):
    lines = ["unit,year,adopted,y"]
    unit_effects = [rng.gauss(0.0, 1.0) for _ in range(60)]
    year_effects = {year: 0.05 * (year - 2010) + rng.gauss(0.0, 0.2) for year in range(2010, 2020)}
    for unit in range(1, 61):
        adopted = rng.choice([2013, 2015, 2017, None])
        for year in range(2010, 2020):
            effect = 0.0
            if adopted is not None and year >= adopted:
                effect = 0.5 + 0.1 * (year - adopted)
            y = 10.0 + unit_effects[unit - 1] + year_effects[year] + effect + rng.gauss(0.0, 0.3)
            adopted_text = "" if adopted is None else str(adopted)
            lines.append(f"{unit},{year},{adopted_text},{y:.4f}")
    return "\n".join(lines) + "\n"


def counties(rng):
    lines = ["county,year,treated,post,emp"]
    for county in range(1, 81):
        treated = int(rng.random() < 0.5)
        base = rng.gauss(50.0, 8.0)
        for year in (2010, 2015):
            post = int(year == 2015)
            emp = base + 2.0 * post + 1.5 * treated * post + rng.gauss(0.0, 1.0)
            lines.append(f"{county},{year},{treated},{post},{emp:.2f}")
    return "\n".join(lines) + "\n"


def main():
    rng = random.Random(20261103)
    folder = Path(__file__).resolve().parent / "data"
    (folder / "sales_wide.csv").write_text(sales_wide(rng), encoding="ascii")
    (folder / "policy.csv").write_text(policy(rng), encoding="ascii")
    (folder / "counties.csv").write_text(counties(rng), encoding="ascii")


if __name__ == "__main__":
    main()
