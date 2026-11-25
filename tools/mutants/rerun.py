"""The shards of the present layout that hold the mutants of shards of an older, coarser one.

Usage: python3 -I tools/mutants/rerun.py "zeff-results 1/32" "zeff-kernels 2/12" ...
       python3 -I tools/mutants/rerun.py --run <id of a weekly run>

The weekly mutation job (.github/workflows/scheduled.yml) deals a package's mutants to its
shards in turn, so mutant i is in shard i mod n. When the layout is made finer by a whole
factor, from n shards to f times n, shard k of n is the shards k, k + n, ..., k + (f - 1) n of
the new layout and nothing else. A shard that was cut off at the job's limit can therefore be
run again alone, in pieces that fit, by naming those shards to the job's `shard` input.

That holds while the package's code outside its tests is what it was in the older run: a line
added or removed there moves every mutant after it to another shard. Then the whole package is
run instead.

With `--run`, the shards are those of that run whose jobs ended as cancelled, which is how a
job cut off at its limit ends; the GitHub command-line program `gh` is asked for them.

Prints the shards, one a line, and the command that starts them by hand.
"""

import json
import re
import subprocess
import sys

WORKFLOW = ".github/workflows/scheduled.yml"


def layout():
    """The number of shards of each package in the job's file."""
    found = {}
    for line in open(WORKFLOW, encoding="utf-8"):
        match = re.match(r"\s*- \{ package: (\S+), shard: \d+, of: (\d+),", line)
        if match:
            found[match.group(1)] = int(match.group(2))
    return found


def cut_off(run):
    """`package k/n` for each mutation job of the run that ended as cancelled."""
    out = subprocess.run(
        ["gh", "api", "--paginate", f"repos/{{owner}}/{{repo}}/actions/runs/{run}/jobs?per_page=100",
         "--jq", ".jobs[] | [.name, (.conclusion // \"\")] | @json"],
        capture_output=True, text=True, check=True,
    ).stdout
    shards = []
    for line in out.splitlines():
        name, conclusion = json.loads(line)
        match = re.match(r"weekly \(mutation testing, (\S+) (\d+) of (\d+)\)$", name)
        if match and conclusion == "cancelled":
            shards.append(f"{match.group(1)} {match.group(2)}/{match.group(3)}")
    return shards


def main():
    arguments = sys.argv[1:]
    if not arguments:
        sys.exit(__doc__)
    if arguments[0] == "--run":
        if len(arguments) != 2:
            sys.exit(__doc__)
        old = cut_off(arguments[1])
        if not old:
            sys.exit(f"no mutation job of run {arguments[1]} ended as cancelled")
    else:
        old = arguments
    now = layout()
    finer = []
    for shard in old:
        match = re.match(r"(\S+) (\d+)/(\d+)$", shard)
        if not match:
            sys.exit(f"`{shard}` is not `package k/n`")
        package, k, n = match.group(1), int(match.group(2)), int(match.group(3))
        if package not in now:
            sys.exit(f"`{package}` has no shards in {WORKFLOW}")
        if now[package] % n != 0 or k >= n:
            sys.exit(
                f"{shard}: the present layout has {now[package]} shards of {package}, "
                f"which is not a whole number of times {n}; run the whole package"
            )
        finer.extend((package, k + step * n, now[package]) for step in range(now[package] // n))
    finer = sorted(set(finer))
    for package, k, n in finer:
        print(f"{package} {k}/{n}")
    named = ",".join(f"{package} {k}/{n}" for package, k, n in finer)
    print(f"\n{len(old)} shards as {len(finer)}; to start them:\n")
    print(f'gh workflow run scheduled.yml --ref main -f job=weekly -f shard="{named}"')


if __name__ == "__main__":
    main()
