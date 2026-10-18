"""Compare two versions of r/maths.json within tolerance (release 0.2.0, step 10).

R computes the elementary functions with the platform's C maths library, the very thing the Zeff
kernel avoids, so the last bits of R's values differ between Linux and macOS and no fixed number
of digits makes the file identical everywhere. Continuous integration therefore regenerates
r/maths.json and runs this script against the committed copy: every value must agree to a relative
1e-13 (the comparison tests use 1e-14 against the committed file), and the inputs, the roundings
and the divisions must agree exactly. The other R files (descriptive.json, frames.json) are
compared byte for byte by the workflow itself.

Usage: python3 -I tests/reference/r/check_maths.py <committed.json> <regenerated.json>
"""

import json
import math
import sys

TOLERANCE = 1e-13


def number(text):
    if text == "NA":
        return math.nan
    if text == "Inf":
        return math.inf
    if text == "-Inf":
        return -math.inf
    return float(text)


def close(a, b):
    a, b = number(a), number(b)
    if math.isnan(a) and math.isnan(b):
        return True
    if a == b:
        return True
    if math.isinf(a) or math.isinf(b):
        return False
    return abs(a - b) <= TOLERANCE * max(abs(a), abs(b))


def main(committed_path, regenerated_path):
    with open(committed_path, encoding="utf-8") as f:
        committed = json.load(f)
    with open(regenerated_path, encoding="utf-8") as f:
        regenerated = json.load(f)
    problems = []
    if committed["grid"] != regenerated["grid"]:
        problems.append("the grid differs")
    for name, values in committed["functions"].items():
        other = regenerated["functions"].get(name)
        if other is None or len(other) != len(values):
            problems.append(f"{name}: a different number of values")
            continue
        for x, a, b in zip(committed["grid"], values, other):
            if not close(a, b):
                problems.append(f"{name}({x}): committed {a}, regenerated {b}")
    for key in ("atan2", "pow"):
        for case_a, case_b in zip(committed[key], regenerated[key]):
            if case_a[:2] != case_b[:2] or not close(case_a[2], case_b[2]):
                problems.append(f"{key}{tuple(case_a[:2])}: committed {case_a[2]}, regenerated {case_b[2]}")
    for key in ("round", "round_significant", "div_mod_int", "div_mod_float"):
        if committed[key] != regenerated[key]:
            problems.append(f"{key} differs exactly where it must not")
    if problems:
        for problem in problems:
            print(problem)
        print(f"{len(problems)} value(s) of r/maths.json differ beyond a relative {TOLERANCE}")
        return 1
    print(f"r/maths.json agrees with the committed copy within a relative {TOLERANCE}")
    return 0


if __name__ == "__main__":
    if len(sys.argv) != 3:
        print(__doc__)
        sys.exit(2)
    sys.exit(main(sys.argv[1], sys.argv[2]))
