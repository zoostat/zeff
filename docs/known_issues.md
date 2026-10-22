# Known issues

The limits and rough edges of the engine as released, kept current from the pre-release routine of
each release (testing handbook, section 2.4) and copied into the release notes. An entry names the
release that introduced it and, where one is planned, the release that removes it. A wrong result
is never listed here as accepted: it is fixed or the function is withdrawn.

## As of 0.3.0

- **Distribution functions with a parameter above 100,000 are accurate to about 1e-10, not the
  class's 1e-12.** The incomplete beta and gamma functions use continued fractions whose stopping
  rule cannot see below a unit in the last place; at such parameters the fraction converges slowly
  and leaves an error of about 2e-11 in the tails (a t distribution with a million degrees of
  freedom meets it). R's own values there carry errors of the same order. The asymptotic
  expansions of Temme close it in a later step. (Since 0.3.0, step 2.)

- **Distribution functions return `NA` where their series cannot converge.** The gamma family,
  and the Poisson and chi-square distributions through it, need about 8√a terms near the centre
  with shape a (or λ, or df/2); past two million terms, from a shape of about 1e11, the answer is
  `NA` rather than a wrong partial sum, and a quantile or a draw that would need it is `NA` too
  (a discrete draw beyond the Ints is `NA` as well). The incomplete beta function, behind the
  beta, t, F, binomial and negative binomial distributions, still converges at 1e14 but loses
  digits slowly near its centre: about 3e-10 at parameters of 1e12 and 6e-10 at 1e13. Asymptotic
  expansions close both in a later step. (Since 0.3.0.)
- **A quantile within 1e-13 of one does not round-trip to six digits.** For a beta distribution
  with a small second shape the quantile of a middling probability lies within 1e-13 of one, where
  the doubles are so far apart that even the nearest one gives back a probability a few parts in a
  million off: `d.cdf(d.quantile(0.92))` prints 0.92094 for such a `d`. The quantile itself is
  right to a few units in the last place (the inverse stops when its bracket is within 4ε of x),
  and R's `qbeta` has the same limit. (Since 0.3.0; a property of doubles, not of the engine.)
- **`correlation` with `method: kendall` costs time quadratic in the rows.** 1,200 rows take a
  moment and 18,412 a few seconds; Knight's O(n log n) algorithm replaces the counting loop later.
  (Since 0.3.0.)
- **Draws are keyed by position.** As in R and Stata, inserting a statement that draws changes the
  draws after it, and inserting a drawing call into a statement changes the later calls of that
  statement. A `seed:` given to one call is a stream of its own, so two calls with the same `seed:`
  in one statement repeat each other. (Since 0.3.0; by design, section 8.3.)
- **The reference for `dist.truncated` is written from base R's functions**, not the `truncdist`
  package, which the rules do not allow to be installed; a discrete truncation keeps the mass at
  `lower:` (the package drops it). (Since 0.3.0.)
- **The fuzz target for distribution parameters runs slowly** (some thousands of inputs in ten
  minutes), since huge parameters make the kernel work to its limit; it is a smoke test of
  termination and of internal errors, not a search of the whole parameter space. (Since 0.3.0.)

## As of 0.2.0

- **Printed tables count width in characters.** An East Asian wide character takes two cells in a
  terminal, so a frame or table holding such text misaligns by one cell per character. Values are
  never cut. A Unicode width table is a dependency decision for a later release. (Since 0.1.0.)
- **`summary` and the grouped tables print fixed three-decimal statistics.** Values below 0.0005 in
  magnitude print as `0.000` and very large values print every digit before the point (section
  10.2's form). A statistic on its own (`mean`, `sd`, `quantile`, …) prints six significant digits
  and is the way to read such columns. `quantile` with a list of probabilities prints one value per
  line, not a table. (Since 0.1.0.)
- **Dates are read from CSV only in ISO form or with a declared format.** A column that looks like
  dates in another form is read as text, with a note proposing the `types:` entry. (Since 0.1.0.)
- **Labels, value labels and categories come only from the shipped datasets.** A CSV carries none
  (the import note says so), and `label`, `category` and `recode` arrive at later releases, so a
  script cannot yet attach them. A labelled variable prints its labels; its stored number is what
  computes and what `print` of the bare column shows. (Since 0.2.0.)
- **A line beginning with `-` is refused (Z0305).** The grammar of section 3.4 ends a statement
  at the line break, so `-7` on its own line has nothing on its left; write `print(-7)` or
  `let x = -7`, and continue a long expression with the operator at the end of the line. (Since
  0.1.0; by design.)
- **`import` reads delimited text only** (UTF-8 or a declared encoding, with every option of
  section 6.1 for delimited text). The other formats arrive at the releases section 6.1 names:
  fixed width, Excel, OpenDocument, Parquet and Arrow, Stata and JSON at 0.11.0; SPSS, SAS, R and
  HTML tables at 1.1; local databases after 1.1. The `zeff:` data root holds the two shipped
  datasets as CSV and Arrow IPC; their Parquet copies arrive with the reader and writer of 0.11.0.
  (Since 0.1.0; the formats narrowed at 0.2.0; the releases corrected at 0.3.0.)
- **The console's `:undo` re-runs the whole session** without its last statement, so a long
  session with large imports takes a moment to undo; the result cache that makes it immediate is
  0.11.0. (Since 0.2.0.)
- **The browser engine bundle is 1.15 MB compressed** (brotli; 1.10 MB at 0.2.0, 355 kB at
  0.1.0), because the two datasets are embedded and the special functions and distributions were
  added; it stays well under the 5 MB warning level of risk R4, and the cold start is 140 ms
  against 1,000 ms. (Since 0.2.0; the figures are those of 0.3.0.)
- **Each project pins the exact engine version** (`zeff = "0.2.0"`). An engine of another
  version refuses the project with exit code 4; there is no launcher yet to pick the pinned engine
  (0.9.0). (Since 0.1.0.)
- **The function catalogue's status column** is updated at each release close, not per function,
  so between releases it lags the registry; `zeff help <package>` shows what is shipped. (Since
  0.1.0.)

## Resolved at 0.2.0

- `by:` on every statistic, on `summary` and on `describe` (listed at 0.1.0 as missing).
- `thousands:`, `encoding:`, `quote:` and `decimal:` on `import` (listed at 0.1.0 as missing).
