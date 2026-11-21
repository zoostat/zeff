# R's binary-outcome fits for release 0.9.0, step 4 (Engine and Project Specification, section 14,
# the row of 0.9.0: the likelihood trait and the shared optimiser): `glm(family = binomial)` with
# the logit and the probit links, iterated to the maximum (epsilon 1e-15, and ten refits from
# R's own answer) so that the reference is the maximum itself and not where R's rule stops.
# Two inputs: a generated design whose every value is made by whole-number arithmetic, so that
# R and the Rust tests build the same rows bit for bit, and the shipped wages
# (`union ~ educ + exper + female`, its complete rows). For each fit: the coefficients, the
# log-likelihood, the deviance, the variance of the estimates from the expected information
# (`vcov`, dispersion one) and the number of rows. Base R only. Values to 17 significant digits.
# Run with `Rscript tests/reference/r/binary.R`.
source(file.path(dirname(sub("^--file=", "", commandArgs(FALSE)[grepl("^--file=", commandArgs(FALSE))])), "common.R"))
root <- normalizePath(file.path(here(), "..", "..", ".."))

num17 <- function(x) ifelse(is.na(x), "NA", sprintf("%.17g", x))

# The generated design: 400 rows; x1 and x2 from residues, the outcome from a third residue that
# leans on x1 and x2, so that both slopes are clearly away from zero and nothing separates.
generated <- function() {
  i <- 0:399
  x1 <- (i %% 17) / 4 - 2
  x2 <- ((i * 37) %% 23) / 5 - 2.2
  threshold <- 4 + (i %% 17) %/% 3 - ((i * 37) %% 23) %/% 6
  y <- as.numeric(((i * 7919) %% 11) < threshold)
  data.frame(y = y, x1 = x1, x2 = x2)
}

control <- glm.control(epsilon = 1e-15, maxit = 200)

# R's rule stops on the change of the deviance alone, which near the maximum is flat to second
# order: for a probit (whose scoring converges linearly) the coefficients are then still some
# 1e-10 from the maximum and the variance, taken at the last iteration's weights, some 1e-7.
# Each refit from the previous coefficients takes at least one more step; ten take the fit to
# the maximum to the last digits R can hold.
fit_case <- function(name, formula, data, link) {
  m <- glm(formula, family = binomial(link = link), data = data, control = control)
  for (again in 1:10) {
    m <- glm(formula, family = binomial(link = link), data = data, control = control,
             start = coef(m))
  }
  if (!m$converged) stop(sprintf("%s did not converge", name))
  v <- vcov(m)
  list(
    name = name,
    link = link,
    n = nrow(model.frame(m)),
    terms = as.list(names(coef(m))),
    coefficients = as.list(num17(unname(coef(m)))),
    log_likelihood = num17(as.numeric(logLik(m))),
    deviance = num17(deviance(m)),
    vcov = lapply(seq_len(nrow(v)), function(r) as.list(num17(unname(v[r, ]))))
  )
}

g <- generated()
wages <- read_dataset(file.path(root, "packages", "tier1", "datasets", "data", "wages.csv"))
wages <- wages[complete.cases(wages[, c("union", "educ", "exper", "female")]), ]

cases <- list(
  fit_case("generated_logit", y ~ x1 + x2, g, "logit"),
  fit_case("generated_probit", y ~ x1 + x2, g, "probit"),
  fit_case("wages_logit", union ~ educ + exper + female, wages, "logit"),
  fit_case("wages_probit", union ~ educ + exper + female, wages, "probit")
)

# A fit that converges with fitted probabilities numerically 0 or 1 (release 0.9.0, step 6b):
# tests/datasets/extreme_regressor.csv, 240 rows that overlap and three far out on `x`. R warns
# ("fitted probabilities numerically 0 or 1 occurred") by the rule of `glm.fit`, a fitted value
# within ten machine epsilons of 0 or 1; the count of such rows is written beside the fit.
extreme <- read_dataset(file.path(root, "tests", "datasets", "extreme_regressor.csv"))
extreme_case <- function(name, link) {
  warned <- character(0)
  m <- withCallingHandlers(
    glm(y ~ x + z, family = binomial(link = link), data = extreme, control = control),
    warning = function(w) {
      warned <<- c(warned, conditionMessage(w))
      invokeRestart("muffleWarning")
    }
  )
  fit <- suppressWarnings(fit_case(name, y ~ x + z, extreme, link))
  for (again in 1:10) {
    m <- suppressWarnings(glm(y ~ x + z, family = binomial(link = link), data = extreme,
                              control = control, start = coef(m)))
  }
  eps <- 10 * .Machine$double.eps
  mu <- fitted(m)
  fit$numerically_0_or_1 <- as.integer(sum(mu < eps | mu > 1 - eps))
  fit$r_warned <- any(grepl("numerically 0 or 1", warned))
  fit
}
extremes <- list(extreme_case("extreme_logit", "logit"), extreme_case("extreme_probit", "probit"))

