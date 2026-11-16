# R's two-stage least squares for release 0.8.0, step 10 (Language Reference, section 7.1; Engine
# and Project Specification, section 13): the estimates and their standard errors written out in
# base R, and held inside this script to fixest 0.14.2's `feols(y ~ exog | endog ~ instr)` where
# fixest has the form. With X the regressors (the exogenous ones and the instrumented ones), Z
# the instruments (the exogenous regressors and the excluded instruments) and W the weights: the
# first stage is the weighted regression of each column of X on Z, the fitted regressors are
# Xhat = Z Pi, the estimates solve Xhat'W Xhat b = Xhat'W y, and the residuals are the structural
# ones, e = y - X b. The variance is that of least squares with Xhat for the regressors and e
# for the residuals: classical s2 (Xhat'W Xhat)^-1 with s2 = e'We / (N - k); HC1, HC2 and HC3
# with the leverage of Xhat; and clustered (CR1). Analytic and sampling weights are scaled to sum to the rows
# used; frequency weights are checked against the rows written out. The data: Card's (1995) NLS
# extract and wage2 of Wooldridge's textbook, from the package wooldridge 1.4-7; card is written
# beside the references as CSV, as wage1 and wage2 are, so that the engine fits the same rows.
# Inputs scaled by 1e7 to 1e12 stand beside the working cases. Values to 17 significant digits.
# Run with `Rscript tests/reference/r/iv.R`.
source(file.path(dirname(sub("^--file=", "", commandArgs(FALSE)[grepl("^--file=", commandArgs(FALSE))])), "common.R"))
Sys.setlocale("LC_COLLATE", "C")
root <- normalizePath(file.path(here(), "..", "..", ".."))
.libPaths(c(file.path(here(), "library"), .libPaths()))
suppressMessages({
  library(wooldridge)
  library(fixest)
})
options(warn = -1)

num17 <- function(x) unname(ifelse(is.na(x), "NA", sprintf("%.17g", x)))

data(card)
data(wage2)
write_data <- function(data, file) {
  text <- data
  for (col in names(text)) text[[col]] <- ifelse(is.na(data[[col]]), "", sprintf("%.17g", data[[col]]))
  write.csv(text, file, row.names = FALSE, quote = FALSE, na = "")
}
write_data(card, file.path(root, "tests", "reference", "data", "card.csv"))

# Two-stage least squares by hand. `exog` and `endog` are named lists of columns in the order of
# the formula's regressors (`order` gives the names in that order), `instr` the excluded
# instruments; the intercept is first in X and in Z.
tsls <- function(y, exog, endog, instr, order, w = NULL, weight_type = NULL, cluster = NULL) {
  rows <- complete.cases(as.data.frame(c(list(y = y), exog, endog, instr)))
  # A row with a zero weight is left out of the estimate and of the count of rows.
  if (!is.null(w)) rows <- rows & !is.na(w) & w > 0
  if (!is.null(cluster)) rows <- rows & !is.na(cluster)
  pick <- function(columns) lapply(columns, function(v) v[rows])
  y <- y[rows]
  regressors <- c(pick(exog), pick(endog))[order]
  X <- cbind("(intercept)" = 1, do.call(cbind, regressors))
  Z <- cbind("(intercept)" = 1, do.call(cbind, c(pick(exog), pick(instr))))
  n_rows <- nrow(X)
  k <- ncol(X)
  ww <- if (is.null(w)) rep(1, n_rows) else w[rows]
  if (identical(weight_type, "analytic") || identical(weight_type, "sampling")) ww <- ww * n_rows / sum(ww)
  N <- if (identical(weight_type, "frequency")) sum(ww) else n_rows
  sw <- sqrt(ww)
  Xw <- X * sw
  Zw <- Z * sw
  yw <- y * sw
  # By QR with the columns scaled to unit norm, so that inputs of very different sizes are
  # solved as well as the working ones: the normal equations of a column times 1e12 are not.
  solve_ls <- function(A, rhs) {
    scale <- sqrt(colSums(A^2))
    qr.coef(qr(sweep(A, 2, scale, "/"), tol = 1e-12), rhs) / scale
  }
  Pi <- solve_ls(Zw, Xw)
  dimnames(Pi) <- list(colnames(Z), colnames(X))
  Xh <- Zw %*% Pi
  scale <- sqrt(colSums(Xh^2))
  B <- chol2inv(qr.R(qr(sweep(Xh, 2, scale, "/"), tol = 1e-12))) / outer(scale, scale)
  b <- as.vector(solve_ls(Xh, yw))
  e <- as.vector(yw - Xw %*% b)
  rss <- sum(e^2)
  h <- rowSums((Xh %*% B) * Xh)
  copies <- if (identical(weight_type, "frequency")) ww else rep(1, n_rows)
  sandwich <- function(factor, scale) {
    meat <- crossprod(Xh * e * sqrt(factor / copies))
    sqrt(diag(scale * B %*% meat %*% B))
  }
  se <- list(
    classical = sqrt(diag(rss / (N - k) * B)),
    hc1 = sandwich(rep(1, n_rows), N / (N - k)),
    hc2 = sandwich(1 / (1 - h / copies), 1),
    hc3 = sandwich(1 / (1 - h / copies)^2, 1)
  )
  if (!is.null(cluster)) {
    g <- cluster[rows]
    G <- length(unique(g))
    S <- rowsum(Xh * e, g)
    se$cluster <- sqrt(diag(G / (G - 1) * (N - 1) / (N - k) * B %*% crossprod(S) %*% B))
    se$clusters <- G
  }
  list(names = colnames(X), n = n_rows, k = k, N = N, dropped = sum(!rows), b = b, rss = rss, se = se,
       first = Pi[, names(endog), drop = FALSE], instruments = colnames(Z), rows = rows)
}

