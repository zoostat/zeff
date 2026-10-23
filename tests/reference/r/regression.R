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
  q <- qt(1 - (1 - level) / 2, df)
  ci <- cbind(table[, 1] - q * table[, 2], table[, 1] + q * table[, 2])
  rownames(ci) <- rownames(table)
  columns <- list()
  for (term in names(all)[kept]) {
    columns[[term]] <- list(
      estimate = num17(table[term, 1]),
      std_error = num17(table[term, 2]),
      t_value = num17(table[term, 3]),
      p_value = num17(table[term, 4]),
      ci_lower = num17(ci[term, 1]),
      ci_upper = num17(ci[term, 2])
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
    log_likelihood = num17(as.numeric(logLik(fit)))
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

out <- list(reference = reference_block(), cases = cases)
write_json(out, file.path(here(), "regression.json"))
