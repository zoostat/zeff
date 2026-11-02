# fixest's fixed-effects regressions for release 0.6.0, step 18 (Language Reference, sections 7.2
# and 10.2; Engine and Project Specification, section 13; question 4 of the release, decided 1
# November 2026): `feols` with absorbed effects on the shipped wages and on Wooldridge's wagepan,
# with classical, heteroskedasticity-robust and clustered errors (fixest's small-sample defaults,
# `fixef.K = "nested"`), analytic and sampling weights, and an interacted effect. The projections
# run to a tolerance of 1e-11 so that the comparison tests the estimator and not the stopping
# rule. Values to 17 significant digits. Run with `Rscript tests/reference/r/fixed_effects.R`.
source(file.path(dirname(sub("^--file=", "", commandArgs(FALSE)[grepl("^--file=", commandArgs(FALSE))])), "common.R"))
Sys.setlocale("LC_COLLATE", "C")
root <- normalizePath(file.path(here(), "..", "..", ".."))
.libPaths(c(file.path(here(), "library"), .libPaths()))
suppressMessages(library(fixest))
library(wooldridge)

num17 <- function(x) ifelse(is.na(x), "NA", sprintf("%.17g", x))
TOL <- 1e-11

industry_levels <- c("Agriculture", "Mining", "Utilities", "Construction", "Manufacturing", "Retail",
                     "Transport", "Finance", "Services", "Education", "Health", "Public administration")

# fixest writes `I(x^2)` where Zeff writes `x^2`, and `(Intercept)` never appears.
zeff_name <- function(name) {
  while (grepl("^I\\(.*\\)$", name)) name <- sub("^I\\((.*)\\)$", "\\1", name)
  name
}

# One case: `zeff` is the formula as a script writes it, `fe_text` the `fe:` argument, `fe` the
# formula's `| fixed effects` part; `se` is classical, HC1 or cluster (with `cluster` the
# variable's name); `weights` a variable's name with `weight_type` analytic (fixest's weights
# with classical errors) or sampling (with HC1 errors, as Stata's pweight). Zeff scales analytic
# and sampling weights to sum to the rows, which changes no estimate and no error.
case <- function(zeff, data_file, lhs, rhs, fe, fe_text, data, se = "classical", cluster = NULL,
                 weights = NULL, weight_type = "sampling", level = 0.95) {
  f <- as.formula(paste0(lhs, " ~ ", rhs, " | ", fe))
  vcov <- if (se == "cluster") as.formula(paste0("~", cluster)) else if (se == "HC1") "hetero" else "iid"
  args <- list(fml = f, data = data, vcov = vcov, fixef.tol = TOL)
  if (!is.null(weights)) args$weights <- as.formula(paste0("~", weights))
  fit <- do.call(feols, args)
  # The singletons: the complete rows (the fit with none removed) less the rows the fit used.
  complete <- do.call(feols, c(args, list(fixef.rm = "none")))
  all <- coef(fit)
  names(all) <- vapply(names(all), zeff_name, "")
  omitted <- if (is.null(fit$collin.var)) character(0) else unname(vapply(fit$collin.var, zeff_name, ""))
  table <- coeftable(fit)
  rownames(table) <- vapply(rownames(table), zeff_name, "")
  ci <- confint(fit, level = level)
  rownames(ci) <- vapply(rownames(ci), zeff_name, "")
  columns <- list()
  for (term in names(all)) {
    columns[[term]] <- list(
      estimate = num17(table[term, 1]),
      std_error = num17(table[term, 2]),
      t_value = num17(table[term, 3]),
      p_value = num17(table[term, 4]),
      ci_lower = num17(ci[term, 1]),
      ci_upper = num17(ci[term, 2])
    )
  }
  r <- r2(fit)
  used <- obs(fit)
  first <- head(used, 20)
  new <- head(data, 10)
  predicted <- suppressWarnings(as.numeric(predict(fit, newdata = new)))
  list(
    formula = zeff, fe = fe_text, data = data_file, level = num(level),
    se = se, cluster = if (is.null(cluster)) "" else cluster,
    weights = if (is.null(weights)) "" else weights, weight_type = weight_type,
    terms = as.list(names(all)),
    omitted = as.list(omitted),
    columns = columns,
    nobs = nobs(fit),
    singletons = as.integer(nobs(complete) - nobs(fit)),
    fixef_sizes = as.list(as.integer(fit$fixef_sizes)),
    df_residual = as.integer(degrees_freedom(fit, "resid")),
    df_t = as.integer(degrees_freedom(fit, "t")),
    r_squared = num17(r[["r2"]]),
    adj_r_squared = num17(r[["ar2"]]),
    within_r_squared = num17(r[["wr2"]]),
    adj_within_r_squared = num17(r[["war2"]]),
    rss = num17(sum(resid(fit)^2 * (if (is.null(fit$weights)) 1 else fit$weights))),
    rows_20 = as.list(as.integer(first)),
    fitted_20 = lapply(head(as.numeric(fitted(fit)), 20), num17),
    residuals_20 = lapply(head(as.numeric(resid(fit)), 20), num17),
    predict_10 = lapply(predicted, num17)
  )
}

