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
# The cases with fixed effects (step 11) are fixest's own figures, one of them also written out
# by hand. Run with `Rscript tests/reference/r/iv.R`.
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
# The region of residence in 1966, one variable from the extract's nine indicators, for the
# cases with fixed effects.
card$region <- as.integer(as.matrix(card[, paste0("reg66", 1:9)]) %*% 1:9)
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

# With fixed effects (release 0.8.0, step 11): the outcome, the regressors and the instruments
# projected out together, then the two stages. The figures are fixest's own, with its
# small-sample defaults (`fixef.K = "nested"`, the ruling of release 0.6.0), the projections run
# to 1e-11 and the singletons removed, as Zeff removes them. `zeff` names the terms as a script
# writes them and in its order, `theirs` as fixest names them. One case is also written out by
# hand, with the within transformation in closed form, and held to fixest here.
TOL <- 1e-11
fe_cases <- list()
fe_case <- function(name, data, zeff, endogenous, instruments, fe_text, f, names, theirs, clusters = NULL,
                    weights = NULL, weight_type = NULL) {
  args <- list(fml = f, data = data, fixef.tol = TOL, fixef.rm = "singleton")
  if (!is.null(weights)) args$weights <- as.formula(paste0("~", weights))
  fit <- suppressMessages(do.call(feols, args))
  args$fixef.rm <- "none"
  complete <- suppressMessages(do.call(feols, args))
  stopifnot(setequal(names(coef(fit)), theirs))
  e <- resid(fit)
  w <- if (is.null(weights)) rep(1, length(e)) else weights(fit)
  w <- w * length(e) / sum(w)
  out <- list(
    data = "card", formula = zeff, fe = fe_text, endogenous = endogenous, instruments = instruments,
    n = as.integer(nobs(fit)), dropped = as.integer(nrow(data) - nobs(complete)),
    singletons = as.integer(nobs(complete) - nobs(fit)),
    fixef_sizes = as.list(as.integer(fit$fixef_sizes)),
    names = as.list(names),
    coefficients = as.list(num17(coef(fit)[theirs])),
    rss = num17(sum(w * e^2)),
    se_classical = as.list(num17(se(fit, vcov = "iid")[theirs])),
    se_hc1 = as.list(num17(se(fit, vcov = "hetero")[theirs])),
    se_cluster = list()
  )
  for (cluster in clusters) {
    out$se_cluster[[cluster]] <- as.list(num17(se(fit, vcov = as.formula(paste0("~", cluster)))[theirs]))
  }
  if (!is.null(weights)) { out$weights <- weights; out$weight_type <- weight_type }
  fe_cases[[name]] <<- out
  invisible(fit)
}

# The model of the specification's sample script (section 7): the region absorbed.
f <- fe_case("script", card, "lwage ~ exper + exper^2 + black + educ", "educ", "nearc4", "region",
             lwage ~ exper + I(exper^2) + black | region | educ ~ nearc4,
             c("exper", "exper^2", "black", "educ"), c("exper", "I(I(exper^2))", "black", "fit_educ"),
             clusters = c("age", "region"))
# The same by hand: every column less its mean within the region, two stages without an
# intercept, and N - k - 9 for the nine regions (N - k - 1 when the clusters are the regions, in
# which the effect is nested).
within <- function(v) v - ave(v, card$region)
Xd <- cbind(within(card$exper), within(card$exper^2), within(card$black), within(card$educ))
Zd <- cbind(Xd[, 1:3], within(card$nearc4))
yd <- within(card$lwage)
Xh <- Zd %*% qr.solve(Zd, Xd)
b <- qr.solve(Xh, yd)
e <- drop(yd - Xd %*% b)
A <- solve(crossprod(Xh))
n_rows <- length(yd)
fixest_order <- c("exper", "I(I(exper^2))", "black", "fit_educ")
agree(b, coef(f)[fixest_order])
agree(sqrt(diag(A) * sum(e^2) / (n_rows - 4 - 9)), se(f, vcov = "iid")[fixest_order])
S <- Xh * e
agree(sqrt(diag(A %*% crossprod(S) %*% A) * n_rows / (n_rows - 4 - 9)), se(f, vcov = "hetero")[fixest_order])
Sc <- rowsum(S, card$age)
agree(sqrt(diag(A %*% crossprod(Sc) %*% A) * 11 / 10 * (n_rows - 1) / (n_rows - 4 - 9)), se(f, vcov = ~age)[fixest_order])
Sc <- rowsum(S, card$region)
agree(sqrt(diag(A %*% crossprod(Sc) %*% A) * 9 / 8 * (n_rows - 1) / (n_rows - 4 - 1)), se(f, vcov = ~region)[fixest_order])

