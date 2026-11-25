# Known issues

The limits and rough edges of the engine as released, kept current from the pre-release routine of
each release (testing handbook, section 2.4) and copied into the release notes. An entry names the
release that introduced it and, where one is planned, the release that removes it. A wrong result
is never listed here as accepted: it is fixed or the function is withdrawn.

## As of 0.9.0

- **`glm` fits a binary outcome, and nothing else yet.** `family: binomial` with the logit or the
  probit link is the whole of it at this release: the other families of the catalogue (Poisson,
  negative binomial, gamma) and fixed effects absorbed by `fe:` are of a later release, and
  `fe:` on `glm` is refused until then (Z0304); a category among the regressors does the same
  for a few groups. Analytic weights are refused, since the variance of a 0 or 1 is fixed by its
  mean. The intervals of the coefficients are Wald's, estimate plus and minus a normal quantile
  of its standard error, as R's `confint.default` and Stata give; R's `confint` of a `glm` fit
  gives profile-likelihood intervals, which differ a little in small samples. Odds ratios,
  `summary(exponentiate: true)`, are for the logit link, where a coefficient is the logarithm
  of one. (Since 0.9.0.)
- **`margins` gives effects and predictions, each with its own interval, and does not compare
  them.** The rows of one table, the effect at several held values or the predictions of
  several groups, are separate statements; a test of one against another, a contrast or a
  second difference, is not given. A prediction from a fit with absorbed fixed effects needs
  the level of each effect and is refused (Z0401); the effects of its variables are given.
  Standard errors are by the delta method from the fit's own variance. (Since 0.9.0.)
