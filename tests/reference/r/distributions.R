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

# The Weibull and the logistic (release 0.10.0, step 5), from dweibull, pweibull, qweibull and
# dlogis, plogis, qlogis, appended so that the entries before them keep their places. Each has
# everyday parameters and parameters of 1e7 and 1e12 beside them. A shape of 1e7 is given a scale
# of 1e6 so that no point of the grid lies beyond the scale: there R's dweibull multiplies an
# overflowed power by an underflowed exponential and gives NaN where the density is nought, and
# the engine's tests hold that case to nought by itself.
weibull_family <- function(shape, scale) {
  family("weibull", list(shape = shape, scale = scale),
         function(x) dweibull(x, shape, scale),
         function(x, lower.tail = TRUE) pweibull(x, shape, scale, lower.tail = lower.tail),
         function(p) qweibull(p, shape, scale))
}
logistic_family <- function(location, scale) {
  family("logistic", list(location = location, scale = scale),
         function(x) dlogis(x, location, scale),
         function(x, lower.tail = TRUE) plogis(x, location, scale, lower.tail = lower.tail),
         function(p) qlogis(p, location, scale))
}
trunc_weibull <- local({
  a <- 0.5; b <- 4; Z <- pweibull(b, 1.5, 2) - pweibull(a, 1.5, 2)
  composite("dist.truncated(dist.weibull(shape: 1.5, scale: 2), lower: 0.5, upper: 4)",
    function(x) ifelse(x < a | x > b, 0, dweibull(x, 1.5, 2) / Z),
    function(x) pmin(1, pmax(0, (pweibull(x, 1.5, 2) - pweibull(a, 1.5, 2)) / Z)),
    function(x) pmin(1, pmax(0, (pweibull(x, 1.5, 2, lower.tail = FALSE) - pweibull(b, 1.5, 2, lower.tail = FALSE)) / Z)),
    function(p) qweibull(pweibull(a, 1.5, 2) + p * Z, 1.5, 2))
})
mix_logistic_weibull <- local({
  dmix <- function(x) 0.4 * dlogis(x) + 0.6 * dweibull(x, 2, 1)
  pmix <- function(x) 0.4 * plogis(x) + 0.6 * pweibull(x, 2, 1)
  smix <- function(x) 0.4 * plogis(x, lower.tail = FALSE) + 0.6 * pweibull(x, 2, 1, lower.tail = FALSE)
  qmix <- function(p) vapply(p, function(pp) {
    if (pp <= 0) return(-Inf)
    if (pp >= 1) return(Inf)
    uniroot(function(x) pmix(x) - pp, c(-800, 800), tol = 1e-14)$root
  }, 0)
  composite("dist.mixture([dist.logistic(), dist.weibull(shape: 2)], probs: [0.4, 0.6])", dmix, pmix, smix, qmix)
})
families <- c(families, list(
  weibull_family(1.5, 1), weibull_family(0.5, 2), weibull_family(5, 100),
  weibull_family(1e7, 1e6), weibull_family(2, 1e12),
  logistic_family(0, 1), logistic_family(100, 15), logistic_family(-2.5, 0.01),
  logistic_family(0, 1e7), logistic_family(1e12, 1e7),
  trunc_weibull, mix_logistic_weibull
))

