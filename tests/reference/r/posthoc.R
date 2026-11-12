# R's answers for posthoc (release 0.7.0, steps 19 and 19b). First, step 19: the pairwise
# comparisons of the group means after a one-way anova, in R 4.6.0. Each pair is written as the first group minus the second, in
# the order (1, 2), (1, 3), ..., (2, 3), ..., the groups in the order of factor() with its levels
# sorted by code point.
#
# - Tukey's HSD (the Tukey-Kramer form for unequal sizes): TukeyHSD(aov()), whose rows are the
#   second group minus the first, so the differences change sign and the limits change places.
#   Its p-values come from ptukey and its limits from qtukey, which carry R's own error (the help
#   page of dist.tukey gives its size), so the comparison tests hold them to that and not to 1e-9.
# - The pairwise t-tests: pairwise.t.test() with pool.sd = TRUE (the pooled variance of all the
#   groups, N - k degrees of freedom) and pool.sd = FALSE (Welch's test of each pair), under each
#   method of p.adjust(); Welch's tests need two values and some spread in every group.
# - Sidak: emmeans' pairs(adjust = "sidak") on the pooled tests, and the formula 1 - (1 - p)^m it
#   uses written out on Welch's tests.
# - Scheffe: DescTools::ScheffeTest(), which compares t^2 / (k - 1) with F on k - 1 and N - k
#   degrees of freedom; its rows are the second group minus the first as well.
# - Intervals at 95% for the single-step methods: TukeyHSD and ScheffeTest for their own;
#   emmeans' confint(adjust = "bonferroni" | "sidak" | "none") on the pooled variance; and
#   t.test(conf.level = ) at the level each method gives a single comparison for Welch's tests.
#
# The cases are those of anova.R: 2 to 6 groups of unequal sizes and variances, ties, a group of
# one observation (pooled tests only), NA in the variable and in the groups, text groups, a large
# sample, groups far apart, two groups; and hours by industry on the shipped wages, 12 groups and
# 66 comparisons.
#
# Then, step 19b: the comparisons of the estimated marginal means of one term after a factorial
# anova, which are emmeans' pairs(emmeans(lm(), ~ term)) under each of its adjustments (tukey,
# scheffe, sidak, bonferroni, holm, hochberg, hommel, BH, BY, none), with confint() for the
# single-step ones. The means are written in our order, the first factor of the term changing
# slowest, and a mean that cannot be estimated is left out before the pairs are formed, so the
# family is the means that remain, as emmeans counts it. The layouts are those of the factorial
# cases of anova.R: balanced and unbalanced, the terms in both orders, a covariate alone and in
# an interaction with the factor compared, a covariate inside log(), three factors, an empty
# cell with and without the interaction, rows with NA, frequency weights against the rows
# written out and analytic weights. Under analytic weights a covariate is held at its weighted
# mean, which emmeans is told by `at =`; its own default there is the unweighted mean. On a
# balanced layout the comparisons of a main effect are also checked against TukeyHSD(aov()).
# Run with `Rscript tests/reference/r/posthoc.R`.
source(file.path(dirname(sub("^--file=", "", commandArgs(FALSE)[grepl("^--file=", commandArgs(FALSE))])), "common.R"))
options(warn = -1)
.libPaths(c(file.path(here(), "library"), .libPaths()))
suppressMessages({
  library(DescTools)
  library(emmeans)
})

LEVEL <- 0.95
series <- function(n, digits = 6) round(3 * sin(seq_len(n) * 1.3) + 0.4 + seq_len(n) / 17, digits)
same <- function(a, b, tolerance = 1e-10) all(abs(a - b) <= tolerance * pmax(1, abs(a), abs(b)))

# The lower triangle of pairwise.t.test()'s matrix, column by column: the pairs in our order.
triangle <- function(p) unname(p[lower.tri(p, diag = TRUE)])

