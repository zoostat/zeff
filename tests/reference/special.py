"""Reference values of the special functions (release 0.3.0, step 2).

Writes `tests/reference/special.json`: the gamma and beta functions, the error functions and the
normal tails, the regularised incomplete gamma and beta functions and their inverses, each at the
points of a fixed grid, computed with Python's `decimal` module at 70 significant digits and
rounded once to the nearest double, so that the file is the same on every machine (Python's
`math` module is the platform's C library). Also writes `tests/reference/data/special_cases.csv`,
the same inputs as exact hexadecimal floats for the R script beside it. Run with
`python3 -I tests/reference/special.py`; continuous integration re-runs it and fails if anything
committed differs.
"""

import decimal
import json
import math
import os
import platform
from decimal import Decimal
from fractions import Fraction

here = os.path.dirname(os.path.abspath(__file__))

PREC = 70
decimal.getcontext().prec = PREC
decimal.getcontext().Emax = 999_999
decimal.getcontext().Emin = -999_999


def D(x):
    """A double as an exact decimal."""
    return Decimal(x)


# ----- constants -----

def compute_pi():
    decimal.getcontext().prec = PREC + 20
    three = Decimal(3)
    lasts, t, s, n, na, d, da = 0, three, 3, 1, 0, 0, 24
    while s != lasts:
        lasts = s
        n, na = n + na, na + 8
        d, da = d + da, da + 32
        t = (t * n) / d
        s += t
    decimal.getcontext().prec = PREC
    return +s


PI = compute_pi()
SQRT_PI = PI.sqrt()
LN_SQRT_2PI = (2 * PI).ln() / 2


def bernoulli_numbers(count):
    """B_0 .. B_count by the Akiyama–Tanigawa algorithm, exactly (B_1 = +1/2 convention is not
    used: the even-indexed numbers are what the Stirling series needs)."""
    a = []
    out = []
    for m in range(count + 1):
        a.append(Fraction(1, m + 1))
        for j in range(m, 0, -1):
            a[j - 1] = j * (a[j - 1] - a[j])
        out.append(a[0])
    return out


BERNOULLI = bernoulli_numbers(44)


def dsin(x):
    """sin(x) for a decimal x of moderate size."""
    decimal.getcontext().prec += 10
    two_pi = 2 * PI
    k = (x / two_pi).to_integral_value(rounding=decimal.ROUND_HALF_EVEN)
    x = x - k * two_pi
    i, lasts, s, fact, num, sign = 1, 0, x, 1, x, 1
    while s != lasts:
        lasts = s
        i += 2
        fact *= i * (i - 1)
        num *= x * x
        sign *= -1
        s += num / fact * sign
    decimal.getcontext().prec -= 10
    return +s


# ----- gamma and beta -----

def stirling(x):
    """ln Γ(x) by the Stirling series, for x ≥ 50 (22 terms leave less than 1e-60)."""
    total = (x - Decimal("0.5")) * x.ln() - x + LN_SQRT_2PI
    power = x
    x2 = x * x
    for k in range(1, 23):
        b = BERNOULLI[2 * k]
        total += Decimal(b.numerator) / Decimal(b.denominator) / (2 * k * (2 * k - 1)) / power
        power *= x2
    return total


def lgamma(x):
    """ln |Γ(x)| for a decimal x that is not zero or a negative integer."""
    decimal.getcontext().prec += 15
    if x > 0 and x == x.to_integral_value() and x <= 171:
        # ln((n - 1)!) exactly: zero at 1 and 2, where no series should leave a residue.
        value = Decimal(math.factorial(int(x) - 1)).ln()
    elif x < 0:
        # Reflection: Γ(x) Γ(1 - x) = π / sin(πx).
        value = PI.ln() - abs(dsin(PI * x)).ln() - lgamma(1 - x)
    else:
        shift = Decimal(0)
        product = Decimal(1)
        while x + shift < 50:
            product *= x + shift
            shift += 1
        value = stirling(x + shift) - product.ln()
    decimal.getcontext().prec -= 15
    return +value


def gamma_sign(x):
    if x > 0:
        return 1
    # Γ is negative on (-1, 0), positive on (-2, -1), and so on.
    return -1 if int(math.floor(float(x))) % 2 else 1


def gamma(x):
    return gamma_sign(x) * lgamma(x).exp()


