# Zeff Help Pages: Examples of the Help Contract

Date: 17 October 2026 · Companion to the Zeff Specification, section 9.1 · Rendered as a PDF with the two source forms, the enforcement output and the Zemp pane (zeff_help_examples.pdf)

Every function ships with a help page in the form of R's manual pages; Usage and Arguments are generated from the signature, examples are run on every build, and a built-in without a complete page does not build (section 9.1 of the Zeff Specification). These are the pages `zeff help <name>` will print for seven built-in functions, as templates for writing the others. The Accuracy lines show the shape of the conformance record ("Last run: 0.6.0, build 1 934, 48 of 48 cases passed"); no engine has run yet, so every build number and case count is a placeholder. The printed output of every example follows the printing rules of section 3.8 of the Language Reference and, for a result, the layout of section 10.2, because `zeff test` compares an example's output exactly.

## `zeff help reg`

a model with shared arguments, generated warnings, accuracy and references

```
reg — Linear regression by least squares, with fixed effects        regress · tier 2 · since 0.4.0 · 1.0 · closed form

USAGE
    data.reg(formula, fe: Terms, weights: Var, weight_type: frequency | analytic | sampling = sampling,
             se: classical | robust | hc2 | hc3 = classical, cluster: Var, level: Float = 0.95)

DESCRIPTION
    Fits the outcome of `formula` on its regressors by least squares. Categories expand to indicators against
    their base level, `a:b` is an interaction and `a*b` is `a + b + a:b` (Zeff Specification, section 7.2);
    `fe:` absorbs fixed effects by alternating projections before the fit, so a model with a million firm
    effects never builds a million indicator columns.

    Rows with `NA` in any model variable are dropped and counted. Regressors are scaled to unit norm and the
    design is factorised by pivoted QR, never by the normal equations; an absorbed or collinear regressor is
    left out and named in a note. Differences from R and Stata: `robust` is HC1, Stata's `robust`; clustered
    errors are CR1 with reghdfe's rules; with sampling weights `se:` defaults to `robust` (Z0410 otherwise).

ARGUMENTS
    data          Frame. The data. Rows with NA in the outcome, a regressor, a fixed effect, the weight or
                  a cluster variable are dropped and counted.
    formula       Formula. `y ~ x1 + x2`; `- 1` removes the intercept; `log(x)` and `x^2` compute columns
                  before the fit; `base(industry, "Retail")` changes a category's base level.
    fe:           Terms. Fixed effects to absorb: `firm`, `firm + year`, or `firm:year` for an interacted
                  effect. Singletons are dropped and reported.                                      (shared)
    weights:      Var. Weight variable; its meaning is set by `weight_type:`.                        (shared)
    weight_type:  frequency | analytic | sampling = sampling. Stata's fweight, aweight and pweight. (shared)
    se:           classical | robust | hc2 | hc3 = classical. Standard-error type; `bootstrap` from 1.1.
                  With sampling weights the default is `robust`.                                     (shared)
    cluster:      Var. Cluster-robust standard errors (CR1); multiway clustering with Vars after
                  1.0. One-observation clusters are reported.                                       (shared)
    level:        Float = 0.95. Confidence level for `ci` and the printed table.                     (shared)

VALUE
    A `RegFit`. Result interface: summary, estimates, std_errors, p_values, ci, n_obs, export, report.
    Estimator interface: coef, vcov, fit_measures, predict, residuals, fitted, compare, test, derive.
    Linear-model methods: r_squared, influence, vif. `.plot()` draws a coefficient plot.

DETAILS
    Fixed effects are removed by accelerated alternating projections (tolerance 1e-8); the design is then
    factorised by tall-skinny Householder QR in blocks of a million rows. R² within uses the demeaned data;
    clustered degrees of freedom follow reghdfe (absorbed effects nested in clusters are not counted).

WARNINGS AND NOTES
    Note    rows dropped for NA, singletons dropped, absorbed and collinear regressors named, with counts
    W1001   near-singular design, with the condition number        W1002   fewer than 30 clusters
    W1003   extreme weights, with the effective sample size        W1004   little variation left after fe:
    W1006   observations whose leverage drives a coefficient      Z0410   se: classical with sampling weights

ACCURACY
    Closed form: within 1e-6 relative of R 4.4 lm and sandwich 3.1-1, fixest 0.12.1, Stata 18 regress and
    reghdfe 6.12, on wooldridge::wage1, wage2 and a generated two-way panel. Last run: 0.6.0, build 1 934,
    48 of 48 cases passed on all four targets.

REFERENCES
    Davidson & MacKinnon (2004), Econometric Theory and Methods, chapters 3 and 5.
    Correia (2016), Linear models with high-dimensional fixed effects: an efficient and feasible estimator.
    Cameron & Miller (2015), A practitioner's guide to cluster-robust inference, J. Human Resources 50, 317–372.

SEE ALSO
    ivreg, glm, report, derive, test, compare, interact

EXAMPLES
    > let wages = import("zeff:wages.csv")
    > let m = wages.reg(lwage ~ educ + exper + exper^2 + female, fe: firm, cluster: firm)
    > m.coef(educ)
    0.074318
    > m.std_errors(educ)
    0.006104
    > report([m]).export("output/table1.docx")
```