wages <- read_dataset(file.path(root, "packages", "tier1", "datasets", "data", "wages.csv"))
wages$industry <- factor(wages$industry, levels = industry_levels)
wages$lwage <- log(wages$wage)
arith <- "educ + exper + I(exper^2) + female"
arith_text <- "log(wage) ~ educ + exper + exper^2 + female"

cases <- list()
cases$firm <- case(arith_text, "wages", "lwage", arith, "firm", "firm", wages)
cases$firm_robust <- case(arith_text, "wages", "lwage", arith, "firm", "firm", wages, se = "HC1")
cases$industry_year <- case(arith_text, "wages", "lwage", arith, "industry + year", "industry + year", wages)
cases$industry_year_cluster <- case(arith_text, "wages", "lwage", arith, "industry + year", "industry + year", wages,
  se = "cluster", cluster = "worker")
cases$worker_year_cluster <- case(arith_text, "wages", "lwage", arith, "worker + year", "worker + year", wages,
  se = "cluster", cluster = "worker")
cases$industry_by_year <- case("log(wage) ~ educ + female", "wages", "lwage", "educ + female", "industry^year",
  "industry:year", wages)
cases$firm_cluster_worker <- case(arith_text, "wages", "lwage", arith, "firm", "firm", wages,
  se = "cluster", cluster = "worker")
cases$firm_cluster_firm <- case(arith_text, "wages", "lwage", arith, "firm", "firm", wages,
  se = "cluster", cluster = "firm")
cases$firm_analytic <- case("log(wage) ~ educ + female", "wages", "lwage", "educ + female", "firm", "firm", wages,
  weights = "hours", weight_type = "analytic")
cases$firm_sampling <- case("log(wage) ~ educ + female", "wages", "lwage", "educ + female", "firm", "firm", wages,
  se = "HC1", weights = "hours", weight_type = "sampling")
cases$firm_level_90 <- case("log(wage) ~ educ + female", "wages", "lwage", "educ + female", "firm", "firm", wages,
  level = 0.9)

# Wooldridge's wagepan (the testing handbook, section 1.2): 545 men over 1980 to 1987, written
# beside the references with 17 significant digits as wage1 and wage2 are.
data(wagepan)
write_data <- function(data, file) {
  text <- data
  for (col in names(text)) text[[col]] <- ifelse(is.na(data[[col]]), "", sprintf("%.17g", data[[col]]))
  write.csv(text, file, row.names = FALSE, quote = FALSE, na = "")
}
write_data(wagepan, file.path(root, "tests", "reference", "data", "wagepan.csv"))
cases$wagepan_nr <- case("lwage ~ expersq + married + union", "wagepan", "lwage", "expersq + married + union",
  "nr", "nr", wagepan)
cases$wagepan_nr_year <- case("lwage ~ expersq + married + union", "wagepan", "lwage", "expersq + married + union",
  "nr + year", "nr + year", wagepan, se = "cluster", cluster = "nr")
cases$wagepan_educ_absorbed <- case("lwage ~ educ + expersq + married + union", "wagepan", "lwage",
  "educ + expersq + married + union", "nr + year", "nr + year", wagepan, se = "cluster", cluster = "nr")

write_json(list(reference = list(software = R.version.string,
                                 fixest = as.character(packageVersion("fixest")),
                                 note = "fixest::feols with fixef.tol 1e-11; values written with 17 significant digits"),
                cases = cases),
           file.path(here(), "fixed_effects.json"))