comparisons <- function(x, g, data = NULL) {
  ok <- complete.cases(x, g)
  xf <- x[ok]
  gf <- factor(g[ok], levels = sort(unique(g[ok]), method = "radix"))
  lev <- levels(gf)
  k <- nlevels(gf)
  m <- k * (k - 1) / 2
  pair <- combn(k, 2)
  first <- pair[1, ]; second <- pair[2, ]
  n <- as.vector(table(gf))
  means <- as.vector(tapply(xf, gf, mean))
  variances <- as.vector(tapply(xf, gf, var))
  difference <- means[first] - means[second]
  frame <- data.frame(xf = xf, gf = gf)
  fit <- aov(xf ~ gf, data = frame)
  df <- fit$df.residual
  mse <- sum(fit$residuals^2) / df
  se <- sqrt(mse * (1 / n[first] + 1 / n[second]))
  reversed <- paste(lev[second], lev[first], sep = "-")

  # Tukey's HSD.
  tukey <- TukeyHSD(fit, conf.level = LEVEL)$gf
  stopifnot(identical(rownames(tukey), reversed), same(-tukey[, "diff"], difference))

  # Scheffe's test.
  scheffe <- ScheffeTest(fit, conf.level = LEVEL)$gf
  stopifnot(identical(rownames(scheffe), reversed), same(-scheffe[, "diff"], difference))

  # The pooled pairwise tests and their adjustments.
  pooled_p <- function(method) triangle(pairwise.t.test(xf, gf, p.adjust.method = method, pool.sd = TRUE)$p.value)
  em <- emmeans(lm(xf ~ gf, data = frame), "gf")
  raw <- as.data.frame(pairs(em, adjust = "none"))
  stopifnot(same(raw$estimate, difference), same(raw$SE, se), same(raw$df, df),
            same(raw$p.value, pooled_p("none")))
  limits <- function(adjust) {
    ci <- as.data.frame(confint(pairs(em), level = LEVEL, adjust = adjust))
    list(lower = as.list(num(ci$lower.CL)), upper = as.list(num(ci$upper.CL)))
  }
  pooled <- list(
    df = num(df), se = as.list(num(se)), t = as.list(num(difference / se)),
    none = as.list(num(pooled_p("none"))),
    bonferroni = as.list(num(pooled_p("bonferroni"))),
    holm = as.list(num(pooled_p("holm"))),
    hochberg = as.list(num(pooled_p("hochberg"))),
    hommel = as.list(num(pooled_p("hommel"))),
    bh = as.list(num(pooled_p("BH"))),
    by = as.list(num(pooled_p("BY"))),
    sidak = as.list(num(as.data.frame(pairs(em, adjust = "sidak"))$p.value)),
    ci_none = limits("none"), ci_bonferroni = limits("bonferroni"), ci_sidak = limits("sidak"))

  # Welch's test of each pair, where every group has two values and some spread.
  welch <- NULL
  if (all(n >= 2) && all(variances > 0)) {
    welch_p <- function(method) triangle(pairwise.t.test(xf, gf, p.adjust.method = method, pool.sd = FALSE)$p.value)
    tests <- function(level) lapply(seq_len(m), function(i) {
      t.test(xf[gf == lev[first[i]]], xf[gf == lev[second[i]]], var.equal = FALSE, conf.level = level)
    })
    plain <- tests(LEVEL)
    p <- sapply(plain, function(t) t$p.value)
    stopifnot(same(p, welch_p("none")))
    interval <- function(level) {
      ts <- tests(level)
      list(lower = as.list(num(sapply(ts, function(t) t$conf.int[1]))),
           upper = as.list(num(sapply(ts, function(t) t$conf.int[2]))))
    }
    welch <- list(
      df = as.list(num(sapply(plain, function(t) unname(t$parameter)))),
      se = as.list(num(sapply(plain, function(t) t$stderr))),
      t = as.list(num(sapply(plain, function(t) unname(t$statistic)))),
      none = as.list(num(p)),
      bonferroni = as.list(num(welch_p("bonferroni"))),
      holm = as.list(num(welch_p("holm"))),
      hochberg = as.list(num(welch_p("hochberg"))),
      hommel = as.list(num(welch_p("hommel"))),
      bh = as.list(num(welch_p("BH"))),
      by = as.list(num(welch_p("BY"))),
      sidak = as.list(num(pmin(1, 1 - (1 - p)^m))),
      ci_none = interval(LEVEL), ci_bonferroni = interval(1 - (1 - LEVEL) / m),
      ci_sidak = interval(LEVEL^(1 / m)))
  }

  out <- if (is.null(data)) {
    list(x = as.list(num(x)), g = as.list(ifelse(is.na(g), "NA", as.character(g))),
         text_groups = is.character(g))
  } else {
    data
  }
  out <- c(out, list(
    n = as.integer(sum(ok)), groups = as.integer(k),
    first = as.list(lev[first]), second = as.list(lev[second]),
    difference = as.list(num(difference)),
    tukey = list(p = as.list(num(tukey[, "p adj"])),
                 lower = as.list(num(-tukey[, "upr"])), upper = as.list(num(-tukey[, "lwr"]))),
    scheffe = list(p = as.list(num(scheffe[, "pval"])),
                   lower = as.list(num(-scheffe[, "upr.ci"])), upper = as.list(num(-scheffe[, "lwr.ci"]))),
    pooled = pooled))
  if (!is.null(welch)) out$welch <- welch
  out
}