## `zeff help t_test`

one name, three forms; Welch by default, with the reference that says so

```
t_test — t-tests for one mean, two groups or paired measurements      stats · tier 1 · since 0.7.0 · 1.0 · closed form

USAGE
    data.t_test(var, mu: Float = 0, alternative: two_sided | less | greater = two_sided, level: Float = 0.95)
    data.t_test(var, group: Var, equal_var: Bool = false, effect: d | g = g,
                alternative: two_sided | less | greater = two_sided, level: Float = 0.95)
    data.t_test(a, b, paired: true, alternative: two_sided | less | greater = two_sided, level: Float = 0.95)

DESCRIPTION
    Three tests under one name, chosen by the arguments given: a one-sample test of the mean of `var`
    against `mu`, a two-group comparison of `var` between the two levels of `group`, or a paired comparison
    of `a` and `b` row by row. Each prints the hypotheses it tests, H0 and H1, in words and symbols; the
    means; the difference with its confidence interval; t, the degrees of freedom and the p-value; an effect
    size; and a decision at α = 1 − level beside the p-value.

    The two-group form is Welch's test by default, as R's `t.test` and unlike Stata's `ttest`, which assumes
    equal variances; `equal_var: true` gives the pooled test. The effect size is Hedges' g, which corrects
    Cohen's d for small samples; `effect: d` gives d.

ARGUMENTS
    data        Frame. The data; rows with NA in the tested variables are dropped and counted.
    var         Var. The variable tested (one-sample and two-group forms).
    mu:         Float = 0. The hypothesised mean (one-sample form).
    group:      Var. The variable whose two groups are compared; one level or more than two is an error
                that suggests `anova` (Z0412).                                                     (shared)
    equal_var:  Bool = false. Pool the variances (Student's test) instead of Welch's.
    effect:     d | g = g. The effect size reported: Cohen's d or Hedges' g.
    a, b        Var, Var. The two paired measurements (paired form); a row with NA in either is dropped.
    paired:     Bool. Must be `true` in the paired form; it selects that form.
    alternative: two_sided | less | greater = two_sided. The alternative tested, which H1, the p-value and
                the interval follow: `less` means the first-named quantity (the mean, the first group, `a`)
                is less than the second or than `mu`.                                                  (shared)
    level:      Float = 0.95. Confidence level.                                                     (shared)
    weights:    Var. Frequency or analytic weights; sampling weights are refused (Z0411) and point to
                regression with robust errors.                                                      (shared)

VALUE
    A `TestResult`, with the result interface (summary, estimates, p_values, ci, n_obs, export, report)
    and its own methods statistic(), df(), effect_size(), h0(), h1() and alternative(): the statistic,
    the degrees of freedom, the p-value, the estimate (the mean or the difference) with its interval,
    the effect size, and the hypotheses as sentences.

DETAILS
    Welch degrees of freedom by the Welch–Satterthwaite equation. Hedges' g = d × (1 − 3 / (4(n₁ + n₂) − 9)).
    The paired test is the one-sample test on `a − b`. p-values come from `dist.t` with the tail computed
    directly, never as 1 − CDF.

WARNINGS AND NOTES
    Note    rows dropped for NA, with the count
    Z0412   group: has one level or more than two              Z0411   weights: with weight_type: sampling

ACCURACY
    Closed form: within 1e-6 relative of R 4.4 t.test and effectsize 0.8.9, and of Stata 18 ttest with
    `unequal`, on wooldridge::wage1 and R's sleep data. Last run: 0.7.0, build 2 017, 36 of 36 passed.

REFERENCES
    Welch (1947), The generalization of "Student's" problem when several different population variances
    are involved, Biometrika 34, 28–35.
    Hedges (1981), Distribution theory for Glass's estimator of effect size, J. Educational Statistics 6, 107–128.

SEE ALSO
    wilcoxon_test, anova, proportion_test, reg, power.t_test (from 1.17)

EXAMPLES
    > let wages = import("zeff:wages.csv")
    > wages.t_test(lwage, group: female)
    t_test · Welch two-sample · lwage by female
      H0: the population means of lwage are equal in male and female        μ(male) − μ(female) = 0
      H1: the population means of lwage differ between male and female      μ(male) − μ(female) ≠ 0

      group      N      mean      sd
      male    9,604    3.214   0.512
      female  8,597    3.061   0.498

    difference (male − female) = 0.153   95% CI [0.138, 0.168]
    t = 20.42   df = 18,074.2   p < 0.001
    Hedges' g = 0.30   95% CI [0.27, 0.33]
    decision at α = 0.05: reject H0 (p < 0.001)
    > wages.t_test(lwage, mu: 3, alternative: greater)
    t_test · one-sample · lwage
      H0: the population mean of lwage equals 3            μ = 3
      H1: the population mean of lwage is greater than 3   μ > 3

    N = 18,201   mean = 3.142   sd = 0.511
    difference (mean − 3) = 0.142   95% CI [0.136, Inf)
    t = 37.48   df = 18,200   p < 0.001
    decision at α = 0.05: reject H0 (p < 0.001)
    > import("zeff:sleep.csv").t_test(extra_1, extra_2, paired: true)
```

