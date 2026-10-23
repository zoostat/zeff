# R's linear regressions for release 0.4.0, step 10 (Language Reference, sections 7.2 and 10.2;
# Engine and Project Specification, section 13): `lm`, `summary`, `confint`, `logLik` on the
# shipped wages (its industry levels in the Arrow file's order), tests/datasets/wages_large.csv
# and the edge-case files, under Zeff's term names and contrast conventions (the base, the first
# level, is the omitted level under treatment and sum). Values to 17 significant digits. Run with
# `Rscript tests/reference/r/regression.R`.
source(file.path(dirname(sub("^--file=", "", commandArgs(FALSE)[grepl("^--file=", commandArgs(FALSE))])), "common.R"))
Sys.setlocale("LC_COLLATE", "C")
root <- normalizePath(file.path(here(), "..", "..", ".."))
# sandwich (question 2 of release 0.4.0): from the harness's own library when install.R made it.
.libPaths(c(file.path(here(), "library"), .libPaths()))
library(sandwich)

num17 <- function(x) ifelse(is.na(x), "NA", sprintf("%.17g", x))

industry_levels <- c("Agriculture", "Mining", "Utilities", "Construction", "Manufacturing", "Retail",
                     "Transport", "Finance", "Services", "Education", "Health", "Public administration")

zeff_sum <- function(k) {
  C <- contr.sum(k)
  C[c(k, seq_len(k - 1)), , drop = FALSE]
}

level_name <- function(level) {
  if (grepl("[=:+]", level)) paste0('"', gsub('"', '\\\\"', gsub("\\\\", "\\\\\\\\", level)), '"') else level
}

zeff_name <- function(name, factors) {
  if (name == "(Intercept)") return("(intercept)")
  part_name <- function(part) {
    for (var in names(factors)) {
      f <- factors[[var]]
      k <- length(f$levels)
      if (f$contrast %in% c("treatment", "full")) {
        for (l in f$levels) if (part == paste0(var, l)) return(paste0(var, "=", level_name(l)))
      } else if (f$contrast %in% c("sum", "helmert")) {
        for (i in seq_len(k - 1)) if (part == paste0(var, i)) return(paste0(var, "=", level_name(f$levels[i + 1])))
      } else if (f$contrast == "poly") {
        poly_names <- c(".L", ".Q", ".C", if (k > 4) paste0("^", 4:(k - 1)))
        trends <- c("linear", "quadratic", "cubic", if (k > 4) paste0("degree", 4:(k - 1)))
        for (i in seq_len(k - 1)) if (part == paste0(var, poly_names[i])) return(paste0(var, "=", trends[i]))
      }
    }
    if (part == "femaleTRUE") return("female")
    part <- sub("^I\\((.*)\\)$", "\\1", part)
    gsub("`", "", part, fixed = TRUE)
  }
  for (var in names(factors)) {
    for (l in factors[[var]]$levels) if (name == paste0(var, l)) return(paste0(var, "=", level_name(l)))
  }
  paste(vapply(strsplit(name, ":", fixed = TRUE)[[1]], part_name, ""), collapse = ":")
}

