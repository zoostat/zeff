# Known issues

The limits and rough edges of the engine as released, kept current from the pre-release routine of
each release (testing handbook, section 2.4) and copied into the release notes. An entry names the
release that introduced it and, where one is planned, the release that removes it. A wrong result
is never listed here as accepted: it is fixed or the function is withdrawn.

## As of 0.4.0

- **A regression refuses an outcome it cannot compute with.** An outcome above 1e150 in size has
  a square the arithmetic cannot represent, and an outcome so small against its regressors that
  an estimate falls below 2.2e-308 (the smallest normal number) keeps a few digits of that
  estimate only; both are refused with Z0803 and the advice to rescale the outcome by a power of
  ten. A regressor above 1e150 or below 1e-150 in size is dropped with a note instead. Weights
  more than 1e150 apart are refused (Z0224), as are frequency weights whose total exceeds 2^53,
  the largest count held exactly. (Since 0.4.0; limits of the arithmetic, found by the formula
  fuzz target.)
- **`predict` gives no standard error of prediction**, only the fitted values on new data; the
  interval of a prediction arrives with the panel models. (Since 0.4.0.)
- **A test's result has no interval**: `std_errors()` and `ci()` on it give `NA`, and
  `estimates()` one row; the tests of `stats` bring their intervals at 0.7.0. (Since 0.4.0.)
- **`test` reads linear restrictions only.** Constants, coefficients, their sums and
  differences, and products and quotients by a constant, on both sides of one `==`; a product or
  quotient of two coefficients, a power or a function is refused (Z0415), and `derive` is the way
  to such a quantity. `derive` differentiates `+`, `-`, `*`, `/`, `^` and eight functions of one
  argument (`log`, `exp`, `sqrt`, `log1p`, `expm1`, `log10`, `log2`, `abs`); `log` with a base
  and any other function are refused (Z0417). (Since 0.4.0; by design, section 7.2.)
- **`compare` needs two fits on the same rows, one nested in the other** by its term names;
  fits on frames with different rows dropped for `NA` are refused (Z0416) with the advice to drop
  those rows first. (Since 0.4.0; by design.)
- **`report` labels terms by identifier.** `labels: {educ: "Years of schooling"}` renames a term
  whose name is an identifier; a term such as `industry=Retail` or `(intercept)` keeps its name,
  since a text key in a record is a syntax question for a later release. (Since 0.4.0.)
- **`influence` gives no Cook's distance or studentized residual with one residual degree of
  freedom**, nor for a row of leverage one; under frequency weights the leverage printed is the
  per-observation one, h/w, as R computes on the replicated rows. (Since 0.4.0; by design.)
- **Mutation testing of a slow package against the whole workspace's tests times out.** The
  weekly job's five-fold limit on a baseline measured on the mutated package alone falls short
  when another package's tests run under load; its shards therefore run 69 to 224 minutes, and a
  local rerun passes `--timeout 600` or tests only the packages holding the tests. The eight
  recorded hangs of the numeric kernel are caught by the limit alone. (Since 0.3.0; understood at
  0.4.0, step 22a.)
- **An exactly perfect fit prints `Inf` t values.** When every residual is exactly zero (a
  constant outcome on an intercept, or data on an exact line in dyadic numbers) the standard
  errors are zero and the t values print as `Inf` with p < 0.001, under the note that the fit is
  essentially perfect; an outcome whose squared mean underflows (values below about 1e-154) gets
  the `Inf` without the note, since the rule's threshold is then zero. R prints the same
  quantities. (Since 0.4.0.)
- **The Edgeworth series of Spearman's exact tail is pinned only by the stats package's
  comparison with R**, not by the kernel's own tests, so a mutation run against the kernel alone
  reports its lines as missed. (Since 0.4.0, step 22d; a gap in the kernel's tests to close.)

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
- **The browser engine bundle is 1.34 MB compressed** (brotli; 1.15 MB at 0.3.0, 1.10 MB at
  0.2.0, 355 kB at 0.1.0), because the two datasets are embedded and the regression and results
  packages were added; it stays under the 5 MB warning level of risk R4. The cold start is 639 ms
  against 1,000 ms (140 ms at 0.3.0): the module loads in 17 ms and the first call takes 620 ms,
  a rise to be understood before the next release adds to it. (Since 0.2.0; the figures are those
  of 0.4.0.)
- **Each project pins the exact engine version** (`zeff = "0.2.0"`). An engine of another
  version refuses the project with exit code 4; there is no launcher yet to pick the pinned engine
  (0.9.0). (Since 0.1.0.)
- **The function catalogue's status column** is updated at each release close, not per function,
  so between releases it lags the registry; `zeff help <package>` shows what is shipped. (Since
  0.1.0.)

## Resolved at 0.4.0

- `label`, `category` and `recode` attach labels, value labels and categories from a script
  (listed at 0.2.0 as missing). A CSV still carries none, as its import note says; a labelled
  variable prints its labels, and its stored number is what computes.

## Resolved at 0.2.0

- `by:` on every statistic, on `summary` and on `describe` (listed at 0.1.0 as missing).
- `thousands:`, `encoding:`, `quote:` and `decimal:` on `import` (listed at 0.1.0 as missing).
