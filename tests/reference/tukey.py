"""Reference values of the studentized range (release 0.7.0, step 18).

Writes `tests/reference/tukey.json`: the lower tail, the upper tail and the density of the
studentized range of 2 to 200 groups on 1 to 1e12 degrees of freedom, computed with Python's
`decimal` module at 70 significant digits and rounded once to the nearest double, so that the
file is the same on every machine. Also writes `tests/reference/data/tukey_cases.csv`, the same
inputs as exact hexadecimal floats for the R script beside it.

The method is the kernel's in outline and independent of it in every detail. The range W of k
standard normals has three integrals over the line (its lower tail, its upper tail and its
density, below at `range_value`), each summed by the trapezoidal rule, which for a smooth
integrand that dies away at both ends converges faster than any power of the step. The
studentized range integrates the same function of the range at q s against the density of s,
the root of a chi-squared over its degrees of freedom, after two changes of variable that the
kernel does not make (below at `weight`). The normal tail comes from a 95-digit `erfc` and is
carried between the points of a lattice by its Taylor series. Nothing is taken from a float
except places to start looking, and those are rounded to six digits so that the platform's C
library cannot move a node. Before writing, the script checks itself against the two cases with
a closed form (two groups on one and on two degrees of freedom) and checks that the two tails,
each summed on its own, add to one.

Run with `python3 -I tests/reference/tukey.py`; continuous integration re-runs it and fails if
anything committed differs.
"""

import decimal
import json
import math
import multiprocessing
import os
import platform
from decimal import Decimal

here = os.path.dirname(os.path.abspath(__file__))

PREC = 70
GUARD = 15
ctx = decimal.getcontext()
ctx.prec = PREC
# Wide exponents: far from its peak the weight of a million or more degrees of freedom is
# smaller than any power of ten a narrower range could hold, and must not read as zero.
ctx.Emax = 10**17
ctx.Emin = -(10**17)
INF = float("inf")


# ----- the normal distribution -----

def compute_pi():
    lasts, t, s, n, na, d, da = 0, Decimal(3), 3, 1, 0, 0, 24
    while s != lasts:
        lasts = s
        n, na = n + na, na + 8
        d, da = d + da, da + 32
        t = (t * n) / d
        s += t
    return s


ctx.prec = PREC + 2 * GUARD
PI = compute_pi()
SQRT_2 = Decimal(2).sqrt()
SQRT_2PI = (2 * PI).sqrt()
SPACING = Decimal(1) / 5
POLES = [Decimal(n * n) * SPACING * SPACING for n in range(1, 76)]
BELLS = [(-a).exp() for a in POLES]
ctx.prec = PREC


def erfc(x):
    """erfc(x) for x >= 0 to about 95 digits: the Maclaurin series of erf below 1/2; above it
    the trapezoidal rule on exp(-t^2) / (t^2 + x^2) with the correction for its pole."""
    if x < Decimal("0.5"):
        total = term = x
        n = 0
        while True:
            n += 1
            term = -term * x * x / n
            piece = term / (2 * n + 1)
            total += piece
            if abs(piece) <= Decimal("1e-98") * abs(total):
                return 1 - 2 * total / PI.sqrt()
    x2 = x * x
    total = 1 / (2 * x2)
    for a, e in zip(POLES, BELLS):
        total += e / (a + x2)
    value = 2 * SPACING * x * (-x2).exp() / PI * total
    if x * SPACING < PI:
        value -= 2 / ((2 * PI * x / SPACING).exp() - 1)
    return value


LATTICE = 32
NODES = {}


def node(j):
    """The upper tail and the density at j/32, with the series that carry both a little way:
    exp(-x u - u^2/2) = sum b_n u^n, where (n + 1) b_(n+1) = -x b_n - b_(n-1)."""
    ctx.prec = PREC + 2 * GUARD
    x = Decimal(j) / LATTICE
    tail = erfc(x / SQRT_2) / 2
    height = (-x * x / 2).exp() / SQRT_2PI
    b = [Decimal(1), -x]
    reach = Decimal(1) / (2 * LATTICE)
    power = reach
    while abs(b[-1]) * power > Decimal("1e-90") or abs(b[-2]) * power > Decimal("1e-90"):
        n = len(b) - 1
        b.append((-x * b[n] - b[n - 1]) / (n + 1))
        power *= reach
    entry = (
        x,
        tail,
        [-height * c / (n + 1) for n, c in enumerate(b)][::-1],
        [height * c for c in b][::-1],
    )
    ctx.prec = PREC
    NODES[j] = entry
    return entry