## `zeff help where`

a data verb: short, with the NA rule that differs from Stata

```
where — Keep the rows where a condition is true                              data · tier 1 · since 0.2.0 · 1.0 · exact

USAGE
    data.where(condition)

DESCRIPTION
    Returns the rows for which `condition` is `true`. Rows where it is `NA` are dropped and counted, so
    `where(age > 30)` and `where(!(age > 30))` both drop rows with missing age, and the note of each says
    so; the two never partition a frame silently. Stata's `keep if age > 30` keeps missing age.

    Row order is kept. Nothing is copied: the result records which rows were kept and shares every column
    with its parent until a later step reads one (Zeff Specification, section 6).

ARGUMENTS
    data        Frame. The frame to filter.
    condition   Column<Bool>. An expression over the frame's variables, such as
                `year >= 2010 && !is_missing(wage)`. A single Bool, or a column of another type, is an
                error that names the problem (Z0203).

VALUE
    A `Frame` with the same columns and a subset of the rows; a panel declaration survives.

MISSING VALUES
    A row where the condition is `NA` is not kept, and a note counts those rows and says how to list them.
    To keep them on purpose, say so: `where(is_missing(age) || age > 30)`.

WARNINGS AND NOTES
    Note    rows dropped because the condition was NA, with the count and a statement that lists them
    Note    no row kept: the result is empty

COMING FROM R OR STATA
    Equivalents: R subset(), dplyr::filter(); Stata keep if.

    Zeff                 R                                                      Stata
    d.where(wage > 20)   subset(d, wage > 20), or dplyr::filter(d, wage > 20)   keep if wage > 20 & !missing(wage)

    In Stata a missing value is larger than any number, so `keep if wage > 20` keeps the rows with no wage;
    R's `subset` and `filter` leave them out and say nothing.

ACCURACY
    Exact: the same rows, in the same order, as dplyr::filter() once its NA rows are removed; part of
    every verb test. Last run: 0.2.0, build 611, 22 of 22 cases passed on all four targets.

SEE ALSO
    drop_missing, select, sort, distinct, sample, is_missing

EXAMPLES
    // The rows that meet two conditions.
    > let wages = import("zeff:wages.csv")
    > wages.where(year >= 2010 && female == 1).n_rows()
    4217

    // The note counts the rows where the condition was `NA`.
    > wages.where(exper > 10).n_rows()
    7902
    note: 13 rows dropped where `exper > 10` was NA; `wages.where(is_missing(exper))` lists them
```

