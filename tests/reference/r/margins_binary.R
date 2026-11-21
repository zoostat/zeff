# R's marginal effects of a binary model for release 0.9.0, step 12a (open decisions #88, #195
# and #202): what `margins` gives after `glm(family: binomial)`, on the scale of the probability,
# written out in base R on `glm` fits of the shipped wages iterated to their maximum.
#
# With p = g(eta) the inverse link of the linear predictor eta = x'b, and X the model matrix at
# the rows the effects are taken at: the slope of a number is g'(eta) times the derivative of
# eta, the derivative of the model matrix (through every term the variable enters) times b; the
# effect of a 0/1 variable or of a level of a category is the difference g(eta) at the two
# values; the adjusted prediction is g(eta). Each is averaged over the rows (by the fit's
# weights under weights) or taken at the means, a number at its mean and each level of a
# category at its share. The standard error is by the delta method, sqrt(J V J') with J the
# gradient of the averaged quantity in b,
#   slope:       mean of g''(eta) (dX b) X + g'(eta) dX
#   difference:  mean of g'(eta1) X1 - g'(eta0) X0
#   prediction:  mean of g'(eta) X
# and V the fit's variance: classical, or `sandwich`'s with `type = "HC1"` as Zeff's variance
# layer has it for every model (open decision #198). The z table is the normal distribution's.
# Values to 17 significant digits. Run with `Rscript tests/reference/r/margins_binary.R`.
source(file.path(dirname(sub("^--file=", "", commandArgs(FALSE)[grepl("^--file=", commandArgs(FALSE))])), "common.R"))
Sys.setlocale("LC_COLLATE", "C")
root <- normalizePath(file.path(here(), "..", "..", ".."))
.libPaths(c(file.path(here(), "library"), .libPaths()))
library(sandwich)

num17 <- function(x) ifelse(is.na(x), "NA", sprintf("%.17g", x))
control <- glm.control(epsilon = 1e-14, maxit = 100)

industry_levels <- c("Agriculture", "Mining", "Utilities", "Construction", "Manufacturing", "Retail",
                     "Transport", "Finance", "Services", "Education", "Health", "Public administration")

wages <- read_dataset(file.path(root, "packages", "tier1", "datasets", "data", "wages.csv"))
wages$industry <- factor(wages$industry, levels = industry_levels)

# The inverse link with its first two derivatives.
inverse_link <- function(link) {
  if (link == "logit") {
    list(g = plogis, g1 = dlogis, g2 = function(eta) dlogis(eta) * (1 - 2 * plogis(eta)))
  } else {
    list(g = pnorm, g1 = dnorm, g2 = function(eta) -eta * dnorm(eta))
  }
}

# The derivative of the model matrix in the numeric variable `var` (see margins.R).
slope_matrix <- function(fit, data, var) {
  tt <- delete.response(terms(fit))
  frame <- model.frame(tt, data, xlev = fit$xlevels)
  X <- model.matrix(tt, frame, contrasts.arg = fit$contrasts)
  factors <- attr(tt, "factors")
  assign <- attr(X, "assign")
  total <- matrix(0, nrow(X), ncol(X), dimnames = dimnames(X))
  for (name in rownames(factors)) {
    expr <- parse(text = name)[[1]]
    if (!(var %in% all.vars(expr))) next
    if (is.call(expr) && identical(expr[[1]], as.name("I"))) expr <- expr[[2]]
    changed <- frame
    changed[[name]] <- rep_len(as.numeric(eval(D(expr, var), data)), nrow(data))
    Xd <- model.matrix(tt, changed, contrasts.arg = fit$contrasts)
    enters <- assign > 0 & factors[name, pmax(assign, 1)] > 0
    total[, enters] <- total[, enters] + Xd[, enters]
  }
  total
}

matrix_of <- function(fit, data) {
  tt <- delete.response(terms(fit))
  model.matrix(tt, model.frame(tt, data, xlev = fit$xlevels), contrasts.arg = fit$contrasts)
}

matrix_at <- function(fit, data, var, value) {
  data[[var]] <- if (is.factor(data[[var]])) factor(value, levels = levels(data[[var]])) else value
  matrix_of(fit, data)
}

