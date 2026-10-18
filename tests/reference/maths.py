"""Reference values of the maths kernel on a fixed grid (release 0.2.0, step 5).

Writes `tests/reference/maths.json`: for every elementary function its correctly rounded value at
each point of a fixed grid, computed with Python's `decimal` module at 50 significant digits and
then rounded once to the nearest double, so that the file is the same on every machine (Python's
`math` module is the platform's C library and differs between systems, which is the very thing the
kernel avoids); the roundings of a fixed set of numbers half to even (Python's `round`, decided on
the exact binary value as R decides it) and half away from zero (ROUND_HALF_UP on the exact value,
as Stata and Excel round); and floored division and remainder as Python computes them. Run with
`python3 -I tests/reference/maths.py`; continuous integration re-runs it and fails if anything
committed differs.
"""

import decimal
import json
import math
import os
import platform
from decimal import Decimal

here = os.path.dirname(os.path.abspath(__file__))

decimal.getcontext().prec = 50
decimal.getcontext().Emax = 999_999
decimal.getcontext().Emin = -999_999

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


# ----- the elementary functions in exact decimal arithmetic -----

def pi():
    """Pi to 420 digits (the recipe of the decimal module's documentation): enough to reduce an
    argument as large as 1e300 to its angle with 50 digits to spare."""
    decimal.getcontext().prec = 422
    three = Decimal(3)
    lasts, t, s, n, na, d, da = 0, three, 3, 1, 0, 0, 24
    while s != lasts:
        lasts = s
        n, na = n + na, na + 8
        d, da = d + da, da + 32
        t = (t * n) / d
        s += t
    decimal.getcontext().prec = 420
    s = +s
    decimal.getcontext().prec = 50
    return s


PI = pi()


def reduce_angle(x):
    """x reduced to [-pi, pi], at a precision that holds every digit of x before the point."""
    with decimal.localcontext() as ctx:
        ctx.prec = 60 + max(0, x.adjusted())
        two_pi = 2 * PI
        k = (x / two_pi).to_integral_value(rounding=decimal.ROUND_HALF_EVEN)
        reduced = x - k * two_pi
    return +reduced


def dsin(x):
    extra = 4 + max(0, x.adjusted())
    decimal.getcontext().prec += extra
    x = reduce_angle(x)
    i, lasts, s, fact, num, sign = 1, 0, x, 1, x, 1
    while s != lasts:
        lasts = s
        i += 2
        fact *= i * (i - 1)
        num *= x * x
        sign *= -1
        s += num / fact * sign
    decimal.getcontext().prec -= extra
    return +s


def dcos(x):
    extra = 4 + max(0, x.adjusted())
    decimal.getcontext().prec += extra
    x = reduce_angle(x)
    i, lasts, s, fact, num, sign = 0, 0, 1, 1, 1, 1
    while s != lasts:
        lasts = s
        i += 2
        fact *= i * (i - 1)
        num *= x * x
        sign *= -1
        s += num / fact * sign
    decimal.getcontext().prec -= extra
    return +s


def datan(x):
    """arctan by its series after halving the argument until it is small."""
    decimal.getcontext().prec += 4
    halvings = 0
    while abs(x) > Decimal("0.2"):
        x = x / (1 + (1 + x * x).sqrt())
        halvings += 1
    i, lasts, s, num, sign = 1, 0, x, x, 1
    while s != lasts:
        lasts = s
        i += 2
        num *= x * x
        sign *= -1
        s += sign * num / i
    s *= 2 ** halvings
    decimal.getcontext().prec -= 4
    return +s


def to_double(r):
    """A decimal rounded once to the nearest double, as text."""
    y = float(r)
    if math.isinf(y):
        return "Inf" if y > 0 else "-Inf"
    if y == 0:
        return "-0.0" if r.is_signed() else "0.0"
    return repr(y)


def value_of(name, x):
    """The function at the double x, correctly rounded to a double, as text: a shortest form,
    `Inf`/`-Inf`, or `NA` where the function has no value. Zeff's answers at the edges (section
    4.1): a logarithm of zero is `-Inf`."""
    d = Decimal(x)
    # At zero and for tiny arguments the odd functions return x itself, sign included: no decimal
    # arithmetic could keep the sign of -0.0 or the last digits of 1e-300.
    if name in ("sin", "tan", "asin", "atan", "log1p", "expm1") and abs(x) < 1e-17:
        return text(x)
    try:
        if name == "exp":
            if x > 709.8:
                return "Inf"
            if x < -746:
                return "0.0"
            r = d.exp()
        elif name == "log":
            if x == 0:
                return "-Inf"
            if x < 0:
                return "NA"
            r = d.ln()
        elif name == "log2":
            if x == 0:
                return "-Inf"
            if x < 0:
                return "NA"
            r = d.ln() / Decimal(2).ln()
        elif name == "log10":
            if x == 0:
                return "-Inf"
            if x < 0:
                return "NA"
            r = d.log10()
        elif name == "log1p":
            if x == -1:
                return "-Inf"
            if x < -1:
                return "NA"
            r = (1 + d).ln()
        elif name == "expm1":
            if x > 709.8:
                return "Inf"
            if x < -40:
                return "-1.0"
            r = d.exp() - 1
        elif name == "sqrt":
            if x < 0:
                return "NA"
            r = d.sqrt()
        elif name == "sin":
            r = dsin(d)
        elif name == "cos":
            r = dcos(d)
        elif name == "tan":
            r = dsin(d) / dcos(d)
        elif name == "atan":
            r = datan(d)
        elif name == "asin":
            if abs(x) > 1:
                return "NA"
            if abs(x) == 1:
                r = PI / 2 * (1 if x > 0 else -1)
            else:
                r = datan(d / (1 - d * d).sqrt())
        elif name == "acos":
            if abs(x) > 1:
                return "NA"
            if x == 1:
                return "0.0"
            if x == -1:
                r = PI
            else:
                r = PI / 2 - datan(d / (1 - d * d).sqrt())
        else:
            raise ValueError(name)
    except (decimal.InvalidOperation, decimal.Overflow):
        return "NA"
    return to_double(r)