fe_case("over", card, "lwage ~ exper + expersq + black + smsa + educ", "educ", "nearc4 + nearc2", "region",
        lwage ~ exper + expersq + black + smsa | region | educ ~ nearc4 + nearc2,
        c("exper", "expersq", "black", "smsa", "educ"), c("exper", "expersq", "black", "smsa", "fit_educ"),
        clusters = "age")
# Two effects; clustered by one of them, which is then nested in the clusters.
fe_case("two_way", card, "lwage ~ black + smsa + educ", "educ", "nearc4 + nearc2", "region + age",
        lwage ~ black + smsa | region + age | educ ~ nearc4 + nearc2,
        c("black", "smsa", "educ"), c("black", "smsa", "fit_educ"), clusters = "age")
# An interacted effect.
fe_case("interacted", card, "lwage ~ black + smsa + educ", "educ", "nearc4 + nearc2", "region:smsa66",
        lwage ~ black + smsa | region^smsa66 | educ ~ nearc4 + nearc2,
        c("black", "smsa", "educ"), c("black", "smsa", "fit_educ"))
# An effect with missing values and with levels of one row: the IQ score.
fe_case("singletons", card, "lwage ~ black + smsa + educ", "educ", "nearc4 + nearc2", "IQ",
        lwage ~ black + smsa | IQ | educ ~ nearc4 + nearc2,
        c("black", "smsa", "educ"), c("black", "smsa", "fit_educ"))
# Two instrumented regressors under an effect.
fe_case("two_endogenous", card, "lwage ~ black + smsa + educ + exper", "educ + exper", "nearc4 + nearc2 + age",
        "region", lwage ~ black + smsa | region | educ + exper ~ nearc4 + nearc2 + age,
        c("black", "smsa", "educ", "exper"), c("black", "smsa", "fit_educ", "fit_exper"))
# Weights: fixest's weights are analytic with classical errors and sampling with robust ones.
fe_case("weighted", card, "lwage ~ exper + expersq + black + smsa + educ", "educ", "nearc4 + nearc2", "region",
        lwage ~ exper + expersq + black + smsa | region | educ ~ nearc4 + nearc2,
        c("exper", "expersq", "black", "smsa", "educ"), c("exper", "expersq", "black", "smsa", "fit_educ"),
        clusters = "age", weights = "weight", weight_type = "analytic")

