"""Line coverage of the kernel crates from a cargo-llvm-cov JSON report.

Usage: python3 -I tools/coverage/kernel_coverage.py coverage.json 80

Prints a table of line coverage per crate (the kernel crates first), then the kernel's total, and
exits 1 when that total is below the threshold given (the testing handbook asks for 80% or more
in the kernel). The table is Markdown, so it reads well in a job summary.
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
    for crate in sorted(lines, key=lambda c: (not c.startswith("kernel/"), c)):
        covered, total = lines[crate]
        pct = 100.0 * covered / total if total else 100.0
        print(f"| {crate} | {covered:,} | {total:,} | {pct:.1f}% |")
    kernel_covered = sum(v[0] for c, v in lines.items() if c.startswith("kernel/"))
    kernel_total = sum(v[1] for c, v in lines.items() if c.startswith("kernel/"))
    kernel_pct = 100.0 * kernel_covered / kernel_total if kernel_total else 100.0
    print()
    print(f"Kernel total: {kernel_pct:.1f}% of {kernel_total:,} lines (threshold {threshold:g}%).")
    if kernel_pct < threshold:
        print()
        print(f"The kernel is below {threshold:g}%: write the missing tests.")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