## `zeff help merge`

a verb whose errors are part of its contract

```
merge — Join two frames on key variables                                     data · tier 1 · since 0.6.0 · 1.0 · exact

IN BRIEF
    - `a.merge(b, on: id)`: every row of `a`, with the columns of `b` where the key matches.
    - `a.merge(b, on: firm, relation: many_to_one)`: the key structure stated, and checked before joining.

USAGE
    data.merge(frame, on: Vars, join: left | inner | full = left,
               relation: one_to_one | many_to_one | one_to_many, suffixes: List)

DESCRIPTION
    Joins `data` (the left frame) with `frame` (the right) on the key variables in `on:`, which must exist
    in both with the same type. Every merge prints how many rows matched and how many did not on each side,
    as Stata's `_merge` table does. `NA` keys never match and are reported as unmatched (dplyr matches them).
    The result keeps the left frame's row order, and within one left row the right frame's order.

    `relation:` states the key structure you expect and refuses the data when it differs: `many_to_one`
    means each left key appears at most once on the right. A many-to-many merge is always refused (Z0603),
    since it is nearly always a mistake. A non-key column present in both frames is an error (Z0604) that
    lists the columns, unless `suffixes:` gives the two suffixes that tell them apart.

ARGUMENTS
    data        Frame. The left frame.
    frame       Frame. The right frame.
    on:         Vars. The key variables, present in both frames with the same type; when the names differ,
                `rename` one frame first.
    join:       left | inner | full = left. Which rows to keep: all left rows, matched rows only, or all rows
                of both frames.
    relation:   one_to_one | many_to_one | one_to_many. The expected key structure, checked before joining.
    suffixes:   List of two Str, such as `["_l", "_r"]`. Added to non-key columns that both frames share.

VALUE
    A `Frame` with the left frame's columns, then the right frame's non-key columns; labels, categories and
    a panel declaration on other keys are kept.

MISSING VALUES
    A key that is `NA` matches nothing, on either side: its row is counted as unmatched.

WARNINGS AND NOTES
    Note    matched and unmatched rows on each side, with counts
    Z0603   the keys form a many-to-many relation, or relation: does not hold
    Z0604   a non-key column exists in both frames and no suffixes: is given
    Z0101   a key variable is missing from one frame

A COMMON MISTAKE
    Joining on one variable when the frames share another, which cannot be one column of the result:

    > wages.merge(firms, on: firm)
    error[Z0604]: columns `size` exist in both frames and are not keys
      = help: give suffixes: ["_w", "_f"], or drop one of them first

    Give the suffixes, or drop the column from one frame first.

COMING FROM R OR STATA
    Equivalents: dplyr::left_join(); Stata merge.

    Zeff                 R                                   Stata
    a.merge(b, on: id)   dplyr::left_join(a, b, by = "id")   merge 1:1 id using b, keep(master match)

    dplyr matches a key that is `NA` with another that is `NA`; here `NA` matches nothing. Stata's `merge`
    asks for the relation and adds `_merge`; here the note counts the matched and unmatched rows.

ACCURACY
    Exact: the same rows and values as dplyr 1.1 joins with NA keys excluded, in the order stated above.
    Last run: 0.6.0, build 1 902, 31 of 31 cases passed on all four targets.

SEE ALSO
    append, to_long, to_wide, aggregate, rename

EXAMPLES
    // Many rows of wages to one firm; the note counts the rows matched and unmatched.
    > let firms = import("zeff:firms.csv")
    > let wages = import("zeff:wages.csv").merge(firms, on: firm, relation: many_to_one)
    merge: 18,201 rows matched; 0 left rows unmatched; 3 right rows unmatched (firms with no workers)

    // The matched rows alone, with a column that both frames have told apart by its suffixes.
    > wages.merge(firms, on: firm, join: inner, suffixes: ["_w", "_f"]).n_rows()
    18201
```