cases <- list()
# Distinct values, k groups of unequal sizes and unequal variances.
for (k in 2:6) {
  n <- 7 * k + 3
  g <- rep(seq_len(k), length.out = n)
  x <- series(n) * (0.5 + 0.4 * g) + 0.3 * g
  cases[[length(cases) + 1]] <- comparisons(x, g)
}
# Ties: values rounded to one digit, then to integers.
cases[[length(cases) + 1]] <- comparisons(series(24, digits = 1), rep(1:3, each = 8))
cases[[length(cases) + 1]] <- comparisons(round(series(30, digits = 0)), rep(1:4, length.out = 30))
# A group of one observation: the pooled tests only.
cases[[length(cases) + 1]] <- comparisons(series(12), c(rep(1, 6), rep(2, 5), 3))
# NA in the variable and in the groups.
x <- series(20); x[c(3, 11)] <- NA
g <- rep(1:3, length.out = 20); g[c(7, 11)] <- NA
cases[[length(cases) + 1]] <- comparisons(x, g)
# Text groups, with ties.
cases[[length(cases) + 1]] <- comparisons(series(18, digits = 1), rep(c("north", "south", "east"), each = 6))
# A large sample.
cases[[length(cases) + 1]] <- comparisons(series(300, digits = 2), rep(1:5, length.out = 300))
# Groups far apart: tiny p-values.
g <- rep(1:4, each = 10)
cases[[length(cases) + 1]] <- comparisons(series(40) + 6 * g, g)
# Groups apart by less: p-values on both sides of 0.05, so the step-wise methods differ.
g <- rep(1:5, length.out = 60)
cases[[length(cases) + 1]] <- comparisons(series(60) + 1.1 * g, g)
# Two groups: every method gives the p-value of the t-test.
cases[[length(cases) + 1]] <- comparisons(series(16) * rep(c(1, 2), each = 8), rep(c(1, 2), each = 8))
cases[[length(cases) + 1]] <- comparisons(series(15), rep(c(1, 2), length.out = 15))

# The shipped wages: hours by industry, 12 groups of unequal sizes.
root <- normalizePath(file.path(here(), "..", "..", ".."))
wages <- read_dataset(file.path(root, "packages", "tier1", "datasets", "data", "wages.csv"))
shipped <- comparisons(wages$hours, wages$industry,
                       data = list(file = "packages/tier1/datasets/data/wages.csv", x = "hours", g = "industry"))

# ----- after a factorial anova (step 19b) -----

# The layout of the factorial cases of anova.R.
layout <- function(n) {
  i <- seq_len(n)
  data.frame(
    y = round(3 * sin(i * 1.3) + 0.4 + i / 17, 4),
    a = c("p", "p", "q", "p", "r", "q", "p")[1 + (i - 1) %% 7],
    b = c("u", "v", "w", "z", "u", "v", "u", "w", "z", "u", "v")[1 + (i - 1) %% 11],
    c = c("m", "n", "n", "m", "n")[1 + (i - 1) %% 5],
    x = round(2 * cos(i * 0.7) + i / 20, 4),
    z = round(sin(i * 0.31) * 3 + (i %% 4), 4),
    stringsAsFactors = FALSE)
}