def upper(x):
    """The upper tail of the standard normal at x >= 0."""
    j = int(x * LATTICE + Decimal("0.5"))
    x0, tail, series, _ = NODES.get(j) or node(j)
    ctx.prec = PREC + GUARD
    e = x - x0
    total = Decimal(0)
    for c in series:
        total = total * e + c
    value = tail + total * e
    ctx.prec = PREC
    return value


def density(x):
    """The density of the standard normal at x >= 0."""
    j = int(x * LATTICE + Decimal("0.5"))
    x0, _, _, series = NODES.get(j) or node(j)
    ctx.prec = PREC + GUARD
    e = x - x0
    total = Decimal(0)
    for c in series:
        total = total * e + c
    ctx.prec = PREC
    return total


# ----- one integral -----

LN_10 = math.log(10.0)
FIRST = 0.7
SMALL = Decimal("1e-22")
AGREE = Decimal("1e-6")


def six(x):
    """A float rounded to six significant digits: a place to start, the same on every machine."""
    return float(f"{x:.6g}")


def log_of(v):
    if v <= 0:
        return -INF
    e = v.adjusted()
    return math.log(float(v.scaleb(-e))) + e * LN_10


def integrate(f, guess, stride, crude=False, search=None):
    """The integral over the line of a positive function with one peak, by the trapezoidal
    rule. Find the peak and its width, on floats; sum outwards from it until the terms stop
    mattering, on three grids set a third of a step apart. Their mean is the rule at a third
    of the step, and their spread measures the error of each wherever the grid falls (two
    grids half a step apart agree by accident when the peak sits a quarter of a step from a
    node). The step is divided by three until the spread is below 1e-6 of the mean, which for
    a rule whose error is cubed when the step is divided by three puts the mean far below the
    last digit of a double. `search`, when given, is a rougher form of the function that is
    good enough to look for the peak with."""
    look = search or f
    guess, stride = six(guess), six(stride)
    a, b, c = guess - stride, guess, guess + stride
    fa, fb, fc = (log_of(look(Decimal(x))) for x in (a, b, c))
    d = stride
    for _ in range(400):
        if fb >= fa and fb >= fc:
            break
        d *= 2
        if fc > fb:
            a, fa, b, fb = b, fb, c, fc
            c = b + d
            fc = log_of(look(Decimal(c)))
        else:
            c, fc, b, fb = b, fb, a, fa
            a = b - d
            fa = log_of(look(Decimal(a)))
    if fb == -INF:
        return Decimal(0)
    for _ in range(400):
        da, dc = fb - fa, fb - fc
        left, right = b - a, c - b
        if da <= 2.0 and dc <= 2.0 and left <= 2.0 * right and right <= 2.0 * left:
            break
        if left > 2.0 * right or (right <= 2.0 * left and da > dc):
            m = 0.5 * (a + b)
            fm = log_of(look(Decimal(m)))
            if fm > fb:
                c, fc, b, fb = b, fb, m, fm
            else:
                a, fa = m, fm
        else:
            m = 0.5 * (b + c)
            fm = log_of(look(Decimal(m)))
            if fm > fb:
                a, fa, b, fb = b, fb, m, fm
            else:
                c, fc = m, fm
    curvature = -2.0 * ((fc - fb) / (c - b) - (fb - fa) / (b - a)) / (c - a)
    sigma = 1.0 / math.sqrt(curvature) if curvature > 0 else 0.5 * (c - a)
    mode = Decimal(b)
    step = Decimal(six((1.2 if crude else FIRST) * sigma))
    small = Decimal("1e-6") if crude else SMALL

    def sweep(step, offset):
        sums = [Decimal(0), Decimal(0)]
        total = Decimal(0)
        for direction in (1, -1):
            i = 0 if direction == 1 else 1
            prev = None
            quiet = 0
            while True:
                term = f(mode + (direction * i + offset) * step)
                sums[i % 2] += term
                total += term
                if term <= small * total and (prev is None or term <= prev):
                    quiet += 1
                    if quiet == 2:
                        break
                else:
                    quiet = 0
                prev = term
                i += 1
        return sums

    parts = [sum(sweep(step, Decimal(j) / 3)) * step for j in range(1 if crude else 3)]
    if crude:
        return parts[0]
    for _ in range(8):
        mean = sum(parts) / 3
        if max(parts) - min(parts) <= AGREE * mean:
            return mean
        step = step / 3
        parts = [mean] + [sum(sweep(step, Decimal(j) / 3)) * step for j in (1, 2)]
    raise ArithmeticError("the quadrature did not settle")


