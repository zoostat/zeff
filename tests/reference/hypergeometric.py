"""Exact values of the hypergeometric distribution (release 0.10.0, step 6).

Writes `tests/reference/hypergeometric.json`: for populations from a handful to a few thousand,
the probability of each number of successes and both tails of the distribution function, as
quotients of whole numbers in Python's `fractions` and `math.comb`, each rounded once to the
nearest double. Nothing is approximated on the way, so a tail of 1e-200 is as right as a
probability near one half, and the engine's sums are held to these within 1e-12 relative.

With m successes, n failures and d draws without replacement, the probability of k successes is
C(m, k) C(n, d - k) / C(m + n, d). The lower tail is the sum up to and including k and the upper
tail the sum above k; the two are summed apart, each from its own end.

Run with `python3 -I tests/reference/hypergeometric.py`; continuous integration re-runs it and
fails if anything committed differs.
"""

import json
import os
import platform
from fractions import Fraction
from math import comb

here = os.path.dirname(os.path.abspath(__file__))


def dec(value):
    """A fraction as the nearest double, written so that it reads back to the same double."""
    return repr(float(value))


def case(successes, failures, draws, ks):
    first, last = max(0, draws - failures), min(draws, successes)
    whole = comb(successes + failures, draws)
    counts = {k: comb(successes, k) * comb(failures, draws - k) for k in range(first, last + 1)}
    assert sum(counts.values()) == whole
    below = {}
    running = 0
    for k in range(first, last + 1):
        running += counts[k]
        below[k] = running
    rows = {"k": [], "pdf": [], "lower": [], "upper": []}
    for k in ks:
        assert first <= k <= last, (successes, failures, draws, k)
        rows["k"].append(k)
        rows["pdf"].append(dec(Fraction(counts[k], whole)))
        rows["lower"].append(dec(Fraction(below[k], whole)))
        rows["upper"].append(dec(Fraction(whole - below[k], whole)))
    mean = Fraction(draws * successes, successes + failures)
    return {"successes": successes, "failures": failures, "draws": draws, "mean": dec(mean), **rows}


def spread(first, last, count):
    """About `count` whole numbers from first to last, both ends among them."""
    if last - first < count:
        return list(range(first, last + 1))
    step = (last - first) / (count - 1)
    return sorted({first + round(i * step) for i in range(count)})


cases = [
    # Every value of small urns, the degenerate ones among them.
    case(10, 7, 8, list(range(1, 9))),
    case(5, 5, 3, [0, 1, 2, 3]),
    case(1, 1, 1, [0, 1]),
    case(0, 6, 4, [0]),
    case(6, 0, 4, [4]),
    case(9, 4, 0, [0]),
    case(9, 4, 13, [9]),
    case(3, 2, 4, [2, 3]),
    # Rare successes and rare failures.
    case(50, 950, 100, spread(0, 50, 26)),
    case(950, 50, 100, spread(50, 100, 26)),
    # A few thousand, from one end of the range to the other: the far values are of the order
    # of 1e-200 and less.
    case(300, 700, 400, spread(0, 300, 41)),
    case(1000, 1000, 1000, spread(0, 1000, 41)),
    case(2500, 1500, 3000, spread(1500, 2500, 41)),
    case(40, 3960, 2000, spread(0, 40, 41)),
]

out = {
    "reference": {
        "software": "Python fractions and math.comb (standard library), exact rational arithmetic",
        "python": platform.python_version(),
    },
    "cases": cases,
}
with open(os.path.join(here, "hypergeometric.json"), "w", encoding="utf-8") as f:
    json.dump(out, f, indent=1, ensure_ascii=False)
    f.write("\n")
values = sum(3 * len(c["k"]) for c in cases)
print(f"Python {platform.python_version()}: wrote hypergeometric.json, {len(cases)} cases, {values} values")