case <- function(zeff, r_formula, link, vars, se = "classical", cluster = NULL, weights = NULL,
                 weight_type = "sampling", binary = c("female"), type = "ame", at = list()) {
  data <- wages
  w <- NULL
  iterate <- function(fit_once) {
    m <- fit_once(NULL)
    for (again in 1:10) m <- fit_once(coef(m))
    if (!m$converged) stop("a fit did not converge")
    m
  }
  if (!is.null(weights) && weight_type == "frequency") {
    times <- ifelse(is.na(data[[weights]]), 0L, as.integer(data[[weights]]))
    data <- data[rep(seq_len(nrow(data)), times), , drop = FALSE]
    fit <- iterate(function(start) glm(r_formula, family = binomial(link = link), data = data,
                                       control = control, start = start))
  } else if (!is.null(weights)) {
    data$zeff_w <- data[[weights]]
    fit <- iterate(function(start) suppressWarnings(
      glm(r_formula, family = binomial(link = link), data = data, weights = zeff_w,
          control = control, start = start)))
  } else {
    fit <- iterate(function(start) glm(r_formula, family = binomial(link = link), data = data,
                                       control = control, start = start))
  }
  used <- data[rownames(model.frame(fit)), , drop = FALSE]
  if (!is.null(weights) && weight_type != "frequency") {
    used <- used[used$zeff_w > 0, , drop = FALSE]
    w <- used$zeff_w
  }
  V <- vcov(fit)
  if (se == "cluster") {
    g <- data[[cluster]][as.integer(match(rownames(model.frame(fit)), rownames(data)))]
    V <- vcovCL(fit, cluster = g, type = "HC1")
  } else if (se != "classical") {
    V <- vcovHC(fit, type = se)
  }
  beta <- coef(fit)
  stopifnot(!anyNA(beta))
  il <- inverse_link(link)
  is_category <- function(var) is.factor(wages[[var]])
  if (type == "mem") {
    model_vars <- all.vars(delete.response(terms(fit)))
    ww <- if (is.null(w)) rep(1, nrow(used)) else w
    factors_in <- model_vars[vapply(model_vars, is_category, TRUE)]
    one <- used[1, , drop = FALSE]
    for (v in setdiff(model_vars, factors_in)) one[[v]] <- sum(used[[v]] * ww) / sum(ww)
    if (length(factors_in) == 0) {
      used <- one
      w <- NULL
    } else {
      # At the means the linear predictor is the model matrix at the means times b: the
      # weighted sum of the grid's rows, one row of X, not one prediction for each cell.
      grid <- expand.grid(lapply(setNames(factors_in, factors_in), function(v) levels(used[[v]])),
                          stringsAsFactors = FALSE)
      cell <- rep(1, nrow(grid))
      rows_at <- one[rep(1, nrow(grid)), , drop = FALSE]
      for (v in factors_in) {
        share <- tapply(ww, used[[v]], sum) / sum(ww)
        share[is.na(share)] <- 0
        cell <- cell * share[grid[[v]]]
        rows_at[[v]] <- factor(grid[[v]], levels = levels(used[[v]]))
      }
      used <- rows_at
      w <- as.numeric(cell)
    }
  }
  for (v in names(at)) {
    used[[v]] <- if (is_category(v)) factor(at[[v]], levels = levels(wages[[v]])) else at[[v]]
  }
  # At the means the rows of `used` are the cells of one point: their matrices are averaged
  # into one row before the link is applied. Otherwise every row is a point of its own.
  point <- function(M) {
    if (type == "mem") {
      matrix(if (is.null(w)) colMeans(M) else colSums(M * w) / sum(w), nrow = 1,
             dimnames = list(NULL, colnames(M)))
    } else M
  }
  wp <- if (type == "mem") NULL else w
  average <- function(M) if (is.null(wp)) colMeans(M) else colSums(M * wp) / sum(wp)
  mean_of <- function(x) if (is.null(wp)) mean(x) else sum(x * wp) / sum(wp)
  rows <- list()
  add <- function(term, contrast, estimate, J) {
    std_error <- sqrt(as.numeric(t(J) %*% V %*% J))
    z_value <- estimate / std_error
    q <- qnorm(0.975)
    rows[[length(rows) + 1]] <<- list(
      term = term, contrast = contrast,
      estimate = num17(estimate), std_error = num17(std_error), z_value = num17(z_value),
      p_value = num17(2 * pnorm(abs(z_value), lower.tail = FALSE)),
      ci_lower = num17(estimate - q * std_error), ci_upper = num17(estimate + q * std_error)
    )
  }
  difference <- function(term, contrast, X1, X0) {
    X1 <- point(X1); X0 <- point(X0)
    eta1 <- as.numeric(X1 %*% beta); eta0 <- as.numeric(X0 %*% beta)
    add(term, contrast, mean_of(il$g(eta1) - il$g(eta0)),
        average(il$g1(eta1) * X1 - il$g1(eta0) * X0))
  }
  if (length(vars) == 0) {
    X <- point(matrix_of(fit, used))
    eta <- as.numeric(X %*% beta)
    add("prediction", "", mean_of(il$g(eta)), average(il$g1(eta) * X))
  }
  for (var in vars) {
    if (is.factor(used[[var]])) {
      levels_of <- levels(used[[var]])
      low <- matrix_at(fit, used, var, levels_of[1])
      for (level in levels_of[-1]) {
        difference(paste0(var, "=", level), paste0(level, " − ", levels_of[1]),
                   matrix_at(fit, used, var, level), low)
      }
    } else if (var %in% binary) {
      difference(var, "1 − 0", matrix_at(fit, used, var, 1), matrix_at(fit, used, var, 0))
    } else {
      X <- point(matrix_of(fit, used))
      dX <- point(slope_matrix(fit, used, var))
      eta <- as.numeric(X %*% beta)
      s <- as.numeric(dX %*% beta)
      add(var, "dy/dx", mean_of(il$g1(eta) * s), average(il$g2(eta) * s * X + il$g1(eta) * dX))
    }
  }
  list(formula = zeff, link = link, se = se, cluster = if (is.null(cluster)) "" else cluster,
       weights = if (is.null(weights)) "" else weights, weight_type = weight_type,
       type = type, at = lapply(at, function(v) if (is.character(v)) v else num17(v)),
       vars = as.list(vars), nobs = nobs(fit), rows = rows)
}

