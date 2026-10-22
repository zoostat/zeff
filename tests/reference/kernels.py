"""Exact reference values for the linear-algebra kernel (release 0.4.0, step 2).

Writes `tests/reference/kernels.json`: least-squares cases with their exact solutions, computed in
rational arithmetic with Python's `fractions` on the exact binary values of the inputs and rounded
once to the nearest double (the inputs come from Python's seeded generator through additions and
multiplications only, never through the platform's maths library, so the file is the same on
every machine); the collinearity rule of section 7.2 of the Language Reference (R's
`dqrdc2`: a column whose remaining norm is below 1e-7 of its own norm is left out) applied
exactly; a triangular system with its exact solution and inverse; a symmetric positive-definite
system; and sums whose compensation matters. Every case has inputs from 1e7 to 1e12 beside its
working values, as the risk register's R3 asks. Also writes `tests/reference/data/kernels_cases.csv`,
the same inputs as exact hexadecimal floats for the R script beside it. Run with
`python3 -I tests/reference/kernels.py`; continuous integration re-runs it and fails if anything
committed differs.
"""

import json
import os
import random
from fractions import Fraction

here = os.path.dirname(os.path.abspath(__file__))

TOLERANCE = Fraction(1, 10_000_000)  # the collinearity rule's 1e-7, relative to a column's norm


def text(x):
    """A double as the shortest text that reads back to the same value."""
    return repr(float(x))


def exact(x):
    """The exact rational value of a double."""
    return Fraction(float(x))


def rounded(q):
    """A rational rounded once to the nearest double, as text."""
    return text(float(q))


# ----- exact linear algebra on rationals -----

def dot(a, b):
    return sum((x * y for x, y in zip(a, b)), Fraction(0))


def solve(a, b):
    """Solve the square system a x = b exactly by Gaussian elimination with pivoting on the
    largest absolute entry (exact arithmetic, so the choice of pivot changes nothing)."""
    n = len(a)
    m = [row[:] + [b[i]] for i, row in enumerate(a)]
    for col in range(n):
        pivot = max(range(col, n), key=lambda r: abs(m[r][col]))
        if m[pivot][col] == 0:
            raise ValueError("singular")
        m[col], m[pivot] = m[pivot], m[col]
        for r in range(n):
            if r != col and m[r][col] != 0:
                factor = m[r][col] / m[col][col]
                m[r] = [x - factor * y for x, y in zip(m[r], m[col])]
    return [m[i][n] / m[i][i] for i in range(n)]


def inverse(a):
    n = len(a)
    columns = [solve(a, [Fraction(int(i == j)) for i in range(n)]) for j in range(n)]
    return [[columns[j][i] for j in range(n)] for i in range(n)]


def columns_of(x):
    return [[row[j] for row in x] for j in range(len(x[0]))]


def accepted_columns(x):
    """The columns the limited pivoting keeps, in order: a column whose residual norm after the
    accepted columns before it have been projected out is below the tolerance times its own norm
    is left out (a zero column counts as having norm one, so it is left out too)."""
    cols = columns_of(x)
    kept = []
    for j, c in enumerate(cols):
        norm2 = dot(c, c)
        if kept:
            basis = [cols[k] for k in kept]
            gram = [[dot(p, q) for q in basis] for p in basis]
            rhs = [dot(p, c) for p in basis]
            coefficients = solve(gram, rhs)
            fitted = [sum((coefficients[k] * basis[k][i] for k in range(len(basis))), Fraction(0)) for i in range(len(c))]
            residual = [ci - fi for ci, fi in zip(c, fitted)]
            remaining2 = dot(residual, residual)
        else:
            remaining2 = norm2
        original2 = norm2 if norm2 != 0 else Fraction(1)
        if remaining2 < TOLERANCE * TOLERANCE * original2:
            continue
        kept.append(j)
    return kept


