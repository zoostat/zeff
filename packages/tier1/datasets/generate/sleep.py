"""The public `sleep` dataset (Engine and Project Specification, section 8.5): Student's data of
1908 on the extra hours of sleep gained by ten patients under two soporific drugs, from Cushny
and Peebles (1905), as R ships them in `datasets::sleep`. The values are in the public domain
(Student 1908; the author died in 1937). Run with
`python3 -I packages/tier1/datasets/generate/sleep.py`; it rewrites
`packages/tier1/datasets/data/sleep.csv` from the values written here, so the derivation is the
file itself.

Source: Student (1908), The probable error of a mean, Biometrika 6, 1-25, table on page 20, the
data of Cushny, A. R. and Peebles, A. R. (1905), The action of optical isomers II: hyoscines,
Journal of Physiology 32, 501-510. Retrieved from the published paper; no licence is needed for a
work of 1908.
"""

import csv
import os

GROUP_1 = [0.7, -1.6, -0.2, -1.2, -0.1, 3.4, 3.7, 0.8, 0.0, 2.0]
GROUP_2 = [1.9, 0.8, 1.1, 0.1, -0.1, 4.4, 5.5, 1.6, 4.6, 3.4]


def main():
    here = os.path.dirname(os.path.abspath(__file__))
    path = os.path.join(here, "..", "data", "sleep.csv")
    with open(path, "w", encoding="utf-8", newline="\n") as f:
        writer = csv.writer(f, lineterminator="\n")
        writer.writerow(["extra", "group", "id"])
        for group, values in ((1, GROUP_1), (2, GROUP_2)):
            for patient, extra in enumerate(values, start=1):
                writer.writerow([repr(extra), group, patient])
    print("wrote", path)


if __name__ == "__main__":
    main()