# A truncation near its bounds (release 0.8.0, step 20i). Close to a bound the formula above,
# (F(x) - F(a)) / Z, subtracts two nearly equal numbers and keeps few digits, so the reference
# takes the probability between two near points from the series of the integral of the density
# instead: for the normal, with He_n the Hermite polynomials,
#   integral from a to a + h of phi = phi(a) * sum over n of (-1)^n He_n(a) h^(n+1) / (n+1)!,
# and for Student's t from 0, the binomial series of (1 + x^2/df)^(-(df+1)/2) integrated term by
# term. Farther than 0.1 from a bound the tails are subtracted on the side where they are small.
# The exponential has closed forms through expm1 and log1p. Every quantile is the root of the
# same distribution function by Newton's steps, and the script stops if a root neither gives its
# probability back to fourteen digits nor is the double nearest the root.
near_normal <- function(a, h) {
  previous <- 0; he <- 1; power <- h; sign <- 1; total <- 0
  for (n in 0:150) {
    total <- total + sign * he * power
    following <- a * he - n * previous; previous <- he; he <- following
    power <- power * h / (n + 2); sign <- -sign
  }
  dnorm(a) * total
}
normal_between <- function(a, b) {
  if (is.infinite(b)) return(pnorm(a, lower.tail = FALSE))
  if (b - a <= 0.1) return(near_normal(a, b - a))
  if (a >= 0) return(pnorm(a, lower.tail = FALSE) - pnorm(b, lower.tail = FALSE))
  if (b <= 0) return(pnorm(b) - pnorm(a))
  1 - (pnorm(a) + pnorm(b, lower.tail = FALSE))
}
near_xs <- function(a, b, steps) c(a + steps, if (is.finite(b)) rev(b - steps))
near_ps <- c(1e-300, 1e-200, 1e-100, 1e-20, 1e-12, 1e-6, 0.01, 0.3, 0.5, 0.7, 0.99, 1 - 1e-6, 1 - 1e-12)
near_case <- function(inner, params, a, b, xs, density, lower_mass, upper_mass) {
  Z <- lower_mass(b)
  root <- function(p) {
    q <- 1 - p
    x <- if (p <= 0.5) a + p * Z / density(a) else if (is.finite(b)) b - q * Z / density(b) else a + 1
    x <- min(b, max(a, x))
    for (i in 1:300) {
      step <- if (p <= 0.5) (lower_mass(x) / Z - p) else -(upper_mass(x) / Z - q)
      x <- min(b, max(a, x - step * Z / density(x)))
    }
    # The root gives its probability back, or it is the double nearest the root: its two
    # neighbours stand on either side of the probability.
    miss <- function(x) if (p <= 0.5) lower_mass(x) / Z - p else q - upper_mass(x) / Z
    target <- if (p <= 0.5) p else q
    beside <- x + c(-2, 2) * .Machine$double.eps * max(abs(x), .Machine$double.xmin)
    stopifnot(abs(miss(x)) <= 1e-14 * target || miss(max(a, beside[1])) * miss(min(b, beside[2])) <= 0)
    x
  }
  list(
    inner = inner,
    params = lapply(params, dec),
    bounds = list(lower = num(a), upper = num(b)),
    x = as.list(dec(xs)),
    lower = as.list(num(vapply(xs, function(x) lower_mass(x) / Z, 0))),
    upper = as.list(num(vapply(xs, function(x) upper_mass(x) / Z, 0))),
    p = as.list(dec(near_ps)),
    quantile = as.list(num(vapply(near_ps, root, 0)))
  )
}
near_normal_case <- function(a, b, xs) {
  near_case("normal", list(mean = 0, sd = 1), a, b, xs, dnorm,
            function(x) normal_between(a, x), function(x) normal_between(x, b))
}
near_exponential_case <- function(rate, a, b, xs) {
  width <- expm1(-rate * (b - a))
  case <- near_case("exponential", list(rate = rate), a, b, xs,
                    function(x) rate * exp(-rate * (x - a)),
                    function(x) -expm1(-rate * (x - a)),
                    function(x) -exp(-rate * (x - a)) * expm1(-rate * (b - x)))
  # The closed form of the quantile, in place of the root.
  case$quantile <- as.list(num(vapply(near_ps, function(p) {
    if (p <= 0.5) a - log1p(p * width) / rate else b - log1p((1 - p) * expm1(rate * (b - a))) / rate
  }, 0)))
  case
}
near_t_case <- function(df, xs) {
  from_zero <- function(x) {
    if (x > 0.1) return(pt(x, df) - 0.5)
    term <- x; total <- 0
    for (n in 0:60) {
      total <- total + term / (2 * n + 1)
      term <- term * (-(df + 1) / 2 - n) / (n + 1) * x^2 / df
    }
    dt(0, df) * total
  }
  near_case("t", list(df = df), 0, Inf, xs, function(x) dt(x, df),
            function(x) if (is.infinite(x)) 0.5 else from_zero(x),
            function(x) pt(x, df, lower.tail = FALSE))
}
tiny <- c(1e-300, 1e-200, 1e-100, 1e-50, 1e-20, 1.196288e-12, 1e-8, 1e-4, 0.01)
near_bound <- list(
  near_normal_case(0, 2, c(tiny, 0.3, 1, 1.7, 2 - c(1e-4, 1e-8, 1e-12))),
  near_normal_case(5, 6, near_xs(5, 6, c(1e-12, 1e-8, 1e-4, 0.01, 0.3))),
  near_normal_case(-6, -5, near_xs(-6, -5, c(1e-12, 1e-8, 1e-4, 0.01, 0.3))),
  near_normal_case(-0.001, 0.001, c(-0.001 + 1e-15, -0.0005, 0, 0.0003, 0.000999)),
  near_normal_case(8, Inf, 8 + c(1e-10, 0.001, 0.1, 1, 4)),
  near_exponential_case(2, 3, 4, near_xs(3, 4, c(1e-12, 1e-6, 0.01, 0.5))),
  near_exponential_case(2, 30, 31, near_xs(30, 31, c(1e-12, 1e-6, 0.01, 0.5))),
  near_t_case(5, c(tiny, 0.05, 0.5, 2, 10))
)

write_json(list(reference = reference_block(), families = families, near_bound = near_bound), file.path(here(), "distributions.json"))
