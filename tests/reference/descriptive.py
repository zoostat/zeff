"""Reference values for the descriptive reducers (sum, mean, variance, sd).

Python's `statistics` module computes in exact rational arithmetic and rounds once at the end, so
its answers are the correctly rounded true values: the reference for release 0.1.0 (testing
handbook, section 1.1). Run with `python3 -I tests/reference/descriptive.py`; it rewrites
`tests/reference/descriptive.json`.

Every number is stored as the shortest text that reads back to the same double, and the Rust
tests parse it with `str::parse::<f64>`, which is correctly rounded, so no value changes on the way.
"""

import json
import os
import platform
import random
import statistics
from fractions import Fraction

rng = random.Random(20261018)  # Mersenne Twister: the same stream on every Python 3

datasets = {
    "small": [2.5, 3.1, 4.7, 5.0, 5.9, 6.2, 7.4, 8.8, 9.1, 10.3, 11.6, 12.0],
    # A large offset with a small spread: the case the two-pass algorithm exists for.
    "offset": [1e9 + round(rng.uniform(0, 1), 6) for _ in range(1000)],
    # Values over sixteen orders of magnitude, both signs.
    "wide": [rng.uniform(-1, 1) * 10.0 ** rng.randint(-8, 8) for _ in range(500)],
    "constant": [3.25] * 7,
    "two": [-1.5, 2.5],
}


def text(x):
    return repr(float(x))


out = {
    "reference": {
        "software": "Python statistics module (standard library), exact rational arithmetic",
        "python": platform.python_version(),
    },
    "datasets": {},
}
for name, values in datasets.items():
    exact_sum = sum(Fraction(v) for v in values)
    out["datasets"][name] = {
        "values": [text(v) for v in values],
        "sum": text(exact_sum),
        "mean": text(statistics.mean(values)),
        "variance": text(statistics.variance(values)),
        "sd": text(statistics.stdev(values)),
    }

here = os.path.dirname(os.path.abspath(__file__))
path = os.path.join(here, "descriptive.json")
with open(path, "w", encoding="utf-8") as f:
    json.dump(out, f, indent=1)
    f.write("\n")
print("wrote", path)

# The same datasets as one-column CSV files, for the R scripts in r/ (testing handbook, section
# 1.1): every value is the shortest text that reads back to the same double, so R reads the very
# numbers Python computed with.
data_dir = os.path.join(here, "data")
os.makedirs(data_dir, exist_ok=True)
for name, values in datasets.items():
    with open(os.path.join(data_dir, f"{name}.csv"), "w", encoding="utf-8", newline="\n") as f:
        f.write("x\n")
        for v in values:
            f.write(text(v) + "\n")
print("wrote", len(datasets), "datasets to", data_dir)