# ----- the range of k standard normals -----

def one_minus_power(r, n):
    """1 - (1 - r)^n, keeping the digits of a tiny r."""
    if r > Decimal("1e-10"):
        return 1 - (1 - r) ** n
    total = term = n * r
    for j in range(2, n + 1):
        term = -term * (n - j + 1) * r / j
        total += term
        if abs(term) <= Decimal("1e-80") * abs(total):
            break
    return total


def largest(k):
    """Roughly where the largest of k standard normals sits: only a place to start looking."""
    t = math.sqrt(2.0 * math.log(k + 1.0))
    return t - (2.515517 + 0.802853 * t + 0.010328 * t * t) / (
        1.0 + 1.432788 * t + 0.189269 * t * t + 0.001308 * t ** 3)


def range_value(kind, w, k, crude=False):
    """The range of k standard normals at w > 0: its lower tail, its upper tail or its density.

    lower:   k * integral of phi(z) [Phi(z) - Phi(z - w)]^(k-1)
    upper:   k * integral of phi(z) Phi(z)^(k-1) {1 - [1 - Phi(z - w)/Phi(z)]^(k-1)}
    density: k (k-1) * integral of phi(z) phi(z - w) [Phi(z) - Phi(z - w)]^(k-2)
    """
    def f(z):
        y = z - w
        qz, qy = upper(abs(z)), upper(abs(y))
        if kind == "upper":
            below_z = 1 - qz if z >= 0 else qz
            below_y = 1 - qy if y >= 0 else qy
            return density(abs(z)) * below_z ** (k - 1) * one_minus_power(below_y / below_z, k - 1)
        band = qy - qz if y >= 0 else (qz - qy if z <= 0 else 1 - qz - qy)
        if kind == "lower":
            return density(abs(z)) * band ** (k - 1)
        return density(abs(z)) * density(abs(y)) * band ** (k - 2)

    m = largest(k)
    h = float(w) / 2
    guess = {"lower": min(m, h), "upper": max(m, h), "density": h}[kind]
    value = integrate(f, guess, 1.0 / math.sqrt(k), crude)
    return (k * (k - 1) if kind == "density" else k) * value


# ----- the studentized range -----

NORMS = {}


def weight(nu):
    """With s the root of a chi-squared over its nu degrees of freedom, s = ln(1 + e^t) and
    t = u - e^(-u): a function giving s and the density of u up to a constant, and the
    constant. The first change of variable keeps the integrand smooth far from the real line;
    the second makes its left tail die as fast as its right one."""
    a = Decimal(nu) / 2

    def shape(u):
        if u < -12:
            return Decimal(0), Decimal(0)
        back = (-u).exp()
        e = (u - back).exp()
        s = (1 + e).ln() if e > Decimal("1e-20") else e * (1 - e / 2 + e * e / 3)
        return s, (-a * (s * s - 1 - 2 * s.ln())).exp() / s * e / (1 + e) * (1 + back)

    if nu not in NORMS:
        NORMS[nu] = integrate(lambda u: shape(u)[1], place(1.0), 1.0 / math.sqrt(2.0 * nu + 2.0))
    return shape, NORMS[nu]