def lbeta(a, b):
    return lgamma(a) + lgamma(b) - lgamma(a + b)


# ----- the error functions -----

def erf_series(x):
    """erf(x) by its Maclaurin series, at a precision that absorbs the growth of the terms."""
    if x == 0:
        return Decimal(0)
    extra = 20 + int(float(x * x) * 0.9)
    decimal.getcontext().prec += extra
    term = x
    total = x
    n = 0
    x2 = x * x
    while True:
        n += 1
        term *= -x2 / n
        contribution = term / (2 * n + 1)
        total += contribution
        if abs(contribution) < abs(total) * Decimal(10) ** -(PREC + 10):
            break
    total = total * 2 / SQRT_PI
    decimal.getcontext().prec -= extra
    return +total


def erfc_fraction(x):
    """erfc(x) by its continued fraction, for x ≥ 5: exp(-x²)/√π · 1/(x + ½/(x + 1/(x + 3/2/(x + …))))."""
    decimal.getcontext().prec += 10
    value = Decimal(0)
    for k in range(400, 0, -1):
        value = (Decimal(k) / 2) / (x + value)
    value = (-x * x).exp() / SQRT_PI / (x + value)
    decimal.getcontext().prec -= 10
    return +value


def erf(x):
    if x >= 5:
        return 1 - erfc_fraction(x)
    if x <= -5:
        return erfc_fraction(-x) - 1
    return erf_series(x)


def erfc(x):
    if x >= 5:
        return erfc_fraction(x)
    if x <= -5:
        return 2 - erfc_fraction(-x)
    decimal.getcontext().prec += 20
    value = 1 - erf_series(x)
    decimal.getcontext().prec -= 20
    return +value


SQRT_2 = Decimal(2).sqrt()


def normal_lower(x):
    return erfc(-x / SQRT_2) / 2


def normal_upper(x):
    return erfc(x / SQRT_2) / 2


# ----- the regularised incomplete gamma function -----

def gamma_prefix(a, x):
    return (a * x.ln() - x - lgamma(a)).exp()


def incomplete_gamma(a, x):
    """(P, Q): the series below a + 1, the continued fraction above, the other as the complement."""
    decimal.getcontext().prec += 15
    if x == 0:
        p, q = Decimal(0), Decimal(1)
    elif x < a + 1:
        term = 1 / a
        total = term
        n = 0
        while True:
            n += 1
            term *= x / (a + n)
            total += term
            if abs(term) < abs(total) * Decimal(10) ** -(PREC + 10):
                break
        p = gamma_prefix(a, x) * total
        q = 1 - p
    else:
        tiny = Decimal(10) ** -(PREC + 60)
        b = x + 1 - a
        c = 1 / tiny
        d = 1 / b
        h = d
        i = 0
        while True:
            i += 1
            an = -i * (i - a)
            b += 2
            d = an * d + b
            if abs(d) < tiny:
                d = tiny
            c = b + an / c
            if abs(c) < tiny:
                c = tiny
            d = 1 / d
            delta = d * c
            h *= delta
            if abs(delta - 1) < Decimal(10) ** -(PREC + 10):
                break
        q = gamma_prefix(a, x) * h
        p = 1 - q
    decimal.getcontext().prec -= 15
    return +p, +q


# ----- the regularised incomplete beta function -----

def beta_prefix(a, b, x):
    return (a * x.ln() + b * (1 - x).ln() - lbeta(a, b)).exp()


def beta_fraction(a, b, x):
    tiny = Decimal(10) ** -(PREC + 60)
    qab = a + b
    qap = a + 1
    qam = a - 1
    c = Decimal(1)
    d = 1 - qab * x / qap
    if abs(d) < tiny:
        d = tiny
    d = 1 / d
    h = d
    m = 0
    while True:
        m += 1
        m2 = 2 * m
        aa = m * (b - m) * x / ((qam + m2) * (a + m2))
        d = 1 + aa * d
        if abs(d) < tiny:
            d = tiny
        c = 1 + aa / c
        if abs(c) < tiny:
            c = tiny
        d = 1 / d
        h *= d * c
        aa = -(a + m) * (qab + m) * x / ((a + m2) * (qap + m2))
        d = 1 + aa * d
        if abs(d) < tiny:
            d = tiny
        c = 1 + aa / c
        if abs(c) < tiny:
            c = tiny
        d = 1 / d
        delta = d * c
        h *= delta
        if abs(delta - 1) < Decimal(10) ** -(PREC + 10):
            break
    return h