# `rename` maps R's name of a term to Zeff's where the two differ beyond the rules above: Zeff
# keeps the brackets of an arithmetic term as written, `(educ * 2)`, where R writes `I(educ * 2)`.
# `se` is classical, HC1, HC2, HC3 or cluster (with `cluster` the grouping vector); `weights` a
# weight vector with `weight_type` frequency, analytic or sampling. Frequency weights replicate the
# rows in R; analytic and sampling weights go to lm's `weights`, the sampling fit taking HC1 errors
# from sandwich, as Stata's pweight does. The degrees of freedom of the t table are n − k, or G − 1
# when clustered (the rulings of release 0.4.0).
case <- function(zeff, data_file, r_formula, data, factors = list(), level = 0.95, rename = c(),
                 se = "classical", cluster = NULL, weights = NULL, weight_type = "sampling") {
  # The first ten rows as `predict` sees them: before any replication under frequency weights.
  new <- head(data, 10)
  if (!is.null(weights) && weight_type == "frequency") {
    keep <- !is.na(weights) & weights > 0
    data <- data[rep(seq_len(nrow(data)), ifelse(is.na(weights), 0L, as.integer(weights))), , drop = FALSE]
    if (!is.null(cluster)) cluster <- rep(cluster, ifelse(is.na(weights), 0L, as.integer(weights)))
    fit <- lm(r_formula, data = data)
  } else if (!is.null(weights)) {
    data$zeff_w <- weights
    fit <- lm(r_formula, data = data, weights = zeff_w)
  } else {
    fit <- lm(r_formula, data = data)
  }
  all <- coef(fit)
  to_zeff <- function(name) {
    if (name %in% names(rename)) rename[[name]] else zeff_name(name, factors)
  }
  names(all) <- vapply(names(all), to_zeff, "")
  kept <- !is.na(all)
  s <- summary(fit)
  table <- coef(s)
  rownames(table) <- vapply(rownames(table), to_zeff, "")
  df <- fit$df.residual
  V <- vcov(fit)
  rownames(V) <- colnames(V) <- vapply(rownames(V), to_zeff, "")
  if (se != "classical") {
    # The rows lm kept (an NA cluster drops its row with the others); sandwich's HC1 to HC3 and
    # vcovCL with the HC1 small-sample adjustment and the cluster adjustment, Stata's CR1.
    used <- !is.na(residuals(fit)) & (if (is.null(fit$weights)) TRUE else fit$weights > 0)
    V <- if (se == "cluster") {
      g <- cluster[as.integer(rownames(model.frame(fit)))]
      df <- length(unique(g)) - 1
      vcovCL(fit, cluster = g, type = "HC1", cadjust = TRUE)
    } else {
      vcovHC(fit, type = se)
    }
    rownames(V) <- colnames(V) <- vapply(rownames(V), to_zeff, "")
    ses <- sqrt(diag(V))[rownames(table)]
    table[, 2] <- ses
    table[, 3] <- table[, 1] / ses
    table[, 4] <- 2 * pt(abs(table[, 3]), df, lower.tail = FALSE)
  }
  # The variance matrix in Zeff's names over the terms with an estimate (step 16), the
  # predictions for the first ten rows, and the first twenty residuals and fitted values of the
  # rows lm used.
  V_out <- V[rownames(table), rownames(table), drop = FALSE]
  predicted <- as.numeric(predict(fit, newdata = new))
  q <- qt(1 - (1 - level) / 2, df)
  ci <- cbind(table[, 1] - q * table[, 2], table[, 1] + q * table[, 2])
  rownames(ci) <- rownames(table)
  # The standardized coefficients of question 3 (d): each estimate times the standard deviation
  # of its model-matrix column over the outcome's, the rows as lm saw them (replicated under
  # frequency weights, unweighted otherwise); 0 for the intercept, whose column has no variation.
  X <- model.matrix(fit)
  yv <- model.response(model.frame(fit))
  betas <- coef(fit) * apply(X, 2, sd) / sd(yv)
  names(betas) <- names(all)
  columns <- list()
  for (term in names(all)[kept]) {
    columns[[term]] <- list(
      estimate = num17(table[term, 1]),
      std_error = num17(table[term, 2]),
      t_value = num17(table[term, 3]),
      p_value = num17(table[term, 4]),
      ci_lower = num17(ci[term, 1]),
      ci_upper = num17(ci[term, 2]),
      beta = num17(betas[[term]])
    )
  }
  list(
    formula = zeff, data = data_file, level = num(level),
    se = se, cluster = if (is.null(cluster)) "" else attr(cluster, "name"),
    weights = if (is.null(weights)) "" else attr(weights, "name"), weight_type = weight_type,
    terms = as.list(names(all)[kept]),
    omitted = as.list(names(all)[!kept]),
    columns = columns,
    nobs = nobs(fit),
    df_residual = as.integer(df),
    r_squared = num17(s$r.squared),
    adj_r_squared = num17(s$adj.r.squared),
    sigma = num17(s$sigma),
    rss = num17(deviance(fit)),
    log_likelihood = num17(as.numeric(logLik(fit))),
    aic = num17(AIC(fit)),
    bic = num17(BIC(fit)),
    vcov_terms = as.list(rownames(V_out)),
    vcov = lapply(seq_len(nrow(V_out)), function(i) lapply(V_out[i, ], num17)),
    predict_10 = lapply(predicted, num17),
    residuals_20 = lapply(head(as.numeric(residuals(fit)), 20), num17),
    fitted_20 = lapply(head(as.numeric(fitted(fit)), 20), num17),
    hat_20 = lapply(head(as.numeric(hatvalues(fit)), 20), num17),
    cooks_20 = lapply(head(as.numeric(cooks.distance(fit)), 20), num17),
    rstudent_20 = lapply(head(as.numeric(rstudent(fit)), 20), num17)
  )
}