## `zeff help dist.normal`

a distribution and its four methods on one page

```
dist.normal — The normal distribution                                dist · tier 1 · since 0.3.0 · 1.0 · distribution

USAGE
    dist.normal(mean: Float = 0, sd: Float = 1)
    d.pdf(x)     d.cdf(x, upper: Bool = false)     d.quantile(p)     d.draw(n: Int, seed: Int)

DESCRIPTION
    A `Distribution` value for N(mean, sd²). Like every distribution it answers `pdf`, `cdf`, `quantile` and
    `draw`, can be truncated with `dist.truncated` and mixed with `dist.mixture`, and is a value like any
    other: `let d = dist.normal(sd: 2)`.

    `cdf(x, upper: true)` computes the upper tail directly, so a p-value of 1e-200 is exact rather than 0;
    this is why Zeff never writes `1 - cdf(x)`.

ARGUMENTS
    mean:       Float = 0. Location.
    sd:         Float = 1. Scale; must be positive (Z0413).
    x, p, n     Of the methods: a value or a column; a probability in [0, 1]; the number of draws.
    upper:      Bool = false. Return P(X > x) instead of P(X ≤ x).
    seed:       Int. Random seed for `draw`; chosen and recorded when omitted.                      (shared)

VALUE
    A `Distribution`. `pdf`, `cdf` and `quantile` return a Float or a `Column<Float>` matching their input;
    `draw` returns a `Column<Float>` of n values.

DETAILS
    cdf by Cody's rational approximations to erf and erfc, relative error below 1e-16 over the range;
    quantile by Wichura's algorithm AS 241; draws by inverse CDF from the Philox stream (section 8.3), so a
    seed gives the same draws on every platform.

WARNINGS AND NOTES
    Z0413   sd: not positive; p outside [0, 1]

ACCURACY
    Distribution class: pdf, cdf and quantile within 1e-12 relative of R 4.4 (dnorm, pnorm, qnorm) and the
    Boost.Math reference tables, including tails down to 1e-300. Draws: identical for a seed on every
    platform; against R, agreement within Monte Carlo error over 50 seeds. Last run: 0.3.0, build 844, all
    passed.

REFERENCES
    Cody (1969), Rational Chebyshev approximations for the error function, Mathematics of Computation 23, 631–637.
    Wichura (1988), Algorithm AS 241: the percentage points of the normal distribution, Applied Statistics 37, 477–484.

SEE ALSO
    dist.t, dist.lognormal, dist.truncated, dist.mixture, pdf, cdf, quantile, draw

EXAMPLES
    > dist.normal().cdf(1.96)
    0.975002
    > dist.normal().quantile(0.975)
    1.95996
    > dist.normal().cdf(30, upper: true)
    4.90671e-198
    > dist.normal(mean: 100, sd: 15).draw(5, seed: 42)
    Column<Float> · 5 values
    107.4
    93.1
    104.8
    122.9
    96.5
```

## `zeff help attempt`

a core function; the page explains a design decision as well as the call

