# R's values of the distribution functions (release 0.3.0, steps 7 to 10): for each family and
# parameter set, the density, both tails of the distribution function and the quantile on fixed
# grids, written with 15 significant digits. R is the main reference (testing handbook, section
# 1.1); the kernel's special functions are held to the exact decimal reference of special.py. Run
# with `Rscript tests/reference/r/distributions.R`.
source(file.path(dirname(sub("^--file=", "", commandArgs(FALSE)[grepl("^--file=", commandArgs(FALSE))])), "common.R"))

dec <- function(x) sprintf("%.17g", x)
xs <- c(-1e300, -40, -5, -2, -1, -0.5, -1e-10, 0, 1e-10, 0.001, 0.25, 0.5, 1, 1.5, 1.96, 2.5, 3, 5, 10, 40, 1e6)
ps <- c(1e-300, 1e-20, 1e-10, 1e-6, 0.001, 0.025, 0.1, 0.25, 0.5, 0.75, 0.9, 0.975, 0.999, 1 - 1e-6, 1 - 1e-10)

family <- function(name, params, d, p, q) {
  list(
    family = name,
    params = lapply(params, dec),
    x = as.list(dec(xs)),
    pdf = as.list(num(suppressWarnings(d(xs)))),
    lower = as.list(num(suppressWarnings(p(xs)))),
    upper = as.list(num(suppressWarnings(p(xs, lower.tail = FALSE)))),
    p = as.list(dec(ps)),
    quantile = as.list(num(q(ps)))
  )
}

