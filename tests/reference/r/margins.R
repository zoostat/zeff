# R's average marginal effects for release 0.9.0, step 11a (open decisions #88 and #195): the
# effects `margins` gives after `reg`, written out in base R on `lm` fits of the shipped wages.
#
# The slope of a variable is taken through every term that is a function of it, exactly: the
# derivative of each variable of the model frame by `D`, put in its place in the model matrix
# (the product rule over the parts of an interaction), averaged over the rows the fit used. The
# effect of a 0/1 variable, and of each level of a category against the first, is the average
# difference between the model matrix at the two values. The standard error is by the delta
# method with the regressors held fixed, sqrt(J V J'), J the averaged row and V the fit's
# variance (classical, HC1, HC3 or clustered, as `regression.R` computes them); the t table is
# on n - k degrees of freedom, or G - 1 when clustered. Under sampling weights the average is
# weighted by them; frequency weights replicate the rows. Values to 17 significant digits. Run
# with `Rscript tests/reference/r/margins.R`.
#
# Step 11b: `at:` sets a variable to a value in every row, the others as observed; `type: mem`
# takes each effect at the means of the regressors, a number at its mean and each level of a
# category at its share of the rows (Stata's `atmeans`), which is the weighted sum over the grid
# of the categories' levels, each cell weighted by the product of its levels' shares, since
# every column of the model matrix is a product of its parts; with no variable named, the
# adjusted prediction, J the averaged row of the model matrix itself.
source(file.path(dirname(sub("^--file=", "", commandArgs(FALSE)[grepl("^--file=", commandArgs(FALSE))])), "common.R"))
Sys.setlocale("LC_COLLATE", "C")
root <- normalizePath(file.path(here(), "..", "..", ".."))
.libPaths(c(file.path(here(), "library"), .libPaths()))
library(sandwich)

num17 <- function(x) ifelse(is.na(x), "NA", sprintf("%.17g", x))

industry_levels <- c("Agriculture", "Mining", "Utilities", "Construction", "Manufacturing", "Retail",
                     "Transport", "Finance", "Services", "Education", "Health", "Public administration")

wages <- read_dataset(file.path(root, "packages", "tier1", "datasets", "data", "wages.csv"))
wages$industry <- factor(wages$industry, levels = industry_levels)

# The derivative of the model matrix in the numeric variable `var`, row by row: for each
# variable of the model frame that is a function of `var`, the matrix with that variable
# replaced by its derivative, kept in the columns of the terms the variable enters; summed.
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
    # I(...) keeps arithmetic out of the formula's own operators; inside it is plain arithmetic.
    if (is.call(expr) && identical(expr[[1]], as.name("I"))) expr <- expr[[2]]
    changed <- frame
    changed[[name]] <- rep_len(as.numeric(eval(D(expr, var), data)), nrow(data))
    Xd <- model.matrix(tt, changed, contrasts.arg = fit$contrasts)
    enters <- assign > 0 & factors[name, pmax(assign, 1)] > 0
    total[, enters] <- total[, enters] + Xd[, enters]
  }
  total
}

# The model matrix with `var` set to `value` in every row.
matrix_at <- function(fit, data, var, value) {
  tt <- delete.response(terms(fit))
  data[[var]] <- if (is.factor(data[[var]])) factor(value, levels = levels(data[[var]])) else value
  model.matrix(tt, model.frame(tt, data, xlev = fit$xlevels), contrasts.arg = fit$contrasts)
}

