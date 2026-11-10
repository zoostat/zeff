# R's answers for the one-way form of anova (release 0.7.0, step 16): oneway.test() for Welch's
# F and aov() for the classical F in R 4.6.0 (oneway.test(var.equal = TRUE) agrees where it runs). A fixed series split into 2 to 6 groups of unequal
# sizes, each group scaled and shifted so that the variances differ; ties (rounded values); a
# group of one observation, where Welch's test is undefined and the classical one is not; NA in
# the variable and in the grouping variable; text groups; a large sample; groups far apart, so
# that the p-value is tiny; and two groups, where Welch's F is the square of Welch's t. The
# statistics, degrees of freedom, p-values, sums of squares and eta squared are held to a relative
# 1e-9. Run with `Rscript tests/reference/r/anova.R`.
source(file.path(dirname(sub("^--file=", "", commandArgs(FALSE)[grepl("^--file=", commandArgs(FALSE))])), "common.R"))
options(warn = -1)

series <- function(n, digits = 6) round(3 * sin(seq_len(n) * 1.3) + 0.4 + seq_len(n) / 17, digits)

one_way <- function(x, g) {
  ok <- complete.cases(x, g)
  xf <- x[ok]; gf <- factor(g[ok])
  welch <- tryCatch(oneway.test(xf ~ gf, var.equal = FALSE), error = function(e) NULL)
  # The classical F from aov(), which admits a group of one where oneway.test() does not; where
  # both run they agree to the last digit.
  a <- anova(lm(xf ~ gf))
  classical <- list(statistic = a[1, "F value"], parameter = c(a[1, "Df"], a[2, "Df"]), p.value = a[1, "Pr(>F)"])
  if (all(table(gf) >= 2)) {
    o <- oneway.test(xf ~ gf, var.equal = TRUE)
    stopifnot(abs(unname(o$statistic) - classical$statistic) <= 1e-12 * classical$statistic)
  }
  list(x = as.list(num(x)),
       g = if (is.character(g)) as.list(ifelse(is.na(g), "NA", g)) else as.list(ifelse(is.na(g), "NA", as.character(g))),
       text_groups = is.character(g),
       welch_f = if (is.null(welch)) "NA" else num(unname(welch$statistic)),
       welch_df1 = if (is.null(welch)) "NA" else num(unname(welch$parameter[1])),
       welch_df2 = if (is.null(welch)) "NA" else num(unname(welch$parameter[2])),
       welch_p = if (is.null(welch)) "NA" else num(welch$p.value),
       f = num(unname(classical$statistic)),
       df1 = num(unname(classical$parameter[1])), df2 = num(unname(classical$parameter[2])),
       p_value = num(classical$p.value),
       ss_between = num(a[1, "Sum Sq"]), ss_within = num(a[2, "Sum Sq"]),
       eta_squared = num(a[1, "Sum Sq"] / sum(a[, "Sum Sq"])),
       n = as.integer(sum(ok)), groups = as.integer(nlevels(gf)))
}

anova_cases <- list()
# Distinct values, k groups of unequal sizes and unequal variances.
for (k in 2:6) {
  n <- 7 * k + 3
  g <- rep(seq_len(k), length.out = n)
  x <- series(n) * (0.5 + 0.4 * g) + 0.3 * g
  anova_cases[[length(anova_cases) + 1]] <- one_way(x, g)
}
# Ties: values rounded to one digit, then to integers.
anova_cases[[length(anova_cases) + 1]] <- one_way(series(24, digits = 1), rep(1:3, each = 8))
anova_cases[[length(anova_cases) + 1]] <- one_way(round(series(30, digits = 0)), rep(1:4, length.out = 30))
# A group of one observation: Welch's test is undefined.
anova_cases[[length(anova_cases) + 1]] <- one_way(series(12), c(rep(1, 6), rep(2, 5), 3))
# NA in the variable and in the groups.
x <- series(20); x[c(3, 11)] <- NA
g <- rep(1:3, length.out = 20); g[c(7, 11)] <- NA
anova_cases[[length(anova_cases) + 1]] <- one_way(x, g)
# Text groups, with ties.
anova_cases[[length(anova_cases) + 1]] <- one_way(series(18, digits = 1), rep(c("north", "south", "east"), each = 6))
# A large sample.
anova_cases[[length(anova_cases) + 1]] <- one_way(series(300, digits = 2), rep(1:5, length.out = 300))
# Groups far apart: a tiny p-value in both forms.
g <- rep(1:4, each = 10)
anova_cases[[length(anova_cases) + 1]] <- one_way(series(40) + 6 * g, g)
# Two groups of equal size: Welch's F is the square of Welch's t.
anova_cases[[length(anova_cases) + 1]] <- one_way(series(16) * rep(c(1, 2), each = 8), rep(c(1, 2), each = 8))
# Two groups, the classical F equal to the pooled t squared, printed by t_test with equal_var.
anova_cases[[length(anova_cases) + 1]] <- one_way(series(15), rep(c(1, 2), length.out = 15))

write_json(list(reference = reference_block(), anova = anova_cases),
           file.path(here(), "anova.json"))
