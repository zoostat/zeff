"""The fixed datasets of the test suite and the reference values of the descriptive functions.

Two things come out of this script, both committed: the datasets in `tests/datasets/` (the ordinary
ones and the edge cases of the testing handbook, section 1: all `NA`, one row, zero variance,
Unicode labels, awkward column names, infinities, extreme magnitudes, the guessing boundary), and
`tests/reference/functions.json`, which holds for every numeric column of every dataset what the
descriptive functions of release 0.1.0 must give: the count of values, the count missing, the mean,
the sample standard deviation, the minimum and the maximum. Python's `statistics` module computes
in exact rational arithmetic (testing handbook, section 1.1), so these are the correctly rounded
true values. Run with `python3 -I tests/reference/functions.py`; continuous integration re-runs it
and fails if anything committed differs.

A value is missing when its trimmed text is empty, `NA` or `.`, and `NaN` counts as missing too,
as the reader turns it into `NA`. A column is numeric when every other value is `true`/`false`
(counted as 1 and 0) or reads as a number; `Inf` and `-Inf` are numbers.
"""

import csv
import io
import json
import math
import os
import platform
import random
import statistics

here = os.path.dirname(os.path.abspath(__file__))
root = os.path.dirname(os.path.dirname(here))
folder = os.path.join(root, "tests", "datasets")
os.makedirs(folder, exist_ok=True)

rng = random.Random(20261018)


def rows_to_csv(header, rows):
    out = io.StringIO()
    writer = csv.writer(out, lineterminator="\n")
    writer.writerow(header)
    writer.writerows(rows)
    return out.getvalue()


def fmt(x):
    """The shortest text that reads back to the same double."""
    return repr(float(x))


# ----- the datasets -----

datasets = {}

datasets["wages"] = rows_to_csv(
    ["id", "wage", "hours", "name"],
    [[1, 23.4, 40, "Ann"], [2, 19.0, 35, "Bob"], [3, 31.2, "", "Cy"]],
)

regions = ["São Paulo", "Zürich", "東京", "Łódź", "Québec"]
rows = []
for i in range(1, 1201):
    wage = round(rng.lognormvariate(3.0, 0.5), 2)
    hours = "" if rng.random() < 0.05 else rng.randint(20, 60)
    female = "true" if rng.random() < 0.48 else "false"
    region = rng.choice(regions)
    educ = rng.randint(8, 20)
    rows.append([i, wage, hours, female, region, educ])
datasets["wages_large"] = rows_to_csv(["id", "wage", "hours", "female", "region", "educ"], rows)

datasets["awkward_names"] = rows_to_csv(
    ["Gross Sales (total)", "x y", "`case`", "日本語", "1st", "Wage", "wage_2"],
    [[1200.5, 1, 2, 3, 4, 10, 11], [980.25, 2, 3, 4, 5, 20, 21], [1500.0, 3, 4, 5, 6, 30, 31]],
)

datasets["all_na"] = rows_to_csv(["x", "y", "z"], [["", "NA", "."], ["NA", "", "."], [".", "NA", ""]])

datasets["one_row"] = rows_to_csv(["x", "y", "flag"], [[42, 3.5, "true"]])

datasets["zero_variance"] = rows_to_csv(
    ["constant", "constant_with_na", "big_constant"],
    [[3.25, 7, 1e9 + 0.5], [3.25, "", 1e9 + 0.5], [3.25, 7, 1e9 + 0.5], [3.25, 7, 1e9 + 0.5]],
)

datasets["empty"] = rows_to_csv(["x", "y"], [])

datasets["infinities"] = rows_to_csv(
    ["plus", "minus", "both", "nan_mixed", "finite"],
    [[1, 1, 1, 1.5, 1], ["Inf", "-Inf", "Inf", "NaN", 2], [2, 2, "-Inf", "nan", 3]],
)

rows = []
for i in range(500):
    rows.append([1e9 + round(rng.uniform(0, 1), 6), rng.uniform(-1, 1) * 10.0 ** rng.randint(-8, 8), 5e-324 if i == 0 else rng.uniform(0, 1e-300)])
datasets["magnitudes"] = rows_to_csv(["offset", "wide", "tiny"], rows)

datasets["bools"] = rows_to_csv(
    ["treated", "mixed_case", "with_na"],
    [["true", "TRUE", "true"], ["false", "False", ""], ["true", "false", "NA"], ["true", "TRUE", "false"]],
)

rows = [[i, 30 + i % 7] for i in range(1, 1001)]
rows.append([1001, "varies"])
datasets["guess_boundary"] = rows_to_csv(["id", "hours"], rows)

datasets["unicode_labels"] = rows_to_csv(
    ["city", "note", "n"],
    [["café", "naïve", 1], ["café", "straße", 2], ["東京", "😀", 3], ["Zürich", "", 4]],
)

# ----- the reference values -----


def is_missing(text):
    t = text.strip()
    return t == "" or t == "NA" or t == "." or t.lower() == "nan"


def as_number(text):
    t = text.strip()
    if t.lower() == "true":
        return 1.0
    if t.lower() == "false":
        return 0.0
    try:
        return float(t)
    except ValueError:
        return None


def column_stats(values):
    present = [v for v in values if not is_missing(v)]
    missing = len(values) - len(present)
    numbers = [as_number(v) for v in present]
    if any(n is None for n in numbers):
        return None
    stats = {"n": len(numbers), "missing": missing, "mean": None, "sd": None, "min": None, "max": None}
    if numbers:
        has_inf = any(math.isinf(n) for n in numbers)
        if has_inf:
            plus = any(n == math.inf for n in numbers)
            minus = any(n == -math.inf for n in numbers)
            stats["mean"] = "NA" if (plus and minus) else ("Inf" if plus else "-Inf")
            stats["sd"] = "NA"
        else:
            stats["mean"] = fmt(statistics.mean(numbers))
            if len(numbers) >= 2:
                stats["sd"] = fmt(statistics.stdev(numbers))
        stats["min"] = fmt(min(numbers)) if not has_inf else ("-Inf" if minus else fmt(min(numbers)))
        stats["max"] = fmt(max(numbers)) if not has_inf else ("Inf" if plus else fmt(max(numbers)))
    return stats


# The import options a dataset needs before it can be read at all, as a script writes them.
import_options = {"guess_boundary": ', types: {hours: Str}'}

out = {
    "reference": {
        "software": "Python statistics module (standard library), exact rational arithmetic",
        "python": platform.python_version(),
    },
    "datasets": {},
}
for name, text in datasets.items():
    with open(os.path.join(folder, name + ".csv"), "w", encoding="utf-8", newline="") as f:
        f.write(text)
    reader = csv.reader(io.StringIO(text))
    header = next(reader)
    rows = list(reader)
    columns = {}
    for i, column in enumerate(header):
        stats = column_stats([r[i] for r in rows])
        if stats is not None:
            columns[column] = stats
    out["datasets"][name] = {
        "file": f"tests/datasets/{name}.csv",
        "rows": len(rows),
        "import_options": import_options.get(name, ""),
        "columns": columns,
    }

with open(os.path.join(here, "functions.json"), "w", encoding="utf-8") as f:
    json.dump(out, f, indent=1, ensure_ascii=False)
    f.write("\n")
print(f"Python {platform.python_version()}: wrote {len(datasets)} datasets and functions.json")