families <- list(
  family("normal", list(mean = 0, sd = 1), dnorm, pnorm, qnorm),
  family("normal", list(mean = 100, sd = 15),
         function(x) dnorm(x, 100, 15), function(x, lower.tail = TRUE) pnorm(x, 100, 15, lower.tail = lower.tail), function(p) qnorm(p, 100, 15)),
  family("normal", list(mean = -2.5, sd = 0.01),
         function(x) dnorm(x, -2.5, 0.01), function(x, lower.tail = TRUE) pnorm(x, -2.5, 0.01, lower.tail = lower.tail), function(p) qnorm(p, -2.5, 0.01)),
  family("lognormal", list(meanlog = 0, sdlog = 1), dlnorm, plnorm, qlnorm),
  family("lognormal", list(meanlog = 3, sdlog = 0.5),
         function(x) dlnorm(x, 3, 0.5), function(x, lower.tail = TRUE) plnorm(x, 3, 0.5, lower.tail = lower.tail), function(p) qlnorm(p, 3, 0.5)),
  family("uniform", list(min = 0, max = 1), dunif, punif, qunif),
  family("uniform", list(min = 2, max = 5),
         function(x) dunif(x, 2, 5), function(x, lower.tail = TRUE) punif(x, 2, 5, lower.tail = lower.tail), function(p) qunif(p, 2, 5)),
  family("exponential", list(rate = 1), dexp, pexp, qexp),
  family("exponential", list(rate = 0.1),
         function(x) dexp(x, 0.1), function(x, lower.tail = TRUE) pexp(x, 0.1, lower.tail = lower.tail), function(p) qexp(p, 0.1)),
  family("exponential", list(rate = 25),
         function(x) dexp(x, 25), function(x, lower.tail = TRUE) pexp(x, 25, lower.tail = lower.tail), function(p) qexp(p, 25)),
  family("t", list(df = 1), function(x) dt(x, 1), function(x, lower.tail = TRUE) pt(x, 1, lower.tail = lower.tail), function(p) qt(p, 1)),
  family("t", list(df = 3), function(x) dt(x, 3), function(x, lower.tail = TRUE) pt(x, 3, lower.tail = lower.tail), function(p) qt(p, 3)),
  family("t", list(df = 24), function(x) dt(x, 24), function(x, lower.tail = TRUE) pt(x, 24, lower.tail = lower.tail), function(p) qt(p, 24)),
  family("t", list(df = 1e6), function(x) dt(x, 1e6), function(x, lower.tail = TRUE) pt(x, 1e6, lower.tail = lower.tail), function(p) qt(p, 1e6)),
  family("chi2", list(df = 1), function(x) dchisq(x, 1), function(x, lower.tail = TRUE) pchisq(x, 1, lower.tail = lower.tail), function(p) qchisq(p, 1)),
  family("chi2", list(df = 5), function(x) dchisq(x, 5), function(x, lower.tail = TRUE) pchisq(x, 5, lower.tail = lower.tail), function(p) qchisq(p, 5)),
  family("chi2", list(df = 100), function(x) dchisq(x, 100), function(x, lower.tail = TRUE) pchisq(x, 100, lower.tail = lower.tail), function(p) qchisq(p, 100)),
  family("f", list(df1 = 3, df2 = 20), function(x) df(x, 3, 20), function(x, lower.tail = TRUE) pf(x, 3, 20, lower.tail = lower.tail), function(p) qf(p, 3, 20)),
  family("f", list(df1 = 1, df2 = 1), function(x) df(x, 1, 1), function(x, lower.tail = TRUE) pf(x, 1, 1, lower.tail = lower.tail), function(p) qf(p, 1, 1)),
  family("f", list(df1 = 100, df2 = 1000), function(x) df(x, 100, 1000), function(x, lower.tail = TRUE) pf(x, 100, 1000, lower.tail = lower.tail), function(p) qf(p, 100, 1000)),
  family("gamma", list(shape = 0.5, rate = 1), function(x) dgamma(x, 0.5, 1), function(x, lower.tail = TRUE) pgamma(x, 0.5, 1, lower.tail = lower.tail), function(p) qgamma(p, 0.5, 1)),
  family("gamma", list(shape = 2, rate = 3), function(x) dgamma(x, 2, 3), function(x, lower.tail = TRUE) pgamma(x, 2, 3, lower.tail = lower.tail), function(p) qgamma(p, 2, 3)),
  family("gamma", list(shape = 100, rate = 0.1), function(x) dgamma(x, 100, 0.1), function(x, lower.tail = TRUE) pgamma(x, 100, 0.1, lower.tail = lower.tail), function(p) qgamma(p, 100, 0.1)),
  family("beta", list(a = 0.5, b = 0.5), function(x) dbeta(x, 0.5, 0.5), function(x, lower.tail = TRUE) pbeta(x, 0.5, 0.5, lower.tail = lower.tail), function(p) qbeta(p, 0.5, 0.5)),
  family("beta", list(a = 2, b = 3), function(x) dbeta(x, 2, 3), function(x, lower.tail = TRUE) pbeta(x, 2, 3, lower.tail = lower.tail), function(p) qbeta(p, 2, 3)),
  family("beta", list(a = 100, b = 100), function(x) dbeta(x, 100, 100), function(x, lower.tail = TRUE) pbeta(x, 100, 100, lower.tail = lower.tail), function(p) qbeta(p, 100, 100)),
  family("binomial", list(n = 10, p = 0.3), function(x) dbinom(x, 10, 0.3), function(x, lower.tail = TRUE) pbinom(x, 10, 0.3, lower.tail = lower.tail), function(p) qbinom(p, 10, 0.3)),
  family("binomial", list(n = 1000, p = 0.5), function(x) dbinom(x, 1000, 0.5), function(x, lower.tail = TRUE) pbinom(x, 1000, 0.5, lower.tail = lower.tail), function(p) qbinom(p, 1000, 0.5)),
  family("binomial", list(n = 1e6, p = 1e-3), function(x) dbinom(x, 1e6, 1e-3), function(x, lower.tail = TRUE) pbinom(x, 1e6, 1e-3, lower.tail = lower.tail), function(p) qbinom(p, 1e6, 1e-3)),
  family("poisson", list(lambda = 2), function(x) dpois(x, 2), function(x, lower.tail = TRUE) ppois(x, 2, lower.tail = lower.tail), function(p) qpois(p, 2)),
  family("poisson", list(lambda = 0.01), function(x) dpois(x, 0.01), function(x, lower.tail = TRUE) ppois(x, 0.01, lower.tail = lower.tail), function(p) qpois(p, 0.01)),
  family("poisson", list(lambda = 1000), function(x) dpois(x, 1000), function(x, lower.tail = TRUE) ppois(x, 1000, lower.tail = lower.tail), function(p) qpois(p, 1000)),
  family("negbinomial", list(size = 3, p = 0.4), function(x) dnbinom(x, 3, 0.4), function(x, lower.tail = TRUE) pnbinom(x, 3, 0.4, lower.tail = lower.tail), function(p) qnbinom(p, 3, 0.4)),
  family("negbinomial", list(mu = 4.5, size = 3), function(x) dnbinom(x, size = 3, mu = 4.5), function(x, lower.tail = TRUE) pnbinom(x, size = 3, mu = 4.5, lower.tail = lower.tail), function(p) qnbinom(p, size = 3, mu = 4.5)),
  family("negbinomial", list(size = 0.5, p = 0.1), function(x) dnbinom(x, 0.5, 0.1), function(x, lower.tail = TRUE) pnbinom(x, 0.5, 0.1, lower.tail = lower.tail), function(p) qnbinom(p, 0.5, 0.1))
)