def incomplete_beta(a, b, x):
    """(I, 1 - I)."""
    decimal.getcontext().prec += 15
    if x == 0:
        lower, upper = Decimal(0), Decimal(1)
    elif x == 1:
        lower, upper = Decimal(1), Decimal(0)
    elif x < (a + 1) / (a + b + 2):
        lower = beta_prefix(a, b, x) * beta_fraction(a, b, x) / a
        upper = 1 - lower
    else:
        upper = beta_prefix(b, a, 1 - x) * beta_fraction(b, a, 1 - x) / b
        lower = 1 - upper
    decimal.getcontext().prec -= 15
    return +lower, +upper


# ----- inverses by bisection -----

def solve(f, derivative, target, lo, hi, log_space=False):
    """The x in (lo, hi) with increasing f(x) = target: bisection to about twelve digits (in
    ln x when `log_space`, for a root deep in a lower tail), then Newton steps on the exact
    function with the bracket kept, which square the digits each time."""
    if log_space:
        ulo, uhi = lo.ln(), hi.ln()
        for _ in range(64):
            mid = (ulo + uhi) / 2
            if f(mid.exp()) < target:
                ulo = mid
            else:
                uhi = mid
        lo, hi = ulo.exp(), uhi.exp()
    else:
        for _ in range(40):
            mid = (lo + hi) / 2
            if f(mid) < target:
                lo = mid
            else:
                hi = mid
    x = (lo + hi) / 2
    close = Decimal(10) ** -(PREC - 4)
    for _ in range(100):
        fx = f(x)
        if fx == target:
            break
        if fx < target:
            lo = x
        else:
            hi = x
        slope = derivative(x)
        nxt = x - (fx - target) / slope if slope != 0 else None
        if nxt is None or not (lo < nxt < hi):
            nxt = (lo + hi) / 2
        step = nxt - x
        x = nxt
        if abs(step) <= abs(x) * close or hi - lo <= abs(x) * close:
            break
    return x


def normal_density(x):
    return (-x * x / 2).exp() / (2 * PI).sqrt()


def gamma_density(a, x):
    return gamma_prefix(a, x) / x


def beta_density(a, b, x):
    return beta_prefix(a, b, x) / (x * (1 - x))


def normal_quantile(p, upper):
    if upper:
        return -normal_quantile(p, False)
    if p == Decimal("0.5"):
        return Decimal(0)
    return solve(normal_lower, normal_density, p, Decimal(-40), Decimal(40))


def incomplete_gamma_inverse(a, p, upper):
    hi = a + 60 * a.sqrt() + 60
    while incomplete_gamma(a, hi)[0] < 1 - Decimal(10) ** -60 and hi < Decimal(10) ** 6:
        hi *= 2
    # Solve on the smaller tail: a lower tail may lie deep (log space), an upper tail never.
    if upper and p > Decimal("0.5"):
        return incomplete_gamma_inverse(a, 1 - p, False)
    if not upper and p > Decimal("0.5"):
        return incomplete_gamma_inverse(a, 1 - p, True)
    if upper:
        return solve(lambda x: -incomplete_gamma(a, x)[1], lambda x: gamma_density(a, x), -p, Decimal(0), hi)
    return solve(lambda x: incomplete_gamma(a, x)[0], lambda x: gamma_density(a, x), p,
                 Decimal(10) ** -320, hi, log_space=True)


def incomplete_beta_inverse(a, b, p, upper):
    if upper:
        # 1 - I_x(a, b) = p is I_{1-x}(b, a) = p.
        return 1 - incomplete_beta_inverse(b, a, p, False)
    if p > Decimal("0.5"):
        # The smaller tail is the upper one: I_x(a, b) = p is I_{1-x}(b, a) = 1 - p.
        return 1 - incomplete_beta_inverse(b, a, 1 - p, False)
    return solve(lambda x: incomplete_beta(a, b, x)[0], lambda x: beta_density(a, b, x), p,
                 Decimal(10) ** -320, Decimal(1), log_space=True)


# ----- the grids -----

