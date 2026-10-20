"""Compare a regenerated R reference file with its committed copy within tolerance (release 0.2.0,
step 10).

R computes the elementary functions with the platform's C maths library and accumulates means in
extended precision where the processor has it (x86), the very things the Zeff kernel avoids, so
the last bits of R's values differ between Linux and macOS and no fixed number of digits makes
these files identical everywhere. Continuous integration therefore regenerates them and runs this
script against the committed copies: in `maths.json` every function value must agree to a
relative 1e-13 and the inputs, roundings and whole-number divisions exactly; in
`descriptive.json` every count must agree exactly and every statistic to a relative 1e-12 (R's
variance on the offset dataset carries that much error of its own); in `special.json` every value to a
relative 1e-12, the class tolerance of the distribution functions, since R's `pgamma` and `pbeta`
carry errors of that order in their far tails on some platforms. `frames.json` holds only row
numbers and is compared byte for byte by the workflow itself.

Usage: python3 -I tests/reference/r/check_r.py maths|descriptive|special|distributions|association <committed.json> <regenerated.json>
"""

import json
import math
import sys

TOLERANCE = {"maths": 1e-13, "descriptive": 1e-12, "special": 1e-12, "distributions": 1e-12, "association": 1e-12}


def number(text):
    if text == "NA":
        return math.nan
    if text == "Inf":
        return math.inf
    if text == "-Inf":
        return -math.inf
    return float(text)


def close(a, b, tolerance):
    a, b = number(a), number(b)
    if math.isnan(a) and math.isnan(b):
        return True
    if a == b:
        return True
    if math.isinf(a) or math.isinf(b):
        return False
    return abs(a - b) <= tolerance * max(abs(a), abs(b))


def check_maths(committed, regenerated, problems):
    tolerance = TOLERANCE["maths"]
    if committed["grid"] != regenerated["grid"]:
        problems.append("the grid differs")
    for name, values in committed["functions"].items():
        other = regenerated["functions"].get(name)
        if other is None or len(other) != len(values):
            problems.append(f"{name}: a different number of values")
            continue
        for x, a, b in zip(committed["grid"], values, other):
            if not close(a, b, tolerance):
                problems.append(f"{name}({x}): committed {a}, regenerated {b}")
    for key in ("atan2", "pow"):
        for case_a, case_b in zip(committed[key], regenerated[key]):
            if case_a[:2] != case_b[:2] or not close(case_a[2], case_b[2], tolerance):
                problems.append(f"{key}{tuple(case_a[:2])}: committed {case_a[2]}, regenerated {case_b[2]}")
    for key in ("round", "round_significant", "div_mod_int"):
        if committed[key] != regenerated[key]:
            problems.append(f"{key} differs exactly where it must not")


def check_descriptive(committed, regenerated, problems):
    tolerance = TOLERANCE["descriptive"]
    exact = ("n", "missing", "n_distinct")
    for name, dataset in committed["datasets"].items():
        other = regenerated["datasets"].get(name)
        if other is None or dataset["file"] != other["file"]:
            problems.append(f"{name}: missing or another file")
            continue
        for column, stats in dataset["columns"].items():
            other_stats = other["columns"].get(column)
            if other_stats is None:
                problems.append(f"{name}.{column}: missing")
                continue
            for key, value in stats.items():
                if key == "quantiles":
                    for kind, values in value.items():
                        for p, a, b in zip(value["probs"], values, other_stats["quantiles"][kind]):
                            if not close(a, b, tolerance):
                                problems.append(f"{name}.{column} {kind} at {p}: committed {a}, regenerated {b}")
                elif key in exact:
                    if value != other_stats[key]:
                        problems.append(f"{name}.{column} {key}: committed {value}, regenerated {other_stats[key]}")
                elif not close(value, other_stats[key], tolerance):
                    problems.append(f"{name}.{column} {key}: committed {value}, regenerated {other_stats[key]}")


def check_special(committed, regenerated, problems):
    """Every list of the file (special.json and distributions.json alike): scalars compared within
    tolerance, inputs exactly."""
    tolerance = TOLERANCE["special"]

    def walk(path, a, b):
        if isinstance(a, dict) and isinstance(b, dict):
            for key in a:
                if key not in b:
                    problems.append(f"{path}.{key}: missing")
                else:
                    walk(f"{path}.{key}", a[key], b[key])
            return
        if isinstance(a, list) and isinstance(b, list):
            if len(a) != len(b):
                problems.append(f"{path}: {len(a)} against {len(b)} entries")
                return
            for i, (x, y) in enumerate(zip(a, b)):
                walk(f"{path}[{i}]", x, y)
        elif isinstance(a, str) and isinstance(b, str):
            try:
                same = close(a, b, tolerance)
            except ValueError:
                same = a == b  # a name, not a number
            if not same:
                problems.append(f"{path}: committed {a}, regenerated {b}")
        elif a != b:
            problems.append(f"{path}: committed {a!r}, regenerated {b!r}")

    for key, value in committed.items():
        if key == "reference":
            continue
        if key not in regenerated:
            problems.append(f"{key}: missing")
            continue
        walk(key, value, regenerated[key])


def main(which, committed_path, regenerated_path):
    with open(committed_path, encoding="utf-8") as f:
        committed = json.load(f)
    with open(regenerated_path, encoding="utf-8") as f:
        regenerated = json.load(f)
    problems = []
    {"maths": check_maths, "descriptive": check_descriptive, "special": check_special, "distributions": check_special, "association": check_special}[which](committed, regenerated, problems)
    if problems:
        for problem in problems:
            print(problem)
        print(f"{len(problems)} value(s) of r/{which}.json differ beyond a relative {TOLERANCE[which]}")
        return 1
    print(f"r/{which}.json agrees with the committed copy within a relative {TOLERANCE[which]}")
    return 0


if __name__ == "__main__":
    if len(sys.argv) != 4 or sys.argv[1] not in TOLERANCE:
        print(__doc__)
        sys.exit(2)
    sys.exit(main(sys.argv[1], sys.argv[2], sys.argv[3]))
