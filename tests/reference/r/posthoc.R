# R's answers for posthoc (release 0.7.0, step 19): the pairwise comparisons of the group means
# after a one-way anova, in R 4.6.0. Each pair is written as the first group minus the second, in
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
# 66 comparisons. Run with `Rscript tests/reference/r/posthoc.R`.
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

write_json(list(reference = c(reference_block(),
                              list(DescTools = as.character(packageVersion("DescTools")),
                                   emmeans = as.character(packageVersion("emmeans")))),
                cases = cases, wages = shipped),
           file.path(here(), "posthoc.json"))