cases <- list()
# The average effects of a logit with a square.
cases$logit <- case("union ~ educ + exper + exper^2 + female",
  union ~ educ + exper + I(exper^2) + female, "logit", c("educ", "exper", "female"))
# A probit with an interaction and a logarithm, robust errors.
cases$probit_robust <- case("union ~ educ * female + exper + log(hours)",
  union ~ educ * female + exper + log(hours), "probit", c("educ", "female", "hours", "exper"),
  se = "HC1")
# A category and an interaction with it, clustered errors.
cases$logit_category <- case("union ~ industry + educ + female + female:educ + industry:exper",
  union ~ industry + educ + female + female:educ + industry:exper, "logit",
  c("industry", "female", "educ", "exper"), se = "cluster", cluster = "worker")
wide <- union ~ educ * female + exper + I(exper^2) + industry
wide_zeff <- "union ~ educ * female + exper + exper^2 + industry"
# Variables held at values, the others as observed.
cases$logit_at <- case(wide_zeff, wide, "logit", c("exper", "female", "educ"), se = "HC1",
  at = list(exper = 10, industry = "Mining"))
# At the means: the link is applied at the one point, not averaged over the cells.
cases$logit_mem <- case(wide_zeff, wide, "logit", c("educ", "exper", "female", "industry"),
  type = "mem")
cases$probit_mem_at <- case(wide_zeff, wide, "probit", c("exper", "female"), type = "mem",
  se = "HC3", at = list(exper = 10.5, industry = "Finance"))
# The adjusted prediction: the average probability, at values, and at the means.
cases$prediction <- case(wide_zeff, wide, "logit", character(0))
cases$prediction_at <- case(wide_zeff, wide, "probit", character(0), se = "HC1",
  at = list(exper = 10, industry = "Mining", female = 1))
cases$prediction_mem <- case(wide_zeff, wide, "logit", character(0), type = "mem",
  at = list(educ = 16))
# Sampling weights: the average is weighted, the errors HC1.
cases$logit_sampling <- case("union ~ educ * female + exper + exper^2",
  union ~ educ * female + exper + I(exper^2), "logit", c("educ", "female", "exper"),
  se = "HC1", weights = "hours", weight_type = "sampling")
cases$probit_sampling_mem <- case("union ~ educ * female + exper + exper^2",
  union ~ educ * female + exper + I(exper^2), "probit", c("educ", "female"),
  se = "HC1", weights = "hours", weight_type = "sampling", type = "mem")
# Frequency weights: each row stands for as many observations.
cases$logit_frequency <- case("union ~ exper + exper^2 + female + female:exper",
  union ~ exper + I(exper^2) + female + female:exper, "logit", c("exper", "female"),
  weights = "educ", weight_type = "frequency")

reference <- reference_block()
reference$note <- "base R and sandwich; numbers written with 17 significant digits"
write_json(list(reference = reference, cases = cases), file.path(here(), "margins_binary.json"))
