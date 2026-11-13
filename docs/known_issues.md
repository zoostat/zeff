# Known issues

The limits and rough edges of the engine as released, kept current from the pre-release routine of
each release (testing handbook, section 2.4) and copied into the release notes. An entry names the
release that introduced it and, where one is planned, the release that removes it. A wrong result
is never listed here as accepted: it is fixed or the function is withdrawn.

## As of 0.6.0

- **`save` writes CSV and TSV only.** Parquet, Arrow, Stata, Excel and JSON are refused with
  Z0716, which names 0.11.0, the release of the reader and writer interface (SPSS 1.1); an
  opaque column is refused with the advice to drop it. What is written reads back bit for bit.
  (Since 0.6.0; by the build order.)
- **Under `fe:` a fit has no leverage.** `se: hc2` and `se: hc3` are refused (Z0410),
  `influence` is refused (Z0304, until 0.8.0) and W1006 is not judged, because an observation's
  leverage in the full design cannot be had from the demeaned columns. `fe:` takes variables and
  interactions `a:b` joined by `+`; `*`, a range or an expression is refused (Z0401), as is a
  fixed effect that is also a variable of the formula. `ivreg` with `fe:` arrives at 0.8.0 and
  two-way clustering at 1.1. (Since 0.6.0.)
- **The fixed effects are projected out by an iteration**, to a tolerance of 1e-8 on each
  column, so the estimates agree with fixest on the reference fits within 1e-7 rather than to
  the last digits of a closed form. A projection that has not converged in 10,000 passes stops
  the fit with an internal error rather than giving a result; no data met in testing reaches it.
  `predict` gives `NA` with W0207 for a level of an effect the fit did not see. (Since 0.6.0.)
- **A fixed-effects fit runs on one thread.** On the benchmark panel the two-way fit takes
  0.44 s at a million rows and 4.2 s at ten million on the development machine, against 0.23 s
  and 2.2 s for fixest on the same files, within the factor of two the release set itself;
  `--threads` arrives at 1.1. (Since 0.6.0.)
- **`event` needs units that are treated at different times, or never.** When every unit is
  treated at the same period and the fit has time effects, the leads and lags are the time
  effects themselves: they are dropped as collinear with a note, and a fit left with no
  regressor is refused (Z0802). A frame with no treated row is refused (Z0401) and a `ref:`
  that is not a relative period of the data is Z0507, listing the periods present. `did` and
  `pre_trends` arrive at 1.2. (Since 0.6.0; by design.)
- **A panel's time is an Int or a `Date`, and a `Date` counts days.** A gap is one missing
  integer or one missing calendar day, so monthly, quarterly or yearly observations dated as
  `Date`s are a day apart to no one and `lag`, `lead` and `diff` give `NA`; declare the panel on
  a whole-number period (`year(d) * 12 + month(d)`, say) until the `Period` type of a 1.x
  release. A unit is one variable; a composite unit comes with it. `lag`, `lead` and `diff` need the
  declaration (Z0602) and work inside `add` and `update` only. (Since 0.6.0; by design, section
  6.)
- **Dates and times carry no time zone and no duration.** A `DateTime` is a wall-clock instant;
  text with `Z` or an offset is converted to UTC and counted (W0702). `Date - Date` is days;
  every other arithmetic on a date is refused (Z0221) with the function to use (`add_days`,
  `add_months`, `days_between`, `months_between`). Years run from 1 to 9999, a result outside
  them is `NA` (W0203); `add_months` clamps to the month's last day, so it does not undo itself
  there; `format` prints weekday and month names in English. (Since 0.6.0; by design.)
- **`today()` and `now()` are recorded only by their note.** Each use prints W0703 with the
  value it gave; the run record that keeps it, and `zeff run --replay`, arrive at 0.11.0. Both
  are refused inside a function (Z0110). (Since 0.6.0.)
