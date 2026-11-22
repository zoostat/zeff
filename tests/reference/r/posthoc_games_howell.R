# R's answers for the Games-Howell comparison of posthoc (release 0.9.0, step 29; open decision
# #85), in R 4.6.0. Each pair is written as the first group minus the second, in the order
# (1, 2), (1, 3), ..., (2, 3), ..., the groups in the order of factor() with its levels sorted by
# code point.
#
# The comparison is Tukey's without the pooled variance (Games and Howell, 1976). For the groups
# i and j of k, with means m, variances v and sizes n:
#   the standard error      sqrt(v_i / n_i + v_j / n_j),
#   t                       (m_i - m_j) / the standard error,
#   the degrees of freedom  Welch's and Satterthwaite's, as t.test(var.equal = FALSE) has them,
#   the p-value             ptukey(|t| * sqrt(2), k, df, lower.tail = FALSE),
#   the interval            the difference -/+ qtukey(level, k, df) / sqrt(2) * the standard error.
# It is written here in base R from those formulas: the standard error, t and the degrees of
# freedom are checked against t.test() of each pair, and for two groups the whole against
# t.test(), where the studentized range of two is Student's t. The p-values come from ptukey
# and the limits from qtukey, which carry R's own error (the help page of dist.tukey gives its
# size), so the comparison tests hold them to that and not to 1e-9. Below two degrees of freedom,
# which a pair of groups of two values has, ptukey and qtukey give NaN, written here as NA: the
# engine's studentized range is defined from one degree of freedom, and its values there are held
# by the tests of dist.tukey and by the tail its own quantile leaves.
#
# Where the package rstatix is installed, every case at 95% is also checked against
# rstatix::games_howell_test(), whose rows are the second group minus the first; the package is
# not one of the reference library's (tests/reference/r/install.R), so the check is passed over
# where it is absent and nothing of it is written to the file.
#
# The cases are the one-way cases of posthoc_level.R that have two values and some spread in
# every group: 2 to 6 groups of unequal sizes and variances, ties, NA in the variable and in the
# groups, text groups, groups apart by a little, two groups; then groups of very unequal
# variances and sizes, where the comparison differs most from Tukey's, and groups of three and of
# two values, with few degrees of freedom; and hours by industry on
# the shipped wages, 12 groups and 66 comparisons. Each at the levels 0.90, 0.95 and 0.99.
# Run with `Rscript tests/reference/r/posthoc_games_howell.R`.
source(file.path(dirname(sub("^--file=", "", commandArgs(FALSE)[grepl("^--file=", commandArgs(FALSE))])), "common.R"))
options(warn = -1)
.libPaths(c(file.path(here(), "library"), .libPaths()))

