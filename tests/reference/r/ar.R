# R's Anderson-Rubin test and confidence set for release 0.8.0, step 15 (the release's ruling on
# the weak-instrument statistics; the owner's answer to question 3(c)), written out in base R.
# With the exogenous regressors (and the fixed effects) projected out of the outcome y, the
# instrumented regressor x and the m excluded instruments Z, weighted as the fit is: the
# statistic at a value b0 of the coefficient is the F, or the Wald statistic over m on the robust
# or clustered variance, of Z in the regression of y - b0 x on Z. It is referred to F on m and
# N - L - (absorbed parameters) degrees of freedom, or G - 1 when clustered, L the number of
# instruments in all. The confidence set at a level holds the values b0 the test does not
# reject: under classical errors the solution of a quadratic inequality, written out here; for
# the other kinds the statistic is computed by a regression at each value of a wide grid and
# the ends of the set are the roots between its points. A set is an interval, two rays, the
# whole line or empty. With several instrumented regressors the test is of all their
# coefficients at zero and there is no set. No package of the reference library computes the
# test (ivmodel and ivDiag would; they are not installed), so the classical set is held to the
# search as well as to its quadratic. The data: Card's extract and wage2, as iv.R writes them.
# Values to 17 significant digits. Run with `Rscript tests/reference/r/ar.R` after iv.R.
source(file.path(dirname(sub("^--file=", "", commandArgs(FALSE)[grepl("^--file=", commandArgs(FALSE))])), "common.R"))
Sys.setlocale("LC_COLLATE", "C")
root <- normalizePath(file.path(here(), "..", "..", ".."))

num17 <- function(x) unname(ifelse(is.na(x), "NA", sprintf("%.17g", x)))
card <- read.csv(file.path(root, "tests", "reference", "data", "card.csv"))
wage2 <- read.csv(file.path(root, "tests", "reference", "data", "wage2.csv"))
agree <- function(a, b, tolerance = 1e-8) stopifnot(max(abs(as.vector(a) / as.vector(b) - 1)) < tolerance)

partial_out <- function(v, A, w) v - A %*% solve(crossprod(A * w, A), crossprod(A * w, v))

# The partialled, weighted columns of a model: y, X (n x p) and Z (n x m), with the counts.
prepare <- function(y, X, Z2, W, w = NULL, analytic = FALSE, fe = NULL, absorbed = 0) {
  rows_n <- nrow(X)
  ww <- if (is.null(w)) rep(1, rows_n) else as.numeric(w)
  if (analytic) ww <- ww * rows_n / sum(ww)
  y <- cbind(y)
  if (!is.null(fe)) {
    centre <- function(v) v - ave(v * ww, fe, FUN = sum) / ave(ww, fe, FUN = sum)
    X <- apply(X, 2, centre); Z2 <- apply(Z2, 2, centre); y <- apply(y, 2, centre)
    if (!is.null(W)) W <- apply(W, 2, centre)
  }
  if (!is.null(W)) {
    X <- apply(X, 2, partial_out, A = W, w = ww); Z2 <- apply(Z2, 2, partial_out, A = W, w = ww)
    y <- apply(y, 2, partial_out, A = W, w = ww)
  }
  sw <- sqrt(ww)
  L <- ncol(Z2) + if (is.null(W)) 0 else ncol(W)
  list(y = drop(y) * sw, X = X * sw, Z = Z2 * sw, n = rows_n, df = rows_n - L - absorbed)
}

# The statistic of the excluded instruments in the regression of u on them.
statistic <- function(u, s, kind, cluster = NULL, cluster_df = NULL) {
  Z <- s$Z; m <- ncol(Z)
  A <- solve(crossprod(Z)); pi <- A %*% crossprod(Z, u); e <- drop(u - Z %*% pi)
  V <- if (kind == "classical") {
    A * sum(e^2) / s$df
  } else if (kind == "hc1") {
    A %*% crossprod(Z * e) %*% A * s$n / s$df
  } else {
    Sc <- rowsum(Z * e, cluster); G <- nrow(Sc)
    A %*% crossprod(Sc) %*% A * G / (G - 1) * (s$n - 1) / (if (is.null(cluster_df)) s$df else cluster_df)
  }
  drop(t(pi) %*% solve(V) %*% pi) / m
}

