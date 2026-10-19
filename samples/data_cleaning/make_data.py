"""Write data/staff_de.csv, the file the sample project cleans: a staff list as a German
spreadsheet exports it. Fields are separated by `;`, the decimal mark is a comma, a point
separates groups of three digits, the letters are Windows-1252, a missing value is -99, and
the headers carry spaces. Python's standard library only; the file is the same on every run.
"""

import random
from pathlib import Path

REGIONS = ["Köln", "Zürich", "Málaga", "Genève", "Düsseldorf"]
REGION_EFFECT = [0.00, 0.22, -0.18, 0.15, 0.04]


def german(number, decimals):
    """1234.5 -> '1.234,50'."""
    text = f"{number:,.{decimals}f}"
    return text.replace(",", "\x00").replace(".", ",").replace("\x00", ".")


def main():
    rng = random.Random(20261019)
    lines = ["Staff Id;Region;Monthly Wage;Weekly Hours;Age;Years of Education;Female"]
    for staff_id in range(1, 361):
        region = rng.randrange(len(REGIONS))
        female = rng.random() < 0.46
        educ = min(20, max(8, round(rng.gauss(13.5, 2.6))))
        age = rng.randint(16, 70)
        exper = max(0, age - educ - 6)
        latent = (7.45 + 0.07 * educ + 0.03 * exper - 0.0005 * exper * exper
                  - 0.11 * female + REGION_EFFECT[region] + rng.gauss(0, 0.28))
        wage = round(2.718281828459045 ** latent, 2)
        hours = round(min(70.0, max(5.0, rng.gauss(38, 7) - 3 * female)), 1)
        if rng.random() < 0.04:
            hours_text = "-99"
        else:
            hours_text = german(hours, 1)
        wage_text = "-99" if rng.random() < 0.025 else german(wage, 2)
        lines.append(f"{staff_id};{REGIONS[region]};{wage_text};{hours_text};{age};{educ};{int(female)}")
    out = Path(__file__).resolve().parent / "data" / "staff_de.csv"
    out.write_bytes(("\n".join(lines) + "\n").encode("windows-1252"))
    print(f"wrote {out} ({len(lines) - 1} rows)")


if __name__ == "__main__":
    main()