- **`curve` draws the ROC curve alone.** `type: roc` is its one type; the precision and recall
  curve and the calibration of a binary fit are proposed for 0.10.0 (open decision #218).
  (Since 0.9.0.)
- **61 help pages are not yet of the quality of the rest.** The pages of `core`, `text`, `time`,
  `data`, `datasets`, `dist` and `stats`, and of the functions new at 0.8.0 and 0.9.0, carry
  what release 0.8.0 asks of a page: the commonest forms first, missing values under their own
  heading, a caption on every example, a common mistake with its real message, and the
  equivalents in R and Stata. The pages of `results` (23), `regress` (7) and `plot` (31) follow
  at 0.10.0. The Stata column of the revised pages was written from Stata's documented
  behaviour and has not been run in Stata; where a Stata command could not be named with
  certainty the page says "none of its own" and no more. (Since 0.8.0, when 145 pages were
  listed; open decision #103.)
- **NIST's Filip problem is not solved: the tenth power is dropped as collinear.** The problem
  fits a polynomial of degree ten in one variable, the hardest of NIST's linear-regression
  problems. The collinearity rule of the Language Reference (section 7.2: a pivot below 1e-7
  of its column's norm) drops `x^10`, with a note and the warning of near-singular regressors,
  so the certified values are not reproduced (a log relative error of 0). R's `lm` drops the
  same term at its default tolerance, which is the same number, and reaches 7 digits when the
  tolerance is lowered to 1e-10; Zeff has no such option (open decision #214). Every other
  linear-regression problem of NIST is solved to within 0.3 of a digit of what R reaches,
  but Longley (11.7 digits against R's 13.0). (Since 0.4.0; found at
  0.9.0.)
- **On NIST's three hardest problems of analysis of variance, F agrees to 3.1 to 3.3 digits.**
  The values of SmLs07, SmLs08 and SmLs09 have thirteen constant leading digits
  (1000000000000.4 and its neighbours), so a number as a computer stores it keeps three or
  four digits of the part that varies: exact arithmetic on the stored numbers gives F to 4.2
  to 4.4 digits and no more. `anova` forms its sum of squares between the groups from the
  group means, which are stored at the same size, and loses one digit more; R's
  `oneway.test` gives 3.3. The class of `anova` asks for 1e-6. Data that vary in their
  leading digits are not affected: the other eight problems agree to 8.8 digits and more.
  Subtracting a constant from such a variable before the test gives the digits its stored
  values allow. (Since 0.7.0; found at 0.9.0.)
- **On NIST's Wampler5 problem three coefficients agree to 5.8 to 6.2 digits, at the edge of
  the class.** The class of `reg` asks for 1e-6 relative. The certified coefficients are 1 and
  their certified standard deviations are up to 2.4e7, so the difference is 7e-14 of a standard
  error; the standard errors themselves agree to 13 digits, and R's `lm` reaches the same 5.8.
  The digits are what sixteen-digit arithmetic leaves on that design. (Since 0.4.0; found at
  0.9.0.)
- **The exact rank-sum test is slow on samples with a few ties near fifty values a group.** With
  ties, `wilcoxon_test(y, group: g)` searches its exact interval shift by shift and counts the
  distribution of the tied ranks at each. Where most values are tied, as with scores on a short
  scale, the same shifts repeat and are counted once (since 0.9.0: two groups of 38 and 36
  scores take a quarter of a second in the released build). Where few are
  tied, as with measurements rounded to a decimal, nearly every shift is its own: two groups of
  40 take about five seconds in the released build on a recent laptop (twelve in R 4.6.0, which
  searches in the same way), and more as the groups near 50. The figures are right.
  `exact: false` gives the normal approximation at once, and from 50 values in either group
  the approximation is the default. (Since 0.7.0.)
- **Three small things met at this release are not yet mended.** `mean(x, na.rm: true)`, a habit
  from R, is answered with a message about an interaction in a formula (Z0306), which does not
  say what the reader needs: that missing values are skipped without asking, with a warning
  that counts them. `mad(x, constant: -1)` takes a negative constant and gives a negative
  spread, as R does, where a constant that is not above zero should be refused. And a message
  raised inside a built-in written in Zeff may still name an inner function (see the entry of
  0.5.0). None gives a wrong number for a call that makes sense. (Since 0.5.0 and 0.7.0; noted
  at 0.9.0.)
- **The separation check gives no answer on one kind of large, exactly balanced design.** Before
  a fit, `glm` decides by a linear program whether the outcomes are separated. Where every
  column of the design has the same sum over the rows of outcome 1 as over the rows of outcome
  0, each step of that program is a step in place, and with forty columns and more of unrelated
  numbers it can end without a verdict (found by a test written at this release: a design of
  drawn numbers in which each row is met once with each outcome). The fit then goes ahead as if
  unchecked, and nothing wrong is printed: such outcomes overlap. A separated design on which
  the check gave no answer would be met by the fit's own warnings, of fitted probabilities
  numerically 0 or 1 or of a fit that did not converge, in place of Z0803. With matched pairs
  and a dummy for each pair, the design in which such balance is met in practice, the check
  answers at 200 pairs. To be mended at 0.10.0 (open decision #226). (Since 0.9.0.)
- **No reader outside the project has read Zeff's scripts yet.** The testing handbook asks, from
  0.8.0, that short scripts be given to students and referees who have not written Zeff and that
  what they misread be corrected (section 2.4, item 4). The test was not run at 0.8.0, by the
  owner's decision of 20 November 2026. At this release it is one test for Zeff and Zemp Lite,
  on the hosted alpha of Zemp Lite with readers the owner chooses (his answer of 21 November
  2026); its sheet and key are ready, and it had not been reported when this list was written.
  Until it is run, that a script reads as it computes is the project's own judgement. (Since
  0.8.0.)

## As of 0.8.0

- **`ivreg` is two-stage least squares, with the standard errors of `reg`.** Limited-information
  maximum likelihood and the generalised method of moments are after 1.0. The standard errors
  divide by N − k and use the t table, as R's `ivreg`, fixest and Stata's `ivregress 2sls,
  small` do; Stata's default, which divides by N and uses the normal table, gives slightly
  smaller ones (open decision #150). `influence` and `vif` take a fit of `reg` only. (Since
  0.8.0.)
- **The Anderson–Rubin confidence set is for one instrumented regressor.** With several,
  `ar_test` gives the joint test that all their coefficients are zero and no set. Under
  clustered errors the test needs more clusters than instruments outside the equation, and so
  do the F of the first stage and the Kleibergen–Paap F, which print as "not available" below
  that: a clustered variance has no higher rank than the clusters less one. (Since 0.8.0.)
- **A first stage that fits exactly prints an F of twenty-one digits.** An instrumented
  regressor that is an exact combination of the instruments has no residual in its first
  stage; its F is infinite, and what prints is the rounding of a division by nearly nothing.
  The estimates are those of least squares and are right. R warns of an "essentially perfect
  fit" in the same place; Zeff does so for `reg` and not yet for a first stage. (Since 0.8.0.)
- **`export` writes Typst, and the engine sets no PDF from it.** A `.typ` file is set by a Typst
  the user has installed; the library is not inside the engine, for its size and for the
  licences of some of its hyphenation patterns (open decisions #73 and #172). In the LaTeX
  form the notes of a test's result are rows across the table and widen its last column when
  they are long, which the Typst form no longer does; and a table exported without a title
  writes an empty comment line as its second line. (Since 0.8.0; the LaTeX form since 0.4.0.)
- **A probability below the smallest double has few digits, and a truncation far in a tail
  shows it.** A double holds a number below 2.2e-308 with fewer digits and nothing below
  4.9e-324, so `dist.normal().cdf(-38.4)` is 6.4e-323 to two digits, in Zeff as in R. Inside a
  truncation that lies wholly that far out the same loss appears in probabilities of ordinary
  size: for `dist.truncated(dist.normal(), upper: -37)` the distribution function is right to
  nine digits down to −37.8, 5 per cent off at −38.4 and 0 from −38.5 (R on the log scale:
  1.15e-23 at −38.4), and `quantile(1e-300)` is −38.49 where −52.44 is right; an exponential
  cut at 700 mean lifetimes behaves the same beyond 740. Nearer than 37 standard deviations
  nothing is lost. A log scale for the tails, as R's `log.p`, is open decision #173. (Since 0.3.0 for the
  distributions; seen in a truncation at 0.8.0.)
- **`dist.truncated` refuses equal bounds.** A count distribution cannot be cut to one number by
  writing it twice, `lower: 2, upper: 2`; `lower: 2, upper: 2.5` does it, since a truncated
  count distribution keeps the whole numbers of its interval. (Since 0.3.0.)
- **A warning is said once for a statement, not once for a script.** The same observation
  driving three fits made in one statement is one warning with a count (W1006, since 0.8.0);
  three fits on three lines are three warnings. The passes of a loop are counted as one since
  0.9.0 (see "Resolved at 0.9.0"). (Since 0.8.0 for the count; before it, every repetition.)

## As of 0.7.0

- **In a script, a loop variable does not stand for a term of a fit.** `for v in vars(educ) {
  m.coef(v) }` asks for a term called `v` and stops with Z0414 listing the terms;
  `coef(name_of(v))` reads the term the variable names. Inside a function the parameters and
  bindings of the function are the terms they hold, since 0.8.0 (open decision #147). An
  error, never another number. (Since 0.4.0; found at 0.7.0.)
- **A function written on the spot is taken only where a function is expected**, as in
  `map(xs, fn(x) x * 2)`: its body is one expression, and `let f = fn(x) ...` is refused
  (Z0205); a function with a name is written with `fn name(...)`. `filter` refuses an `NA` from
  its function, and `map`, `filter` and `reduce` a list that holds a failed item. (Since 0.7.0;
  functions bound to a name after 1.0.)
- **The tests take one variable at a time and no sampling weights.** `each(...)`, for the same
  test over several variables, is not built: a loop over `vars(...)` does it, as the `tests`
  sample shows. Sampling weights are refused by every test (Z0411), since a design weight gives
  no distribution for a p-value; frequency weights are taken, analytic weights by some. Under
  analytic weights the one-way and the factorial forms of `anova` scale the weights differently
  and can differ on the same data. (Since 0.7.0.)
- **Some tests stop at a size.** `shapiro_test` is refused above 5,000 values (Z0421), as in R.
  `fisher_test` on a table larger than 2 × 2 gives the p-value alone, no odds ratio and no
  one-sided form (Z0420), and is refused when its enumeration would pass a million tables,
  where R's network algorithm reaches further; `chi2_test` is the way then. `ks_test` is exact
  below 100 values without ties and asymptotic otherwise, and estimates the parameters for the
  normal family alone (Z0422). `wilcoxon_test` is exact below 50 in each group. Hommel's
  adjustment in `p_adjust` takes time quadratic in the number of p-values (20,000 in 0.3 s,
  100,000 in 6.5 s). (Since 0.7.0.)
- **An effect size has no exact interval.** Eta, omega and epsilon squared of `anova` have none,
  and the interval of Hedges' g in `t_test` is the large-sample one: the exact intervals need
  the noncentral t and F distributions, which `dist` does not have. (Since 0.7.0; open decision
  #93, for 0.8.0.)
- **`posthoc` compares the means of an `anova` by ten methods and no other.** Games and
  Howell's, Dunnett's, Student, Newman and Keuls's and Duncan's are not built (open decision
  #85 for the first). The step-wise methods (`holm`, `hochberg`, `hommel`, `bh`, `by`) have no
  intervals, and `level:` written with one of them changes nothing (note W0401). `ci(level:)`
  on the result at another level than the result's is refused: ask `posthoc(level:)`. A
  comparison is the first group less the second, where R's `TukeyHSD` and Stata's `pwcompare`
  print the second less the first, and it is read by name in that order alone. No table of the
  marginal means is printed; that belongs with `margins`. (Since 0.7.0.)
- **`anova(effect: omega_f)` rests on Stata's documented formula, not on a run of Stata.** The
  figure is held to the formula written out in R; no Stata output is in the reference library.
  (Since 0.7.0; open decision #101.)
- **`dist.tukey` is slow.** A value of the studentized range's distribution takes about 5 ms (35
  ms on one or two degrees of freedom) and a quantile about a tenth of a second, since each is
  computed to the accuracy class of the distributions (1e-12), which R's `ptukey` does not
  reach; `draw` from it is slow for many values. (Since 0.7.0.)
- **`tabulate` takes one or two variables and no weights.** (Since 0.7.0; weights with open
  decision #53, at 1.1.)
- **Text functions use the simple forms of Unicode.** `upper` and `lower` map one character to
  one (`ß` stays as it is), `ignore_case:` folds case in the same way, and a pattern has no
  back-reference and no look-around (Z0419). `replace` writes its replacement as plain text, with
  no reference to a group. `split` takes one text and not a column, and keeps a last empty
  piece, which R drops. (Since 0.7.0.)
- **A figure's text is set in IBM Plex and no other face**, upright only: there is no italic,
  no ligature and no shaping of scripts that need it. A character the faces lack (Chinese, an
  emoji) is drawn as an empty box, with a note. (Since 0.7.0; italic by open decision #119,
  after 1.0; wider coverage with the fetch store of 0.11.0.)
- **A figure file is plain.** SVG: the text is drawn as outlines, so a dense figure is large
  (0.85 to 3 MB for 18,201 points) and has no title or description for a screen reader. PDF:
  version 1.4, one page, each face embedded whole (a small figure is about 91 kB), no tagging.
  PNG: a white page at 300 dots to the inch, with no option for another. A page too small for
  its labels is refused and never set smaller; no text is below 8 points. (Since 0.7.0;
  accessible files at 1.2.)
- **Only the two axes take an expression.** `x:` and `y:` may be `log(wage)` or a `case`; the
  other mappings (`color:`, `fill:`, `size:`, `shape:`, `group:`, `label:`) take a variable or
  a constant, and a variable computed with `add` first. The same holds wherever a frame method
  names a variable: `mean(wage * 2)` is refused (Z0401). A reducer, a project function and
  `if` are not taken in a mapping. (Since 0.7.0.)
- **A plot has eight colours and eight shapes for categories**, which repeat from the ninth
  with a note; a colour scale of dates is refused; every level of a category that has no row
  is left out of the legend (an argument of `scale` to keep them is not built before 1.0, open
  decision #109). (Since 0.7.0.)
- **Some marks and scales refuse one another.** Bars and areas are refused on a logarithmic
  axis, and a value at or below zero there is an error that names the rows; `area` and `ribbon`
  are refused on an axis of categories; a histogram has no axis of dates; free scales are
  refused in fixed and in polar coordinates; fixed and flipped coordinates are not offered
  together; polar coordinates start at the top and turn clockwise, with no argument to change
  it (open decision #132). A panel smaller than 12.7 mm is refused when the figure is written.
  (Since 0.7.0.)
- **Zeff's figures are not ggplot2's.** By choice: limits are never rounded outward; the breaks
  of a logarithmic axis are 5, 10, 20, 50, 100; areas of several groups overlay unless stacked;
  a gap in the data breaks a line or an area; legends above or below a panel start at its left
  edge; titles start at the page's margin; a box plot uses Tukey's hinges; a histogram's bins
  start as Stata's do; the density is computed exactly at 512 places, so it agrees with R's
  binned `density()` to about three places. (Since 0.7.0.)
- **The shorthands and the plots of a result take few options.** `scatter(fit: linear)` draws a
  95% band and no other; `coef_plot` takes no option and leaves the intercept out; the plot of
  an event study does not join its points; `heatmap` draws Pearson's correlations without their
  numbers; a refusal names the grammar's function and not the shorthand. A fitted line as a
  layer of its own is `smooth`, at 1.2. (Since 0.7.0.)
- **The figures are read by eye, in part.** Every target must write the same bytes for each of
  the 31 figure files of the `plots` sample, which the identity check holds on every push; that
  the bytes draw what the script says was read by eye for the fifteen figures, in the PNG form
  for all and in the PDF and SVG forms for some, on one machine. (Since 0.7.0.)
- **The language server has been driven by tests, not yet by an editor.** `zeff lsp` serves
  diagnostics as one types, completion, signature help, hover, formatting, folding, the
  templates of common analyses and the statement ranges an interface runs (since 0.8.0), and
  answers any other request as not served: quick fixes, navigation, rename and inlay hints are
  1.1. A template is offered by an engine that has every function it calls: the one for
  instrumental variables arrives with `ivreg`, the one for a logistic regression with `glm`
  at 0.9.0. A changed document is sent whole. Completion offers a frame's variables with their types; their counts of missing
  values are known for a shipped dataset or from the last run, and before a project's first run
  a file's variables show their types alone. The tree-sitter grammar has likewise been held to
  the engine's parser on 131 scripts and seen in no editor. (Since 0.7.0.)
- **`zeff fmt` has one style and no options**, formats the whole project and leaves a script
  that does not parse as it is. `zeff test` runs a script only when its text calls `assert`,
  and does not run the examples of a function private to its file. (Since 0.7.0.)
- **A run is cancelled between two steps, not inside one.** A single call that takes long, a
  regression on ten million rows, ends before the cancel takes effect; a loop is left before
  its next pass. Ctrl-C cancels `zeff run`; the interrupt is tested on Linux and macOS.
  (Since 0.7.0.)
- **The memory budget is an estimate that leans high**, looked at before each top-level
  statement: a file read as text counts five times its size, a list built by a loop three
  times its values, a statement that builds from frames one more copy of them. It may refuse
  what would have fitted, and says its figure; it does not see an `import` inside a function or
  a loop, a many-to-many `merge`, `to_long`, `append` or a fit's model matrix. The command line
  has no budget unless `memory` in `zeff.toml` or `--memory` sets one. (Since 0.7.0; open
  decision #143.)
- **In a browser the engine needs a page that is cross-origin isolated**, since a run is
  cancelled through shared memory; the worker is tested in headless Chrome and in no other
  browser. Only the frames held by a run's top-level bindings cross to the interface; packages
  loaded on first use are not yet built, so the whole engine loads at once. (Since 0.7.0.)
- **A completion file holds the words of the engine that wrote it** and is written again after
  an upgrade; the PowerShell file is run in CI alone. (Since 0.7.0.)
- **The reference file of `posthoc` after a formula was written with emmeans 2.0.3**, while CI
  holds it to the pinned 2.0.4 within tolerance; `emmeans` is not on the development machine,
  so that script is regenerated in CI alone. (Since 0.7.0; the installation is the owner's.)
- **The weekly mutation job runs each shard in two passes**, since 0.8.0: every mutant against
  its own package's tests, then the mutants that pass against the tests of the packages that
  use the mutated one, never against the whole workspace. A mutant that only a package outside
  that set would catch is counted as a survivor, which errs on the side of reading one more.
  The second pass as first built (step 3) counted every mutant as caught, by a failure of
  memory and not of a test; it was repaired and proven at step 24a1, and the run of 19
  November 2026 is the first whose second pass can be trusted. The accepted survivors are a
  list, `tools/mutants/recorded.txt`, and each shard's note prints what is not on it. A whole run
  takes about fourteen hours at fourteen shards at a time. A mutant that an early test catches and
  under which a slower test then runs on is reported as a timeout; seventeen such are recorded under
  the key `late`. The corrections made after the run of 20 November 2026 were each tried by hand,
  and the scheduled run of 23 November is the first whole run to read them. (Since 0.7.0; the job
  repaired twice at 0.8.0.)
- **A one-sided interval of `wilcoxon_test` at a level of exactly 50 per cent is not unique under
  the normal approximation.** The standardised statistic can equal the quantile, which is 0
  there, along a whole step of the shift, and any point of the step is a bound. Zeff gives the
  left end of the step; R gives the point at which its solver stops (1.3 where Zeff prints
  1.000, for the upper bound on the five values 0, 1, −2, 3, 4). At every other level the two
  agree within R's solver tolerance. (Since 0.7.0.)

## As of 0.6.0

- **`save` writes CSV and TSV only.** Parquet, Arrow, Stata, Excel and JSON are refused with
  Z0716, which names 0.11.0, the release of the reader and writer interface (SPSS 1.1); an
  opaque column is refused with the advice to drop it. What is written reads back bit for bit.
  (Since 0.6.0; by the build order.)
- **Under several effects in `fe:` a fit has no leverage.** `se: hc2` and `se: hc3` are refused
  (Z0410), `influence` is refused (Z0304) and W1006 is not judged, because an observation's
  leverage in the full design cannot be had from the columns with two or more effects projected
  out; the messages name release 1.1. With one effect, an interaction `a:b` included, the
  leverage is that in the demeaned columns plus the row's share of its level's weight, and all
  three work as without `fe:`, in `reg` and in `ivreg` (from 0.8.0; before it every fit with
  `fe:` was refused). `influence` and `vif` take a `reg` fit only. `fe:` takes variables and
  interactions `a:b` joined by `+`; `*`, a range or an expression is refused (Z0401), as is a
  fixed effect that is also a variable of the formula. Two-way clustering arrives at 1.1.
  (Since 0.6.0; `ivreg` takes `fe:` from 0.8.0.)
- **`wilcoxon_test` with `exact: true` on large samples stores every pair.** The exact
  interval reads its bounds off the sorted Walsh averages or pairwise differences, so its
  memory grows with the square of the sample; the exact distribution is itself for small
  samples, and the rule without `exact:` uses the approximation from 50 observations, whose
  memory grows with the sample alone (from 0.8.0: 190,757 rows in 1.4 s within 220 MB, where
  0.7.0 could not finish within 7 GB). (Since 0.7.0.)
- **`zeff report-bug` shrinks the data files it brings, within limits.** With `--include-data`
  or `--synthetic` a text file (CSV, TSV) the kept statements read is shrunk by lines; a
  shipped dataset, a frame written out in the script and a file that is not text are not
  shrunk. A result believed wrong (`--line` with `--reference`) keeps every row, since its
  reference was computed on them, and its values are never made up. The first line of a file
  is always kept as it is, as the line of the variables' names: a file without such a line
  keeps its first row, with `--synthetic` too, and the report says the first line is as it
  was. Made-up values are chosen by column, so a column the failure needs keeps all the values
  of the lines kept. The shrinking stops after 500 trial runs and says so. (Since 0.8.0.)
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
  every other arithmetic on a date is refused (Z0221) with the function to use (`add_time`,
  `time_between`). Years run from 1 to 9999, a result outside
  them is `NA` (W0203); `add_time` by months clamps to the month's last day, so it does not undo itself
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
- **Four functions planned in Zeff are written in Rust** (five until `row_mean` and `row_sum`
  became `row` at step 8k of 0.8.0), each with the reason in its catalogue
  row: `row` (the kernel's compensated row reduction, which `+` over columns
  does not reproduce), `diff` (its `lag` reads the panel of the frame around the call, which a
  body does not pass on), `n_missing` (a list holding failed values reaches it), `summary` (its
  table's formatting and combined warning need `format` and `join` of `text`, its title the
  frame's name, and it takes `weights:`) and `duplicates` (its pieces came later in 0.6.0);
  `vif` stays in Rust too though `design()` exists (open decision #136, not before 1.0).
  `drop_missing` returned to Zeff at step 7b of 0.8.0, once a body could raise the engine's
  codes and see a list of variables resolved against the frame (open decisions #89 and #90). `fill_missing` and
  `quantile_group` returned to Zeff at steps 4.1 and 4.4 of 0.7.0; `quantile_group` became
  the option `n:` of `bin`, in Rust, at step 8h of 0.8.0, and `fill_missing` took in
  `fill_forward` and moved to `data`, in Rust, at step 8i (open decision #97). Of the functions of 0.7.0
  that were planned in Zeff, `t_test`, `proportion_test`, `chi2_test`, `kruskal_test` and
  `p_adjust` are in Rust, since a body has no way to build a test's result (open decision #92,
  decided at 0.8.0: the tests stay in Rust), and so is `coef_plot`. Nothing a script sees differs, and section 8.2's totals follow
  the catalogue. (Since 0.5.0; the list as of the close of 0.7.0.)
- **Eight conditions carry codes numbered at this release**, the next free numbers of their
  families, each logged in `docs/spec_changes.md` and listed in section 11 of the Language
  Reference: Z0507 (`event`'s `ref:`), Z0601 and Z0602 (the panel declaration and the panel
  functions), Z0609, Z0610 and Z0611 (`append`, `to_long`, `to_wide`), W0210 (`bin`) and W0701
  (a data root's file changed since the last run). W0701 compares hashes kept in the project's
  cache, which stand in for the run record until 0.11.0. (Since 0.6.0.)

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
- **`case` over rows evaluates every arm on the whole frame**, as R's `ifelse` does, so a warning
  such as a division by zero may count rows the arm does not supply; a Category column is refused
  as a value of `case` (Z0214; `to(x, Str)` gives its labels). (Since 0.5.0; a Date or DateTime
  column is accepted from 0.6.0.)
- **A failed value reaches six functions**: `is`, `type_of`, `print`, `report`, `len` and
  `n_missing`; anywhere else it is Z0220 reporting the failure it holds, so a list of fits made
  under `attempt` is read in a loop that steps over the failed ones. (Since 0.5.0; by design,
  section 3.7.)
- **A message raised inside a built-in written in Zeff may name an inner function, not the one
  the script called**: `median` needs a Bool, Int or Float, from `mad`; `mean`, from `skewness`
  and `kurtosis`. The body's own column is no longer named: from step 8o of 0.8.0 `winsorize`,
  `standardize` and `center` name the argument as written and warn once, and from 0.9.0 `mad`
  does, where it warned a second time of `deviation`. The message is placed at the
  script's call and its code is on the function's page. (Since 0.5.0; the warning of `skewness`
  and `kurtosis` for too few values carries W0205 from step 4.3 of 0.7.0.)
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
- **The result of `test` or `compare` has no interval**: `std_errors()` and `ci()` on it give
  `NA`, and `estimates()` one row. The tests of `stats` carry their intervals from 0.7.0, at the
  level the test was made at. (Since 0.4.0.)
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
  in one statement repeat each other. From 0.7.0 the key is the statement's place among the
  statements that draw, as section 8.3 has it, so that `seed(n)` at the top repeats a run and a
  statement that draws nothing moves no draw; the numbers under a script-wide `seed(n)` are
  therefore not those of 0.3.0 to 0.6.0. (Since 0.3.0; by design, section 8.3.)
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
- **The build a browser loads is 2.5 MB compressed** (2,535 kB with Brotli, 3,659 kB with gzip,
  13.4 MB before compression), measured from 0.7.0 on the engine's worker build, which is what
  an interface loads; until 0.6.0 the measure was the conformance build's (1.65 MB at 0.6.0,
  1.46 MB at 0.5.0, 1.34 MB at 0.4.0, 1.15 MB at 0.3.0, 1.10 MB at 0.2.0, 355 kB at 0.1.0;
  1.93 MB at 0.7.0). The three datasets are embedded, and this release adds the plot renderer
  with its three faces of type (about 0.6 MB), the tests and the regular expressions (216 kB).
  It stays under the 5 MB warning level of risk R4, with a little under half its room left. The
  cold start is 52 ms against 1,000 ms: the module loads in 11 ms and a one-line script on a
  project just opened takes 41 ms. (Since 0.2.0; the figures are those of 0.7.0, on the
  development machine.)
- **Each project pins the exact engine version** (`zeff = "0.2.0"`). An engine of another
  version refuses the project with exit code 4; there is no launcher yet to pick the pinned engine
  (0.9.0). (Since 0.1.0.)
- **The function catalogue's status column** is updated at each release close, not per function,
  so between releases it lags the registry; `zeff help <package>` shows what is shipped. (Since
  0.1.0.)

## Resolved at 0.9.0

- **The gaps that the review against R met**, listed at 0.8.0 with a way round each, are closed,
  each as its open decision recommends: the length of each text value, `n_chars` (#165, step
  31a); a row number, `row_number()` (#167, step 31b); a count of missing values across
  variables in each row, `row(vars, stat: n_missing)` (#166, step 31c); one value taken out of
  a column, `x[i]`, `first` and `last` (#168, step 32a); a column guessed as whole numbers on
  import that meets a decimal further down, now read as Float with a note (#169, step 32b);
  the six smaller refusals (#170, step 33); and the six printed layouts (#171, step 34): a
  coefficient of `report` that printed as `0.000` is written to two significant digits with
  its sign, `labels:` reaches the parts of an interaction, degrees of freedom have thousands
  separators, a facet of a number is titled `kids = 0`, hypotheses too long for the page have
  their symbols under their words, and `estimates()` of a t-test gives the standard error.
  Each was an error or a layout, never a wrong number. Printed output changed with the
  layouts, and the release notes say where.
- **A warning repeated word for word in the passes of a loop** was printed in every pass. It
  is said once, with the count of passes, as a note has been since 0.8.0 (#149, step 30).
  Warnings of two statements are still two. Listed since 0.8.0.

- **A test said nothing of a row its weight left out.** Under `weights:`, a row whose weight was
  missing or zero was left out of `t_test`, `wilcoxon_test`, the one-way `anova`,
  `kruskal_test`, `chi2_test`, `fisher_test` and `proportion_test` without a word, so the note
  on the rows dropped counted too few: 26 where 29 were left out, in a weighted t-test on the
  wages of 2019. The N of the table and every figure of the test were right. The note counts
  such rows and names the weight, and says how many rows a zero weight left out (step 40b).
  From 0.7.0 to 0.8.0.
- **Three calls that were refused or failed without cause.** `update(x = NA, where: …)`, typed
  or alone, was refused with a message that advised the very form it refused (step 38e).
  `merge` with `suffixes:` kept a panel declaration that named a variable the suffix had
  renamed; the declaration is dropped with a note (step 38e). `x.attempt()`, the method form,
  ended in an internal error or let the failure through; it is `attempt(x)` (step 42a). Each
  was an error or a wrong header, never a wrong number.
- **84 help pages were brought to the quality of the rest**: those of `data`, `datasets`, `dist`
  and `stats` (steps 38a1 to 40), of the 145 listed at 0.8.0. The 61 that remain are in the
  section of 0.9.0.
- **The exact rank-sum test on heavily tied samples took minutes.** Two groups of 38 and 36
  scores on a scale of five took two minutes in the test build; a run of equal shifts is now
  counted once (step 42d), and they take a quarter of a second in the released build. The
  figures did not change. What remains slow is in the section of 0.9.0. Since 0.7.0.

## Resolved at 0.8.0

- `winsorize` warned twice and named `value`, the column of its own body, where the script's
  variable was meant, and refused a share outside 0 to 0.5 through an `assert`. Since step 8o a
  built-in written in Zeff names the argument as the script wrote it and its warning is said
  once: `warning[W0201]: 211 rows with NA in wage were skipped`; the share is refused with
  Z0401 and the reason. Listed since 0.6.0.
- **Wrong results, corrected, none of them in a released version unless said.** A count
  distribution truncated beside a whole number ended in an internal error (0.8.0 only, step
  20n). With no more clusters than instruments the first-stage F and the Kleibergen–Paap F of
  `ivreg` printed numbers of seventeen digits; fixed effects that leave no residual degree of
  freedom ended in an internal error, in `reg` since 0.6.0; a dropped instrumented regressor
  was still counted in the statistics of the first stage (steps 24a2a and 24a2c). The
  probability of a truncated distribution close to its bound kept five digits where thirteen
  are right, since 0.3.0 (step 20i). `wilcoxon_test` stored every pair of two groups, 662 MB
  for the shipped wages by sex, and now stores none (step 20d).

## Resolved at 0.7.0

- The pages of the built-ins written in Zeff list the warnings and errors they can raise, and
  no others (listed at 0.5.0 as listing more than they can raise). Each of the fourteen carries
  the codes of the functions it calls that its body cannot raise, with the reason, and the codes
  of its own operators and conditions; the tests raise every code left on a page through the
  function, and no test raises one a page leaves out. Since step 32f; open decision #139.
- **A wrong result, corrected.** From 0.5.0 to 0.6.0, `mad`, `skewness` and `kurtosis` gave a
  wrong number on a frame that had a variable called `centre`, whatever variable was asked for:
  the bodies bound that name and used it inside a method of the frame, where the frame's
  variable came first. The run carried the warning W0102 ("`centre` is both a variable of data
  and a binding; the variable was used") and no error. `skewness` and `kurtosis` of a variable
  itself called `dev2` were also wrong, under a note that `dev2` replaced a variable. An
  analysis is affected only if its data had a variable of one of those two names and it called
  one of the three functions; such a run printed the warning or the note. Corrected at step
  32d of 0.7.0: on every other frame the three give the same bits as before. `iqr`, `center`,
  `standardize`, `winsorize` and `quantile_group` were not affected.
- A loop over a range walks it: `for i in a..b` and `[for i in a..b { ... }]` take the next
  number at each pass and hold one item (listed at 0.5.0 as holding every item before the first
  pass, 184 bytes each, 1.8 GB for ten million passes). A loop of three million passes now runs
  in 24 MB; it begins at once and can be cancelled before its first pass; the results and the
  draws of every pass are the same. Under a memory budget such a loop counts one item, and a
  list built by one counts its values with the room a growing list takes. Since step 38g.
- **A wrong result, corrected.** From 0.3.0 to 0.6.0, `correlation` with `method: pearson`
  printed a coefficient of 0.000 for a variable that held a value above 1e150 in size, whatever
  the data: the products of the deviations overflowed. R prints the same. An analysis is
  affected only if a variable had such a value, which is seldom a measurement and often a code
  for a missing value that was not declared. `t_test` and the one-way `anova`, new at 0.7.0,
  had the same fault before they were released (a standard deviation of `Inf`, t = 0, p = 1;
  an F of `Inf`). The three now refuse the variable (Z0803), as `reg` and the factorial `anova`
  do, and at 1e150 itself give what the same data give at their ordinary size. Found by the
  mutation job at step 40a of 0.7.0, through a mutant of `anova` that no test caught.
- `zeff run` says why it will not run when the engine's own registry does not build (found at
  step 38d of 0.7.0: it ended with exit code 2 and printed nothing, where `zeff check` printed
  the reason). Only a development build can be in that state. Since step 40c: the command looks
  at the registry first, as `zeff check` does.
- `iqr`, `mad`, `skewness` and `kurtosis` take a frame that has a variable called `var` or
  `type` (listed at 0.6.0 as refused with Z0106), and a function of a project gives the same
  result whatever the variables of the frame it is given are called (listed as depending on
  them: Z0106 for a parameter, the variable under the warning W0102 for a `let`). Inside the
  body of a function the names the function binds come first, inside a frame's methods too; a
  script is unchanged. Since step 38c; open decision #140, option A.
- A function written in a script sees, by its short name, a module function the script brought
  in with `use` (listed at 0.6.0 as refused with Z0103 unless written `checks::positive(x)`):
  the script's `use` lines are read before the bodies of its functions, so a body sees what the
  top level sees, as section 5.3 of the Language Reference says. Since step 38b.
- A misspelt name is compared with the project's own functions (listed at 0.6.0 as compared
  with the engine's alone): after `use functions::clean`, `twise(2)` is answered with "did you
  mean `twice`?". A public function of a module the script has not brought in is answered with
  the `use` line that brings it in, and a private function of another file with the reason it
  is not seen. Since step 38b.
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