# The set by search: the roots of statistic(b0) = critical value over a grid from -1e8 to 1e8,
# and whether the far ends are inside.
searched <- function(s, crit, kind, cluster, cluster_df) {
  f <- function(b0) statistic(s$y - b0 * s$X[, 1], s, kind, cluster, cluster_df) - crit
  grid <- c(-10^seq(8, -8, by = -0.05), 0, 10^seq(-8, 8, by = 0.05))
  v <- sapply(grid, f)
  if (anyNA(v)) stop("the statistic is NA at ", paste(head(grid[is.na(v)], 3), collapse = ", "), " under ", kind)
  roots <- c()
  for (i in seq_len(length(grid) - 1)) {
    if (v[i] * v[i + 1] < 0) roots <- c(roots, uniroot(f, c(grid[i], grid[i + 1]), tol = 1e-15)$root)
  }
  list(roots = roots, inside_far = v[1] <= 0)
}
shape_of <- function(roots, inside_far) {
  if (length(roots) == 0) return(if (inside_far) "whole line" else "empty")
  if (length(roots) == 2) return(if (inside_far) "two rays" else "interval")
  "other"
}

cases <- list()
ar_case <- function(name, data_name, formula, endogenous, instruments, s, level = 0.95, clusters = list(),
                    fe = NULL, weights = NULL, weight_type = NULL) {
  m <- ncol(s$Z); p <- ncol(s$X)
  out <- list(data = data_name, formula = formula, endogenous = endogenous, instruments = instruments,
              level = num17(level), df1 = as.integer(m), kinds = list())
  if (!is.null(fe)) out$fe <- fe
  if (!is.null(weights)) { out$weights <- weights; out$weight_type <- weight_type }
  kinds <- c(list(classical = NULL, hc1 = NULL), clusters)
  for (kind_name in names(kinds)) {
    cluster <- kinds[[kind_name]]
    kind <- if (kind_name %in% c("classical", "hc1")) kind_name else "cluster"
    cluster_df <- if (kind == "cluster" && !is.null(attr(cluster, "absorbed"))) s$df + attr(cluster, "plain") - attr(cluster, "absorbed") else NULL
    df2 <- if (kind == "cluster") length(unique(cluster)) - 1 else s$df
    at_zero <- statistic(s$y, s, kind, cluster, cluster_df)
    crit <- qf(level, m, df2)
    one <- list(statistic = num17(at_zero), df2 = as.integer(df2),
                p_value = num17(pf(at_zero, m, df2, lower.tail = FALSE)))
    if (p == 1) {
      found <- searched(s, crit, kind, cluster, cluster_df)
      if (kind == "classical") {
        # The quadratic: A(b) - q B(b) <= 0 with A the explained and B the residual sum of
        # squares of y - b x on Z, q = m crit / df.
        x <- s$X[, 1]; y <- s$y; Z <- s$Z
        P <- function(a, b) sum((Z %*% solve(crossprod(Z), crossprod(Z, a))) * b)
        M <- function(a, b) sum(a * b) - P(a, b)
        q <- m * crit / s$df
        qa <- P(x, x) - q * M(x, x); qb <- -2 * (P(x, y) - q * M(x, y)); qc <- P(y, y) - q * M(y, y)
        disc <- qb^2 - 4 * qa * qc
        by_quadratic <- if (disc > 0) sort((-qb + c(-1, 1) * sqrt(disc)) / (2 * qa)) else numeric(0)
        stopifnot(length(by_quadratic) == length(found$roots))
        if (length(by_quadratic) > 0) agree(by_quadratic, found$roots, 1e-7)
        stopifnot(identical(shape_of(by_quadratic, qa < 0), shape_of(found$roots, found$inside_far)))
        found$roots <- by_quadratic
      }
      one$shape <- shape_of(found$roots, found$inside_far)
      one$ends <- as.list(num17(found$roots))
    }
    out$kinds[[kind_name]] <- one
  }
  cases[[name]] <<- out
}
ones <- function(data) cbind("(intercept)" = rep(1, nrow(data)))
columns <- function(data, names) as.matrix(data[, names, drop = FALSE])