# Truncations and mixtures (step 10), from base R's functions: a truncated density is d(x) / Z on
# the interval with Z the interval's probability, its distribution function (F(x) - F(a-)) / Z, its
# quantile Q(F(a-) + p Z); a mixture's density and distribution function are weighted sums, its
# quantile the root of the mixed distribution function (uniroot). These are the formulas of the
# truncdist package, written out so that no package is needed.
composite <- function(expression, d, p, s, q) {
  list(
    family = "composite",
    expression = expression,
    x = as.list(dec(xs)),
    pdf = as.list(num(suppressWarnings(d(xs)))),
    lower = as.list(num(suppressWarnings(p(xs)))),
    upper = as.list(num(suppressWarnings(s(xs)))),
    p = as.list(dec(ps)),
    quantile = as.list(num(q(ps)))
  )
}
trunc_norm <- local({
  a <- -1; b <- 2; Z <- pnorm(b) - pnorm(a)
  composite("dist.truncated(dist.normal(), lower: -1, upper: 2)",
    function(x) ifelse(x < a | x > b, 0, dnorm(x) / Z),
    function(x) pmin(1, pmax(0, (pnorm(x) - pnorm(a)) / Z)),
    function(x) pmin(1, pmax(0, (pnorm(x, lower.tail = FALSE) - pnorm(b, lower.tail = FALSE)) / Z)),
    function(p) qnorm(pnorm(a) + p * Z))
})
trunc_t <- local({
  a <- 0; Z <- 1 - pt(a, 5)
  composite("dist.truncated(dist.t(df: 5), lower: 0)",
    function(x) ifelse(x < a, 0, dt(x, 5) / Z),
    function(x) pmin(1, pmax(0, (pt(x, 5) - pt(a, 5)) / Z)),
    function(x) pmin(1, pmax(0, pt(x, 5, lower.tail = FALSE) / Z)),
    function(p) qt(pt(a, 5) + p * Z, 5))
})
trunc_pois <- local({
  a <- 1; b <- 6; Z <- ppois(b, 3) - ppois(a - 1, 3)
  composite("dist.truncated(dist.poisson(lambda: 3), lower: 1, upper: 6)",
    function(x) ifelse(x < a | x > b, 0, dpois(x, 3) / Z),
    function(x) pmin(1, pmax(0, (ppois(x, 3) - ppois(a - 1, 3)) / Z)),
    function(x) pmin(1, pmax(0, (ppois(x, 3, lower.tail = FALSE) - ppois(b, 3, lower.tail = FALSE)) / Z)),
    function(p) qpois(ppois(a - 1, 3) + p * Z, 3))
})
mix_norm <- local({
  dmix <- function(x) 0.3 * dnorm(x) + 0.7 * dnorm(x, 3, 0.5)
  pmix <- function(x) 0.3 * pnorm(x) + 0.7 * pnorm(x, 3, 0.5)
  smix <- function(x) 0.3 * pnorm(x, lower.tail = FALSE) + 0.7 * pnorm(x, 3, 0.5, lower.tail = FALSE)
  qmix <- function(p) vapply(p, function(pp) {
    if (pp <= 0) return(-Inf)
    if (pp >= 1) return(Inf)
    uniroot(function(x) pmix(x) - pp, c(-60, 60), tol = 1e-14)$root
  }, 0)
  composite("dist.mixture([dist.normal(), dist.normal(mean: 3, sd: 0.5)], probs: [0.3, 0.7])", dmix, pmix, smix, qmix)
})
families <- c(families, list(trunc_norm, trunc_t, trunc_pois, mix_norm))

write_json(list(reference = reference_block(), families = families), file.path(here(), "distributions.json"))
