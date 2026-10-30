# R's values for the functions of `core` written in Zeff (release 0.5.0, step 14): qlogis(),
# plogis() and pmin(pmax()). Every value is written with 17 significant digits, so the Rust test
# reads back the exact double R computed. `qlogis` is log(p / (1 - p)) and `plogis` is
# 1 / (1 + exp(-x)) in R's C code as in Zeff's bodies, so the two agree to the last bit wherever
# the two logarithms and exponentials do; the test counts the exact matches and allows a relative
# difference of 1e-15 for the rest. Run with `Rscript tests/reference/r/zeff_functions.R`.
source(file.path(dirname(sub("^--file=", "", commandArgs(FALSE)[grepl("^--file=", commandArgs(FALSE))])), "common.R"))

dec <- function(x) ifelse(is.na(x), "NA", ifelse(is.infinite(x), ifelse(x > 0, "Inf", "-Inf"), sprintf("%.17g", x)))

# Probabilities: the working grid and the edges, with values down to 1e-12 from both ends.
p <- c(0, 1, 0.5, 0.1, 0.9, 0.25, 0.75, 0.01, 0.99, 1e-3, 1 - 1e-3, 1e-6, 1 - 1e-6,
       1e-9, 1 - 1e-9, 1e-12, 1 - 1e-12, 0.3333333333333333, 0.6666666666666666, 1e-300,
       2.2250738585072014e-308, 0.4999999999999999, 0.5000000000000001)
# Log odds from -1e12 to 1e12, with the range where exp(-x) overflows and underflows.
x <- c(-1e12, -1e7, -800, -745, -100, -40, -20, -10, -5, -2.5, -1, -0.5, -1e-5, 0, 1e-5, 0.5, 1,
       2.5, 5, 10, 20, 40, 100, 709, 710, 800, 1e7, 1e12)
ps <- suppressWarnings(qlogis(p))
xs <- plogis(x)

# clamp: pmin(pmax(x, low), high) on a grid of values and ranges, including low above high.
values <- c(-1e12, -3.5, -1, 0, 0.5, 2, 3, 3.5, 1e7, 1e12)
ranges <- list(c(0, 3), c(-1, 1), c(2, 2), c(5, 1), c(-1e12, 1e12), c(0.1, 0.2))
clamp_cases <- list()
for (r in ranges) for (v in values) {
  clamp_cases[[length(clamp_cases) + 1]] <- list(
    x = dec(v), low = dec(r[1]), high = dec(r[2]), expected = dec(pmin(pmax(v, r[1]), r[2]))
  )
}

write_json(list(
  reference = reference_block(),
  logit = lapply(seq_along(p), function(i) list(p = dec(p[i]), expected = dec(ps[i]))),
  inv_logit = lapply(seq_along(x), function(i) list(x = dec(x[i]), expected = dec(xs[i]))),
  clamp = clamp_cases
), file.path(here(), "zeff_functions.json"))
