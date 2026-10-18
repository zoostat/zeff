"""Reference values of the maths kernel on a fixed grid (release 0.2.0, step 5).

Writes `tests/reference/maths.json`: for every elementary function the value Python's `math`
module gives at each point of a fixed grid (the platform's C library; the kernel must agree within
two units in the last place, exactly where the answer is exact), the roundings of a fixed set of
numbers half to even (Python's `round`, decided on the exact binary value as R decides it) and half
away from zero (the `decimal` module's ROUND_HALF_UP on the exact value, as Stata and Excel round),
and floored division and remainder as Python computes them. Run with
`python3 -I tests/reference/maths.py`; continuous integration re-runs it and fails if anything
committed differs.
"""

import decimal
import json
import math
import os
import platform

here = os.path.dirname(os.path.abspath(__file__))

# A grid that reaches the special points and the ranges a user meets: around zero, around one,
# large and small magnitudes, negative values, and the boundaries of the inverse trigonometric
# functions. Every value is written in a form that reads back to the same double.
grid = [
    0.0, -0.0, 1e-300, -1e-300, 1e-10, 0.001, 0.1, 0.25, 0.5, 0.75, 0.9, 0.999, 1.0, 1.001, 1.5,
    2.0, math.e, 3.0, math.pi, 4.0, 5.5, 8.0, 10.0, 12.5, 100.0, 1000.0, 123456.789, 1e6, 1e10,
    1e15, 1e100, 1e300, -1e-10, -0.001, -0.25, -0.5, -0.75, -1.0, -1.5, -2.0, -3.0, -math.pi,
    -10.0, -100.0, -1e6, -1e100, 0.7853981633974483, 1.5707963267948966, 3.141592653589793,
    6.283185307179586, 100.5, -99.5, 1e-5, 2.5e-8, 7.0, 0.3, 0.6, 0.123456789,
]

functions = {
    "exp": math.exp,
    "log": math.log,
    "log2": math.log2,
    "log10": math.log10,
    "log1p": math.log1p,
    "expm1": math.expm1,
    "sin": math.sin,
    "cos": math.cos,
    "tan": math.tan,
    "asin": math.asin,
    "acos": math.acos,
    "atan": math.atan,
    "sqrt": math.sqrt,
}


def value(f, x):
    """The function at x as text: a double's shortest form, `NA` where it has no value, or an
    infinity's name. Python raises at a logarithm of zero where Zeff gives `-Inf` (Language
    Reference, section 4.1); the reference writes Zeff's answer there."""
    if f in (math.log, math.log2, math.log10) and x == 0:
        return "-Inf"
    if f is math.log1p and x == -1:
        return "-Inf"
    try:
        y = f(x)
    except ValueError:
        return "NA"
    except OverflowError:
        return "Inf"
    if math.isnan(y):
        return "NA"
    if math.isinf(y):
        return "Inf" if y > 0 else "-Inf"
    return repr(y)


def power_value(x, y):
    """x ** y as IEEE pow gives it: Python raises at 0 ** negative, where the power is an infinity
    (the rule of 1 / 0 in Language Reference 4.1)."""
    if x == 0 and y < 0:
        return "Inf"
    return value(lambda _: math.pow(x, y), 0)


def text(x):
    if math.isinf(x):
        return "Inf" if x > 0 else "-Inf"
    return repr(x)


out = {
    "reference": {
        "software": "Python math module (the platform C library) and decimal module",
        "python": platform.python_version(),
    },
    "grid": [text(x) for x in grid],
    "functions": {name: [value(f, x) for x in grid] for name, f in functions.items()},
    "atan2": [[text(y), text(x), value(lambda _: math.atan2(y, x), 0)] for y in (0.0, 1.0, -1.0, 2.5) for x in (0.0, 1.0, -1.0, -0.5)],
    "pow": [[text(x), text(y), power_value(x, y)] for x in (2.0, 0.5, 10.0, -2.0, 1.5, 0.0) for y in (0.0, 1.0, 2.0, 3.0, 0.5, -1.0, 10.0, -0.5)],
}

# Rounding: numbers chosen for their ties, near-ties and magnitudes.
to_round = [
    0.5, 1.5, 2.5, -2.5, 2.675, 1.005, 0.125, -0.125, 9.995, 0.995, 123.456, 1250.0, 1350.0,
    -0.4, 1e300, 123456.789, 2.345, 2.355, 7.5, -7.5, 1.0000000000000002, 0.1 + 0.2,
    1234567.891, 0.045, 0.055, 1.45, 1.55, 100.0, 5.000000000000001, 3.14159,
]
decimal.getcontext().prec = 500
decimal.getcontext().Emax = 999_999
decimal.getcontext().Emin = -999_999
rounding = []
for x in to_round:
    for digits in (0, 1, 2, 3, -1, -2):
        even = round(x, digits)
        exact = decimal.Decimal(x)
        quantum = decimal.Decimal(1).scaleb(-digits)
        away = float(exact.quantize(quantum, rounding=decimal.ROUND_HALF_UP))
        rounding.append([repr(x), digits, repr(float(even)), repr(away)])
out["round"] = rounding

significant = []
for x in (123456.0, 0.00123456, 1.25, 9.99, -987.654, 5.0, 0.5, 1500.0, 2.5e-7):
    for sig in (1, 2, 3, 6):
        if x == 0:
            continue
        exponent = math.floor(math.log10(abs(x)))
        digits = sig - 1 - exponent
        significant.append([repr(x), sig, repr(float(round(x, digits)))])
out["round_significant"] = significant

divisions = []
for a in (7, -7, 6, -6, 0, 1, -1, 9223372036854775807, -9223372036854775808, 100, -100):
    for b in (2, -2, 3, -3, 7, -7, 1, -1, 10):
        divisions.append([a, b, a // b, a % b])
out["div_mod_int"] = divisions
float_divisions = []
for a in (7.5, -7.5, 6.0, -6.0, 0.0, 1e10, -1e10, 0.3):
    for b in (2.0, -2.0, 3.0, 0.1, -0.1, 7.0):
        float_divisions.append([repr(a), repr(b), repr(math.floor(a / b)), repr(a - b * math.floor(a / b))])
out["div_mod_float"] = float_divisions

with open(os.path.join(here, "maths.json"), "w", encoding="utf-8") as f:
    json.dump(out, f, indent=1, ensure_ascii=False)
    f.write("\n")
print("wrote maths.json:", len(grid), "grid points,", len(rounding), "roundings")
