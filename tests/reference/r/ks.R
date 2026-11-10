# R's answers for ks_test (release 0.7.0, step 15): the one-sample ks.test() in R 4.6.0 against
# the ten continuous families and the Poisson, at sizes on both sides of the exact rule (n < 100
# without ties), with ties, and with NA; and nortest's lillie.test() for estimated: true at sizes
# on both sides of its two formulas (n <= 100) and in every branch of Stephens' form above 0.1.
# R sums the limiting Kolmogorov distribution to a tolerance of 1e-6, so each asymptotic case
# also carries the limit's upper tail summed to convergence ("p_tight"); the engine is held to the tight
# value within a relative 1e-9 and to R's within 1e-3, and to the exact and Lilliefors p-values
# within 1e-9. Run with `Rscript tests/reference/r/ks.R`.
source(file.path(dirname(sub("^--file=", "", commandArgs(FALSE)[grepl("^--file=", commandArgs(FALSE))])), "common.R"))
.libPaths(c(file.path(here(), "library"), .libPaths()))
suppressMessages(library(nortest))
options(warn = -1)

# Probabilities of a sample of n in a fixed scrambled order, distinct and symmetric.
probs <- function(n) {
  order <- (seq_len(n) * 7919L) %% n + 1L
  (order - 0.5) / n
}

# The limiting Kolmogorov distribution's upper tail at x, summed to convergence: the alternating
# series directly from 1, so that a tiny tail keeps its digits, and one minus the theta form below.
klim_upper <- function(x) {
  if (x <= 0) return(1)
  if (x < 1) {
    k <- seq(1, 399, by = 2)
    return(1 - sqrt(2 * pi) / x * sum(exp(-k^2 * pi^2 / (8 * x^2))))
  }
  k <- 1:400
  2 * sum((-1)^(k - 1) * exp(-2 * k^2 * x^2))
}

ks <- function(x, family, params, pfun) {
  t <- ks.test(x, pfun)
  ok <- !is.na(x)
  n <- sum(ok)
  ties <- length(unique(x[ok])) < n
  exact <- n < 100 && !ties
  d <- unname(t$statistic)
  list(family = family, params = lapply(params, num), x = as.list(num(x)),
       statistic = num(d), p_value = num(t$p.value),
       p_tight = num(if (exact) t$p.value else min(1, max(0, klim_upper(sqrt(n) * d)))),
       exact = exact, ties = ties, n = as.integer(n))
}

cases <- list()
add <- function(case) cases[[length(cases) + 1]] <<- case

# The normal family at every size, the data shifted and scaled so that p-values vary.
for (n in c(1L, 2L, 7L, 25L, 60L, 99L, 100L, 250L, 1000L)) {
  z <- qnorm(probs(n))
  add(ks(z, "normal", list(mean = 0, sd = 1), function(q) pnorm(q)))
  add(ks(0.3 + 1.2 * z, "normal", list(mean = 0, sd = 1), function(q) pnorm(q)))
  add(ks(100 + 15 * z, "normal", list(mean = 100, sd = 15), function(q) pnorm(q, 100, 15)))
}
# Ties: rounded values, at sizes where the exact rule would otherwise apply.
add(ks(round(qnorm(probs(40)), 1), "normal", list(mean = 0, sd = 1), function(q) pnorm(q)))
add(ks(round(qnorm(probs(300)), 1), "normal", list(mean = 0, sd = 1), function(q) pnorm(q)))
# NA among the values.
x <- qnorm(probs(30)); x[c(4, 17)] <- NA
add(ks(x, "normal", list(mean = 0, sd = 1), function(q) pnorm(q)))
# The other families, each at an exact and an asymptotic size, the data drawn from the family
# and from a neighbour of it.
u <- probs(45); v <- probs(150)
for (p in list(u, v)) {
  add(ks(qlnorm(p, 0, 0.5), "lognormal", list(meanlog = 0, sdlog = 0.5), function(q) plnorm(q, 0, 0.5)))
  add(ks(qlnorm(p, 0.2, 0.5), "lognormal", list(meanlog = 0, sdlog = 0.5), function(q) plnorm(q, 0, 0.5)))
  add(ks(p, "uniform", list(min = 0, max = 1), function(q) punif(q)))
  add(ks(2 + 3 * p^1.3, "uniform", list(min = 2, max = 5), function(q) punif(q, 2, 5)))
  add(ks(qexp(p, 2), "exponential", list(rate = 2), function(q) pexp(q, 2)))
  add(ks(qexp(p, 2.5), "exponential", list(rate = 2), function(q) pexp(q, 2)))
  add(ks(qt(p, 5), "t", list(df = 5), function(q) pt(q, 5)))
  add(ks(qt(p, 5), "normal", list(mean = 0, sd = 1), function(q) pnorm(q)))
  add(ks(qchisq(p, 3), "chi2", list(df = 3), function(q) pchisq(q, 3)))
  add(ks(qf(p, 4, 9), "f", list(df1 = 4, df2 = 9), function(q) pf(q, 4, 9)))
  add(ks(qgamma(p, 2, 1), "gamma", list(shape = 2, rate = 1), function(q) pgamma(q, 2, 1)))
  add(ks(qbeta(p, 2, 5), "beta", list(a = 2, b = 5), function(q) pbeta(q, 2, 5)))
  add(ks(qbeta(p, 2, 5), "uniform", list(min = 0, max = 1), function(q) punif(q)))
}
# A discrete family: Poisson counts, tied by nature.
add(ks(qpois(probs(80), 3), "poisson", list(lambda = 3), function(q) ppois(q, 3)))

lf <- function(x) {
  t <- lillie.test(x)
  list(x = as.list(num(x)), statistic = num(unname(t$statistic)), p_value = num(t$p.value),
       n = as.integer(sum(!is.na(x))))
}
lilliefors <- list()
for (n in c(5L, 6L, 20L, 50L, 100L, 101L, 250L, 1000L)) {
  z <- qnorm(probs(n))
  lilliefors[[length(lilliefors) + 1]] <- lf(z)
  lilliefors[[length(lilliefors) + 1]] <- lf(exp(z / 2))
  lilliefors[[length(lilliefors) + 1]] <- lf(probs(n))
  lilliefors[[length(lilliefors) + 1]] <- lf(round(z, 1))
  lilliefors[[length(lilliefors) + 1]] <- lf(z^3)
}
x <- qnorm(probs(24)); x[c(3, 11, 20)] <- NA
lilliefors[[length(lilliefors) + 1]] <- lf(x)

write_json(list(reference = reference_block(), ks = cases, lilliefors = lilliefors),
           file.path(here(), "ks.json"))