- **A frame's columns are not known to the checker after `to_wide`, after an import of a file
  that does not exist yet, or after a project function whose returns differ** in their columns;
  such a frame is marked "checked at run time", its variables are checked when the statement
  runs, and after a run the columns the step produced serve the next check as warnings marked
  "from last run". A frame given to `merge` without `suffixes:` while it shares a column is
  checked at run time too, where the merge is refused. (Since 0.6.0, step 21.)
- **`merge` refuses a many-to-many relation**, a key that repeats on both sides (Z0603; `relation:`
  names one of the three others to have it checked), needs `suffixes:` when the frames share a
  column that is not a key (Z0604), and never matches a key that is `NA`. `append` joins Int with
  Float and refuses any other pair of types for one column (Z0609). `aggregate` takes definitions
  that give one value per group and drops a panel declaration with a note, as `to_wide` does.
  (Since 0.6.0; by design, section 6.)
- **`winsorize` warns twice and names `value`.** It is written in Zeff over two calls of
  `quantile`, each of which counts the missing rows of the body's own column, so a column with
  `NA` prints "rows with NA in value were skipped" twice, where the script's variable is meant;
  a share outside 0 to 0.5 is refused by the body's `assert` (Z0001). Both wait for a coded
  warning from a body (open decision #70, 0.7.0). (Since 0.6.0.)
- **Six functions planned in Zeff are written in Rust**, each with the reason in its catalogue
  row: `row_mean` and `row_sum` (the kernel's compensated row reduction, which `+` over columns
  does not reproduce), `diff` (its `lag` reads the panel of the frame around the call, which a
  body does not pass on), `n_missing` (a list holding failed values reaches it), `summary` (its
  table's formatting and combined warning need `format` and `join` of `text`, its title the
  frame's name, and it takes `weights:`) and `duplicates` (its pieces came later in 0.6.0);
  `drop_missing` stays too until a body can resolve a variable range and raise the errors
  its page declares (open decisions #90 and #89, 0.8.0), and `vif` moves at step 30 with
  `design()`. `fill_missing` and `quantile_group` returned to Zeff at steps 4.1 and 4.4 of
  0.7.0. Nothing a script sees differs, and section 8.2's totals follow the catalogue. (Since
  0.5.0; the list as of step 5b of 0.7.0.)
- **Eight conditions carry codes numbered at this release**, the next free numbers of their
  families, each logged in `docs/spec_changes.md` and listed in section 11 of the Language
  Reference: Z0507 (`event`'s `ref:`), Z0601 and Z0602 (the panel declaration and the panel
  functions), Z0609, Z0610 and Z0611 (`append`, `to_long`, `to_wide`), W0210 (`bin`) and W0701
  (a data root's file changed since the last run). W0701 compares hashes kept in the project's
  cache, which stand in for the run record until 0.11.0. (Since 0.6.0.)

## As of 0.5.0

- **A function written in a script does not see, by its short name, a module function the
  script brought in with `use`.** After `use functions::checks`, a call of `positive(x)` inside a
  `fn` of the script is refused as an unknown function (Z0103), though the same call at the top
  level of the script is accepted and section 5.3 of the Language Reference lets a function body
  see the project's functions. Written with its module, `checks::positive(x)`, the call is
  accepted. (Since 0.5.0; found on 13 November 2026 at step 32a of 0.7.0; put right at step 38b
  of 0.7.0.)
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
  body's column (`value`, from `standardize`). The lists are narrowed at step 32d of 0.7.0,
  by the rule of open decision #139. (Since 0.5.0; the warning of `skewness` and `kurtosis` for
  too few values carries W0205 from step 4.3 of 0.7.0.)
- **The mean of values that cancel is within a rounding of the largest input, not of the
  result.** The kernel's mean corrects the compensated quotient by the mean of the deviations,
  whose own rounding is the unit in the last place of the largest value: on a row holding 1e12,
  −1e12 and 1e7 the mean is 8e-12 relative from the exact 3,333,333.33…, where the plain quotient
  and R's long-double sum are exact. Every `mean` of the engine shares it; it is harmless on data
  that does not cancel to a small residual. Reviewed at 0.6.0 with the demeaning kernel (step
  17), as promised: `mean` keeps its corrected form and this entry stands as a documented
  property; the group means of a fixed-effects fit do not use it (a plain compensated quotient),
  so values that cancel within a group lose nothing there. (Since 0.1.0; found at 0.5.0, step
  17; reviewed at 0.6.0.)
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
- **Mutation testing of a slow package against the whole workspace's tests needs a wide budget.**
  cargo-mutants takes each mutant's budget from a baseline measured on the mutated package alone,
  so the floor is the whole budget under the workspace's tests; the weekly job's floor is four
  minutes since 0.5.0 (two before, which timed out 689 mutants on the release's first run), its 34
  shards run 74 to 201 minutes against a limit of 340 at 0.6.0 (39 to 284 at 0.5.0), and a local
  rerun passes a floor of ten minutes. The twelve recorded hangs (eight in the numeric kernel, two
  in the pivoted QR, two in the demeaning kernel's singleton loop) are caught by the limit alone,
  and six mutants that make a fit slow without hanging it (five send the loop of the fixed effects'
  estimates to its cap, one leaves the demeaning kernel's rows unordered) time out on a slow runner
  and are caught when given their time. Since 0.6.0 the job draws its property-test cases from a
  fixed seed: a property test that failed by chance (about once in 350 runs, a fault of the test's
  own data, corrected) had left its failing case in a shard's working copy, where it marked later
  mutants as caught; the clean rerun matched the records. (Since 0.3.0; understood at 0.4.0, the
  budget repaired at 0.5.0, the seed fixed at 0.6.0.)
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
- **The browser engine bundle is 1.65 MB compressed** (brotli; 1.46 MB at 0.5.0, 1.34 MB at
  0.4.0, 1.15 MB at 0.3.0, 1.10 MB at 0.2.0, 355 kB at 0.1.0), because the three datasets are
  embedded and the packages, the interpreter and the functions written in Zeff were added, `time`
  and `text` loaded at start among them; it stays under the 5 MB warning level of risk R4. The
  cold start is 107 ms against 1,000 ms (67 ms at 0.5.0): the module loads in 29 ms and the
  engine's first call, its start on a one-line script, takes 75 ms. The whole conformance report,
  printed for information, takes 1,904 ms with 194 cases. (Since 0.2.0; the figures are those of
  0.6.0.)
- **Each project pins the exact engine version** (`zeff = "0.2.0"`). An engine of another
  version refuses the project with exit code 4; there is no launcher yet to pick the pinned engine
  (0.9.0). (Since 0.1.0.)
- **The function catalogue's status column** is updated at each release close, not per function,
  so between releases it lags the registry; `zeff help <package>` shows what is shipped. (Since
  0.1.0.)

## Resolved at 0.7.0

- The examples of a module function's page are run: `zeff test` runs every `/// >` example of
  every function of `functions/` in the project and compares what it prints with the lines under
  it (listed at 0.5.0 as parsed and syntax-checked but not run). Since steps 32a and 32b.

## Resolved at 0.6.0

- A `Var` parameter of a project function is checked at `zeff check`, against the frame the
  function uses it with, when that frame's columns are known (listed at 0.5.0 as checked only
  when the statement runs). Since step 20, the schema pass.

## Resolved at 0.4.0

- `label`, `category` and `recode` attach labels, value labels and categories from a script
  (listed at 0.2.0 as missing). A CSV still carries none, as its import note says; a labelled
  variable prints its labels, and its stored number is what computes.

## Resolved at 0.2.0

- `by:` on every statistic, on `summary` and on `describe` (listed at 0.1.0 as missing).
- `thousands:`, `encoding:`, `quote:` and `decimal:` on `import` (listed at 0.1.0 as missing).