# One case: the fit as `regression.R` makes it, then one row for each effect asked for. `vars`
# names the variables in order; a factor gives a row for each level after the first.
case <- function(zeff, r_formula, vars, se = "classical", cluster = NULL, weights = NULL,
                 weight_type = "sampling", contrasts = NULL, binary = c("female", "union"),
                 type = "ame", at = list()) {
  data <- wages
  w <- NULL
  if (!is.null(weights) && weight_type == "frequency") {
    times <- ifelse(is.na(data[[weights]]), 0L, as.integer(data[[weights]]))
    data <- data[rep(seq_len(nrow(data)), times), , drop = FALSE]
    fit <- lm(r_formula, data = data, contrasts = contrasts)
  } else if (!is.null(weights)) {
    data$zeff_w <- data[[weights]]
    fit <- lm(r_formula, data = data, weights = zeff_w, contrasts = contrasts)
  } else {
    fit <- lm(r_formula, data = data, contrasts = contrasts)
  }
  used <- data[rownames(model.frame(fit)), , drop = FALSE]
  if (!is.null(weights) && weight_type != "frequency") {
    used <- used[used$zeff_w > 0, , drop = FALSE]
    w <- used$zeff_w
  }
  df <- fit$df.residual
  V <- vcov(fit)
  if (se == "cluster") {
    g <- data[[cluster]][as.integer(match(rownames(model.frame(fit)), rownames(data)))]
    df <- length(unique(g)) - 1
    V <- vcovCL(fit, cluster = g, type = "HC1", cadjust = TRUE)
  } else if (se != "classical") {
    V <- vcovHC(fit, type = se)
  }
  beta <- coef(fit)
  stopifnot(!anyNA(beta))
  # Which variables are 0/1 and which categories is read from the rows as they are.
  is_category <- function(var) is.factor(wages[[var]])
  # The rows the effects are taken at, and their weights. At the means: one row for each cell
  # of the categories' levels, the numbers at their (weighted) means, the cell weighted by the
  # product of the (weighted) shares of its levels.
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
  average <- function(M) if (is.null(w)) colMeans(M) else colSums(M * w) / sum(w)
  rows <- list()
  add <- function(term, contrast, J) {
    estimate <- sum(J * beta)
    std_error <- sqrt(as.numeric(t(J) %*% V %*% J))
    t_value <- estimate / std_error
    q <- qt(0.975, df)
    rows[[length(rows) + 1]] <<- list(
      term = term, contrast = contrast,
      estimate = num17(estimate), std_error = num17(std_error), t_value = num17(t_value),
      p_value = num17(2 * pt(abs(t_value), df, lower.tail = FALSE)),
      ci_lower = num17(estimate - q * std_error), ci_upper = num17(estimate + q * std_error)
    )
  }
  if (length(vars) == 0) {
    tt <- delete.response(terms(fit))
    add("prediction", "", average(model.matrix(tt, model.frame(tt, used, xlev = fit$xlevels),
                                              contrasts.arg = fit$contrasts)))
  }
  for (var in vars) {
    if (is.factor(used[[var]])) {
      levels_of <- levels(used[[var]])
      low <- matrix_at(fit, used, var, levels_of[1])
      for (level in levels_of[-1]) {
        add(paste0(var, "=", level), paste0(level, " − ", levels_of[1]),
            average(matrix_at(fit, used, var, level) - low))
      }
    } else if (var %in% binary) {
      add(var, "1 − 0", average(matrix_at(fit, used, var, 1) - matrix_at(fit, used, var, 0)))
    } else {
      add(var, "dy/dx", average(slope_matrix(fit, used, var)))
    }
  }
  list(formula = zeff, se = se, cluster = if (is.null(cluster)) "" else cluster,
       weights = if (is.null(weights)) "" else weights, weight_type = weight_type,
       type = type, at = lapply(at, function(v) if (is.character(v)) v else num17(v)),
       vars = as.list(vars), nobs = nobs(fit), df = as.integer(df), rows = rows)
}


# Several values in `at:` (step 12b): the case at each combination of the values of `grid`, the
# first variable changing slowest, with `fixed` held throughout; each row is named with the
# values it was taken at, `exper @ exper=5`.
grid_case <- function(zeff, r_formula, vars, grid, fixed = list(), ...) {
  combos <- list(list())
  for (name in names(grid)) {
    combos <- unlist(lapply(combos, function(chosen) {
      lapply(grid[[name]], function(value) c(chosen, setNames(list(value), name)))
    }), recursive = FALSE)
  }
  rows <- list()
  out <- NULL
  for (chosen in combos) {
    one <- case(zeff, r_formula, vars, at = c(chosen, fixed), ...)
    label <- paste(names(chosen), vapply(chosen, as.character, ""), sep = "=", collapse = ", ")
    for (row in one$rows) {
      row$term <- paste0(row$term, " @ ", label)
      rows[[length(rows) + 1]] <- row
    }
    out <- one
  }
  out$rows <- rows
  out$at <- lapply(fixed, function(v) if (is.character(v)) v else num17(v))
  out$grid <- lapply(grid, function(values) lapply(values, function(v) if (is.character(v)) v else num17(v)))
  out
}

cases <- list()
# A quadratic: the slope of exper is b1 + 2 b2 exper, averaged.
cases$quadratic <- case("log(wage) ~ educ + exper + exper^2 + female",
  log(wage) ~ educ + exper + I(exper^2) + female, c("educ", "exper", "female"))
# An interaction with a 0/1 variable and a logarithm, robust errors.
cases$interaction <- case("log(wage) ~ educ * female + exper + log(hours)",
  log(wage) ~ educ * female + exper + log(hours), c("educ", "female", "hours", "exper"), se = "HC1")
# A category interacted with a 0/1 variable and with a number, clustered errors.
cases$category <- case("log(wage) ~ industry * female + industry:educ + educ + exper",
  log(wage) ~ industry * female + industry:educ + educ + exper, c("industry", "female", "educ"),
  se = "cluster", cluster = "worker")
