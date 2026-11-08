# R's answers for wilcoxon_test (release 0.7.0, step 11): wilcox.test() in R 4.6.0, whose exact
# p-values hold below 50 observations even with ties and zeros (the permutation distribution of
# the tied ranks) and whose approximation applies the continuity correction. The signed-rank
# test on a fixed series for every n from 2 to 60, exact and approximate, two-sided and
# one-sided, with the Hodges-Lehmann estimate and interval; the rank-sum test on a grid of group
# sizes; series with ties and with zero differences; the paired form. The data are written with
# 15 significant digits so that the engine ranks the same values. Exact p-values, statistics,
# estimates and exact intervals are held to a relative 1e-12; the approximate intervals are the
# roots uniroot() finds to tol = 1e-4 and are held to 2e-4. Run with
# `Rscript tests/reference/r/wilcoxon.R`.
source(file.path(dirname(sub("^--file=", "", commandArgs(FALSE)[grepl("^--file=", commandArgs(FALSE))])), "common.R"))
options(warn = -1)

series <- function(n) round(3 * sin(seq_len(n) * 1.3) + 0.4 + seq_len(n) / 17, 6)

one <- function(x, mu = 0, exact = NULL, alternative = "two.sided", level = 0.95, correct = TRUE) {
  t <- wilcox.test(x, mu = mu, exact = exact, alternative = alternative, conf.int = TRUE, conf.level = level, correct = correct)
  list(x = as.list(num(x)), mu = num(mu), exact = isTRUE(exact) || (is.null(exact) && length(x) < 50),
       alternative = alternative, level = num(level), correct = correct,
       statistic = num(unname(t$statistic)), p_value = num(t$p.value), estimate = num(unname(t$estimate)),
       ci = as.list(num(as.vector(t$conf.int))))
}
two <- function(x, y, mu = 0, exact = NULL, alternative = "two.sided", level = 0.95, correct = TRUE) {
  t <- wilcox.test(x, y, mu = mu, exact = exact, alternative = alternative, conf.int = TRUE, conf.level = level, correct = correct)
  list(x = as.list(num(x)), y = as.list(num(y)), mu = num(mu),
       exact = isTRUE(exact) || (is.null(exact) && length(x) < 50 && length(y) < 50),
       alternative = alternative, level = num(level), correct = correct,
       statistic = num(unname(t$statistic)), p_value = num(t$p.value), estimate = num(unname(t$estimate)),
       ci = as.list(num(as.vector(t$conf.int))))
}

signed <- list()
for (n in 2:60) {
  x <- series(n)
  alt <- c("two.sided", "less", "greater")[(n %% 3) + 1]
  signed[[length(signed) + 1]] <- one(x, mu = 0.5, exact = TRUE, alternative = alt)
  signed[[length(signed) + 1]] <- one(x, mu = 0.5, exact = FALSE, alternative = alt, level = 0.9)
}
rank_sum <- list()
for (sizes in list(c(2, 3), c(3, 3), c(5, 7), c(8, 13), c(10, 10), c(21, 13), c(30, 25), c(49, 49), c(60, 20), c(4, 60))) {
  x <- series(sizes[1]); y <- rev(series(sizes[2] + 3))[seq_len(sizes[2])] - 0.3
  for (alt in c("two.sided", "less", "greater")) {
    rank_sum[[length(rank_sum) + 1]] <- two(x, y, exact = TRUE, alternative = alt)
    rank_sum[[length(rank_sum) + 1]] <- two(x, y, exact = FALSE, alternative = alt, level = 0.99)
  }
}
v <- c(3, 1, 4, 1, 5, 9, 2, 6, 5, 3, 5, 8, 9, 7, 9)
ties <- list(
  one(v, mu = 5), one(v, mu = 5, alternative = "less"), one(v, mu = 5, alternative = "greater", level = 0.9),
  one(v, mu = 5, exact = FALSE), one(v, mu = 5, exact = FALSE, alternative = "less", correct = FALSE),
  one(v, mu = 4.5), one(c(2, 2, 2, 7, 7, 1, 9, 9, 9, 9, 3), mu = 2),
  two(c(1, 2, 2, 3, 3, 3, 8, 9), c(2, 3, 3, 4, 5, 5, 6)),
  two(c(1, 2, 2, 3, 3, 3, 8, 9), c(2, 3, 3, 4, 5, 5, 6), alternative = "greater"),
  two(c(1, 2, 2, 3, 3, 3, 8, 9), c(2, 3, 3, 4, 5, 5, 6), exact = FALSE, level = 0.9),
  two(rep(1:6, c(2, 3, 4, 5, 4, 3)), rep(2:8, c(1, 2, 2, 3, 3, 2, 1)), mu = 1))
xa <- c(1.83, 0.50, 1.62, 2.48, 1.68, 1.88, 1.55, 3.06, 1.30); ya <- c(0.878, 0.647, 0.598, 2.05, 1.06, 1.29, 1.06, 3.14, 1.29)
paired <- list(a = as.list(num(xa)), b = as.list(num(ya)),
               cases = list(one(xa - ya, alternative = "greater"), one(xa - ya), one(xa - ya, exact = FALSE, level = 0.99)))

out <- list(reference = reference_block(), signed = signed, rank_sum = rank_sum, ties = ties, paired = paired)
write_json(out, file.path(here(), "wilcoxon.json"))