wages <- read_dataset(file.path(root, "packages", "tier1", "datasets", "data", "wages.csv"))
wages$industry <- factor(wages$industry, levels = industry_levels)
large <- read_dataset(file.path(root, "tests", "datasets", "wages_large.csv"))
large$female <- large$female %in% c("true", TRUE)
large_f <- large
large_f$region <- factor(large$region)
region <- list(levels = levels(large_f$region), contrast = "treatment")
industry <- list(levels = industry_levels, contrast = "treatment")

cases <- list()
cases$arithmetic <- case("log(wage) ~ educ + exper + exper^2 + female", "wages",
  log(wage) ~ educ + exper + I(exper^2) + female, wages)
cases$categories <- case("log(wage) ~ educ + industry + female", "wages",
  log(wage) ~ educ + industry + female, wages, list(industry = industry))
summed <- wages
contrasts(summed$industry) <- zeff_sum(12)
cases$sum_contrast <- case("log(wage) ~ category(industry, contrast: sum) + educ", "wages",
  log(wage) ~ industry + educ, summed, list(industry = list(levels = industry_levels, contrast = "sum")))
cases$interaction <- case("log(wage) ~ educ * female + exper", "wages",
  log(wage) ~ educ * female + exper, wages)
cases$no_intercept <- case("wage ~ educ + exper - 1", "wages", wage ~ educ + exper - 1, wages)
cases$collinear <- case("wage ~ educ + hours + (educ * 2)", "wages_large",
  wage ~ educ + hours + I(educ * 2), large, rename = c("I(educ * 2)" = "(educ * 2)"))
cases$magnitudes <- case("wage ~ (educ * 1e7) + (hours * 1e12) + female", "wages_large",
  wage ~ I(educ * 1e7) + I(hours * 1e12) + female, large,
  rename = c("I(educ * 1e+07)" = "(educ * 1e7)", "I(hours * 1e+12)" = "(hours * 1e12)"))
cases$na_rows <- case("wage ~ hours + educ", "wages_large", wage ~ hours + educ, large)
cases$level_90 <- case("wage ~ educ + exper", "wages", wage ~ educ + exper, wages, level = 0.9)
cases$region <- case("wage ~ category(region) + hours", "wages_large",
  wage ~ region + hours, large_f, list(region = region))
cases$polynomial <- case("log(wage) ~ exper + exper^2 + exper^3 + exper^4 + exper^5", "wages",
  log(wage) ~ exper + I(exper^2) + I(exper^3) + I(exper^4) + I(exper^5), wages)

# The variance layer (step 11): sandwich's HC1 to HC3, clusters by worker (3,911) and by industry
# (12), and the three kinds of weight.
named <- function(values, name) { attr(values, "name") <- name; values }
arith <- log(wage) ~ educ + exper + I(exper^2) + female
arith_text <- "log(wage) ~ educ + exper + exper^2 + female"
cases$hc1 <- case(arith_text, "wages", arith, wages, se = "HC1")
cases$hc2 <- case(arith_text, "wages", arith, wages, se = "HC2")
cases$hc3 <- case(arith_text, "wages", arith, wages, se = "HC3")
cases$cluster_worker <- case(arith_text, "wages", arith, wages, se = "cluster", cluster = named(wages$worker, "worker"))
cases$cluster_industry <- case("log(wage) ~ educ + exper + female", "wages", log(wage) ~ educ + exper + female, wages,
  se = "cluster", cluster = named(as.character(wages$industry), "industry"))
cases$cluster_category <- case("log(wage) ~ educ + industry", "wages", log(wage) ~ educ + industry, wages,
  list(industry = industry), se = "cluster", cluster = named(wages$worker, "worker"))
cases$weights_analytic <- case("log(wage) ~ educ + exper + female", "wages", log(wage) ~ educ + exper + female, wages,
  weights = named(wages$hours, "hours"), weight_type = "analytic")
cases$weights_sampling <- case("log(wage) ~ educ + exper + female", "wages", log(wage) ~ educ + exper + female, wages,
  se = "HC1", weights = named(wages$hours, "hours"), weight_type = "sampling")
cases$weights_frequency <- case("log(wage) ~ educ + female", "wages", log(wage) ~ educ + female, wages,
  weights = named(wages$exper, "exper"), weight_type = "frequency")
cases$weights_frequency_robust <- case("log(wage) ~ educ + female", "wages", log(wage) ~ educ + female, wages,
  se = "HC1", weights = named(wages$exper, "exper"), weight_type = "frequency")