gamma_grid = [1e-10, 0.001, 0.1, 0.25, 0.5, 0.75, 1.0, 1.5, 2.0, 2.5, 3.0, 4.5, 7.25, 10.0, 15.5,
              20.0, 30.0, 50.0, 100.0, 170.5, 171.6, 500.0, 1000.0, 12345.678, 1e5, 1e10,
              -0.5, -1.5, -2.5, -3.7, -10.5, -100.5, -0.001]
factorials = [0, 1, 2, 5, 10, 18, 19, 20, 22, 25, 50, 100, 150, 170]
chooses = [(0, 0), (5, 2), (10, 3), (30, 15), (50, 25), (60, 30), (100, 50), (120, 60), (200, 100),
           (1000, 3), (1000, 500), (1029, 514), (1030, 515), (10**12, 2), (10**9, 7)]
beta_pairs = [(0.5, 0.5), (1.0, 1.0), (2.0, 3.0), (0.5, 10.0), (5.0, 5.0), (10.0, 0.5), (30.0, 70.0),
              (100.0, 100.0), (500.0, 2.0), (1000.0, 1000.0), (1e4, 1e4), (5e5, 0.5), (1e6, 0.5),
              (1e-3, 1e-3), (3.5, 1e8)]
erf_grid = [0.0, 1e-10, 0.01, 0.1, 0.5, 1.0, 1.5, 2.0, 3.0, 4.0, 5.0, 5.5, 6.0, 8.0, 10.0, 15.0, 20.0,
            26.0, 27.2, -0.5, -1.0, -3.0, -6.0, -10.0, -27.0]
normal_grid = [0.0, 0.5, 1.0, 1.959963984540054, 2.5, 3.0, 4.0, 5.0, 6.0, 8.0, 10.0, 20.0, 30.0,
               37.0, 38.5, -0.5, -1.0, -2.0, -3.0, -5.0, -8.0, -20.0, -37.0, -38.5]
probabilities = [1e-300, 1e-100, 1e-20, 1e-10, 1e-6, 0.001, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 0.75,
                 0.9, 0.95, 0.975, 0.99, 0.999, 0.999999, 0.9999999999]
ig_shapes = [0.1, 0.5, 1.0, 2.5, 5.0, 10.0, 30.0, 100.0, 1000.0, 1e4, 1e5]
ib_pairs = [(0.5, 0.5), (1.0, 1.0), (2.0, 3.0), (0.5, 10.0), (5.0, 5.0), (10.0, 0.5), (30.0, 70.0),
            (100.0, 100.0), (500.0, 2.0), (1000.0, 1000.0), (1e4, 1e4), (5e5, 0.5), (1.5, 0.5),
            (0.05, 0.05)]
inverse_probabilities = [1e-10, 1e-6, 0.001, 0.025, 0.1, 0.5, 0.9, 0.975, 0.999, 0.999999]
ig_inverse_shapes = [0.1, 0.5, 1.0, 2.5, 10.0, 100.0, 1000.0, 1e4]
ib_inverse_pairs = [(0.5, 0.5), (2.0, 3.0), (0.5, 10.0), (5.0, 5.0), (30.0, 70.0), (1000.0, 1000.0),
                    (1e4, 1e4), (1.5, 0.5)]


def ig_points(a):
    """x values for the incomplete gamma at shape a: the mean, both sides, and the far tails."""
    xs = [1e-300, a / 100, a / 10, a / 2, a * 0.9, a, a * 1.1, a * 1.5, a * 2, a * 3, a + 20 * math.sqrt(a) + 20]
    return [x for x in xs if x > 0]


def ib_points(a, b):
    mean = a / (a + b)
    sd = math.sqrt(a * b / ((a + b) ** 2 * (a + b + 1)))
    xs = [1e-6, 0.01, 0.1, 0.3, mean, 0.7, 0.9, 0.99, 1 - 1e-6, mean - 4 * sd, mean + 4 * sd,
          mean - 8 * sd, mean + 8 * sd]
    return sorted({x for x in xs if 0 < x < 1})


# ----- writing -----

def to_double(r):
    y = float(r)
    if math.isinf(y):
        return "Inf" if y > 0 else "-Inf"
    if y == 0:
        return "-0.0" if r.is_signed() else "0.0"
    return repr(y)


def text(x):
    return repr(float(x))


cases = []  # (kind, a, b, x, p) as doubles, for the R script