# Smooth functions and a product of two numbers, HC3 errors.
cases$functions <- case("log(wage) ~ log(hours) + sqrt(exper + 1) + educ:exper + (educ / (exper + 1)^0.5)",
  log(wage) ~ log(hours) + sqrt(exper + 1) + educ:exper + I(educ / (exper + 1)^0.5), c("hours", "exper", "educ"),
  se = "HC3")
# The effects of a category do not depend on its coding.
summed <- list(industry = contr.sum(12)[c(12, 1:11), ])
cases$sum_contrast <- case("log(wage) ~ category(industry, contrast: sum) * female + educ",
  log(wage) ~ industry * female + educ, c("industry", "female"), contrasts = summed)
# Sampling weights: the average is weighted, the errors HC1.
cases$sampling <- case("log(wage) ~ educ * female + exper + exper^2",
  log(wage) ~ educ * female + exper + I(exper^2), c("educ", "female", "exper"),
  se = "HC1", weights = "hours", weight_type = "sampling")
# Frequency weights: each row stands for as many observations.
cases$frequency <- case("log(wage) ~ exper + exper^2 + female + female:exper",
  log(wage) ~ exper + I(exper^2) + female + female:exper, c("exper", "female"),
  weights = "educ", weight_type = "frequency")


# ----- step 11b: `at:`, `type: mem` and the adjusted prediction -----
wide <- log(wage) ~ educ * female + exper + I(exper^2) + industry + industry:exper
wide_zeff <- "log(wage) ~ educ * female + exper + exper^2 + industry + industry:exper"
# A variable set to a value, the others as observed: the slope of exper at 10 years, the effect
# of female at 12 years of schooling.
cases$at_numbers <- case(wide_zeff, wide, c("exper", "female", "educ"), se = "HC1",
  at = list(exper = 10, educ = 12))
# A category set to a level: the effects in one industry.
cases$at_level <- case("log(wage) ~ industry * female + industry:educ + educ + exper",
  log(wage) ~ industry * female + industry:educ + educ + exper, c("female", "educ"),
  se = "cluster", cluster = "worker", at = list(industry = "Mining"))
# At the means: numbers at their means, each level of the category at its share.
cases$mem <- case(wide_zeff, wide, c("educ", "exper", "female", "industry"), type = "mem")
cases$mem_at <- case(wide_zeff, wide, c("exper", "female"), type = "mem", se = "HC3",
  at = list(exper = 10.5, industry = "Finance"))
# At the means of a model with smooth functions, where the effect at the means is not the
# average effect: the slope of log(hours) is b / mean(hours), not b times the mean of 1 / hours.
cases$mem_functions <- case("log(wage) ~ log(hours) + sqrt(exper + 1) + educ:exper + (educ / (exper + 1)^0.5)",
  log(wage) ~ log(hours) + sqrt(exper + 1) + educ:exper + I(educ / (exper + 1)^0.5), c("hours", "exper", "educ"),
  se = "HC3", type = "mem")
# At the weighted means under sampling weights.
cases$mem_sampling <- case("log(wage) ~ educ * female + exper + exper^2",
  log(wage) ~ educ * female + exper + I(exper^2), c("educ", "female", "exper"),
  se = "HC1", weights = "hours", weight_type = "sampling", type = "mem")
# The adjusted prediction: the average of the predictions, at values, and at the means.
cases$prediction <- case(wide_zeff, wide, character(0))
cases$prediction_at <- case(wide_zeff, wide, character(0), se = "HC1",
  at = list(exper = 10, industry = "Mining", female = 1))
cases$prediction_mem <- case(wide_zeff, wide, character(0), type = "mem",
  at = list(educ = 16))
cases$prediction_frequency <- case("log(wage) ~ exper + exper^2 + female + female:exper",
  log(wage) ~ exper + I(exper^2) + female + female:exper, character(0),
  weights = "educ", weight_type = "frequency", at = list(female = 0))

# ----- step 12b: several values in `at:` -----
cases$grid_effects <- grid_case(wide_zeff, wide, c("exper", "female"), se = "HC1",
  grid = list(exper = list(5, 20), industry = list("Mining", "Finance")), fixed = list(educ = 12))
cases$grid_prediction <- grid_case(wide_zeff, wide, character(0),
  grid = list(exper = list(5, 10.5, 20)))
cases$grid_mem <- grid_case(wide_zeff, wide, c("educ"), type = "mem",
  grid = list(female = list(0, 1)))

reference <- reference_block()
reference$note <- "base R and sandwich; numbers written with 17 significant digits"
write_json(list(reference = reference, cases = cases), file.path(here(), "margins.json"))