cases$weights_cluster <- case("log(wage) ~ educ + female", "wages", log(wage) ~ educ + female, wages,
  se = "cluster", cluster = named(wages$worker, "worker"), weights = named(wages$hours, "hours"), weight_type = "sampling")

# The other contrasts of section 7.2 under a regression: R's contr.helmert and contr.poly, the
# term names as question 5 decided them.
helm <- wages
contrasts(helm$industry) <- contr.helmert(12)
cases$helmert <- case("log(wage) ~ category(industry, contrast: helmert) + educ", "wages",
  log(wage) ~ industry + educ, helm, list(industry = list(levels = industry_levels, contrast = "helmert")))
polyed <- wages
contrasts(polyed$industry) <- contr.poly(12)
cases$poly <- case("log(wage) ~ category(industry, contrast: poly) + educ", "wages",
  log(wage) ~ industry + educ, polyed, list(industry = list(levels = industry_levels, contrast = "poly")))

# The Wooldridge textbook data (question 2 of release 0.4.0; the testing handbook, section 1.2):
# wage1 and wage2 from the wooldridge package, written beside the references as CSV with NA as
# empty cells, so that the Rust tests fit the same rows; the package is part of the harness alone.
library(wooldridge)
data(wage1)
data(wage2)
# Every value with 17 significant digits, which reads back to the same double and is the same
# text on every platform; R's own CSV writer chooses the digits by a routine that differs between
# macOS and Linux in the last place.
write_data <- function(data, file) {
  text <- data
  for (col in names(text)) text[[col]] <- ifelse(is.na(data[[col]]), "", sprintf("%.17g", data[[col]]))
  write.csv(text, file, row.names = FALSE, quote = FALSE, na = "")
}
write_data(wage1, file.path(root, "tests", "reference", "data", "wage1.csv"))
write_data(wage2, file.path(root, "tests", "reference", "data", "wage2.csv"))
cases$wage1_textbook <- case("lwage ~ educ + exper + tenure", "wage1", lwage ~ educ + exper + tenure, wage1)
w1 <- lwage ~ married * female + educ + exper + I(exper^2) + tenure + I(tenure^2)
w1_text <- "lwage ~ married * female + educ + exper + exper^2 + tenure + tenure^2"
cases$wage1_hc1 <- case(w1_text, "wage1", w1, wage1, se = "HC1")
cases$wage1_hc2 <- case(w1_text, "wage1", w1, wage1, se = "HC2")
cases$wage1_hc3 <- case(w1_text, "wage1", w1, wage1, se = "HC3")
cases$wage1_cluster <- case("lwage ~ educ + exper + tenure", "wage1", lwage ~ educ + exper + tenure, wage1,
  se = "cluster", cluster = named(wage1$numdep, "numdep"))
cases$wage1_level_99 <- case("lwage ~ educ + female * married", "wage1", lwage ~ educ + female * married, wage1,
  level = 0.99)
cases$wage2_textbook <- case("lwage ~ educ + exper + tenure + married + black + south + urban", "wage2",
  lwage ~ educ + exper + tenure + married + black + south + urban, wage2)
cases$wage2_analytic <- case("lwage ~ educ + exper + tenure", "wage2", lwage ~ educ + exper + tenure, wage2,
  weights = named(wage2$hours, "hours"), weight_type = "analytic")
cases$wage2_sampling <- case("lwage ~ educ + exper + tenure", "wage2", lwage ~ educ + exper + tenure, wage2,
  se = "HC1", weights = named(wage2$hours, "hours"), weight_type = "sampling")
cases$wage2_frequency <- case("lwage ~ educ + exper", "wage2", lwage ~ educ + exper, wage2,
  weights = named(wage2$sibs, "sibs"), weight_type = "frequency")
cases$wage2_na_rows <- case("lwage ~ educ + meduc + feduc", "wage2", lwage ~ educ + meduc + feduc, wage2)
cases$wage2_cluster_hc <- case("lwage ~ educ + exper + IQ", "wage2", lwage ~ educ + exper + IQ, wage2,
  se = "cluster", cluster = named(wage2$age, "age"))

