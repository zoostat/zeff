"""Compares the surviving mutants of a mutation run with the recorded ones.

Usage: python3 -I tools/mutants/compare.py [--part] <recorded file> <list of survivors>...

With `--part` the lists are one shard of a run and not the whole of it, so nothing is said of
the recorded survivors the lists do not hold.

A list of survivors is what cargo-mutants writes as `missed.txt` or `timeout.txt`: one mutant a
line, `path:line:column: what was replaced in routine`. The recorded file (`tools/mutants/
recorded.txt`) holds one accepted survivor a line, without its line and column, which move with
every edit: the path, the mutation as the tool words it, how many times it is recorded in that
routine, and the key of its reason. A survivor is new when its path and mutation are not
recorded, or are met more often than recorded. The new ones are printed, each a test to write
or a reason to record (testing handbook, section 1); recorded ones that no list holds any more
are counted, since a full run that no longer finds them means their lines can be struck.

Exits 0 always: a surviving mutant is a note to act on, not a failure of the job.
"""

import collections
import re
import sys


def key_of(line):
    """`path` and `mutation in routine` of a line of cargo-mutants, without line and column."""
    match = re.match(r"(\S+?\.rs):\d+:\d+: (.*)$", line.strip())
    return (match.group(1), match.group(2)) if match else None


def recorded(path):
    """The recorded survivors: (path, mutation) -> (count, reason key)."""
    found = {}
    for line in open(path, encoding="utf-8"):
        line = line.rstrip("\n")
        if not line or line.startswith("#"):
            continue
        parts = line.split("\t")
        if len(parts) != 4:
            sys.exit(f"{path}: a line with {len(parts)} fields, not four: {line!r}")
        file, mutation, count, reason = parts
        found[(file, mutation)] = (int(count), reason)
    return found


def main():
    arguments = [a for a in sys.argv[1:] if a != "--part"]
    part = "--part" in sys.argv[1:]
    if len(arguments) < 2:
        sys.exit(__doc__)
    known = recorded(arguments[0])
    met = collections.Counter()
    for path in arguments[1:]:
        for line in open(path, encoding="utf-8", errors="replace"):
            key = key_of(line)
            if key:
                met[key] += 1
    new = []
    for key, count in sorted(met.items()):
        allowed = known.get(key, (0, ""))[0]
        if count > allowed:
            new.append((key, count - allowed))
    print(f"survivors read: {sum(met.values())}; recorded: {sum(min(c, known.get(k, (0, ''))[0]) for k, c in met.items())}; new: {sum(n for _, n in new)}")
    for (file, mutation), extra in new:
        times = "" if extra == 1 else f" ({extra} times)"
        print(f"  new: {file}: {mutation}{times}")
    gone = sum(1 for key in known if key not in met)
    if gone and not part:
        print(f"recorded and not met in these lists: {gone} (a full run that no longer finds them means their lines can be struck)")


if __name__ == "__main__":
    main()
