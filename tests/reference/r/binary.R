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

out <- list(
  software = paste("R", getRversion()),
  generated_rows = nrow(g),
  generated_ones = as.integer(sum(g$y)),
  cases = cases
)
writeLines(to_json(out), file.path(here(), "binary.json"))