# The weak-instrument statistics (release 0.8.0, steps 12b and 12c), written out in base R from
# the instrumented regressors and the excluded instruments with the exogenous regressors (and
# the fixed effects) partialled out of both, weighted as the fit is. With Xp the n x p matrix of
# instrumented regressors, Zp the n x m matrix of excluded instruments, Pi the first-stage
# coefficients on Zp, V the first-stage residuals, L the number of instruments in all and
# df = N - L - (the parameters of the absorbed effects):
#   the first-stage F of a regressor is pi' Zp'Zp pi / m over its residual variance, V'V / df;
#   the Wald form is pi' Var(pi)^-1 pi / m on the robust or clustered variance of the fit;
#   the Cragg-Donald F is the smallest eigenvalue of Svv^-1/2 Pi' Zp'Zp Pi Svv^-1/2 over m, with
#   Svv = V'V / df (Stock and Yogo's form: with one instrumented regressor it is its F).
# fixest's `ivf` and `ivwald` are held to the first two; its `cd` divides by N - p - m - 1 in the
# place of N - L, and is held after that conversion.
partial_out <- function(v, A, w) v - A %*% solve(crossprod(A * w, A), crossprod(A * w, v))
inverse_root <- function(S) {
  e <- eigen(S, symmetric = TRUE)
  e$vectors %*% diag(1 / sqrt(e$values), nrow(S)) %*% t(e$vectors)
}
weak <- function(X, Z2, W, w = NULL, weight_type = NULL, fe = NULL, absorbed = 0, clusters = list()) {
  rows_n <- nrow(X)
  ww <- if (is.null(w)) rep(1, rows_n) else w
  if (identical(weight_type, "analytic") || identical(weight_type, "sampling")) ww <- ww * rows_n / sum(ww)
  N <- if (identical(weight_type, "frequency")) sum(ww) else rows_n
  if (!is.null(fe)) {
    centre <- function(v) v - ave(v * ww, fe, FUN = sum) / ave(ww, fe, FUN = sum)
    X <- apply(X, 2, centre); Z2 <- apply(Z2, 2, centre)
    if (!is.null(W)) W <- apply(W, 2, centre)
  }
  if (!is.null(W)) {
    X <- apply(X, 2, partial_out, A = W, w = ww); Z2 <- apply(Z2, 2, partial_out, A = W, w = ww)
  }
  sw <- sqrt(ww)
  Xp <- X * sw; Zp <- Z2 * sw
  p <- ncol(Xp); m <- ncol(Zp)
  L <- m + if (is.null(W)) 0 else ncol(W)
  df <- N - L - absorbed
  ZZ <- crossprod(Zp); A <- solve(ZZ)
  Pi <- A %*% crossprod(Zp, Xp)
  V <- Xp - Zp %*% Pi
  out <- list(n = N, instruments = L, excluded = m, df = df)
  out$f <- sapply(seq_len(p), function(j) drop(t(Pi[, j]) %*% ZZ %*% Pi[, j]) / m / (sum(V[, j]^2) / df))
  Svv <- crossprod(V) / df
  root <- inverse_root(Svv)
  out$cd <- min(eigen(root %*% (t(Pi) %*% ZZ %*% Pi) %*% root, symmetric = TRUE)$values) / m
  # The variance of the first-stage coefficients, stacked regressor by regressor: the scores of
  # a row are its instruments times its residual, once per frequency weight.
  S <- do.call(cbind, lapply(seq_len(p), function(j) Zp * V[, j]))
  bread <- kronecker(diag(p), A)
  copies <- if (identical(weight_type, "frequency")) ww else rep(1, rows_n)
  variances <- list(hc1 = bread %*% crossprod(S / copies, S) %*% bread * N / df)
  for (name in names(clusters)) {
    Sc <- rowsum(S, clusters[[name]]); G <- nrow(Sc)
    nested <- attr(clusters[[name]], "absorbed")
    dfc <- if (is.null(nested)) df else N - L - nested
    variances[[name]] <- bread %*% crossprod(Sc) %*% bread * G / (G - 1) * (N - 1) / dfc
  }
  out$wald <- lapply(variances, function(Vpi) sapply(seq_len(p), function(j) {
    idx <- (j - 1) * m + seq_len(m)
    drop(t(Pi[, j]) %*% solve(Vpi[idx, idx]) %*% Pi[, j]) / m
  }))
  out
}
weak_cases <- list()
weak_case <- function(name, data_name, formula, endogenous, instruments, stats, fe = NULL, weights = NULL,
                      weight_type = NULL, clusters = character(0)) {
  out <- list(data = data_name, formula = formula, endogenous = endogenous, instruments = instruments,
              df1 = as.integer(stats$excluded), df2 = num17(stats$df),
              f = as.list(num17(stats$f)), cragg_donald = num17(stats$cd),
              wald_hc1 = as.list(num17(stats$wald$hc1)), wald_cluster = list())
  for (cluster in clusters) out$wald_cluster[[cluster]] <- as.list(num17(stats$wald[[cluster]]))
  if (!is.null(fe)) out$fe <- fe
  if (!is.null(weights)) { out$weights <- weights; out$weight_type <- weight_type }
  weak_cases[[name]] <<- out
}
of_stat <- function(fit, type, vcov = NULL) {
  stats <- if (is.null(vcov)) fitstat(fit, type) else fitstat(fit, type, vcov = vcov)
  unname(sapply(stats, function(s) if (is.list(s)) s$stat else s))
}
ones <- function(data) cbind("(intercept)" = rep(1, nrow(data)))
columns <- function(data, names) as.matrix(data[, names, drop = FALSE])

# Card's data, schooling instrumented by the two proximity indicators.
d <- card
exog <- c("exper", "expersq", "black", "south", "smsa")
s <- weak(columns(d, "educ"), columns(d, c("nearc4", "nearc2")), cbind(ones(d), columns(d, exog)),
          clusters = list(age = d$age))
f <- feols(lwage ~ exper + expersq + black + south + smsa | educ ~ nearc4 + nearc2, d)
agree(s$f, of_stat(f, ~ivf))
agree(s$cd, s$f)
agree(s$cd, of_stat(f, ~cd) * (s$n - s$instruments) / (s$n - 1 - 2 - 1))
agree(s$wald$hc1, of_stat(f, ~ivwald, "hetero"))
agree(s$wald$age, of_stat(f, ~ivwald, ~age))
weak_case("card_over", "card", card_text, "educ", "nearc4 + nearc2", s, clusters = "age")
# One instrument.
s <- weak(columns(d, "educ"), columns(d, "nearc4"), cbind(ones(d), columns(d, exog)), clusters = list(age = d$age))
f <- feols(lwage ~ exper + expersq + black + south + smsa | educ ~ nearc4, d)
agree(s$f, of_stat(f, ~ivf))
agree(s$wald$hc1, of_stat(f, ~ivwald, "hetero"))
weak_case("card_just", "card", card_text, "educ", "nearc4", s, clusters = "age")
# Two instrumented regressors, schooling and the test score, by four instruments, on the rows
# where the parents' schooling and the score are known: strong first stages one by one, and a
# Cragg-Donald F far below either, since the instruments move the two together.
both <- c("black", "south", "smsa")
rows <- complete.cases(d[, c("KWW", "fatheduc", "motheduc")])
d2 <- d[rows, ]
s <- weak(columns(d2, c("educ", "KWW")), columns(d2, c("nearc4", "nearc2", "fatheduc", "motheduc")),
          cbind(ones(d2), columns(d2, both)), clusters = list(age = d2$age))
