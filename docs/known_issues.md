# Known issues

The limits and rough edges of the engine as released, kept current from the pre-release routine of
each release (testing handbook, section 2.4) and copied into the release notes. An entry names the
release that introduced it and, where one is planned, the release that removes it. A wrong result
is never listed here as accepted: it is fixed or the function is withdrawn.

## As of 0.5.0

- **The constructs the grammar has but this release does not run** are a `model { }` block
  (1.3), a `dag { }` block (0.10.0), a label written with `as` (1.3) and `use` of a third-party
  package (1.11); each is refused with Z0304, whose explained page lists them. Everything else of
  sections 5.1 to 5.5 runs: `if`, `case`, `for`, `[for ...]`, `break`, `continue`, `fn`, `use
  functions::`, spreads, `attempt` and `iterate`. (Since 0.5.0; by the build order.)
- **A project function is not applied row by row to a column.** A column given to a scalar
  parameter of a function written in a script or a module is refused (Z0218), where a built-in
  written in Zeff applies to the column in one pass; a `Column` parameter or `[for ...]` is the
  way. A function may not call itself (Z0108) and sees its parameters and other functions only.
  (Since 0.5.0; by design, section 5.3.)
- **A `Var` parameter is checked when the statement runs**, against the frame it meets, not at
  `zeff check`, until the schema pass of 0.6.0; the examples of a module function's page are
  parsed and syntax-checked but not run until `zeff test` at 0.7.0. (Since 0.5.0.)
- **`case` over rows evaluates every arm on the whole frame**, as R's `ifelse` does, so a warning
  such as a division by zero may count rows the arm does not supply; a Category column is refused
  as a value of `case` (Z0214; `to(x, Str)` gives its labels). (Since 0.5.0; a Date or DateTime
  column is accepted from 0.6.0.)
- **A failed value reaches six functions**: `is`, `type_of`, `print`, `report`, `len` and
  `n_missing`; anywhere else it is Z0220 reporting the failure it holds, so a list of fits made
  under `attempt` is read in a loop that steps over the failed ones. (Since 0.5.0; by design,
  section 3.7.)
- **The pages of built-ins written in Zeff list more warnings than they can raise.** Their
  "Warnings and notes" is the union of the codes declared by the functions a body calls
  (`frame`'s and the reducers' for `center`, `skewness` and their kin), a message raised by an
  inner reducer names that reducer (`median` needs a Bool, Int or Float, from `mad`) and the
  body's column (`value`, from `standardize`), and the warning of `skewness` and `kurtosis` for
  too few values carries no code in `--format json` (open decision #70). The lists are narrowed
  when `zeff test` runs the pages at 0.7.0. (Since 0.5.0.)
- **Seven functions planned in Zeff are written in Rust**: `fill_missing`, `n_missing`,
  `row_mean`, `row_sum`, `drop_missing`, `vif` and `summary`, each for a piece the language lacks
  (open decisions #68 to #72); nothing a script sees differs, and section 8.2's totals follow
  the catalogue. (Since 0.5.0.)
- **The mean of values that cancel is within a rounding of the largest input, not of the
  result.** The kernel's mean corrects the compensated quotient by the mean of the deviations,
  whose own rounding is the unit in the last place of the largest value: on a row holding 1e12,
  −1e12 and 1e7 the mean is 8e-12 relative from the exact 3,333,333.33…, where the plain quotient
  and R's long-double sum are exact. Every `mean` of the engine shares it; it is harmless on data
  that does not cancel to a small residual, and the correction is reviewed at the next kernel
  step (0.6.0). (Since 0.1.0; found at 0.5.0, step 17.)
- **`vif` gives one factor per column of the design**, so each dummy column of a categorical
  regressor has a row of its own and the generalised factor of the whole term is not given; a
  term dropped as collinear has no row and is named under the table. (Since 0.5.0.)
- **`equal` compares values, not columns or frames** (Z0401 for a column); `sorted` takes lists
  of Bools, Ints, Floats or Strs only; `seq` refuses a count above ten million values and a step
  of zero or of the wrong sign (Z0418), where R counts down by itself; `drop_missing` names its
  variables, the whole frame being `[first..last]`. (Since 0.5.0; by design.)
- **`iterate` prints its number of passes as a note** under the result, since no run record keeps
  it before 0.11.0. (Since 0.5.0.)

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
- **Mutation testing of a slow package against the whole workspace's tests needs a wide
  budget.** cargo-mutants takes each mutant's budget from a baseline measured on the mutated
  package alone, so the floor is the whole budget under the workspace's tests; the weekly job's
  floor is four minutes since 0.5.0 (two before, which timed out 689 mutants on the release's
  first run), its 34 shards run 39 to 284 minutes against a limit of 340, `zeff-numeric` 3 of 16
  (Spearman's exact tail) the longest, and a local rerun passes a floor of ten minutes. The ten
  recorded hangs (eight in the numeric kernel, two in the pivoted QR) are caught by the limit
  alone. (Since 0.3.0; understood at 0.4.0, the budget repaired at 0.5.0.)
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
- **The browser engine bundle is 1.46 MB compressed** (brotli; 1.34 MB at 0.4.0, 1.15 MB at
  0.3.0, 1.10 MB at 0.2.0, 355 kB at 0.1.0), because the two datasets are embedded and the
  packages, the interpreter and the functions written in Zeff were added; it stays under the 5 MB
  warning level of risk R4. The cold start is 67 ms against 1,000 ms: the module loads in 19 ms
  and the engine's first call, its start on a one-line script, takes 48 ms. The 620 ms first call
  reported at 0.4.0 timed the whole conformance report, which the measurement now prints for
  information (1,328 ms with 148 cases and five samples). (Since 0.2.0; the figures are those of
  0.5.0.)
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
