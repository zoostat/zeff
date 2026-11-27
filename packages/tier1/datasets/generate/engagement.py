"""The synthetic `engagement` dataset (Engine and Project Specification, section 8.5; release
0.10.0, step 8; question 5 of the release, taken as recommended on 27 November 2026): a staff
survey answered by 1,200 of the 2,400 employees of `staff`, in two waves, before and after the
training course of that dataset. Six statements about engagement at work were put in the second
wave, each answered from 1, "strongly disagree", to 5, "strongly agree"; one question, "would
you recommend the firm as a place to work?", was put in both waves and answered yes or no. Run
with `python3 -I packages/tier1/datasets/generate/engagement.py`, after `staff.py`, whose file it
reads for the employees' numbers and their training; it rewrites
`packages/tier1/datasets/data/engagement.csv` from draws of Python's Mersenne Twister with a
fixed seed.

The generating model, which the dataset's help page documents and the tests use. The 1,200
respondents are drawn first, without replacement, from the employees of `staff`, and written in
the order of their numbers. Then, for each respondent, in this order of draws:

    engagement   N(0, 1); not written to the file
    eng1..eng6   for item j, x = loading[j] * engagement + sqrt(1 - loading[j]^2) * N(0, 1), and
                 the answer is 1 plus the number of the cut points -1.5, -0.5, 0.5 and 1.5 that
                 x exceeds; the loadings are 0.80, 0.75, 0.70, 0.75, 0.65 and 0.70. The fourth
                 statement is worded in reverse ("I often think of leaving"), so its answer is
                 written as 6 less the answer above
    before       1 with probability 1 / (1 + exp(-u)), u = -0.3 + 0.9 engagement
    after        1 with probability 1 / (1 + exp(-u)),
                 u = -1.2 + 0.9 engagement + 0.8 training + 1.8 before, with the respondent's
                 `training` of `staff`
    missing      each of the six answers is left out with probability 0.03, one draw each, and
                 the answer of the second wave to the yes-or-no question with probability 0.02

The six answers share one factor, so they form a scale once the fourth is scored the right way
round. Cronbach's alpha of that scale in the population the model describes is worked out below
from the model itself, by integrating the bivariate normal over the cells of each pair of
answers, and is 0.8469 to four decimals; the script stops if its own figure is another. With
the fourth item left as written, its correlations with the other five are negative, which is
what an analyst who forgets the reversed wording sees.
"""

import csv
import math
import os
import random

SEED = 20261127
RESPONDENTS = 1200
LOADINGS = [0.80, 0.75, 0.70, 0.75, 0.65, 0.70]
CUTS = [-1.5, -0.5, 0.5, 1.5]
REVERSED = 4
MISSING_ITEM = 0.03
MISSING_AFTER = 0.02
POPULATION_ALPHA = "0.8469"


def phi(x):
    return math.exp(-0.5 * x * x) / math.sqrt(2.0 * math.pi)


def big_phi(x):
    return 0.5 * math.erfc(-x / math.sqrt(2.0))


def population_alpha():
    """Cronbach's alpha of the six answers, each scored the right way round, under the model."""
    edges = [-9.0] + CUTS + [9.0]
    marginal = [big_phi(edges[a + 1]) - big_phi(edges[a]) for a in range(5)]
    marginal[0] = big_phi(CUTS[0])
    marginal[4] = 1.0 - big_phi(CUTS[3])
    mean = sum((a + 1) * p for a, p in enumerate(marginal))
    variance = sum((a + 1 - mean) ** 2 * p for a, p in enumerate(marginal))

    def covariance(rho):
        # E[A B] by Simpson's rule over the first answer's latent value, cell by cell.
        spread = math.sqrt(1.0 - rho * rho)
        product = 0.0
        steps = 2000
        for a in range(5):
            lower, upper = edges[a], edges[a + 1]
            h = (upper - lower) / steps
            total = 0.0
            for i in range(steps + 1):
                x = lower + i * h
                conditional_mean = 0.0
                for b in range(5):
                    below = 0.0 if b == 0 else big_phi((edges[b] - rho * x) / spread)
                    above = 1.0 if b == 4 else big_phi((edges[b + 1] - rho * x) / spread)
                    conditional_mean += (b + 1) * (above - below)
                weight = 1 if i in (0, steps) else (4 if i % 2 == 1 else 2)
                total += weight * phi(x) * conditional_mean
            product += (a + 1) * total * h / 3.0
        return product - mean * mean

    k = len(LOADINGS)
    pairs = sum(
        covariance(LOADINGS[i] * LOADINGS[j]) for i in range(k) for j in range(i + 1, k)
    )
    total_variance = k * variance + 2.0 * pairs
    return k / (k - 1) * (1.0 - k * variance / total_variance)


def sample_alpha(rows):
    """Cronbach's alpha of the complete rows, the fourth answer scored the right way round."""
    complete = [
        [6 - v if j + 1 == REVERSED else v for j, v in enumerate(row)]
        for row in rows
        if all(v is not None for v in row)
    ]
    n, k = len(complete), len(LOADINGS)

    def var(values):
        m = sum(values) / len(values)
        return sum((v - m) ** 2 for v in values) / (len(values) - 1)

    items = sum(var([row[j] for row in complete]) for j in range(k))
    total = var([sum(row) for row in complete])
    return n, k / (k - 1) * (1.0 - items / total)


def main():
    here = os.path.dirname(os.path.abspath(__file__))
    with open(os.path.join(here, "..", "data", "staff.csv"), encoding="utf-8", newline="") as f:
        staff = {int(row["employee"]): int(row["training"]) for row in csv.DictReader(f)}
    alpha = population_alpha()
    if f"{alpha:.4f}" != POPULATION_ALPHA:
        raise SystemExit(f"the model's alpha is {alpha:.6f}, not the documented {POPULATION_ALPHA}")
    rng = random.Random(SEED)
    respondents = sorted(rng.sample(sorted(staff), RESPONDENTS))
    rows = []
    answers = []
    for employee in respondents:
        engagement = rng.gauss(0.0, 1.0)
        items = []
        for j, loading in enumerate(LOADINGS):
            x = loading * engagement + math.sqrt(1.0 - loading * loading) * rng.gauss(0.0, 1.0)
            answer = 1 + sum(1 for cut in CUTS if x > cut)
            items.append(6 - answer if j + 1 == REVERSED else answer)
        before = 1 if rng.random() < 1.0 / (1.0 + math.exp(-(-0.3 + 0.9 * engagement))) else 0
        u = -1.2 + 0.9 * engagement + 0.8 * staff[employee] + 1.8 * before
        after = 1 if rng.random() < 1.0 / (1.0 + math.exp(-u)) else 0
        items = [None if rng.random() < MISSING_ITEM else v for v in items]
        after = None if rng.random() < MISSING_AFTER else after
        answers.append(items)
        rows.append([employee] + items + [before, after])
    header = ["employee"] + [f"eng{j}" for j in range(1, 7)] + ["recommend_before", "recommend_after"]
    path = os.path.join(here, "..", "data", "engagement.csv")
    with open(path, "w", encoding="utf-8", newline="\n") as f:
        writer = csv.writer(f, lineterminator="\n")
        writer.writerow(header)
        for row in rows:
            writer.writerow(["" if v is None else v for v in row])
    complete, in_sample = sample_alpha(answers)
    print(
        "wrote", path, len(rows), "rows; the model's alpha", f"{alpha:.6f}",
        "; alpha of the", complete, "complete rows", f"{in_sample:.6f}",
    )


if __name__ == "__main__":
    main()
