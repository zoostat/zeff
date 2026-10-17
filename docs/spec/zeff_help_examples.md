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
    ivreg, logistic, probit, report, derive, test, compare, interact

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
    data.t_test(var, mu: Float = 0, level: Float = 0.95)
    data.t_test(var, group: Var, equal_var: Bool = false, effect: d | g = g, level: Float = 0.95)
    data.t_test(a, b, paired: true, level: Float = 0.95)

DESCRIPTION
    Three tests under one name, chosen by the arguments given: a one-sample test of the mean of `var`
    against `mu`, a two-group comparison of `var` between the two levels of `group`, or a paired comparison
    of `a` and `b` row by row. Each prints the means, the difference with its confidence interval, t, the
    degrees of freedom, the p-value and an effect size.

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
    level:      Float = 0.95. Confidence level.                                                     (shared)
    weights:    Var. Frequency or analytic weights; sampling weights are refused (Z0411) and point to
                regression with robust errors.                                                      (shared)

VALUE
    A `TestResult`: statistic, df, p_value, estimate (the mean or the difference), ci, effect_size and
    n_obs, with the result interface (summary, estimates, export, report).

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
    wilcoxon_test, anova, proportion_test, reg, power.t_test (from 1.9)

EXAMPLES
    > let wages = import("zeff:wages.csv")
    > wages.t_test(lwage, group: female)
    t_test · Welch two-sample · lwage by female

      group      N      mean      sd
      male    9,604    3.214   0.512
      female  8,597    3.061   0.498

    difference (male − female) = 0.153   95% CI [0.138, 0.168]
    t = 20.42   df = 18,074.2   p < 0.001
    Hedges' g = 0.30   95% CI [0.27, 0.33]
    > wages.t_test(lwage, mu: 3)
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

WARNINGS AND NOTES
    Note    rows dropped because the condition was NA, with the count and a statement that lists them
    Note    no row kept: the result is empty

ACCURACY
    Exact: the same rows, in the same order, as dplyr::filter() once its NA rows are removed; part of
    every verb test. Last run: 0.2.0, build 611, 22 of 22 cases passed on all four targets.

SEE ALSO
    drop_missing, select, sort, distinct, sample, is_missing

EXAMPLES
    > let wages = import("zeff:wages.csv")
    > wages.where(year >= 2010 && female == 1).n_rows()
    4217
    > wages.where(exper > 10).n_rows()
    note: 13 rows dropped where `exper > 10` was NA; `wages.where(is_missing(exper))` lists them
    7902
```

## `zeff help merge`

a verb whose errors are part of its contract

```
merge — Join two frames on key variables                                     data · tier 1 · since 0.6.0 · 1.0 · exact

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

WARNINGS AND NOTES
    Note    matched and unmatched rows on each side, with counts
    Z0603   the keys form a many-to-many relation, or relation: does not hold
    Z0604   a non-key column exists in both frames and no suffixes: is given
    Z0101   a key variable is missing from one frame

ACCURACY
    Exact: the same rows and values as dplyr 1.1 joins with NA keys excluded, in the order stated above.
    Last run: 0.6.0, build 1 902, 31 of 31 cases passed on all four targets.

SEE ALSO
    append, to_long, to_wide, aggregate, rename

EXAMPLES
    > let firms = import("zeff:firms.csv")
    > let wages = import("zeff:wages.csv").merge(firms, on: firm, relation: many_to_one)
    merge: 18,201 rows matched; 0 left rows unmatched; 3 right rows unmatched (firms with no workers)
    > wages.merge(firms, on: firm)
    error[Z0604]: columns `size` exist in both frames and are not keys
      = help: give suffixes: ["_w", "_f"], or drop one of them first
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

ACCURACY
    Exact.

SEE ALSO
    is, iterate, error, warn, on_fail: (shared argument)

EXAMPLES
    > let sims = import("zeff:sims.csv")
    > let fits = [for s in 1..200 {
    >     attempt(sims.where(rep == s).logistic(y ~ treat + x))
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