# One case as the engine reads it: the formula and `instruments:` in Zeff's words, the data's
# name, the options, and R's answers.
cases <- list()
case <- function(name, data_name, formula, endogenous, instruments, fit, weights = NULL, weight_type = NULL,
                 cluster = NULL, names = NULL, instrument_names = NULL) {
  out <- list(
    data = data_name, formula = formula, endogenous = endogenous, instruments = instruments,
    n = as.integer(fit$n), dropped = as.integer(fit$dropped), k = as.integer(fit$k),
    names = as.list(if (is.null(names)) fit$names else names),
    coefficients = as.list(num17(fit$b)),
    rss = num17(fit$rss),
    se_classical = as.list(num17(fit$se$classical)),
    se_hc1 = as.list(num17(fit$se$hc1)),
    se_hc2 = as.list(num17(fit$se$hc2)),
    se_hc3 = as.list(num17(fit$se$hc3)),
    first_stage = lapply(seq_len(ncol(fit$first)), function(j) as.list(num17(fit$first[, j]))),
    first_stage_names = as.list(if (is.null(instrument_names)) fit$instruments else instrument_names)
  )
  if (!is.null(weights)) { out$weights <- weights; out$weight_type <- weight_type }
  if (!is.null(cluster)) {
    out$cluster <- cluster
    out$se_cluster <- as.list(num17(fit$se$cluster))
    out$clusters <- as.integer(fit$se$clusters)
  }
  cases[[name]] <<- out
}

agree <- function(a, b, tolerance = 1e-8) stopifnot(max(abs(as.vector(a) / as.vector(b) - 1)) < tolerance)
# fixest names an instrumented regressor `fit_x` and puts it after the intercept.
of_fixest <- function(values, names) {
  fixed <- ifelse(names == "(intercept)", "(Intercept)", names)
  out <- values[ifelse(fixed %in% names(values), fixed, paste0("fit_", fixed))]
  stopifnot(!anyNA(out))
  out
}

d <- card
controls <- list(exper = d$exper, expersq = d$expersq, black = d$black, south = d$south, smsa = d$smsa)
card_order <- c("exper", "expersq", "black", "south", "smsa", "educ")
card_text <- "lwage ~ exper + expersq + black + south + smsa + educ"

# Just identified: college proximity for schooling (Card, 1995).
just <- tsls(d$lwage, controls, list(educ = d$educ), list(nearc4 = d$nearc4), card_order)
f <- feols(lwage ~ exper + expersq + black + south + smsa | educ ~ nearc4, d)
agree(just$b, of_fixest(coef(f), just$names))
agree(just$se$classical, of_fixest(se(f, vcov = "iid"), just$names))
agree(just$se$hc1, of_fixest(se(f, vcov = "hetero"), just$names))
case("card_just", "card", card_text, "educ", "nearc4", just)