# The comparisons of the marginal means of `term` (its factors, in order) in the model `text`
# on `d`; `covariates` are the numeric variables of the model, held at their weighted means
# under analytic weights.
marginal <- function(d, text, factors, term, covariates = character(0), weights = NULL,
                     weight_type = NULL, expand = FALSE, hsd = FALSE) {
  f <- as.formula(text)
  vars <- all.vars(f)
  used <- d[complete.cases(d[, vars, drop = FALSE]), ]
  w <- NULL
  if (!is.null(weights)) {
    used <- used[!is.na(used[[weights]]) & used[[weights]] > 0, ]
    if (expand) {
      used <- used[rep(seq_len(nrow(used)), used[[weights]]), ]
    } else {
      w <- used[[weights]] * nrow(used) / sum(used[[weights]])
    }
  }
  for (name in factors) used[[name]] <- factor(used[[name]])
  used$.w <- w
  m <- if (is.null(w)) lm(f, used) else lm(f, used, weights = .w)
  at <- list()
  if (!is.null(w)) for (name in covariates) at[[name]] <- weighted.mean(used[[name]], w)
  em <- suppressMessages(if (length(at) == 0) emmeans(m, term) else emmeans(m, term, at = at))
  grid <- as.data.frame(em)
  # Our order: the first factor of the term slowest.
  position <- lapply(term, function(name) as.integer(grid[[name]]))
  ours <- do.call(order, position)
  label <- function(i) paste(sapply(term, function(name) as.character(grid[[name]][i])), collapse = ":")
  labels <- sapply(ours, label)
  estimable <- !is.na(grid$emmean[ours])
  kept <- em[ours[estimable]]
  k <- sum(estimable)
  pair <- combn(k, 2)
  raw <- as.data.frame(pairs(kept, adjust = "none"))
  stopifnot(nrow(raw) == ncol(pair), all(!is.na(raw$p.value)))
  p <- function(adjust) as.list(num(as.data.frame(summary(pairs(kept), adjust = adjust))$p.value))
  limits <- function(adjust) {
    ci <- as.data.frame(confint(pairs(kept), level = LEVEL, adjust = adjust))
    list(lower = as.list(num(ci$lower.CL)), upper = as.list(num(ci$upper.CL)))
  }
  column <- function(name) {
    v <- d[[name]]
    if (is.character(v)) as.list(ifelse(is.na(v), "NA", v)) else as.list(num(v))
  }
  out <- list(
    formula = text, factors = as.list(factors), term = as.list(term),
    columns = setNames(lapply(c(vars, weights), column), c(vars, weights)),
    weights = if (is.null(weights)) "NA" else weights,
    weight_type = if (is.null(weight_type)) "NA" else weight_type,
    means = as.list(labels), estimable = as.list(estimable),
    estimates = as.list(ifelse(estimable, num(grid$emmean[ours]), "NA")),
    first = as.list(labels[estimable][pair[1, ]]), second = as.list(labels[estimable][pair[2, ]]),
    df = num(df.residual(m)),
    difference = as.list(num(raw$estimate)), se = as.list(num(raw$SE)), t = as.list(num(raw$t.ratio)),
    none = p("none"), tukey = p("tukey"), scheffe = p("scheffe"), sidak = p("sidak"),
    bonferroni = p("bonferroni"), holm = p("holm"), hochberg = p("hochberg"), hommel = p("hommel"),
    bh = p("BH"), by = p("BY"),
    ci_none = limits("none"), ci_tukey = limits("tukey"), ci_scheffe = limits("scheffe"),
    ci_sidak = limits("sidak"), ci_bonferroni = limits("bonferroni"))
  if (hsd) {
    # A balanced layout: TukeyHSD of the main effect, whose rows are the second level minus the
    # first.
    stopifnot(length(term) == 1, all(estimable))
    table <- TukeyHSD(aov(f, used), term, conf.level = LEVEL)[[term]]
    lev <- levels(used[[term]])
    stopifnot(identical(rownames(table), paste(lev[pair[2, ]], lev[pair[1, ]], sep = "-")))
    plain <- function(v) as.list(num(unname(v)))
    out$hsd <- list(difference = plain(-table[, "diff"]), p = plain(table[, "p adj"]),
                    lower = plain(-table[, "upr"]), upper = plain(-table[, "lwr"]))
  }
  out
}