LEVELS <- c(0.90, 0.95, 0.99)
series <- function(n, digits = 6) round(3 * sin(seq_len(n) * 1.3) + 0.4 + seq_len(n) / 17, digits)
same <- function(a, b, tolerance = 1e-10) all(abs(a - b) <= tolerance * pmax(1, abs(a), abs(b)))
# The same where R has a value, and none in the same places: ptukey and qtukey give NaN below two
# degrees of freedom, which a pair of groups of two values has.
same_where_defined <- function(a, b, tolerance = 1e-10) {
  identical(is.na(a), is.na(b)) && same(a[!is.na(a)], b[!is.na(b)], tolerance)
}
with_rstatix <- requireNamespace("rstatix", quietly = TRUE)
checked_against_rstatix <- 0

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
  stopifnot(all(n >= 2), all(variances > 0))

  difference <- means[first] - means[second]
  u <- variances[first] / n[first]
  v <- variances[second] / n[second]
  se <- sqrt(u + v)
  t <- difference / se
  df <- (u + v)^2 / (u^2 / (n[first] - 1) + v^2 / (n[second] - 1))
  p <- ptukey(abs(t) * sqrt(2), nmeans = k, df = df, lower.tail = FALSE)

  # The standard error, t and the degrees of freedom are Welch's test of each pair.
  for (i in seq_len(m)) {
    welch <- t.test(xf[gf == lev[first[i]]], xf[gf == lev[second[i]]], var.equal = FALSE)
    stopifnot(same(welch$statistic, t[i]), same(welch$parameter, df[i]), same(welch$stderr, se[i]))
    # Of two groups the studentized range is Student's t: the whole is the t-test.
    if (k == 2) stopifnot(same(welch$p.value, p[i], 1e-6))
  }

  at <- lapply(LEVELS, function(level) {
    half <- qtukey(level, nmeans = k, df = df) / sqrt(2) * se
    if (k == 2) {
      welch <- t.test(xf[gf == lev[1]], xf[gf == lev[2]], var.equal = FALSE, conf.level = level)
      stopifnot(same(welch$conf.int[1], difference - half, 1e-4), same(welch$conf.int[2], difference + half, 1e-4))
    }
    list(level = num(level), lower = as.list(num(difference - half)), upper = as.list(num(difference + half)))
  })

  if (with_rstatix) {
    frame <- data.frame(xf = xf, gf = gf)
    theirs <- as.data.frame(rstatix::games_howell_test(frame, xf ~ gf, conf.level = 0.95, detailed = TRUE))
    half <- qtukey(0.95, nmeans = k, df = df) / sqrt(2) * se
    stopifnot(identical(as.character(theirs$group1), lev[first]),
              identical(as.character(theirs$group2), lev[second]),
              # Their rows are the second group minus the first.
              same(-theirs$estimate, difference),
              same_where_defined(-theirs$conf.high, difference - half),
              same_where_defined(-theirs$conf.low, difference + half),
              same(theirs$df, df),
              same(theirs$statistic, abs(t)),
              # Their p-values are rounded to three decimals.
              identical(is.na(theirs$p.adj), is.na(p)),
              all(abs(theirs$p.adj - p)[!is.na(p)] <= 5.0001e-4))
    checked_against_rstatix <<- checked_against_rstatix + 1
  }

  out <- list(x = as.list(num(x)), g = as.list(ifelse(is.na(g), "NA", as.character(g))),
              text_groups = is.character(g), groups = k,
              first = as.list(lev[first]), second = as.list(lev[second]),
              difference = as.list(num(difference)), se = as.list(num(se)), t = as.list(num(t)),
              df = as.list(num(df)), p = as.list(num(p)), levels = at)
  if (!is.null(data)) {
    out$x <- NULL; out$g <- NULL
    out <- c(data, out)
  }
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
# NA in the variable and in the groups.
x <- series(20); x[c(3, 11)] <- NA
g <- rep(1:3, length.out = 20); g[c(7, 11)] <- NA
cases[[length(cases) + 1]] <- comparisons(x, g)
# Text groups, with ties.
cases[[length(cases) + 1]] <- comparisons(series(18, digits = 1), rep(c("north", "south", "east"), each = 6))
# Groups apart by a little: intervals on both sides of zero.
g <- rep(1:5, length.out = 60)
cases[[length(cases) + 1]] <- comparisons(series(60) + 1.1 * g, g)
# Two groups: the comparison is Welch's t-test.
cases[[length(cases) + 1]] <- comparisons(series(16) * rep(c(1, 2), each = 8), rep(c(1, 2), each = 8))
# Variances that differ by a factor of some hundreds, in groups of equal size.
g <- rep(1:4, each = 9)
cases[[length(cases) + 1]] <- comparisons(series(36) * c(0.1, 1, 5, 20)[g] + g, g)
# The small groups with the large variances, and the reverse: where pooling misleads most.
g <- rep(1:3, times = c(4, 12, 30))
cases[[length(cases) + 1]] <- comparisons(series(46) * c(8, 2, 0.5)[g] + 0.8 * g, g)
g <- rep(1:3, times = c(30, 12, 4))
cases[[length(cases) + 1]] <- comparisons(series(46) * c(8, 2, 0.5)[g] + 0.8 * g, g)
# Groups of three values: few degrees of freedom, all of two or more.
g <- rep(1:4, each = 3)
cases[[length(cases) + 1]] <- comparisons(series(12) * (1 + 0.1 * g), g)
# Groups of two values: degrees of freedom between one and two, where R has no value.
g <- rep(1:4, each = 2)
cases[[length(cases) + 1]] <- comparisons(series(8) * (1 + g), g)
# A large sample: degrees of freedom in the thousands.
g <- rep(1:3, length.out = 6000)
cases[[length(cases) + 1]] <- comparisons(series(6000) * (0.5 + 0.5 * g) + 0.05 * g, g)

# The shipped wages: hours by industry, 12 groups of unequal sizes.
root <- normalizePath(file.path(here(), "..", "..", ".."))
wages <- read_dataset(file.path(root, "packages", "tier1", "datasets", "data", "wages.csv"))
shipped <- comparisons(wages$hours, wages$industry,
                       data = list(file = "packages/tier1/datasets/data/wages.csv", x = "hours", g = "industry"))

if (with_rstatix) {
  cat("checked against rstatix", as.character(packageVersion("rstatix")), "on", checked_against_rstatix, "sets of groups\n")
} else {
  cat("rstatix is not installed: the check against games_howell_test() was passed over\n")
}
write_json(list(reference = reference_block(), cases = cases, wages = shipped),
           file.path(here(), "posthoc_games_howell.json"))