# ----- step 18: the Wald test, nested models and the delta method -----
# A fit with its variance matrix in Zeff's names (classical, sandwich's HC1 or HC3, or vcovCL
# as above) and the degrees of freedom of its t table; restrictions as named coefficients over the
# terms and a constant (Rβ = c), the Wald F written out in base R; `anova` on two nested
# classical fits and the Wald F of the extra terms under robust or clustered errors; the delta
# method through `deriv`, the gradient at the estimates over the full variance matrix, with the t
# and the interval on the fit's degrees of freedom at the 95 per cent level.
fit_v <- function(zeff, data_file, r_formula, data, se = "classical", cluster = NULL) {
  fit <- lm(r_formula, data = data)
  df <- fit$df.residual
  V <- vcov(fit)
  if (se == "cluster") {
    g <- cluster[as.integer(rownames(model.frame(fit)))]
    df <- length(unique(g)) - 1
    V <- vcovCL(fit, cluster = g, type = "HC1", cadjust = TRUE)
  } else if (se != "classical") {
    V <- vcovHC(fit, type = se)
  }
  terms <- vapply(names(coef(fit)), zeff_name, "", factors = list())
  b <- coef(fit)
  names(b) <- terms
  rownames(V) <- colnames(V) <- terms
  list(zeff = zeff, data = data_file, se = se, cluster = if (is.null(cluster)) "" else attr(cluster, "name"),
       fit = fit, b = b, V = V, df = df, terms = terms, level = 0.95)
}

restriction <- function(text, coefs, constant = 0) list(text = text, coefs = coefs, constant = constant)

wald <- function(info, restrictions) {
  k <- length(info$b)
  R <- t(vapply(restrictions, function(x) {
    r <- setNames(rep(0, k), info$terms)
    for (t in names(x$coefs)) r[[t]] <- x$coefs[[t]]
    r
  }, numeric(k)))
  cc <- vapply(restrictions, function(x) x$constant, 0)
  d <- as.numeric(R %*% info$b - cc)
  F <- as.numeric(t(d) %*% solve(R %*% info$V %*% t(R)) %*% d) / nrow(R)
  list(restrictions = lapply(restrictions, function(x) list(text = x$text, coefs = lapply(x$coefs, num17),
                                                            constant = num17(x$constant))),
       f = num17(F), p = num17(pf(F, nrow(R), info$df, lower.tail = FALSE)), q = nrow(R), df = as.integer(info$df))
}

compare_fits <- function(small_key, big_key) {
  small <- fits[[small_key]]
  big <- fits[[big_key]]
  extra <- setdiff(big$terms, small$terms)
  if (big$se == "classical") {
    a <- anova(small$fit, big$fit)
    w <- list(f = num17(a$F[2]), p = num17(a[["Pr(>F)"]][2]), q = length(extra), df = as.integer(big$df))
  } else {
    w <- wald(big, lapply(extra, function(t) restriction(t, setNames(list(1), t))))
    w$restrictions <- NULL
  }
  c(list(small = small_key, big = big_key), w)
}

delta <- function(info, name, text, expr) {
  vars <- all.vars(expr)
  g <- deriv(expr, vars)
  env <- new.env()
  for (v in vars) assign(v, info$b[[v]], envir = env)
  val <- eval(g, env)
  grad <- attr(val, "gradient")
  G <- setNames(rep(0, length(info$b)), info$terms)
  G[colnames(grad)] <- grad[1, ]
  se <- sqrt(as.numeric(t(G) %*% info$V %*% G))
  value <- as.numeric(val)
  t <- value / se
  q <- qt(1 - (1 - info$level) / 2, info$df)
  list(name = name, text = text, value = num17(value), se = num17(se), t = num17(t),
       p = num17(2 * pt(abs(t), info$df, lower.tail = FALSE)),
       lower = num17(value - q * se), upper = num17(value + q * se))
}

fits <- list(
  A = fit_v("log(wage) ~ educ + exper + female", "wages", log(wage) ~ educ + exper + female, wages),
  A0 = fit_v("log(wage) ~ educ + exper", "wages", log(wage) ~ educ + exper, wages),
  B = fit_v("log(wage) ~ educ + exper + female", "wages", log(wage) ~ educ + exper + female, wages, se = "HC1"),
  B0 = fit_v("log(wage) ~ educ + exper", "wages", log(wage) ~ educ + exper, wages, se = "HC1"),
  C = fit_v("log(wage) ~ educ + exper + female", "wages", log(wage) ~ educ + exper + female, wages,
            se = "cluster", cluster = named(wages$worker, "worker")),
  C0 = fit_v("log(wage) ~ educ + exper", "wages", log(wage) ~ educ + exper, wages,
             se = "cluster", cluster = named(wages$worker, "worker")),
  D = fit_v("lwage ~ educ + exper + tenure", "wage1", lwage ~ educ + exper + tenure, wage1),
  D0 = fit_v("lwage ~ educ", "wage1", lwage ~ educ, wage1),
  E = fit_v("lwage ~ educ + exper + exper^2", "wage1", lwage ~ educ + exper + I(exper^2), wage1, se = "HC3"),
  E0 = fit_v("lwage ~ educ", "wage1", lwage ~ educ, wage1, se = "HC3")
)
one <- list(restriction("educ == exper", list(educ = 1, exper = -1)))
two <- list(restriction("educ + exper == 0.1", list(educ = 1, exper = 1), 0.1),
            restriction("female == -0.1", list(female = 1), -0.1))