marginal_cases <- list()
add <- function(case) marginal_cases[[length(marginal_cases) + 1]] <<- case
# Balanced: three levels by two, four observations in each of the six cells.
balanced <- layout(24)
balanced$a <- rep(c("p", "q", "r"), each = 8)
balanced$b <- rep(rep(c("u", "v"), each = 4), 3)
add(marginal(balanced, "y ~ a * b", c("a", "b"), "a", hsd = TRUE))
add(marginal(balanced, "y ~ a * b", c("a", "b"), "b", hsd = TRUE))
add(marginal(balanced, "y ~ a * b", c("a", "b"), c("a", "b")))
# Unbalanced: each main effect and the cells, the terms in both orders, the main effects alone.
d <- layout(60)
add(marginal(d, "y ~ a * b", c("a", "b"), "a"))
add(marginal(d, "y ~ a * b", c("a", "b"), "b"))
add(marginal(d, "y ~ a * b", c("a", "b"), c("a", "b")))
add(marginal(d, "y ~ b * a", c("a", "b"), "a"))
add(marginal(d, "y ~ a + b", c("a", "b"), "a"))
add(marginal(d, "y ~ a + b", c("a", "b"), "b"))
# One factor alone, and one factor of two levels: a single comparison.
add(marginal(d, "y ~ a", "a", "a"))
add(marginal(d, "y ~ a * c", c("a", "c"), "c"))
# A covariate beside the factors, and a covariate that interacts with the factor compared.
add(marginal(d, "y ~ a * b + x", c("a", "b"), "a", covariates = "x"))
add(marginal(d, "y ~ a * b + x", c("a", "b"), c("a", "b"), covariates = "x"))
add(marginal(d, "y ~ a * x", "a", "a", covariates = "x"))
add(marginal(d, "y ~ a * x + b", c("a", "b"), "a", covariates = "x"))
add(marginal(d, "y ~ a * x + b", c("a", "b"), "b", covariates = "x"))
# A covariate inside a function: held at the logarithm of its mean, not the mean of its
# logarithm.
shifted <- d
shifted$x <- round(shifted$x + 5, 4)
add(marginal(shifted, "y ~ a * log(x)", "a", "a", covariates = "x"))
# Three factors with every interaction: a main effect, a two-way term and the cells.
big <- layout(180)
add(marginal(big, "y ~ a * b * c", c("a", "b", "c"), "b"))
add(marginal(big, "y ~ a * b * c", c("a", "b", "c"), c("a", "c")))
add(marginal(big, "y ~ a * b * c", c("a", "b", "c"), c("c", "a")))
add(marginal(big, "y ~ a * b * c", c("a", "b", "c"), c("a", "b", "c")))
# An empty cell, no row with a = r and b = z: with the interaction the means that meet it
# cannot be estimated; without it every mean can.
empty <- d[!(d$a == "r" & d$b == "z"), ]
add(marginal(empty, "y ~ a * b", c("a", "b"), "a"))
add(marginal(empty, "y ~ a * b", c("a", "b"), "b"))
add(marginal(empty, "y ~ a * b", c("a", "b"), c("a", "b")))
add(marginal(empty, "y ~ a + b", c("a", "b"), "a"))
# NA in the outcome, in a factor and in the covariate.
holes <- d
holes$y[c(4, 17)] <- NA; holes$a[c(9, 17)] <- NA; holes$x[31] <- NA
add(marginal(holes, "y ~ a * b + x", c("a", "b"), "b", covariates = "x"))
# Frequency weights, against the rows written out; a zero weight leaves its row out.
weighted <- layout(40)
weighted$w <- c(1, 2, 1, 3, 1, 1, 2, 0, 1, 4)[1 + (seq_len(40) - 1) %% 10]
add(marginal(weighted, "y ~ a * c + x", c("a", "c"), "a", covariates = "x", weights = "w", weight_type = "frequency", expand = TRUE))
add(marginal(weighted, "y ~ a * x + c", c("a", "c"), "a", covariates = "x", weights = "w", weight_type = "frequency", expand = TRUE))
# Analytic weights, the covariate at its weighted mean.
weighted$v <- round(0.5 + (seq_len(40) %% 7) / 3, 4)
add(marginal(weighted, "y ~ a * c + x", c("a", "c"), c("a", "c"), covariates = "x", weights = "v", weight_type = "analytic"))
add(marginal(weighted, "y ~ a * x + c", c("a", "c"), "a", covariates = "x", weights = "v", weight_type = "analytic"))

write_json(list(reference = c(reference_block(),
                              list(DescTools = as.character(packageVersion("DescTools")),
                                   emmeans = as.character(packageVersion("emmeans")))),
                cases = cases, wages = shipped, marginal = marginal_cases),
           file.path(here(), "posthoc.json"))
