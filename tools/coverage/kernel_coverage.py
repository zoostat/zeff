"""Line coverage of the kernel crates and the estimators from a cargo-llvm-cov JSON report.

Usage: python3 -I tools/coverage/kernel_coverage.py coverage.json 80

Prints a table of line coverage per crate (the kernel crates first, then the estimators), then
the kernel's total and the estimators' total (the `results` and `regress` packages: the design,
the estimator and variance layers and the regressions), and exits 1 when either total is below
the threshold given (the testing handbook asks for 80% or more in the kernel and the estimators).
The table is Markdown, so it reads well in a job summary.
"""

import json
import sys
from collections import defaultdict


def crate_of(path):
    """The crate a source file belongs to: `kernel/zeff-numeric`, `crates/zeff-ir`, ..."""
    parts = path.replace("\\", "/").split("/")
    for i, part in enumerate(parts):
        if part in ("kernel", "crates", "tools") and i + 1 < len(parts):
            return f"{part}/{parts[i + 1]}"
        if part == "packages" and i + 2 < len(parts):
            return f"{part}/{parts[i + 1]}/{parts[i + 2]}"
    return "other"


ESTIMATORS = ("packages/tier1/results", "packages/tier2/regress")


def total(lines, chosen):
    """The lines covered and counted over the crates `chosen` accepts, and the percentage."""
    covered = sum(v[0] for c, v in lines.items() if chosen(c))
    count = sum(v[1] for c, v in lines.items() if chosen(c))
    return covered, count, (100.0 * covered / count if count else 100.0)


def main(argv):
    if len(argv) != 3:
        print(__doc__)
        return 2
    with open(argv[1], encoding="utf-8") as f:
        report = json.load(f)
    threshold = float(argv[2])
    lines = defaultdict(lambda: [0, 0])
    for data in report["data"]:
        for entry in data["files"]:
            crate = crate_of(entry["filename"])
            summary = entry["summary"]["lines"]
            lines[crate][0] += summary["covered"]
            lines[crate][1] += summary["count"]
    print("## Weekly: line coverage by crate")
    print()
    print("| Crate | Lines covered | Lines | Coverage |")
    print("| --- | ---: | ---: | ---: |")
    order = lambda c: (not c.startswith("kernel/"), c not in ESTIMATORS, c)
    for crate in sorted(lines, key=order):
        covered, count = lines[crate]
        pct = 100.0 * covered / count if count else 100.0
        print(f"| {crate} | {covered:,} | {count:,} | {pct:.1f}% |")
    kernel_covered, kernel_count, kernel_pct = total(lines, lambda c: c.startswith("kernel/"))
    estimators_covered, estimators_count, estimators_pct = total(lines, lambda c: c in ESTIMATORS)
    print()
    print(f"Kernel total: {kernel_pct:.1f}% of {kernel_count:,} lines (threshold {threshold:g}%).")
    print()
    print(
        f"Estimators total (results and regress): {estimators_pct:.1f}% of {estimators_count:,} lines"
        f" (threshold {threshold:g}%)."
    )
    short = []
    if kernel_pct < threshold:
        short.append("the kernel")
    if estimators_pct < threshold:
        short.append("the estimators")
    if short:
        print()
        print(f"{' and '.join(short).capitalize()} below {threshold:g}%: write the missing tests.")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