def place(s):
    """The u at which the changes of variable of `weight` give s (floats: a place to start)."""
    t = math.log(math.expm1(s)) if s < 30.0 else s
    u = t if t > 1.0 else -math.log(1.0 - t) if t < 0.0 else 0.5
    for _ in range(60):
        u -= (u - math.exp(-u) - t) / (1.0 + math.exp(-u))
    return u


def tukey(kind, q, k, nu):
    """The studentized range of k groups on nu degrees of freedom at q > 0: the lower tail,
    the upper tail or the density, each as its own integral over s of the range at q s."""
    shape, norm = weight(nu)
    typical = 2.0 * largest(k) / float(q)
    # The integrand peaks at s = 1 where the range function is flat there, near the s that
    # brings q s back to a typical range when q is far out, and never beyond the s at which the
    # weight overtakes the power law of a small range.
    rising = math.sqrt(1.0 + (k - 1.0) / nu)
    start = {
        "lower": 1.0 if typical <= 1.0 else min(typical, rising),
        "upper": min(1.0, typical),
        "density": min(typical, rising),
    }[kind]
    guess = place(start)

    def make(crude):
        def f(u):
            s, density_of_u = shape(u)
            if density_of_u == 0:
                return Decimal(0)
            value = range_value(kind, q * s, k, crude) * density_of_u
            return value * s if kind == "density" else value
        return f

    # Looking for the peak may use rough inner integrals; the sum may not.
    value = integrate(make(False), guess, 1.0 / math.sqrt(2.0 * nu + 2.0), search=make(True))
    return value / norm


# ----- the cases -----

# A typical value of the range for each number of groups, and for each number of degrees of
# freedom the multiples of it at which the distribution is read: well below the middle, at the
# middle, above it and far out, further out where the tail is heavier.
TYPICAL = {2: "1", 3: "1.6", 5: "2.3", 10: "3", 30: "4", 100: "5"}
MULTIPLES = {
    1: ("0.3", "1", "4", "40"),
    2: ("0.4", "1", "2.5", "12"),
    5: ("0.5", "1", "1.8", "4"),
    20: ("0.6", "1", "1.5", "2.5"),
    120: ("0.6", "1", "1.4", "2"),
    1e6: ("0.6", "1", "1.4", "2"),
}

EXTRA = [
    # degrees of freedom that are not whole
    ("lower", 3.5, 3, 2.5), ("upper", 4.25, 4, 7.5), ("density", 2.75, 6, 3.5),
    # so many degrees of freedom that the range is almost that of normals with a known variance
    ("lower", 3.0, 5, 1e7), ("upper", 4.5, 5, 1e9), ("upper", 3.0, 2, 1e10),
    ("lower", 2.0, 2, 1e12), ("upper", 5.0, 10, 1e12),
    # a range close to nothing
    ("lower", 0.01, 5, 10), ("lower", 0.001, 3, 4), ("lower", 1e-5, 2, 1), ("lower", 0.05, 20, 3),
    # far out in the upper tail
    ("upper", 50.0, 10, 5), ("upper", 20.0, 2, 2), ("upper", 1000.0, 3, 1), ("upper", 15.0, 5, 30),
    ("upper", 8.0, 3, 1e6), ("upper", 12.0, 50, 120), ("upper", 30.0, 4, 1e6),
    # more groups than the grid has
    ("lower", 6.0, 200, 50), ("upper", 7.5, 200, 50), ("density", 6.0, 200, 50),
    # the density away from the middle
    ("density", 3.0, 5, 10), ("density", 0.5, 2, 3), ("density", 6.0, 10, 20),
    ("density", 9.0, 3, 1), ("density", 0.2, 4, 1e6),
]

# The two closed forms, with t = q / sqrt(2): for two groups on one degree of freedom the lower
# tail is (2 / pi) atan(t), and on two degrees of freedom the upper tail is
# 2 / ((2 + t^2) + t sqrt(2 + t^2)).
CLOSED = [("lower", 1.0, 2, 1), ("lower", 7.0, 2, 1), ("upper", 1.5, 2, 2), ("upper", 20.0, 2, 2)]