three <- list(restriction("2 * educ - exper == 0", list(educ = 2, exper = -1)),
              restriction("female == 0", list(female = 1)),
              restriction("`(intercept)` == 1", list(`(intercept)` = 1), 1))
tests <- list(
  c(list(fit = "A"), wald(fits$A, one)), c(list(fit = "A"), wald(fits$A, two)), c(list(fit = "A"), wald(fits$A, three)),
  c(list(fit = "B"), wald(fits$B, one)), c(list(fit = "B"), wald(fits$B, two)), c(list(fit = "B"), wald(fits$B, three)),
  c(list(fit = "C"), wald(fits$C, one)), c(list(fit = "C"), wald(fits$C, two)), c(list(fit = "C"), wald(fits$C, three)),
  c(list(fit = "D"), wald(fits$D, list(restriction("exper == tenure", list(exper = 1, tenure = -1))))),
  c(list(fit = "D"), wald(fits$D, list(restriction("educ == 0.09", list(educ = 1), 0.09)))),
  c(list(fit = "D"), wald(fits$D, list(restriction("exper == 0", list(exper = 1)), restriction("tenure == 0", list(tenure = 1))))),
  c(list(fit = "E"), wald(fits$E, list(restriction("`exper^2` == 0", list(`exper^2` = 1))))),
  c(list(fit = "E"), wald(fits$E, list(restriction("exper + 50 * `exper^2` == 0", list(exper = 1, `exper^2` = 50)))))
)
comparisons <- list(compare_fits("A0", "A"), compare_fits("B0", "B"), compare_fits("C0", "C"),
                    compare_fits("D0", "D"), compare_fits("E0", "E"))
on_wages <- function(info) list(
  delta(info, "ratio", "educ / exper", quote(educ / exper)),
  delta(info, "gain", "exp(educ * 4) - 1", quote(exp(educ * 4) - 1)),
  delta(info, "root", "sqrt(educ)", quote(sqrt(educ))),
  delta(info, "logged", "log(educ / exper)", quote(log(educ / exper))),
  delta(info, "square", "educ ^ 2", quote(educ^2)),
  delta(info, "product", "educ * female + exper", quote(educ * female + exper)),
  delta(info, "scaled", "100 * educ - 2", quote(100 * educ - 2)),
  delta(info, "tenth", "log10(educ)", quote(log(educ) / log(10))),
  delta(info, "l1p", "log1p(exper)", quote(log(1 + exper))),
  delta(info, "e1", "expm1(educ)", quote(exp(educ) - 1)),
  delta(info, "l2", "log2(educ)", quote(log(educ) / log(2))),
  delta(info, "neg", "-educ / (exper - female)", quote(-educ / (exper - female))),
  delta(info, "power", "(educ + exper) ^ 1.5", quote((educ + exper)^1.5))
)
derivations <- c(
  lapply(on_wages(fits$A), function(d) c(list(fit = "A"), d)),
  lapply(on_wages(fits$B), function(d) c(list(fit = "B"), d)),
  lapply(on_wages(fits$C), function(d) c(list(fit = "C"), d)),
  list(c(list(fit = "E"), delta(fits$E, "ratio2", "exper / `exper^2`", quote(exper / `exper^2`))),
       c(list(fit = "E"), delta(fits$E, "turning", "-exper / (2 * `exper^2`)", quote(-exper / (2 * `exper^2`)))),
       c(list(fit = "D"), delta(fits$D, "years", "tenure / exper", quote(tenure / exper))))
)
inference <- list(
  fits = lapply(fits, function(info) list(formula = info$zeff, data = info$data, se = info$se, cluster = info$cluster,
                                          terms = as.list(unname(info$terms)))),
  tests = tests, compare = comparisons, derive = derivations
)

out <- list(reference = reference_block(), cases = cases, inference = inference)
write_json(out, file.path(here(), "regression.json"))
