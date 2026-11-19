# Regressions

The reproducer of every fixed bug, kept so that the bug cannot come back unseen (Language
Reference, section 11.1, "Every fix becomes a test"; testing handbook, section 3).

Each file is a script of a few statements on a few rows written into it, or a text the
formatter once could not format. Its first lines say what was wrong, since which release it is
corrected, and what the corrected engine gives. The scripts are compiled into the conformance
suite (`REGRESSIONS` in `src/lib.rs`): what each prints, says and writes is a case of
`expected.txt`, compared on the four targets by the identity job and run by `zeff selftest` on
every installation. `tests/regressions.rs` stores the same as a text, so that a fault that
came back is read there and not only seen as a digest that changed.

To add one, when a bug is fixed:

1. Shrink the report with `zeff report-bug` (with `--synthetic` where the rows matter), or
   write the smallest script that showed the fault. Write the rows into the script with
   `frame(...)`, so that it needs no file.
2. Save it here as `<what_was_wrong>.zeff`, with the comment at its head.
3. Add its entry to `REGRESSIONS`: `Held::Run` for a script that is run, `Held::Format` for a
   text the formatter is given.
4. Regenerate `expected.txt` (`cargo run -p zeff-conformance > crates/zeff-conformance/expected.txt`)
   and the stored text of `tests/regressions.rs`, and read both before committing them.

The first three are the faults the pre-release routine of 0.7.0 found and corrected before its
tag: values above 1e150 in the tests that square them, a test's result exported as LaTeX, and
a chain that starts from the lone `.` of a formula.

A script that the check refuses is held like one that runs: nothing is printed, and what the
check says is the case. Two of 0.8.0 are such: a field that a result's record does not have,
and an index into the table of a test's intervals, each once an internal error.

One of 0.8.0 is a wrong number and no error at all: a truncated distribution beside its bound,
where the difference of two nearly equal tails kept five digits.

One of 0.8.0 came from the nightly fuzz target and not from a review: a count distribution
truncated beside a whole number, where the probability of the interval and the quantile read
the bound differently and the quantile ended in an internal error.

One of 0.8.0 is a layout and no number: a test's result as Typst, whose notes widened the last
column of its table and whose hypotheses did not stand under one another.