# The robust and clustered variances of a binary fit (release 0.9.0, step 7b): `sandwich::vcovHC`
# with HC1, HC2 and HC3 and `sandwich::vcovCL` clustered by worker with `type = "HC1"`, which is
# CR1, G/(G − 1) · (N − 1)/(N − k), the factor the variance layer gives every model, on the fit
# iterated to its maximum. For a fit that is not least squares `vcovCL` takes `type = "HC0"`
# unless told, G/(G − 1) alone, which is Stata's factor for `logit` and `probit` too: that
# matrix is written beside it as `cluster_default`, and differs by (N − 1)/(N − k). The rows
# are those complete in the model's variables and in the cluster variable, as `glm` with
# `cluster:` uses them. Two models: the one of the classical reference, and the sample script's
# (section 10 of zeff_samples.zeff), with the industry.
# sandwich: from the harness's own library when install.R made it, as regression.R reads it.
.libPaths(c(file.path(here(), "library"), .libPaths()))
library(sandwich)
matrix_rows <- function(v) lapply(seq_len(nrow(v)), function(r) as.list(num17(unname(v[r, ]))))
robust_case <- function(name, formula, data, link) {
  m <- glm(formula, family = binomial(link = link), data = data, control = control)
  for (again in 1:10) {
    m <- glm(formula, family = binomial(link = link), data = data, control = control,
             start = coef(m))
  }
  if (!m$converged) stop(sprintf("%s did not converge", name))
  list(
    name = name,
    link = link,
    formula = paste(deparse(formula), collapse = " "),
    n = nrow(model.frame(m)),
    clusters = length(unique(data$worker)),
    terms = as.list(names(coef(m))),
    coefficients = as.list(num17(unname(coef(m)))),
    hc1 = matrix_rows(vcovHC(m, type = "HC1")),
    hc2 = matrix_rows(vcovHC(m, type = "HC2")),
    hc3 = matrix_rows(vcovHC(m, type = "HC3")),
    cluster = matrix_rows(vcovCL(m, cluster = data$worker, type = "HC1")),
    cluster_default = matrix_rows(vcovCL(m, cluster = data$worker))
  )
}
by_worker <- read_dataset(file.path(root, "packages", "tier1", "datasets", "data", "wages.csv"))
by_worker <- by_worker[complete.cases(by_worker[, c("union", "educ", "exper", "female", "industry", "worker")]), ]
# The levels of the industry in the order of the shipped dataset's category, so that the base
# level, and with it every coefficient, is the one `glm` has.
by_worker$industry <- factor(by_worker$industry, levels = c(
  "Agriculture", "Mining", "Utilities", "Construction", "Manufacturing", "Retail", "Transport",
  "Finance", "Services", "Education", "Health", "Public administration"))
if (anyNA(by_worker$industry)) stop("an industry outside the dataset's levels")
robust <- list(
  robust_case("wages_logit", union ~ educ + exper + female, by_worker, "logit"),
  robust_case("wages_probit", union ~ educ + exper + female, by_worker, "probit"),
  robust_case("sample_logit", union ~ educ + exper + female + industry, by_worker, "logit"),
  robust_case("sample_probit", union ~ educ + exper + female + industry, by_worker, "probit")
)