def main():
    out = {
        "reference": {
            "software": f"Python {platform.python_version()}, decimal module at {PREC} digits",
            "note": "every value computed exactly and rounded once to the nearest double",
        }
    }
    out["gamma_grid"] = [text(x) for x in gamma_grid]
    out["log_gamma"] = [to_double(lgamma(D(x))) for x in gamma_grid]
    out["gamma"] = []
    for x in gamma_grid:
        lg = lgamma(D(x))
        # Past ln(1.8e308) the double overflows whatever the digits.
        out["gamma"].append("Inf" if lg > 710 else to_double(gamma_sign(D(x)) * lg.exp()))
    for x in gamma_grid:
        cases.append(("gamma", x, None, None, None))
    out["factorial"] = [[n, repr(float(math.factorial(n)))] for n in factorials]
    out["choose"] = []
    for n, k in chooses:
        exact = math.comb(n, k)
        out["choose"].append([n, k, "Inf" if exact.bit_length() > 1024 else repr(float(exact))])
    out["beta"] = []
    for a, b in beta_pairs:
        lb = lbeta(D(a), D(b))
        out["beta"].append([text(a), text(b), to_double(lb), to_double(lb.exp())])
        cases.append(("beta", a, b, None, None))
    out["erf_grid"] = [text(x) for x in erf_grid]
    out["erf"] = [to_double(erf(D(x))) for x in erf_grid]
    out["erfc"] = [to_double(erfc(D(x))) for x in erf_grid]
    for x in erf_grid:
        cases.append(("erf", x, None, None, None))
    out["normal_grid"] = [text(x) for x in normal_grid]
    out["normal_lower"] = [to_double(normal_lower(D(x))) for x in normal_grid]
    out["normal_upper"] = [to_double(normal_upper(D(x))) for x in normal_grid]
    for x in normal_grid:
        cases.append(("normal", x, None, None, None))
    out["normal_quantile"] = []
    for p in probabilities:
        out["normal_quantile"].append([text(p), to_double(normal_quantile(D(p), False)),
                                       to_double(normal_quantile(D(p), True))])
        cases.append(("normal_quantile", None, None, None, p))
    out["incomplete_gamma"] = []
    for a in ig_shapes:
        for x in ig_points(a):
            p, q = incomplete_gamma(D(a), D(x))
            out["incomplete_gamma"].append([text(a), text(x), to_double(p), to_double(q)])
            cases.append(("incomplete_gamma", a, None, x, None))
    out["incomplete_gamma_inverse"] = []
    for a in ig_inverse_shapes:
        for p in inverse_probabilities:
            lower = incomplete_gamma_inverse(D(a), D(p), False)
            upper = incomplete_gamma_inverse(D(a), D(p), True)
            out["incomplete_gamma_inverse"].append([text(a), text(p), to_double(lower), to_double(upper)])
            cases.append(("incomplete_gamma_inverse", a, None, None, p))
    out["incomplete_beta"] = []
    for a, b in ib_pairs:
        for x in ib_points(a, b):
            lower, upper = incomplete_beta(D(a), D(b), D(x))
            out["incomplete_beta"].append([text(a), text(b), text(x), to_double(lower), to_double(upper)])
            cases.append(("incomplete_beta", a, b, x, None))
    out["incomplete_beta_inverse"] = []
    for a, b in ib_inverse_pairs:
        for p in inverse_probabilities:
            lower = incomplete_beta_inverse(D(a), D(b), D(p), False)
            upper = incomplete_beta_inverse(D(a), D(b), D(p), True)
            out["incomplete_beta_inverse"].append([text(a), text(b), text(p), to_double(lower), to_double(upper)])
            cases.append(("incomplete_beta_inverse", a, b, None, p))

    path = os.path.join(here, "special.json")
    with open(path, "w", encoding="utf-8", newline="\n") as f:
        json.dump(out, f, indent=1, ensure_ascii=False)
        f.write("\n")
    print("wrote", path)
    cases_path = os.path.join(here, "data", "special_cases.csv")
    with open(cases_path, "w", encoding="utf-8", newline="\n") as f:
        f.write("kind,a,b,x,p\n")
        for kind, a, b, x, p in cases:
            f.write(",".join([kind] + ["" if v is None else float.hex(float(v)) for v in (a, b, x, p)]) + "\n")
    print("wrote", cases_path)


if __name__ == "__main__":
    main()