d <- card
exog <- c("exper", "expersq", "black", "south", "smsa")
card_text <- "lwage ~ exper + expersq + black + south + smsa + educ"
W <- cbind(ones(d), columns(d, exog))
# Over-identified: an interval; at a low level the two instruments disagree and the set is
# empty; at a very high level it is wide.
s <- prepare(d$lwage, columns(d, "educ"), columns(d, c("nearc4", "nearc2")), W)
ar_case("card_over", "card", card_text, "educ", "nearc4 + nearc2", s, clusters = list(age = d$age))
ar_case("card_over_low_level", "card", card_text, "educ", "nearc4 + nearc2", s, level = 0.6)
ar_case("card_over_high_level", "card", card_text, "educ", "nearc4 + nearc2", s, level = 0.999)
# Just identified by the stronger instrument: an interval close to the usual one.
s <- prepare(d$lwage, columns(d, "educ"), columns(d, "nearc4"), W)
ar_case("card_just", "card", card_text, "educ", "nearc4", s, clusters = list(age = d$age))
# By the weaker instrument alone: two rays.
s <- prepare(d$lwage, columns(d, "educ"), columns(d, "nearc2"), W)
ar_case("card_weak", "card", card_text, "educ", "nearc2", s, clusters = list(age = d$age))
# A weak-instrument case built for the whole line: the first 400 rows and the weaker instrument.
dw <- d[1:400, ]
s <- prepare(dw$lwage, columns(dw, "educ"), columns(dw, "nearc2"), cbind(ones(dw), columns(dw, exog)))
ar_case("card_first_400", "card", card_text, "educ", "nearc2", s)
# The region absorbed, the clusters outside it.
s <- prepare(d$lwage, columns(d, "educ"), columns(d, c("nearc4", "nearc2")),
             columns(d, c("exper", "expersq", "black", "smsa")), fe = d$region, absorbed = 9)
ar_case("card_fe", "card", "lwage ~ exper + expersq + black + smsa + educ", "educ", "nearc4 + nearc2", s,
        clusters = list(age = d$age), fe = "region")
# Analytic weights.
s <- prepare(d$lwage, columns(d, "educ"), columns(d, c("nearc4", "nearc2")), W, w = d$weight, analytic = TRUE)
ar_case("card_analytic", "card", card_text, "educ", "nearc4 + nearc2", s, weights = "weight", weight_type = "analytic")
# Two instrumented regressors: the joint test alone.
d2 <- d[complete.cases(d[, c("KWW", "fatheduc", "motheduc")]), ]
s <- prepare(d2$lwage, columns(d2, c("educ", "KWW")), columns(d2, c("nearc4", "nearc2", "fatheduc", "motheduc")),
             cbind(ones(d2), columns(d2, c("black", "south", "smsa"))))
ar_case("card_two_endogenous", "card", "lwage ~ black + south + smsa + educ + KWW", "educ + KWW",
        "nearc4 + nearc2 + fatheduc + motheduc", s, clusters = list(age = d2$age))
# wage2: schooling by the siblings and the birth order.
w2k <- wage2[complete.cases(wage2[, c("lwage", "exper", "tenure", "educ", "sibs", "brthord")]), ]
s <- prepare(w2k$lwage, columns(w2k, "educ"), columns(w2k, c("sibs", "brthord")),
             cbind(ones(w2k), columns(w2k, c("exper", "tenure"))))
ar_case("wage2_siblings", "wage2", "lwage ~ exper + tenure + educ", "educ", "sibs + brthord", s,
        clusters = list(age = w2k$age))

out <- list(reference = list(software = R.version.string,
                             note = "the Anderson-Rubin test and set written out in base R; 17 significant digits"),
            cases = cases)
write_json(out, file.path(here(), "ar.json"))