```
attempt — Run an expression and carry on if it fails                           core · tier 0 · since 0.5.0 · 1.0 · exact

USAGE
    attempt(expr)

DESCRIPTION
    Evaluates `expr`. If it succeeds, its value is returned unchanged. If it stops with an error, `attempt`
    returns a `Failed` value holding the error's code, message and line instead of ending the script.
    It is the one way to continue after an error: Zeff has no try/catch, because an analysis that fails at
    the top level should stop where it fails, and it is repeated work that needs protection.

    A `Failed` value may sit in any `List<T>`, so `[for ...]` with `attempt` gives a list in which some entries
    failed; `report`, `len` and the other collecting functions skip them and say how many. Using a `Failed`
    value anywhere else is an error that reports the original failure (Z0220).

ARGUMENTS
    expr        Any expression, evaluated once: a model fit, a chain, a function call.

VALUE
    The value of `expr`, or a `Failed`. `x.is(Failed)` tells which; after `if x.is(Failed) { continue }` the
    checker knows `x` succeeded for the rest of the block (Zeff Specification, section 3.7, Narrowing).

WARNINGS AND NOTES
    W0801   failed draws or fits skipped by a collecting function, with the count and the first codes
    Z0220   a Failed value used outside a list

A COMMON MISTAKE
    Using the result as if the expression had succeeded. A `Failed` value is refused wherever a value is
    needed, and the error names the failure it holds:

    > attempt(sims.where(rep == 0).glm(y ~ treat + x, family: binomial)).coef()
    error[Z0220]: a failed value cannot be used as an argument: it failed with Z0801 at line 1: no row remains to fit the model: the frame has no rows
      = help: test it first with `x.is(Failed)`, or keep it in a list for `report` and `len` to skip with a count (section 5.4)

    Test it first, `if f.is(Failed) { ... }`, or keep the results in a list.

COMING FROM R OR STATA
    Equivalents: R tryCatch(); Stata capture.

    Zeff            R                                       Stata
    attempt(expr)   tryCatch(expr, error = function(e) e)   capture noisily command
    x.is(Failed)    inherits(x, "error")                    _rc != 0

    R's `try` and Stata's `capture` leave a flag to be remembered and tested; here the failure is a value
    that stands where the result would, and using it untested is itself an error.

ACCURACY
    Exact.

SEE ALSO
    is, iterate, error, warn, on_fail: (shared argument)

EXAMPLES
    // Two hundred fits in a loop, three of which fail: the loop carries on, and `report` counts them.
    > let sims = import("zeff:sims.csv")
    > let fits = [for s in 1..200 {
    >     attempt(sims.where(rep == s).glm(y ~ treat + x, family: binomial))
    > }]
    > report(fits)
    warning[W0801]: 3 of 200 fits failed (Z0803 at line 3) and were skipped
```

## `zeff help adjustment_sets`

a causal-graph method with its canonical order and references

```
adjustment_sets — Sufficient adjustment sets from a causal graph          identify · tier 3 · since 0.10.0 · 1.0 · exact

USAGE
    g.adjustment_sets(exposure: Var, outcome: Var, type: minimal | all = minimal)

DESCRIPTION
    Lists the sets of observed variables that, conditioned on, identify the total effect of `exposure` on
    `outcome` in the graph `g`, by the generalized adjustment criterion: a set blocks every non-causal path
    and contains no descendant of a variable on a causal path. Latent variables (declared `latent`) cannot be
    in a set; selection variables (declared `selected`) are always conditioned on, since the data were.

    When no set of observed variables exists, the result says so and names what would help: an instrument
    (a cause of the exposure with no other path to the outcome) or a front-door variable.

ARGUMENTS
    g           Dag. The causal graph, from a `dag { }` block.
    exposure:   Var. The cause whose effect is wanted; a node of `g`.
    outcome:    Var. The effect; a node of `g`.
    type:       minimal | all = minimal. Minimal sets only, or every valid set.

VALUE
    A `List<Vars>`, possibly empty, printed one set per line with `{}` for "adjust for nothing"; a note
    explains an empty list.

DETAILS
    Computed on the moral ancestor graph with d-separation tests. Sets are listed in increasing size, then in
    alphabetical order, so the output is canonical. A bidirected edge `a <-> b` stands for an unobserved
    common cause.

WARNINGS AND NOTES
    Note    no adjustment set among observed variables, with the suggestion
    Z0101   exposure: or outcome: is not a node of the graph

ACCURACY
    Exact: the same sets as dagitty 0.3-4 adjustmentSets() on its test graphs and on 2,000 random DAGs.
    Last run: 0.10.0, build 3 310, 2,048 of 2,048 cases passed on all four targets.

REFERENCES
    Shpitser, VanderWeele & Robins (2010), On the validity of covariate adjustment for estimating causal effects, UAI 2010.
    Perković, Textor, Kalisch & Maathuis (2018), Complete graphical characterization and construction of adjustment sets, JMLR 18.
    Textor et al. (2016), Robust causal inference using directed acyclic graphs: the R package dagitty, IJE 45, 1887–1894.

SEE ALSO
    dag, implied_independencies, check, ivreg, exclusion_check

EXAMPLES
    > let g = dag {
    >     ability -> training -> productivity
    >     ability -> productivity
    >     training -> tenure -> productivity
    >     [age, region] -> training
    >     latent ability
    > }
    > g.adjustment_sets(exposure: training, outcome: productivity)
    no adjustment set among observed variables: ability is latent and confounds training -> productivity
      consider an instrument for training, or a front-door variable
    > g.adjustment_sets(exposure: tenure, outcome: productivity)
    { training }
```

