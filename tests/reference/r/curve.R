# R's ROC curves for release 0.9.0, step 14 (catalogue row 515; open decision #112): the points
# of the curve at every distinct threshold and the area under it, for `glm` fits of the shipped
# wages iterated to their maximum, written out in base R.
#
# With the scores the fitted probabilities, in falling order: at each distinct score t, the
# sensitivity is the share of the ones whose score is at least t and the specificity the share
# of the noughts whose score is below t. The area is by the trapezoid rule over those points
# from (0, 0), which is the Mann-Whitney statistic of the scores with ties counted half. Under
# weights every count is a sum of weights; frequency weights are also checked against the rows
# written out as many times. Values to 17 significant digits. Run with
# `Rscript tests/reference/r/curve.R`.
source(file.path(dirname(sub("^--file=", "", commandArgs(FALSE)[grepl("^--file=", commandArgs(FALSE))])), "common.R"))
Sys.setlocale("LC_COLLATE", "C")
root <- normalizePath(file.path(here(), "..", "..", ".."))

num17 <- function(x) ifelse(is.na(x), "NA", sprintf("%.17g", x))
control <- glm.control(epsilon = 1e-14, maxit = 100)

industry_levels <- c("Agriculture", "Mining", "Utilities", "Construction", "Manufacturing", "Retail",
                     "Transport", "Finance", "Services", "Education", "Health", "Public administration")
wages <- read_dataset(file.path(root, "packages", "tier1", "datasets", "data", "wages.csv"))
wages$industry <- factor(wages$industry, levels = industry_levels)

iterated <- function(formula, data, link, weights = NULL) {
  data$zeff_w <- if (is.null(weights)) rep(1, nrow(data)) else data[[weights]]
  fit <- function(start = NULL) {
    suppressWarnings(glm(formula, family = binomial(link = link), data = data, weights = zeff_w,
                         control = control, start = start))
  }
  m <- fit()
  for (again in 1:10) m <- fit(coef(m))
  if (!m$converged) stop("a fit did not converge")
  m
}

# The points and the area from scores, outcomes and weights. R's fitted values of two rows with
# the same regressors may differ in their last binary digit, and whether they do differs
# between platforms (5,721 distinct values of the probit below on macOS, 5,722 on Linux), so the
# scores are rounded to 12 significant digits before ties are counted: rows the model cannot
# tell apart share a threshold, as they do in Zeff, whose fitted values of such rows are equal
# to the last bit.
roc <- function(score, y, w = rep(1, length(y))) {
  score <- signif(score, 12)
  o <- order(-score)
  s <- score[o]; yy <- y[o]; ww <- w[o]
  ones <- sum(ww * yy); noughts <- sum(ww * (1 - yy))
  tp <- cumsum(ww * yy); fp <- cumsum(ww * (1 - yy))
  last <- !duplicated(s, fromLast = TRUE)
  sens <- tp[last] / ones
  fpr <- fp[last] / noughts
  area <- sum(diff(c(0, fpr)) * (c(0, sens[-length(sens)]) + sens) / 2)
  list(threshold = s[last], sensitivity = sens, specificity = 1 - fpr, area = area,
       ones = ones, noughts = noughts)
}

# The Mann-Whitney form for rows without weights: the mean rank of the ones.
mann_whitney <- function(score, y) {
  r <- rank(signif(score, 12))
  n1 <- as.numeric(sum(y == 1)); n0 <- as.numeric(sum(y == 0))
  (sum(r[y == 1]) - n1 * (n1 + 1) / 2) / (n1 * n0)
}

written <- function(curve, n) {
  k <- length(curve$threshold)
  at <- unique(c(1, 2, 3, k %/% 4, k %/% 2, 3 * k %/% 4, k - 2, k - 1, k))
  at <- sort(at[at >= 1 & at <= k])
  list(
    n = n, ones = num17(curve$ones), noughts = num17(curve$noughts),
    area = num17(curve$area), thresholds = as.integer(k),
    points = lapply(at, function(i) list(
      at = as.integer(i), threshold = num17(curve$threshold[i]),
      sensitivity = num17(curve$sensitivity[i]), specificity = num17(curve$specificity[i])))
  )
}

case <- function(zeff, r_formula, link, weights = NULL, weight_type = "", train = NULL) {
  data <- wages
  if (!is.null(train)) data <- wages[train, , drop = FALSE]
  m <- iterated(r_formula, data, link, if (weight_type == "sampling") weights else NULL)
  if (!is.null(weights) && weight_type == "frequency") {
    times <- ifelse(is.na(data[[weights]]), 0L, as.integer(data[[weights]]))
    copies <- data[rep(seq_len(nrow(data)), times), , drop = FALSE]
    m <- iterated(r_formula, copies, link)
    used <- copies[rownames(model.frame(m)), , drop = FALSE]
    curve <- roc(fitted(m), used$union)
    stopifnot(abs(curve$area - mann_whitney(fitted(m), used$union)) < 1e-12)
    out <- written(curve, nobs(m))
  } else if (!is.null(train)) {
    # On new rows: those the fit did not see, with nothing missing in the model's variables.
    rest <- wages[!train, , drop = FALSE]
    rest <- rest[complete.cases(rest[, all.vars(r_formula)]), , drop = FALSE]
    score <- as.numeric(predict(m, newdata = rest, type = "response"))
    curve <- roc(score, rest$union)
    stopifnot(abs(curve$area - mann_whitney(score, rest$union)) < 1e-12)
    out <- written(curve, nrow(rest))
  } else {
    used <- data[rownames(model.frame(m)), , drop = FALSE]
    if (is.null(weights)) {
      curve <- roc(fitted(m), used$union)
      stopifnot(abs(curve$area - mann_whitney(fitted(m), used$union)) < 1e-12)
    } else {
      used <- used[used[[weights]] > 0, , drop = FALSE]
      curve <- roc(fitted(m)[rownames(used)], used$union, used[[weights]])
    }
    out <- written(curve, nrow(used))
  }
  c(list(formula = zeff, link = link, weights = if (is.null(weights)) "" else weights,
         weight_type = weight_type, new_rows = !is.null(train)), out)
}

cases <- list()
# Three regressors of few values each: many rows share a score.
cases$logit <- case("union ~ educ + exper + female", union ~ educ + exper + female, "logit")
# A probit with a category and a square: more distinct scores.
cases$probit <- case("union ~ educ + exper + exper^2 + female + industry",
  union ~ educ + exper + I(exper^2) + female + industry, "probit")
# One regressor: few thresholds, every one written.
cases$few <- case("union ~ female", union ~ female, "logit")
# Sampling weights: every count a sum of weights.
cases$sampling <- case("union ~ educ + exper + female", union ~ educ + exper + female, "logit",
  weights = "hours", weight_type = "sampling")
# Frequency weights: the rows written out as many times.
cases$frequency <- case("union ~ exper + female", union ~ exper + female, "logit",
  weights = "educ", weight_type = "frequency")
# On new rows: fitted to the years up to 2017, the curve on the later years.
cases$new_rows <- case("union ~ educ + exper + female + industry",
  union ~ educ + exper + female + industry, "logit", train = wages$year <= 2017)

reference <- reference_block()
reference$note <- "base R; numbers written with 17 significant digits"
write_json(list(reference = reference, cases = cases), file.path(here(), "curve.json"))