# Weights (release 0.9.0, step 7c), on `union ~ educ + exper + female`, each link, the fit
# iterated to its maximum.
# Sampling weights: `hours`, scaled to a mean of one over the rows used (the estimates and the
# sandwich do not depend on the scale); the rows with hours missing or zero are left out. The
# family is the binomial, which only warns of weights that are not whole numbers. `vcovHC` and
# `vcovCL(type = "HC1")` of the weighted fit, and the weighted log-likelihood by hand, since
# R's `logLik` rounds the weights.
# Frequency weights: `copies`, 1, 2 or 3 by the row's place in the file. The reference is the
# fit on the rows written out `copies` times each, which is what a frequency weight means: its
# classical variance, its sandwiches, its log-likelihood and its count of observations.
raw <- read_dataset(file.path(root, "packages", "tier1", "datasets", "data", "wages.csv"))
raw$copies <- (seq_len(nrow(raw)) - 1) %% 3 + 1
iterated <- function(formula, data, link, weights = NULL) {
  data$.w <- if (is.null(weights)) rep(1, nrow(data)) else weights
  fit <- function(start = NULL) {
    suppressWarnings(glm(formula, family = binomial(link = link), data = data, weights = .w,
                         control = control, start = start))
  }
  m <- fit()
  for (again in 1:10) m <- fit(coef(m))
  if (!m$converged) stop("a weighted fit did not converge")
  m
}
sampling_case <- function(link) {
  d <- raw[complete.cases(raw[, c("union", "educ", "exper", "female", "hours", "worker")]), ]
  d <- d[d$hours > 0, ]
  w <- d$hours / mean(d$hours)
  m <- iterated(union ~ educ + exper + female, d, link, w)
  mu <- fitted(m)
  list(
    name = paste0("sampling_", link),
    kind = "sampling",
    link = link,
    n = nrow(d),
    coefficients = as.list(num17(unname(coef(m)))),
    log_likelihood = num17(sum(w * (d$union * log(mu) + (1 - d$union) * log1p(-mu)))),
    hc1 = matrix_rows(vcovHC(m, type = "HC1")),
    hc2 = matrix_rows(vcovHC(m, type = "HC2")),
    hc3 = matrix_rows(vcovHC(m, type = "HC3")),
    cluster = matrix_rows(vcovCL(m, cluster = d$worker, type = "HC1"))
  )
}
frequency_case <- function(link) {
  d <- raw[complete.cases(raw[, c("union", "educ", "exper", "female", "worker")]), ]
  e <- d[rep(seq_len(nrow(d)), d$copies), ]
  m <- iterated(union ~ educ + exper + female, e, link)
  list(
    name = paste0("frequency_", link),
    kind = "frequency",
    link = link,
    n = nrow(e),
    rows = nrow(d),
    coefficients = as.list(num17(unname(coef(m)))),
    log_likelihood = num17(as.numeric(logLik(m))),
    deviance = num17(deviance(m)),
    vcov = matrix_rows(vcov(m)),
    hc1 = matrix_rows(vcovHC(m, type = "HC1")),
    hc2 = matrix_rows(vcovHC(m, type = "HC2")),
    hc3 = matrix_rows(vcovHC(m, type = "HC3")),
    cluster = matrix_rows(vcovCL(m, cluster = e$worker, type = "HC1"))
  )
}
weighted <- list(sampling_case("logit"), sampling_case("probit"),
                 frequency_case("logit"), frequency_case("probit"))

# A nearly singular design (release 0.9.0, step 7d): `union ~ educ + e2 + x2` with
# e2 = educ + 1e-5 exper and x2 = exper + 1e-4 female, a condition number near 7e10. Two
# estimates near 5e7 cancel in the linear predictor, so neither R nor any double-precision fit
# holds them beyond some four digits, and R says that it did not converge at this tolerance.
# The three regressors span what educ, exper and female span, so the exact answer is known from
# the well-conditioned fit above (`cases`), and the test compares with that; R's fit of this
# design is written here to show what the design allows, and for its standard errors. It is
# written to three significant digits, which is what R holds of it on every machine: on Linux
# and on macOS its fourth digit differs (55567270 and 55578983 for the estimate of `educ`,
# where the exact answer is 55567322), so no more can be compared with a committed copy.
near <- raw[complete.cases(raw[, c("union", "educ", "exper", "female")]), ]
near$e2 <- near$educ + 1e-5 * near$exper
near$x2 <- near$exper + 1e-4 * near$female
near_case <- function(link) {
  fit <- function(start = NULL) {
    suppressWarnings(glm(union ~ educ + e2 + x2, family = binomial(link = link), data = near,
                         control = control, start = start))
  }
  m <- fit()
  for (again in 1:10) m <- fit(coef(m))
  list(
    name = paste0("near_singular_", link),
    link = link,
    n = nrow(near),
    coefficients = as.list(sprintf("%.3g", unname(coef(m)))),
    std_errors = as.list(sprintf("%.3g", unname(sqrt(diag(vcov(m))))))
  )
}
near_singular <- list(near_case("logit"), near_case("probit"))

out <- list(
  software = paste("R", getRversion()),
  generated_rows = nrow(g),
  generated_ones = as.integer(sum(g$y)),
  cases = cases,
  extremes = extremes,
  robust = robust,
  weighted = weighted,
  near_singular = near_singular
)
writeLines(to_json(out), file.path(here(), "binary.json"))