# Over-identified: two instruments for one regressor.
over <- tsls(d$lwage, controls, list(educ = d$educ), list(nearc4 = d$nearc4, nearc2 = d$nearc2), card_order,
             cluster = d$age)
f <- feols(lwage ~ exper + expersq + black + south + smsa | educ ~ nearc4 + nearc2, d)
agree(over$b, of_fixest(coef(f), over$names))
agree(over$se$classical, of_fixest(se(f, vcov = "iid"), over$names))
agree(over$se$hc1, of_fixest(se(f, vcov = "hetero"), over$names))
agree(over$se$cluster, of_fixest(se(f, vcov = ~age), over$names))
case("card_over", "card", card_text, "educ", "nearc4 + nearc2", over, cluster = "age")

# Two instrumented regressors, three instruments.
two <- tsls(d$lwage, list(black = d$black, south = d$south, smsa = d$smsa), list(educ = d$educ, exper = d$exper),
            list(nearc4 = d$nearc4, nearc2 = d$nearc2, age = d$age), c("black", "south", "smsa", "educ", "exper"))
f <- feols(lwage ~ black + south + smsa | educ + exper ~ nearc4 + nearc2 + age, d)
agree(two$b, of_fixest(coef(f), two$names))
agree(two$se$classical, of_fixest(se(f, vcov = "iid"), two$names))
agree(two$se$hc1, of_fixest(se(f, vcov = "hetero"), two$names))
case("card_two_endogenous", "card", "lwage ~ black + south + smsa + educ + exper", "educ + exper",
     "nearc4 + nearc2 + age", two)

# No regressor but the instrumented one: the Wald estimator.
wald <- tsls(d$lwage, list(), list(educ = d$educ), list(nearc4 = d$nearc4), "educ")
reduced <- cov(d$lwage, d$nearc4) / cov(d$educ, d$nearc4)
agree(wald$b[2], reduced)
case("card_wald", "card", "lwage ~ educ", "educ", "nearc4", wald)

# Weights of the three kinds: the NLS sampling weight as analytic and as sampling weights, and
# the number of years of the father's schooling as a count of copies, against the rows written
# out.
analytic <- tsls(d$lwage, controls, list(educ = d$educ), list(nearc4 = d$nearc4, nearc2 = d$nearc2), card_order,
                 w = d$weight, weight_type = "analytic")
f <- feols(lwage ~ exper + expersq + black + south + smsa | educ ~ nearc4 + nearc2, d, weights = ~weight)
agree(analytic$b, of_fixest(coef(f), analytic$names))
agree(analytic$se$classical, of_fixest(se(f, vcov = "iid"), analytic$names))
agree(analytic$se$hc1, of_fixest(se(f, vcov = "hetero"), analytic$names))
case("card_analytic", "card", card_text, "educ", "nearc4 + nearc2", analytic, weights = "weight",
     weight_type = "analytic")
sampling <- tsls(d$lwage, controls, list(educ = d$educ), list(nearc4 = d$nearc4, nearc2 = d$nearc2), card_order,
                 w = d$weight, weight_type = "sampling", cluster = d$age)
agree(sampling$b, analytic$b)
agree(sampling$se$hc1, analytic$se$hc1)
agree(sampling$se$cluster, of_fixest(se(f, vcov = ~age), sampling$names))
case("card_sampling", "card", card_text, "educ", "nearc4 + nearc2", sampling, weights = "weight",
     weight_type = "sampling", cluster = "age")
copies <- ifelse(is.na(d$fatheduc), NA, d$fatheduc)
frequency <- tsls(d$lwage, controls, list(educ = d$educ), list(nearc4 = d$nearc4, nearc2 = d$nearc2), card_order,
                  w = copies, weight_type = "frequency")
keep <- !is.na(copies) & copies > 0
written <- d[rep(which(keep), copies[keep]), ]
out <- tsls(written$lwage, list(exper = written$exper, expersq = written$expersq, black = written$black,
                                south = written$south, smsa = written$smsa),
            list(educ = written$educ), list(nearc4 = written$nearc4, nearc2 = written$nearc2), card_order)