## `zeff help tabulate`

the contract of 0.8.0 (open decision #103): In brief before Usage, Missing values after Value, a heading inside Details, A common mistake with the diagnostic the engine prints, Coming from R or Stata as a table under the catalogue's equivalents, Related tasks, and a caption above every example

```
tabulate — Frequencies and percentages, one- or two-way                data · tier 1 · since 0.7.0 · 1.0 · closed form

IN BRIEF
    - `wages.tabulate(industry)`: counts and percentages of one variable, with a total.
    - `wages.tabulate([female, union])`: a cross-table of counts.
    - `wages.tabulate([female, union], percent: row)`: the same, with the percentage of each row under its
      counts.

USAGE
    data.tabulate(vars, percent: row | col | total)

DESCRIPTION
    One variable gives a table of its values with their counts (`N`) and the percentage of all rows (`%`),
    then a `Total` row. Two variables give a cross-table: the first variable's values down the side, the
    second's across the top, a `Total` column and a `Total` row. With `percent:`, each row of counts is
    followed by a row of percentages: of its row (`row`), of its column (`col`) or of every row of the frame
    (`total`); the `Total` row and column show the matching marginal percentages.

    Only the values present appear, in the order of section 3.7 of the Language Reference (`false` before
    `true`, numbers numerically, text by code point, dates in time order, categories by level); a labelled
    variable prints its labels. Counts print with thousands separators and percentages with one decimal.

ARGUMENTS
    data        Frame. The frame.
    vars        Vars. One variable, `industry`, or two in a list, `[industry, female]`.
    percent:    row | col | total. For two variables, the percentages shown under each row of counts: `row`,
                `col` or `total`. Omitted, counts alone. One variable's table always shows the percentage of
                all rows, so `row` and `col` are refused with one variable.

VALUE
    A `Table`, which prints as shown and can be passed to `print`.

MISSING VALUES
    `NA` is shown as a value of its own, in the last row or column, so that the counts always add up to the
    rows of the frame; no row is dropped and no note is printed. R's `table` and Stata's `tabulate` show it
    only when asked, with `useNA = "ifany"` and `missing`.

DETAILS
    What can be tabulated

    Any variable that can group rows can be tabulated: Bool, Int, Float, Str, Date, DateTime or Category; an
    opaque value cannot. A percentage with nothing to divide by (a frame with no rows) prints `NA`. The
    table has no `weights:` in 1.0 (open decision #53 places frequency weights at 1.1).

WARNINGS AND NOTES
    Z0101   no such variable in the frame
    Z0102   a variable that differs only in case from the name written
    Z0401   more than two variables; an opaque variable; percent: row or col with one variable

A COMMON MISTAKE
    Asking for row percentages of one variable. A one-way table already shows each value's share of all
    rows, so `percent:` is refused there, and the message gives the two ways out:

    > wages.tabulate(industry, percent: row)
    error[Z0401]: `percent: row` and `percent: col` need two variables; one variable's table always shows the percentage of all rows
      = help: give two variables, `tabulate([industry, female], percent: row)`, or leave `percent:` out

    Leave `percent:` out for one variable, or give two: `wages.tabulate([industry, female], percent: row)`.

COMING FROM R OR STATA
    Equivalents: R table(); Stata tabulate.

    Zeff                                      R                                         Stata
    tabulate(industry)                        table(d$industry, useNA = "ifany")        tabulate industry, missing
    tabulate([female, union], percent: row)   prop.table(table(d$female, d$union), 1)   tabulate female union, row
    percent: col                              prop.table(table(d$female, d$union), 2)   tabulate female union, column
    percent: total                            prop.table(table(d$female, d$union))      tabulate female union, cell

    Missing values are counted without asking, as the section above says. A labelled variable prints its
    labels, as Stata does; R prints the codes unless the variable is a factor. Percentages print with one
    decimal, and the `Total` row and column carry the marginal percentages.

RELATED TASKS
    - To test whether two variables are independent, use `chi2_test`.
    - For counts by group as a frame, use `aggregate`.
    - For means and spreads instead of counts, use `summary`.
    - To count the distinct values, use `n_distinct`.

ACCURACY
    Closed form.

REFERENCES
    Agresti (2013), Categorical Data Analysis, third edition, Wiley, chapter 2 (two-way contingency
    tables and their marginal and conditional distributions).

SEE ALSO
    summary, describe, aggregate, n_distinct, chi2_test

EXAMPLES
    // A labelled variable: its labels, its counts and each one's share of all rows.
    > let wages = dataset(wages)
    > wages.tabulate(union)
    tabulate · wages · union

      union        N       %
      no      12,556    68.2
      yes      5,856    31.8
      ──────────────────────
      Total   18,412   100.0

    // A category with twelve levels, in the order of its levels.
    > wages.tabulate(industry)
    tabulate · wages · industry

      industry                     N       %
      Agriculture                535     2.9
      Mining                     200     1.1
      Utilities                  370     2.0
      Construction             1,477     8.0
      Manufacturing            2,574    14.0
      Retail                   2,871    15.6
      Transport                  976     5.3
      Finance                  1,560     8.5
      Services                 2,637    14.3
      Education                1,693     9.2
      Health                   2,098    11.4
      Public administration    1,421     7.7
      ──────────────────────────────────────
      Total                   18,412   100.0

    // A cross-table with the percentage of each row under its counts.
    > wages.tabulate([female, union], percent: row)
    tabulate · wages · female × union · row %

      female       no     yes    Total
      male      6,516   2,975    9,491
                 68.7    31.3    100.0
      female    6,040   2,881    8,921
                 67.7    32.3    100.0
      ────────────────────────────────
      Total    12,556   5,856   18,412
                 68.2    31.8    100.0

    // Missing values are a value of their own, last, so that the counts add up to the rows.
    > let small = frame(sex = ["f", "m", NA, "f", "m", "f"], smoker = [true, false, true, NA, false, false])
    > small.tabulate([sex, smoker])
    tabulate · small · sex × smoker

      sex     false   true   NA   Total
      f           1      1    1       3
      m           2      0    0       2
      NA          0      1    0       1
      ─────────────────────────────────
      Total       3      2    1       6
```

## A project function: the `///` source

```
/// Share of rows where a variable is missing.
///
/// Counts the rows where `var` is `NA` and divides by the row count. An empty frame gives `NA`
/// with a warning, never a division by zero.
///
/// data: The frame to check.
/// var: The variable whose missing values are counted.
/// returns: The share, a Float between 0 and 1.
/// see also: n_missing, is_missing, drop_missing
///
/// > import("zeff:wages.csv").share_missing(educ)
/// 0.0125
fn share_missing(data: Frame, var: Var) -> Float {
    data.where(is_missing(var)).n_rows() / data.n_rows()
}
```