def least_squares(x, y):
    kept = accepted_columns(x)
    cols = columns_of(x)
    basis = [cols[k] for k in kept]
    gram = [[dot(p, q) for q in basis] for p in basis]
    rhs = [dot(p, y) for p in basis]
    beta = solve(gram, rhs)
    fitted = [sum((beta[k] * basis[k][i] for k in range(len(basis))), Fraction(0)) for i in range(len(y))]
    residual = [yi - fi for yi, fi in zip(y, fitted)]
    gram_inverse = inverse(gram)
    leverage = []
    for i in range(len(y)):
        xi = [basis[k][i] for k in range(len(basis))]
        leverage.append(sum((xi[a] * gram_inverse[a][b] * xi[b] for a in range(len(xi)) for b in range(len(xi))), Fraction(0)))
    coefficients = [None] * len(cols)
    for position, j in enumerate(kept):
        coefficients[j] = beta[position]
    full_gram = [[dot(p, q) for q in cols] for p in cols]
    return {
        "rows": len(y),
        "cols": len(cols),
        "x": [[text(v) for v in row] for row in x],
        "y": [text(v) for v in y],
        "kept": kept,
        "rank": len(kept),
        "coefficients": [None if c is None else rounded(c) for c in coefficients],
        "residual_sum_of_squares": rounded(dot(residual, residual)),
        "fitted": [rounded(f) for f in fitted],
        "xtx": [[rounded(v) for v in row] for row in full_gram],
        "kept_xtx_inverse": [[rounded(v) for v in row] for row in gram_inverse],
        "leverage": [rounded(h) for h in leverage],
    }


# ----- the cases -----

def uniform_matrix(rng, rows, cols, scale, decimals=None):
    out = []
    for _ in range(rows):
        row = []
        for j in range(cols):
            u = rng.random()
            v = u * scale[j] if isinstance(scale, list) else u * scale
            if decimals is not None:
                v = round(v, decimals)
            row.append(float(v))
        out.append(row)
    return out


def cases():
    rng = random.Random(20261023)
    out = {}

    x = uniform_matrix(rng, 7, 3, 10.0, decimals=3)
    y = [round(rng.random() * 10, 3) for _ in range(7)]
    out["plain"] = (x, y)

    x = [[1.0] + row for row in uniform_matrix(rng, 20, 2, 5.0, decimals=2)]
    y = [round(2.0 + 0.5 * row[1] - 0.25 * row[2] + (rng.random() - 0.5) * 0.6, 3) for row in x]
    out["intercept"] = (x, y)

    scales = [1.0, 1e7, 1e9, 1e12]
    x = [[1.0] + row for row in uniform_matrix(rng, 25, 4, scales)]
    y = [1e9 * rng.random() + 3e-3 * row[2] - 2e-6 * row[3] + 5e-10 * row[4] for row in x]
    out["magnitudes"] = (x, y)

    base = [[float(rng.randint(-5, 5)) for _ in range(2)] for _ in range(10)]
    x = [[a, b, 2.0 * a + b, float(rng.randint(0, 3))] for a, b in base]
    y = [float(rng.randint(-10, 10)) for _ in range(10)]
    out["duplicate"] = (x, y)

    for name, noise in (("near_collinear_dropped", 1e-9), ("near_collinear_kept", 1e-5)):
        rows = []
        for _ in range(12):
            u = rng.random()
            v = rng.random()
            w = rng.random() * 2 - 1
            rows.append([u, v, float(u + noise * w)])
        y = [rng.random() for _ in range(12)]
        out[name] = (rows, y)

    x = uniform_matrix(rng, 300, 5, [1.0, 2.0, 1e7, 0.5, 1e12])
    y = [row[0] - row[1] + 1e-7 * row[2] + (rng.random() - 0.5) * 0.2 for row in x]
    out["tall"] = (x, y)

    x = [[float(v)] for v in (1.5, 2.5, 3.0, 4.25, 5.0)]
    out["one_column"] = (x, [1.0, 2.0, 2.0, 5.0, 4.0])

    x = [[float(rng.randint(-3, 3)) for _ in range(4)] for _ in range(4)]
    while True:
        try:
            solve([[exact(v) for v in row] for row in x], [Fraction(1)] * 4)
            break
        except ValueError:
            x = [[float(rng.randint(-3, 3)) for _ in range(4)] for _ in range(4)]
    out["square"] = (x, [1.0, -2.0, 0.5, 3.0])

    x = [[float(rng.randint(-4, 4)) for _ in range(5)] for _ in range(3)]
    out["wide"] = (x, [1.0, 2.0, 3.0])

    x = [[rng.random(), 0.0, rng.random() * 1e7] for _ in range(6)]
    out["zero_column"] = (x, [rng.random() for _ in range(6)])

    x = [[1.0, float(i)] for i in range(6)]
    out["exact_fit"] = (x, [3.0 - 0.5 * i for i in range(6)])
    return out