f <- feols(lwage ~ black + south + smsa | educ + KWW ~ nearc4 + nearc2 + fatheduc + motheduc, d2)
agree(s$f, of_stat(f, ~ivf))
agree(s$cd, of_stat(f, ~cd) * (s$n - s$instruments) / (s$n - 2 - 4 - 1))
agree(s$wald$hc1, of_stat(f, ~ivwald, "hetero"))
agree(s$wald$age, of_stat(f, ~ivwald, ~age))
weak_case("card_two_endogenous", "card", "lwage ~ black + south + smsa + educ + KWW", "educ + KWW",
          "nearc4 + nearc2 + fatheduc + motheduc", s, clusters = "age")
# Weights of the three kinds.
s <- weak(columns(d, "educ"), columns(d, c("nearc4", "nearc2")), cbind(ones(d), columns(d, exog)),
          w = d$weight, weight_type = "analytic", clusters = list(age = d$age))
f <- feols(lwage ~ exper + expersq + black + south + smsa | educ ~ nearc4 + nearc2, d, weights = ~weight)
agree(s$f, of_stat(f, ~ivf))
agree(s$wald$hc1, of_stat(f, ~ivwald, "hetero"))
agree(s$wald$age, of_stat(f, ~ivwald, ~age))
weak_case("card_analytic", "card", card_text, "educ", "nearc4 + nearc2", s, weights = "weight",
          weight_type = "analytic", clusters = "age")
keep <- !is.na(d$fatheduc) & d$fatheduc > 0
dk <- d[keep, ]
s <- weak(columns(dk, "educ"), columns(dk, c("nearc4", "nearc2")), cbind(ones(dk), columns(dk, exog)),
          w = dk$fatheduc, weight_type = "frequency")
written <- dk[rep(seq_len(nrow(dk)), dk$fatheduc), ]
s_written <- weak(columns(written, "educ"), columns(written, c("nearc4", "nearc2")),
                  cbind(ones(written), columns(written, exog)))
agree(s$f, s_written$f)
agree(s$cd, s_written$cd)
agree(s$wald$hc1, s_written$wald$hc1)
weak_case("card_frequency", "card", card_text, "educ", "nearc4 + nearc2", s, weights = "fatheduc",
          weight_type = "frequency")
# With the region absorbed: nine parameters join the instruments in the degrees of freedom, and
# one alone when the clusters are the regions.
region_clusters <- d$region
attr(region_clusters, "absorbed") <- 1
s <- weak(columns(d, "educ"), columns(d, c("nearc4", "nearc2")), columns(d, c("exper", "expersq", "black", "smsa")),
          fe = d$region, absorbed = 9, clusters = list(age = d$age, region = region_clusters))
f <- feols(lwage ~ exper + expersq + black + smsa | region | educ ~ nearc4 + nearc2, d)
agree(s$f, of_stat(f, ~ivf))
agree(s$wald$hc1, of_stat(f, ~ivwald, "hetero"))
agree(s$wald$age, of_stat(f, ~ivwald, ~age))
agree(s$wald$region, of_stat(f, ~ivwald, ~region))
weak_case("card_fe", "card", "lwage ~ exper + expersq + black + smsa + educ", "educ", "nearc4 + nearc2", s,
          fe = "region", clusters = c("age", "region"))
# wage2: schooling by the siblings and the birth order.
w2k <- w2[complete.cases(w2[, c("lwage", "exper", "tenure", "educ", "sibs", "brthord")]), ]
s <- weak(columns(w2k, "educ"), columns(w2k, c("sibs", "brthord")), cbind(ones(w2k), columns(w2k, c("exper", "tenure"))),
          clusters = list(age = w2k$age))
f <- feols(lwage ~ exper + tenure | educ ~ sibs + brthord, w2)
agree(s$f, of_stat(f, ~ivf))
agree(s$wald$hc1, of_stat(f, ~ivwald, "hetero"))
agree(s$wald$age, of_stat(f, ~ivwald, ~age))
weak_case("wage2_siblings", "wage2", "lwage ~ exper + tenure + educ", "educ", "sibs + brthord", s, clusters = "age")

out <- list(reference = list(software = R.version.string, fixest = as.character(packageVersion("fixest")),
                             wooldridge = as.character(packageVersion("wooldridge")),
                             note = "two-stage least squares by hand in base R, held to fixest inside the script; 17 significant digits"),
            cases = cases, fixed_effects = fe_cases, weak = weak_cases)
write_json(out, file.path(here(), "iv.json"))
