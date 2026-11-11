# R's answers for kruskal_test and p_adjust (release 0.7.0, step 12): kruskal.test() and
# p.adjust() in R 4.6.0. The Kruskal-Wallis test on a fixed series split into 2 to 6 groups of
# unequal sizes, with ties (rounded values), a group of one observation, NA in the variable and
# in the grouping variable, and text groups; the adjustments under holm, bonferroni, BH and BY
# on vectors with ties, zeros, ones and NA, where R takes the count of non-NA p-values as the
# number of comparisons; from step 18c the same adjustments within the groups of a grouping
# variable. Statistics and p-values are held to a relative 1e-12. Run with
# `Rscript tests/reference/r/kruskal.R`.
source(file.path(dirname(sub("^--file=", "", commandArgs(FALSE)[grepl("^--file=", commandArgs(FALSE))])), "common.R"))
options(warn = -1)

series <- function(n, digits = 6) round(3 * sin(seq_len(n) * 1.3) + 0.4 + seq_len(n) / 17, digits)

kw <- function(x, g) {
  t <- kruskal.test(x, g)
  ok <- complete.cases(x, g)
  list(x = as.list(num(x)),
       g = if (is.character(g)) as.list(ifelse(is.na(g), "NA", g)) else as.list(ifelse(is.na(g), "NA", as.character(g))),
       text_groups = is.character(g),
       statistic = num(unname(t$statistic)), df = as.integer(unname(t$parameter)),
       p_value = num(t$p.value), n = as.integer(sum(ok)), groups = as.integer(length(unique(g[ok]))))
}

kruskal <- list()
# Distinct values, k groups of unequal sizes.
for (k in 2:6) {
  n <- 7 * k + 3
  x <- series(n)
  g <- rep(seq_len(k), length.out = n)
  kruskal[[length(kruskal) + 1]] <- kw(x, g)
}
# Heavy ties: values rounded to one digit, then to integers.
kruskal[[length(kruskal) + 1]] <- kw(series(24, digits = 1), rep(1:3, each = 8))
kruskal[[length(kruskal) + 1]] <- kw(round(series(30, digits = 0)), rep(1:4, length.out = 30))
# A group of one observation.
kruskal[[length(kruskal) + 1]] <- kw(series(12), c(rep(1, 6), rep(2, 5), 3))
# NA in the variable and in the groups.
x <- series(20); x[c(3, 11)] <- NA
g <- rep(1:3, length.out = 20); g[c(7, 11)] <- NA
kruskal[[length(kruskal) + 1]] <- kw(x, g)
# Text groups, with ties.
kruskal[[length(kruskal) + 1]] <- kw(series(18, digits = 1), rep(c("north", "south", "east"), each = 6))
# Larger groups.
kruskal[[length(kruskal) + 1]] <- kw(series(300, digits = 2), rep(1:5, length.out = 300))

adj <- function(p) {
  out <- list(p = as.list(num(p)))
  for (m in c("holm", "bonferroni", "hochberg", "hommel", "BH", "BY", "none")) {
    out[[tolower(m)]] <- as.list(num(p.adjust(p, method = m)))
  }
  # Sidak's single-step adjustment, which p.adjust() does not have: 1 - (1 - p)^n over the n
  # p-values present, through the logarithm so that a small p keeps its digits (step 18b).
  n <- sum(!is.na(p))
  sidak <- if (n <= 1) p else pmin(1, -expm1(n * log1p(-p)))
  sidak[which(p == 0)] <- 0  # a plain zero, whatever sign the platform's log1p gives its own
  out$sidak <- as.list(num(sidak))
  out
}
adjust <- list(
  adj(c(0.01, 0.04, 0.03, 0.005, 0.2)),
  adj(c(0.02, 0.02, 0.5, 0.0001, 1, 0)),
  adj(c(0.3)),
  adj(c(0.03, NA, 0.001, 0.2, NA, 0.049)),
  adj(c(NA, NA)),
  adj(c(0.5, 0.5, 0.5, 0.5)),
  adj(round(abs(sin(1:25)) / 3, 4)),
  adj(c(0.9, 0.95, 0.99, 1)),
  # Step 18b: two and three p-values, cases where Hommel's method gains on Hochberg's, ties
  # among the largest, and NA beside a p-value of 0 and of 1.
  adj(c(0.04, 0.03)),
  adj(c(0.02, 0.03, 0.025)),
  adj(c(0.01, 0.02, 0.022, 0.09)),
  adj(c(0.011, 0.012, 0.013, 0.014, 0.6, 0.6)),
  adj(c(NA, 0, 0.04, 1, 0.04, NA, 0.0004)),
  adj(round(abs(cos(1:40)) / 5, 5))
)

# Step 18c: the adjustment as a column, each group a family of its own (open decision #111).
# Within every group the p-values present are adjusted by the same code as above, which is what
# `dplyr::group_by(g) |> mutate(p.adjust(p))` computes; base R's split() is used so that the
# harness needs no further package. A missing group value is a group, as in dplyr. The groups
# are written as text, "NA" for a missing one, and the Rust test builds the frame from them.
within <- function(p, g) {
  key <- ifelse(is.na(g), "NA", as.character(g))
  out <- list(p = as.list(num(p)), g = as.list(key), text_groups = is.character(g))
  families <- split(seq_along(p), factor(key, levels = unique(key)))
  for (m in c("holm", "bonferroni", "hochberg", "hommel", "bh", "by", "sidak", "none")) {
    column <- rep(NA_real_, length(p))
    for (rows in families) {
      one <- adj(p[rows])[[m]]
      column[rows] <- vapply(one, function(v) if (identical(v, "NA")) NA_real_ else as.numeric(v), 0)
    }
    out[[m]] <- as.list(num(column))
  }
  out
}
grouped <- list(
  # Two families, an NA among the p-values of each.
  within(c(0.01, 0.04, 0.03, 0.2, NA, 0.001, 0.02, NA), c("a", "a", "a", "b", "b", "b", "b", "a")),
  # Rows of three families interleaved, with ties, 0 and 1.
  within(c(0.02, 0.5, 0.02, 0, 1, 0.5, 0.0001, 0.02, 1, 0.3, 0.02, 0.04),
         c("x", "y", "z", "x", "y", "z", "x", "y", "z", "x", "y", "z")),
  # A family of one p-value, a family whose p-values are all NA, and a family of one NA.
  within(c(0.03, 0.2, NA, NA, 0.004, NA, 0.6, 0.05), c(1, 2, 3, 3, 2, 4, 2, 2)),
  # A missing group value is a family of its own.
  within(c(0.01, 0.2, 0.5, 0.3, NA, 0.04, 0.011), c("a", "b", "a", NA, "b", NA, NA)),
  # One family alone: the column form of the list form.
  within(c(0.011, 0.012, 0.013, 0.014, 0.6, 0.6), rep("all", 6)),
  # Every row its own family: nothing is adjusted.
  within(c(0.3, 0.01, NA, 0.7), c(4, 3, 2, 1)),
  # Forty p-values in five families of unequal sizes.
  within(round(abs(cos(1:40)) / 5, 5), c(rep(1:5, length.out = 33), rep(2, 7)))
)

write_json(list(reference = reference_block(), kruskal = kruskal, adjust = adjust, grouped = grouped),
           file.path(here(), "kruskal.json"))
