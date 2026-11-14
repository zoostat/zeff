# R's intervals for posthoc at levels other than 95% (release 0.7.0, step 38f; open decision #105).
# The one-way cases of posthoc.R, each at the levels 0.90 and 0.99:
# - Tukey's HSD: TukeyHSD(aov(), conf.level = ), whose rows are the second group minus the first,
#   so the limits are turned round.
# - Scheffe: DescTools::ScheffeTest(conf.level = ), turned round likewise.
# - none, Bonferroni and Sidak on the pooled variance, by their formulas: the difference plus and
#   minus the t quantile on the pooled degrees of freedom times the standard error, the tail on one
#   side being (1 - level) / 2, that divided by the number of comparisons, and
#   (1 - level^(1 / m)) / 2.
# - The same three for Welch's test of each pair, where every group has two values and some
#   spread: t.test(conf.level = ) at the level, at 1 - (1 - level) / m and at level^(1 / m).
# Run with `Rscript tests/reference/r/posthoc_level.R`.
source(file.path(dirname(sub("^--file=", "", commandArgs(FALSE)[grepl("^--file=", commandArgs(FALSE))])), "common.R"))
options(warn = -1)
.libPaths(c(file.path(here(), "library"), .libPaths()))
suppressMessages(library(DescTools))

LEVELS <- c(0.90, 0.99)
series <- function(n, digits = 6) round(3 * sin(seq_len(n) * 1.3) + 0.4 + seq_len(n) / 17, digits)
same <- function(a, b, tolerance = 1e-10) all(abs(a - b) <= tolerance * pmax(1, abs(a), abs(b)))
limits <- function(lower, upper) list(lower = as.list(num(lower)), upper = as.list(num(upper)))

comparisons <- function(x, g) {
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
  welched <- all(n >= 2) && all(variances > 0)

  at <- lapply(LEVELS, function(level) {
    tukey <- TukeyHSD(fit, conf.level = level)$gf
    stopifnot(identical(rownames(tukey), reversed), same(-tukey[, "diff"], difference))
    scheffe <- ScheffeTest(fit, conf.level = level)$gf
    stopifnot(identical(rownames(scheffe), reversed), same(-scheffe[, "diff"], difference))
    # The tail one side of an interval leaves, for one comparison of m.
    tails <- list(none = (1 - level) / 2, bonferroni = (1 - level) / 2 / m, sidak = (1 - level^(1 / m)) / 2)
    pooled <- lapply(tails, function(tail) {
      half <- qt(1 - tail, df) * se
      limits(difference - half, difference + half)
    })
    out <- list(level = num(level),
                tukey = limits(-tukey[, "upr"], -tukey[, "lwr"]),
                scheffe = limits(-scheffe[, "upr.ci"], -scheffe[, "lwr.ci"]),
                pooled = pooled)
    if (welched) {
      out$welch <- lapply(tails, function(tail) {
        tests <- lapply(seq_len(m), function(i) {
          t.test(xf[gf == lev[first[i]]], xf[gf == lev[second[i]]], var.equal = FALSE, conf.level = 1 - 2 * tail)
        })
        limits(sapply(tests, function(t) t$conf.int[1]), sapply(tests, function(t) t$conf.int[2]))
      })
    }
    out
  })

  list(x = as.list(num(x)), g = as.list(ifelse(is.na(g), "NA", as.character(g))),
       text_groups = is.character(g),
       first = as.list(lev[first]), second = as.list(lev[second]),
       difference = as.list(num(difference)), levels = at)
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
# Groups apart by a little: intervals on both sides of zero.
g <- rep(1:5, length.out = 60)
cases[[length(cases) + 1]] <- comparisons(series(60) + 1.1 * g, g)
# Two groups: every method gives the interval of the t-test.
cases[[length(cases) + 1]] <- comparisons(series(16) * rep(c(1, 2), each = 8), rep(c(1, 2), each = 8))

write_json(list(reference = c(reference_block(), list(DescTools = as.character(packageVersion("DescTools")))),
                cases = cases),
           file.path(here(), "posthoc_level.json"))