agree(frequency$b, out$b)
agree(frequency$se$classical, out$se$classical)
agree(frequency$se$hc1, out$se$hc1)
agree(frequency$se$hc2, out$se$hc2)
agree(frequency$se$hc3, out$se$hc3)
stopifnot(frequency$N == out$n)
case("card_frequency", "card", card_text, "educ", "nearc4 + nearc2", frequency, weights = "fatheduc",
     weight_type = "frequency")

# Inputs of very different sizes beside the working grid: the instrumented regressor in units of
# 1e-7 years, one instrument times 1e12 and the other times 1e9; the estimates of the other
# regressors are those of the over-identified case, and the regressor's is that case's over 1e7.
big <- tsls(d$lwage, controls, list(educ = d$educ * 1e7), list(nearc4 = d$nearc4 * 1e12, nearc2 = d$nearc2 * 1e9),
            card_order)
agree(big$b[-7], over$b[-7], 1e-6)
agree(big$b[7] * 1e7, over$b[7], 1e-6)
case("card_magnitudes", "card", "lwage ~ exper + expersq + black + south + smsa + (educ * 1e7)", "(educ * 1e7)",
     "(nearc4 * 1e12) + (nearc2 * 1e9)", big,
     names = c("(intercept)", "exper", "expersq", "black", "south", "smsa", "(educ * 1e7)"),
     instrument_names = c("(intercept)", "exper", "expersq", "black", "south", "smsa", "(nearc4 * 1e12)",
                          "(nearc2 * 1e9)"))

# wage2: schooling instrumented by the number of siblings and the birth order (which has NA),
# and by the parents' schooling (NA in both).
w2 <- wage2
sibs <- tsls(w2$lwage, list(exper = w2$exper, tenure = w2$tenure), list(educ = w2$educ),
             list(sibs = w2$sibs, brthord = w2$brthord), c("exper", "tenure", "educ"), cluster = w2$age)
f <- feols(lwage ~ exper + tenure | educ ~ sibs + brthord, w2)
agree(sibs$b, of_fixest(coef(f), sibs$names))
agree(sibs$se$classical, of_fixest(se(f, vcov = "iid"), sibs$names))
agree(sibs$se$hc1, of_fixest(se(f, vcov = "hetero"), sibs$names))
agree(sibs$se$cluster, of_fixest(se(f, vcov = ~age), sibs$names))
case("wage2_siblings", "wage2", "lwage ~ exper + tenure + educ", "educ", "sibs + brthord", sibs, cluster = "age")
parents <- tsls(w2$lwage, list(exper = w2$exper, tenure = w2$tenure, black = w2$black), list(educ = w2$educ),
                list(meduc = w2$meduc, feduc = w2$feduc), c("exper", "tenure", "black", "educ"))
f <- feols(lwage ~ exper + tenure + black | educ ~ meduc + feduc, w2)
agree(parents$b, of_fixest(coef(f), parents$names))
agree(parents$se$classical, of_fixest(se(f, vcov = "iid"), parents$names))
agree(parents$se$hc1, of_fixest(se(f, vcov = "hetero"), parents$names))
case("wage2_parents", "wage2", "lwage ~ exper + tenure + black + educ", "educ", "meduc + feduc", parents)
hours <- tsls(w2$lwage, list(exper = w2$exper, tenure = w2$tenure), list(educ = w2$educ),
              list(sibs = w2$sibs, meduc = w2$meduc), c("exper", "tenure", "educ"), w = w2$hours,
              weight_type = "analytic")
f <- feols(lwage ~ exper + tenure | educ ~ sibs + meduc, w2, weights = ~hours)
agree(hours$b, of_fixest(coef(f), hours$names))
agree(hours$se$classical, of_fixest(se(f, vcov = "iid"), hours$names))
case("wage2_analytic", "wage2", "lwage ~ exper + tenure + educ", "educ", "sibs + meduc", hours, weights = "hours",
     weight_type = "analytic")

out <- list(reference = list(software = R.version.string, fixest = as.character(packageVersion("fixest")),
                             wooldridge = as.character(packageVersion("wooldridge")),
                             note = "two-stage least squares by hand in base R, held to fixest inside the script; 17 significant digits"),
            cases = cases)
write_json(out, file.path(here(), "iv.json"))
