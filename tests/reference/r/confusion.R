# R's classification tables for release 0.9.0, step 15 (catalogue row 516): the table of the
# observed outcome against the outcome predicted at a cutoff, for `glm` fits of the shipped
# wages iterated to their maximum, counted in base R.
#
# A row is predicted 1 when its fitted probability is at least the cutoff. The four cells are
# counts of rows, or sums of weights under weights; from them the sensitivity (of the ones, the
# share predicted 1), the specificity (of the noughts, the share predicted 0), the accuracy
# (the share predicted as observed) and the two predictive values (of the rows predicted 1, the
# share that are ones; of the rows predicted 0, the share that are noughts). A share whose
# denominator is nought is NA. Values to 17 significant digits. Run with
# `Rscript tests/reference/r/confusion.R`.
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

counted <- function(score, y, cutoff, w = rep(1, length(y))) {
  predicted <- score >= cutoff
  tp <- sum(w[predicted & y == 1]); fn <- sum(w[!predicted & y == 1])
  fp <- sum(w[predicted & y == 0]); tn <- sum(w[!predicted & y == 0])
  share <- function(a, b) if (b > 0) a / b else NA
  list(
    true_positive = num17(tp), false_negative = num17(fn),
    false_positive = num17(fp), true_negative = num17(tn),
    sensitivity = num17(share(tp, tp + fn)), specificity = num17(share(tn, tn + fp)),
    accuracy = num17(share(tp + tn, tp + fn + fp + tn)),
    positive_predictive_value = num17(share(tp, tp + fp)),
    negative_predictive_value = num17(share(tn, tn + fn))
  )
}

case <- function(zeff, r_formula, link, cutoff, weights = NULL, weight_type = "", train = NULL) {
  data <- wages
  if (!is.null(train)) data <- wages[train, , drop = FALSE]
  if (!is.null(weights) && weight_type == "frequency") {
    times <- ifelse(is.na(data[[weights]]), 0L, as.integer(data[[weights]]))
    copies <- data[rep(seq_len(nrow(data)), times), , drop = FALSE]
    m <- iterated(r_formula, copies, link)
    used <- copies[rownames(model.frame(m)), , drop = FALSE]
    out <- c(list(n = nobs(m)), counted(fitted(m), used$union, cutoff))
  } else if (!is.null(train)) {
    m <- iterated(r_formula, data, link)
    rest <- wages[!train, , drop = FALSE]
    rest <- rest[complete.cases(rest[, all.vars(r_formula)]), , drop = FALSE]
    score <- as.numeric(predict(m, newdata = rest, type = "response"))
    out <- c(list(n = nrow(rest)), counted(score, rest$union, cutoff))
  } else {
    m <- iterated(r_formula, data, link, if (weight_type == "sampling") weights else NULL)
    used <- data[rownames(model.frame(m)), , drop = FALSE]
    if (is.null(weights)) {
      out <- c(list(n = nrow(used)), counted(fitted(m), used$union, cutoff))
    } else {
      used <- used[used[[weights]] > 0, , drop = FALSE]
      out <- c(list(n = nrow(used)),
               counted(fitted(m)[rownames(used)], used$union, cutoff, used[[weights]]))
    }
  }
  c(list(formula = zeff, link = link, cutoff = num17(cutoff),
         weights = if (is.null(weights)) "" else weights, weight_type = weight_type,
         new_rows = !is.null(train)), out)
}

wide <- union ~ educ + exper + I(exper^2) + female + industry
wide_zeff <- "union ~ educ + exper + exper^2 + female + industry"
cases <- list()
cases$half <- case(wide_zeff, wide, "logit", 0.5)
cases$low <- case(wide_zeff, wide, "logit", 0.3)
cases$probit <- case(wide_zeff, wide, "probit", 0.4)
# A cutoff no fitted probability reaches: nothing is predicted 1, and a predictive value is NA.
cases$none <- case("union ~ educ + exper + female", union ~ educ + exper + female, "logit", 0.9)
cases$sampling <- case(wide_zeff, wide, "logit", 0.35, weights = "hours", weight_type = "sampling")
cases$frequency <- case("union ~ exper + female + industry", union ~ exper + female + industry, "logit", 0.35,
  weights = "educ", weight_type = "frequency")
cases$new_rows <- case(wide_zeff, wide, "logit", 0.4, train = wages$year <= 2017)

reference <- reference_block()
reference$note <- "base R; numbers written with 17 significant digits"
write_json(list(reference = reference, cases = cases), file.path(here(), "confusion.json"))