# Points at which both tails are summed and must add up to one.
BOTH = [(2.3, 5, 5), (5.0, 100, 120), (1.6, 3, 1e6)]


def grid():
    cases = []
    for nu, multiples in MULTIPLES.items():
        for k, typical in TYPICAL.items():
            for i, multiple in enumerate(multiples):
                q = float(Decimal(typical) * Decimal(multiple))
                cases.append(("lower", q, k, nu))
                if i >= 2:
                    cases.append(("upper", q, k, nu))
                if i == 1:
                    cases.append(("density", q, k, nu))
    return cases


def compute(job):
    """One value. Each tail is its own integral, and at 70 digits either is also one minus the
    other, so a case of the grid takes the tail that is quicker to sum: the upper one beyond
    the middle of the distribution, the lower one before it.  computes the kind
    named, for the checks that the script makes of itself."""
    as_asked, (kind, q, k, nu) = job
    q = Decimal(q)
    if as_asked or kind == "density":
        return tukey(kind, q, k, nu)
    nearer = "upper" if float(q) > 2.0 * largest(k) else "lower"
    value = tukey(nearer, q, k, nu)
    return value if kind == nearer else 1 - value


def atan(x):
    """atan(x) for x > 0 by Euler's series, turned about pi/2 so that it converges quickly."""
    if x > 1:
        return PI / 2 - atan(1 / x)
    y = x * x / (1 + x * x)
    total = term = y / x
    n = 0
    while abs(term) > Decimal("1e-80") * abs(total):
        n += 1
        term = term * y * (2 * n) / (2 * n + 1)
        total += term
    return total


def closed_form(case):
    kind, q, k, nu = case
    t = Decimal(q) / Decimal(2).sqrt()
    if nu == 1:
        return 2 / PI * atan(t)
    return 2 / ((2 + t * t) + t * (2 + t * t).sqrt())


def to_double(r):
    return repr(float(r))


def main():
    cases = grid() + EXTRA
    jobs = [(False, case) for case in cases]
    jobs += [(True, case) for case in CLOSED]
    jobs += [(True, (kind, *point)) for point in BOTH for kind in ("lower", "upper")]
    with multiprocessing.get_context("spawn").Pool(min(4, os.cpu_count() or 1)) as pool:
        values = pool.map(compute, jobs, chunksize=1)
    checks = values[len(cases):]
    for case, value in zip(CLOSED, checks):
        error = abs(value / closed_form(case) - 1)
        if error > Decimal("1e-20"):
            raise ArithmeticError(f"the closed form of {case} is missed by {error:.1e}")
    for i, point in enumerate(BOTH):
        error = abs(checks[len(CLOSED) + 2 * i] + checks[len(CLOSED) + 2 * i + 1] - 1)
        if error > Decimal("1e-20"):
            raise ArithmeticError(f"the two tails at {point} miss one by {error:.1e}")
    out = {
        "reference": {
            "software": f"Python {platform.python_version()}, decimal module at {PREC} digits",
            "note": "every value computed exactly and rounded once to the nearest double; "
                    "`kind` is the lower tail, the upper tail or the density at `q`",
        },
        "cases": [
            {"kind": kind, "q": repr(q), "groups": k, "df": repr(float(nu)), "value": to_double(value)}
            for (kind, q, k, nu), value in zip(cases, values)
        ],
    }
    path = os.path.join(here, "tukey.json")
    with open(path, "w", encoding="utf-8", newline="\n") as f:
        json.dump(out, f, indent=1, ensure_ascii=False)
        f.write("\n")
    print("wrote", path)
    cases_path = os.path.join(here, "data", "tukey_cases.csv")
    with open(cases_path, "w", encoding="utf-8", newline="\n") as f:
        f.write("kind,q,groups,df\n")
        for kind, q, k, nu in cases:
            f.write(f"{kind},{q.hex()},{k},{float(nu).hex()}\n")
    print("wrote", cases_path)


if __name__ == "__main__":
    main()