def atan2_value(y, x):
    """The angle of the point (x, y), as IEEE atan2 defines it at the axes and the origin."""
    if x == 0 and y == 0:
        if math.copysign(1, x) > 0:
            return "0.0" if math.copysign(1, y) > 0 else "-0.0"
        return to_double(PI) if math.copysign(1, y) > 0 else to_double(-PI)
    if x == 0:
        return to_double(PI / 2 if y > 0 else -PI / 2)
    a = datan(Decimal(y) / Decimal(x))
    if x < 0:
        a = a + PI if y >= 0 else a - PI
    if y == 0 and x < 0:
        return to_double(PI) if math.copysign(1, y) > 0 else to_double(-PI)
    return to_double(a)


def power_value(x, y):
    """x ** y as IEEE pow gives it, correctly rounded: a negative base with a fractional exponent
    has no real value (`NA`) and 0 ** negative is an infinity, as 1 / 0 (section 4.1)."""
    if x == 0 and y < 0:
        return "Inf"
    if x == 0 and y == 0:
        return "1.0"
    if x == 0:
        return "0.0"
    try:
        if y == int(y):
            r = Decimal(x) ** int(y)
        else:
            if x < 0:
                return "NA"
            r = Decimal(x) ** Decimal(y)
    except (decimal.InvalidOperation, decimal.Overflow):
        return "NA"
    return to_double(r)


def text(x):
    if math.isinf(x):
        return "Inf" if x > 0 else "-Inf"
    return repr(x)


names = ["exp", "log", "log2", "log10", "log1p", "expm1", "sin", "cos", "tan", "asin", "acos", "atan", "sqrt"]

out = {
    "reference": {
        "software": "Python decimal module at 50 significant digits, rounded once to a double",
        "python": platform.python_version(),
    },
    "grid": [text(x) for x in grid],
    "functions": {name: [value_of(name, x) for x in grid] for name in names},
    "atan2": [[text(y), text(x), atan2_value(y, x)] for y in (0.0, 1.0, -1.0, 2.5) for x in (0.0, 1.0, -1.0, -0.5)],
    "pow": [[text(x), text(y), power_value(x, y)] for x in (2.0, 0.5, 10.0, -2.0, 1.5, 0.0) for y in (0.0, 1.0, 2.0, 3.0, 0.5, -1.0, 10.0, -0.5)],
}

# Rounding: numbers chosen for their ties, near-ties and magnitudes.
to_round = [
    0.5, 1.5, 2.5, -2.5, 2.675, 1.005, 0.125, -0.125, 9.995, 0.995, 123.456, 1250.0, 1350.0,
    -0.4, 1e300, 123456.789, 2.345, 2.355, 7.5, -7.5, 1.0000000000000002, 0.1 + 0.2,
    1234567.891, 0.045, 0.055, 1.45, 1.55, 100.0, 5.000000000000001, 3.14159,
]
rounding = []
for x in to_round:
    for digits in (0, 1, 2, 3, -1, -2):
        even = round(x, digits)
        exact = Decimal(x)
        quantum = Decimal(1).scaleb(-digits)
        with decimal.localcontext() as ctx:
            ctx.prec = 500  # 1e300 at two decimals needs 303 digits
            away = float(exact.quantize(quantum, rounding=decimal.ROUND_HALF_UP))
        rounding.append([repr(x), digits, repr(float(even)), repr(away)])
out["round"] = rounding

significant = []
for x in (123456.0, 0.00123456, 1.25, 9.99, -987.654, 5.0, 0.5, 1500.0, 2.5e-7):
    for sig in (1, 2, 3, 6):
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
        # Python's // and % are exact (the remainder from fmod, the quotient from it), where
        # floor(a / b) would round 7.5 / 0.1 up to 75 and miss the true remainder; R's %/% and %%
        # agree with them.
        float_divisions.append([repr(a), repr(b), repr(a // b), repr(a % b)])
out["div_mod_float"] = float_divisions

with open(os.path.join(here, "maths.json"), "w", encoding="utf-8") as f:
    json.dump(out, f, indent=1, ensure_ascii=False)
    f.write("\n")
print("wrote maths.json:", len(grid), "grid points,", len(rounding), "roundings")

# The inputs for the R script r/maths.R, each with its exact hexadecimal form: R's own reading
# of a decimal such as 1e300 is a few bits off (its parser multiplies by powers of ten), and a
# function as sensitive as sin at 1e100 would then be evaluated at another point. R reads a
# hexadecimal float exactly.
inputs_path = os.path.join(here, "data", "maths_inputs.csv")
os.makedirs(os.path.dirname(inputs_path), exist_ok=True)
with open(inputs_path, "w", encoding="utf-8", newline="\n") as f:
    f.write("set,decimal,hex\n")
    for name, values in (("grid", grid), ("round", to_round), ("significant", [x for x, _, _ in [(123456.0, 0, 0), (0.00123456, 0, 0), (1.25, 0, 0), (9.99, 0, 0), (-987.654, 0, 0), (5.0, 0, 0), (0.5, 0, 0), (1500.0, 0, 0), (2.5e-7, 0, 0)]])):
        for x in values:
            f.write(f"{name},{text(x)},{float.hex(x)}\n")
print("wrote", inputs_path)
