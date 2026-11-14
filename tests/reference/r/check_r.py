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

Usage: python3 -I tests/reference/r/check_r.py maths|descriptive|special|distributions|association|kernels|types|design|regression|fixed_effects|event|dates|time|format|text|text_regex|panel|running|quantile_group|distinct|aggregate|merge|reshape|tabulate|t_test|counts|wilcoxon|kruskal|fisher|shapiro|ks|anova|tukey|posthoc|posthoc_level <committed.json> <regenerated.json>

`dates.json`, `time.json`, `format.json`, `text.json` and `text_regex.json` hold day counts, texts, seconds, date parts and ISO texts, which are exact on every platform, and are compared exactly apart from the software line.

`aggregate.json` holds R's grouped statistics written with 17 significant digits: the keys, the
counts and the exact reducers (`min`, `max`, `median`, `mad`) must agree exactly and the
closed-form ones (`mean`, `sd`, `variance`, `sum`, `q90`, `iqr`, `skewness`, `kurtosis`) to a
relative 1e-12, since R's means accumulate in extended precision on x86.
"""

import json
import math
import sys

TOLERANCE = {"maths": 1e-13, "descriptive": 1e-12, "special": 1e-12, "distributions": 1e-12, "association": 1e-12, "kernels": 1e-12, "types": 1e-12, "design": 1e-12, "regression": 1e-10, "fixed_effects": 1e-9, "event": 1e-9, "dates": 0.0, "time": 0.0, "format": 0.0, "text": 0.0, "text_regex": 0.0, "panel": 0.0, "running": 0.0, "quantile_group": 0.0, "distinct": 0.0, "aggregate": 1e-12, "merge": 0.0, "reshape": 0.0, "tabulate": 1e-12, "t_test": 1e-6, "counts": 1e-6, "wilcoxon": 1e-12, "kruskal": 1e-9, "fisher": 1e-3, "shapiro": 1e-9, "ks": 1e-9, "anova": 1e-9, "tukey": 1e-9, "posthoc": 1e-9, "posthoc_level": 1e-9}


def number(text):
    if text == "NA":
        return math.nan
    if text == "Inf":
        return math.inf
    if text == "-Inf":
        return -math.inf
    return float(text)


def close(a, b, tolerance, absolute=0.0):
    a, b = number(a), number(b)
    if math.isnan(a) and math.isnan(b):
        return True
    if a == b:
        return True
    if math.isinf(a) or math.isinf(b):
        return False
    return abs(a - b) <= tolerance * max(abs(a), abs(b)) or abs(a - b) <= absolute


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


# An absolute floor beside the relative tolerance, for files whose values are bounded: a
# correlation in [-1, 1] that is exactly zero comes out as 0 on one platform and 6.5e-18 on another,
# and a far tail of R's `ptukey` is one minus a sum, of which only the rounding is left. The
# p-values of `posthoc` have the same far tails, under Tukey's method and under Šidák's, which
# emmeans computes as 1 − (1 − p)ⁿ.
ABSOLUTE = {"association": 1e-15, "tukey": 1e-12, "posthoc": 1e-12, "posthoc_level": 1e-12}


def check_special(committed, regenerated, problems, which="special"):
    """Every list of the file (special.json, distributions.json and association.json alike):
    scalars compared within tolerance, inputs exactly."""
    tolerance = TOLERANCE[which]
    absolute = ABSOLUTE.get(which, 0.0)

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
                same = close(a, b, tolerance, absolute)
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


def check_kernels(committed, regenerated, problems):
    """kernels.json: each quantity within what its conditioning allows. A least-squares case's
    coefficients, fitted values and leverages carry the condition number of its equilibrated
    design (R's own values move by that much between platforms); the raw condition number and
    the small singular values are computed by LAPACK to an absolute accuracy of about epsilon
    times the largest singular value; the entries of the R factor in a column left out are
    residual-sized; and the residual sum of squares of an exact fit is rounding noise."""
    base = TOLERANCE["kernels"]
    for name, case in committed["least_squares"].items():
        other = regenerated["least_squares"].get(name)
        if other is None:
            problems.append(f"least_squares.{name}: missing")
            continue
        kappa = number(case["condition_number_scaled"])
        raw = number(case["condition_number"])
        tolerance = max(base, 1e-14 * kappa)
        if case["rank"] != other["rank"] or case["pivot"] != other["pivot"]:
            problems.append(f"least_squares.{name}: rank or pivot order differs")
            continue
        for key in ("coefficients", "fitted", "leverage"):
            values = [number(v) for v in case[key]]
            scale = max((abs(v) for v in values if not math.isnan(v)), default=1.0)
            for i, (a, b) in enumerate(zip(case[key], other[key])):
                if not close(a, b, tolerance, tolerance * max(scale, 1.0)):
                    problems.append(f"least_squares.{name}.{key}[{i}]: committed {a}, regenerated {b}")
        fitted_scale = max((abs(number(v)) for v in case["fitted"]), default=1.0)
        if not close(case["residual_sum_of_squares"], other["residual_sum_of_squares"], tolerance, 1e-20 * max(fitted_scale, 1.0) ** 2):
            problems.append(f"least_squares.{name}.residual_sum_of_squares: committed {case['residual_sum_of_squares']}, regenerated {other['residual_sum_of_squares']}")
        rows = case["r"]
        for j in range(len(rows[0])):
            column_scale = max(abs(number(row[j])) for row in rows)
            for i, (row_a, row_b) in enumerate(zip(rows, other["r"])):
                if not close(row_a[j], row_b[j], base, 1e-11 * column_scale):
                    problems.append(f"least_squares.{name}.r[{i}][{j}]: committed {row_a[j]}, regenerated {row_b[j]}")
        largest = max(number(v) for v in case["singular_values"])
        for i, (a, b) in enumerate(zip(case["singular_values"], other["singular_values"])):
            if not close(a, b, base, 1e-12 * largest):
                problems.append(f"least_squares.{name}.singular_values[{i}]: committed {a}, regenerated {b}")
        if not close(case["condition_number"], other["condition_number"], 1e-13 * max(raw, 1.0)):
            problems.append(f"least_squares.{name}.condition_number: committed {case['condition_number']}, regenerated {other['condition_number']}")
        if not close(case["condition_number_scaled"], other["condition_number_scaled"], tolerance):
            problems.append(f"least_squares.{name}.condition_number_scaled: committed {case['condition_number_scaled']}, regenerated {other['condition_number_scaled']}")
    rest_committed = {k: v for k, v in committed.items() if k not in ("reference", "least_squares")}
    rest_regenerated = {k: v for k, v in regenerated.items() if k not in ("reference", "least_squares")}
    check_special(rest_committed, rest_regenerated, problems, "kernels")


def check_design(committed, regenerated, problems):
    """design.json: every column of every case, its first rows, sum and sum of squares, within
    the tolerance (the polynomial contrasts come from R's QR, which differs between platforms in
    its last bits)."""
    tolerance = TOLERANCE["design"]
    for name, case in committed["cases"].items():
        other = regenerated["cases"].get(name)
        if other is None:
            problems.append(f"cases.{name}: missing")
            continue
        if case["rows_used"] != other["rows_used"] or sorted(case["columns"]) != sorted(other["columns"]):
            problems.append(f"cases.{name}: the rows or the columns differ")
            continue
        for column, values in case["columns"].items():
            theirs = other["columns"][column]
            for i, (a, b) in enumerate(zip(values["first_20"], theirs["first_20"])):
                if not close(a, b, tolerance, tolerance):
                    problems.append(f"cases.{name}.{column}[{i}]: committed {a}, regenerated {b}")
            for key in ("sum", "sum_squares"):
                if not close(values[key], theirs[key], tolerance, tolerance):
                    problems.append(f"cases.{name}.{column}.{key}: committed {values[key]}, regenerated {theirs[key]}")


def check_regression(committed, regenerated, problems):
    """regression.json: every term's statistics and every fit measure of every case within the
    tolerance (R's lm on the platform's LAPACK), the terms, the omitted ones and the counts exactly."""
    tolerance = TOLERANCE["regression"]
    for name, case in committed["cases"].items():
        other = regenerated["cases"].get(name)
        if other is None:
            problems.append(f"cases.{name}: missing")
            continue
        for key in ("terms", "omitted", "nobs", "df_residual"):
            if case[key] != other[key]:
                problems.append(f"cases.{name}.{key}: committed {case[key]}, regenerated {other[key]}")
        for term, values in case["columns"].items():
            theirs = other["columns"].get(term, {})
            for key, a in values.items():
                b = theirs.get(key)
                if b is None or not close(a, b, tolerance, 1e-14):
                    problems.append(f"cases.{name}.{term}.{key}: committed {a}, regenerated {b}")
        for key in ("r_squared", "adj_r_squared", "sigma", "rss", "log_likelihood"):
            if not close(case[key], other[key], tolerance, 1e-14):
                problems.append(f"cases.{name}.{key}: committed {case[key]}, regenerated {other[key]}")


def check_fixed_effects(committed, regenerated, problems):
    """fixed_effects.json: every term's statistics, the fit measures, the fitted values and the
    predictions of every case within the tolerance (fixest's projections stop at 1e-11), the terms,
    the omitted ones, the counts and the degrees of freedom exactly."""
    tolerance = TOLERANCE["fixed_effects"]
    for name, case in committed["cases"].items():
        other = regenerated["cases"].get(name)
        if other is None:
            problems.append(f"cases.{name}: missing")
            continue
        for key in ("terms", "omitted", "nobs", "singletons", "fixef_sizes", "df_residual", "df_t", "rows_20"):
            if case[key] != other[key]:
                problems.append(f"cases.{name}.{key}: committed {case[key]}, regenerated {other[key]}")
        for term, values in case["columns"].items():
            theirs = other["columns"].get(term, {})
            for key, a in values.items():
                b = theirs.get(key)
                if b is None or not close(a, b, tolerance, 1e-14):
                    problems.append(f"cases.{name}.{term}.{key}: committed {a}, regenerated {b}")
        for key in ("r_squared", "adj_r_squared", "within_r_squared", "adj_within_r_squared", "rss"):
            if not close(case[key], other[key], tolerance, 1e-14):
                problems.append(f"cases.{name}.{key}: committed {case[key]}, regenerated {other[key]}")
        for key in ("fitted_20", "residuals_20", "predict_10"):
            for i, (a, b) in enumerate(zip(case[key], other[key])):
                if not close(a, b, tolerance, 1e-12):
                    problems.append(f"cases.{name}.{key}[{i}]: committed {a}, regenerated {b}")


def check_event(committed, regenerated, problems):
    """event.json: every term's statistics and both R² forms of every case within the tolerance,
    the terms, the omitted ones, the counts and the degrees of freedom exactly."""
    tolerance = TOLERANCE["event"]
    for name, case in committed["cases"].items():
        other = regenerated["cases"].get(name)
        if other is None:
            problems.append(f"cases.{name}: missing")
            continue
        for key in ("terms", "omitted", "nobs", "df_t"):
            if case[key] != other[key]:
                problems.append(f"cases.{name}.{key}: committed {case[key]}, regenerated {other[key]}")
        for term, values in case["columns"].items():
            theirs = other["columns"].get(term, {})
            for key, a in values.items():
                b = theirs.get(key)
                if b is None or not close(a, b, tolerance, 1e-14):
                    problems.append(f"cases.{name}.{term}.{key}: committed {a}, regenerated {b}")
        for key in ("r_squared", "within_r_squared"):
            if not close(case[key], other[key], tolerance, 1e-14):
                problems.append(f"cases.{name}.{key}: committed {case[key]}, regenerated {other[key]}")


def check_aggregate(committed, regenerated, problems):
    """Keys, counts and the exact reducers exactly; the closed-form statistics within tolerance."""
    tolerance = TOLERANCE["aggregate"]
    exact = ("key", "n", "n_distinct", "min", "max", "median", "mad")
    if len(committed["cases"]) != len(regenerated["cases"]):
        problems.append("a different number of cases")
        return
    for i, (case, other) in enumerate(zip(committed["cases"], regenerated["cases"])):
        for field in ("keys", "value", "distinct_of"):
            if case[field] != other[field]:
                problems.append(f"case {i} {field}: committed {case[field]}, regenerated {other[field]}")
        if len(case["groups"]) != len(other["groups"]):
            problems.append(f"case {i}: a different number of groups")
            continue
        for group, other_group in zip(case["groups"], other["groups"]):
            key = "/".join(group["key"])
            for stat, value in group.items():
                if stat in exact:
                    if value != other_group[stat]:
                        problems.append(f"case {i} {key} {stat}: committed {value}, regenerated {other_group[stat]}")
                elif not close(value, other_group[stat], tolerance):
                    problems.append(f"case {i} {key} {stat}: committed {value}, regenerated {other_group[stat]}")


def check_dates(committed, regenerated, problems):
    """Every value exact: day counts, seconds and ISO texts carry no rounding."""
    for key in committed:
        if key == "reference":
            continue
        if committed[key] != regenerated.get(key):
            problems.append(f"{key} differs")


def main(which, committed_path, regenerated_path):
    with open(committed_path, encoding="utf-8") as f:
        committed = json.load(f)
    with open(regenerated_path, encoding="utf-8") as f:
        regenerated = json.load(f)
    problems = []
    checks = {"maths": check_maths, "descriptive": check_descriptive, "kernels": check_kernels, "design": check_design, "regression": check_regression, "fixed_effects": check_fixed_effects, "event": check_event, "dates": check_dates, "time": check_dates, "format": check_dates, "panel": check_dates, "running": check_dates, "quantile_group": check_dates, "distinct": check_dates, "aggregate": check_aggregate, "merge": check_dates, "reshape": check_dates}
    if which in checks:
        checks[which](committed, regenerated, problems)
    else:
        check_special(committed, regenerated, problems, which)
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