def main():
    out = {
        "reference": {
            "software": "Python fractions: exact rational arithmetic on the binary values of the inputs",
            "note": "every value rounded once to the nearest double; the collinearity rule (1e-7 of a column's own norm) applied exactly",
        },
        "least_squares": {},
    }
    csv_rows = []
    for name, (x, y) in cases().items():
        xq = [[exact(v) for v in row] for row in x]
        yq = [exact(v) for v in y]
        out["least_squares"][name] = least_squares(xq, yq)
        for i, row in enumerate(x):
            for j, v in enumerate(row):
                csv_rows.append((name, "x", i, j, float.hex(v)))
        for i, v in enumerate(y):
            csv_rows.append((name, "y", i, 0, float.hex(v)))

    # A triangular system with small integers: exact solution and inverse.
    r = [[2.0, -1.0, 3.0, 0.5, 1.0], [0.0, 4.0, 1.0, -2.0, 0.25], [0.0, 0.0, -1.5, 2.0, 1.0], [0.0, 0.0, 0.0, 0.8, -3.0], [0.0, 0.0, 0.0, 0.0, 1e7]]
    b = [1.0, -2.0, 3.5, 0.0, 2e7]
    rq = [[exact(v) for v in row] for row in r]
    xq = solve(rq, [exact(v) for v in b])
    rinv = inverse(rq)
    out["triangular"] = {
        "r": [[text(v) for v in row] for row in r],
        "b": [text(v) for v in b],
        "x": [rounded(v) for v in xq],
        "inverse": [[rounded(v) for v in row] for row in rinv],
    }
    for i, row in enumerate(r):
        for j, v in enumerate(row):
            csv_rows.append(("triangular", "r", i, j, float.hex(v)))
    for i, v in enumerate(b):
        csv_rows.append(("triangular", "b", i, 0, float.hex(v)))

    # A symmetric positive-definite system, BᵀB + D, with magnitudes up to 1e12 on the diagonal.
    rng = random.Random(7)
    bm = [[float(rng.randint(-3, 3)) for _ in range(5)] for _ in range(8)]
    a = [[sum(bm[k][i] * bm[k][j] for k in range(8)) + (1e12 if i == j == 4 else 1.0 if i == j else 0.0) for j in range(5)] for i in range(5)]
    rhs = [1.0, 2.0, -1.0, 0.5, 1e12]
    aq = [[exact(v) for v in row] for row in a]
    out["positive_definite"] = {
        "a": [[text(v) for v in row] for row in a],
        "b": [text(v) for v in rhs],
        "x": [rounded(v) for v in solve(aq, [exact(v) for v in rhs])],
    }
    for i, row in enumerate(a):
        for j, v in enumerate(row):
            csv_rows.append(("positive_definite", "a", i, j, float.hex(v)))
    for i, v in enumerate(rhs):
        csv_rows.append(("positive_definite", "b", i, 0, float.hex(v)))

    # Sums whose compensation matters: large and small values mixed, exact in rationals.
    values = []
    for i in range(1000):
        values.append(float(1e12 if i % 7 == 0 else -1e12 if i % 7 == 1 else rng.random() * 1e-3 if i % 7 == 2 else rng.random() - 0.5))
    others = [float(rng.random() * 1e7 - 5e6) for _ in range(1000)]
    vq = [exact(v) for v in values]
    oq = [exact(v) for v in others]
    out["sums"] = {
        "values": [text(v) for v in values],
        "others": [text(v) for v in others],
        "sum": rounded(sum(vq, Fraction(0))),
        "dot": rounded(dot(vq, oq)),
        "sum_of_squares": rounded(dot(oq, oq)),
    }

    with open(os.path.join(here, "kernels.json"), "w", encoding="utf-8") as f:
        json.dump(out, f, indent=1)
        f.write("\n")
    with open(os.path.join(here, "data", "kernels_cases.csv"), "w", encoding="utf-8", newline="\n") as f:
        f.write("case,kind,row,col,value\n")
        for row in csv_rows:
            f.write(",".join(str(v) for v in row) + "\n")
    print("wrote kernels.json and data/kernels_cases.csv:", len(out["least_squares"]), "least-squares cases")


if __name__ == "__main__":
    main()
