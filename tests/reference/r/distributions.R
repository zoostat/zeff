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
    pdf = as.list(num(d(xs))),
    lower = as.list(num(p(xs))),
    upper = as.list(num(p(xs, lower.tail = FALSE))),
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
         function(x) dexp(x, 25), function(x, lower.tail = TRUE) pexp(x, 25, lower.tail = lower.tail), function(p) qexp(p, 25))
)

write_json(list(reference = reference_block(), families = families), file.path(here(), "distributions.json"))
