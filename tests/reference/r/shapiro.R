# R's answers for shapiro_test (release 0.7.0, step 14): shapiro.test() in R 4.6.0, Royston's
# (1995) algorithm AS R94. At every size the catalogue's step names, n = 3, 4, 5, 11, 12, 50,
# 2,000 and 5,000 (the two sides of each branch of the approximation and the largest size
# allowed), on data of four shapes: the normal quantiles themselves, a skewed transform of them,
# the quantiles rounded to one decimal so that they tie heavily, and a smooth non-normal series;
# the largest sizes carry two shapes each to keep the file small. One case holds NA. W and the
# p-value are held to a relative 1e-9, since the p-value passes through the platform's log and
# exp. Run with `Rscript tests/reference/r/shapiro.R`.
source(file.path(dirname(sub("^--file=", "", commandArgs(FALSE)[grepl("^--file=", commandArgs(FALSE))])), "common.R"))
options(warn = -1)

# The normal quantiles of a sample of n, in a fixed scrambled order (W does not depend on the
# order, but the engine must sort for itself).
normal_like <- function(n) {
  order <- (seq_len(n) * 7919L) %% n + 1L
  qnorm((order - 0.5) / n)
}
series <- function(n) 3 * sin(seq_len(n) * 1.3) + 0.4 + seq_len(n) / 17

sw <- function(x, shape) {
  t <- shapiro.test(x)
  list(shape = shape, x = as.list(num(x)), w = num(unname(t$statistic)), p_value = num(t$p.value),
       n = as.integer(sum(!is.na(x))))
}

cases <- list()
for (n in c(3L, 4L, 5L, 11L, 12L, 50L, 2000L, 5000L)) {
  z <- normal_like(n)
  cases[[length(cases) + 1]] <- sw(z, "normal")
  cases[[length(cases) + 1]] <- sw(round(z, 1), "ties")
  if (n <= 50L) {
    cases[[length(cases) + 1]] <- sw(exp(z / 2), "skewed")
    cases[[length(cases) + 1]] <- sw(series(n), "series")
  }
}
# NA among the values, and a skewed sample of 2,000 whose p-value is far in the tail.
x <- normal_like(24); x[c(3, 11, 20)] <- NA
cases[[length(cases) + 1]] <- sw(x, "na")
cases[[length(cases) + 1]] <- sw(exp(normal_like(2000) / 2), "skewed")

write_json(list(reference = reference_block(), shapiro = cases), file.path(here(), "shapiro.json"))
