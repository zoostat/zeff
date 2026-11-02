# fixest's event-study terms for release 0.6.0, step 19 (catalogue row 252; question 2 of the
# release, decided 1 November 2026): `i(rel, ref = c(-1, -1000))` with the never-treated rows at
# the reference of their own, on a generated staggered panel with never-treated units, under
# two-way fixed effects with clustered errors, pooled with an intercept, and with a Date time
# whose relative periods are days. The panel is written beside the references with 17
# significant digits. Values to 17 significant digits. Run with `Rscript tests/reference/r/event.R`.
source(file.path(dirname(sub("^--file=", "", commandArgs(FALSE)[grepl("^--file=", commandArgs(FALSE))])), "common.R"))
Sys.setlocale("LC_COLLATE", "C")
root <- normalizePath(file.path(here(), "..", "..", ".."))
.libPaths(c(file.path(here(), "library"), .libPaths()))
suppressMessages(library(fixest))

num17 <- function(x) ifelse(is.na(x), "NA", sprintf("%.17g", x))
TOL <- 1e-11

# The panel: 60 units over 2010 to 2017; cohorts first treated in 2012, 2014 and 2016 (15 units
# each) and 15 never treated; y = unit effect + year effect + 0.3 x + a dynamic effect of
# 0.2 (k + 1) from the period of treatment on + noise, with R's generator under a fixed seed
# (the file, not the draws, is the reference input).
set.seed(20261102)
units <- 60; years <- 2010:2017
d <- expand.grid(year = years, unit = seq_len(units))[, c("unit", "year")]
cohort <- rep(c(2012, 2014, 2016, NA), each = 15)
d$treat <- cohort[d$unit]
unit_effect <- rnorm(units, 0, 0.5); year_effect <- 0.05 * (years - 2010) + rnorm(length(years), 0, 0.1)
d$x <- rnorm(nrow(d))
rel <- d$year - d$treat
effect <- ifelse(!is.na(rel) & rel >= 0, 0.2 * (rel + 1), 0)
d$y <- unit_effect[d$unit] + year_effect[d$year - 2009] + 0.3 * d$x + effect + rnorm(nrow(d), 0, 0.3)
d$day <- as.Date("2020-01-01") + 7 * (d$year - 2010)
d$treat_day <- as.Date("2020-01-01") + 7 * (d$treat - 2010)

write_data <- function(data, file) {
  text <- data
  for (col in names(text)) {
    v <- data[[col]]
    text[[col]] <- if (inherits(v, "Date")) ifelse(is.na(v), "", format(v, "%Y-%m-%d")) else ifelse(is.na(v), "", sprintf("%.17g", v))
  }
  write.csv(text, file, row.names = FALSE, quote = FALSE, na = "")
}
write_data(d, file.path(root, "tests", "reference", "data", "event_panel.csv"))

zeff_name <- function(name) {
  if (grepl("^rel(_days)?::", name)) return(paste0("event=", sub("^rel(_days)?::", "", name)))
  if (name == "(Intercept)") return("(intercept)")
  name
}

case <- function(zeff, fe_text, fml, data, se = "classical", cluster = NULL) {
  vcov <- if (se == "cluster") as.formula(paste0("~", cluster)) else "iid"
  fit <- feols(fml, data = data, vcov = vcov, fixef.tol = TOL)
  all <- coef(fit)
  names(all) <- vapply(names(all), zeff_name, "")
  table <- coeftable(fit)
  rownames(table) <- vapply(rownames(table), zeff_name, "")
  ci <- confint(fit)
  rownames(ci) <- vapply(rownames(ci), zeff_name, "")
  columns <- list()
  for (term in names(all)) {
    columns[[term]] <- list(
      estimate = num17(table[term, 1]), std_error = num17(table[term, 2]),
      t_value = num17(table[term, 3]), p_value = num17(table[term, 4]),
      ci_lower = num17(ci[term, 1]), ci_upper = num17(ci[term, 2]))
  }
  r <- r2(fit)
  omitted <- if (is.null(fit$collin.var)) character(0) else unname(vapply(fit$collin.var, zeff_name, ""))
  list(formula = zeff, fe = fe_text, data = "event_panel", se = se,
       cluster = if (is.null(cluster)) "" else cluster,
       terms = as.list(names(all)), omitted = as.list(omitted), columns = columns,
       nobs = nobs(fit), df_t = as.integer(degrees_freedom(fit, "t")),
       r_squared = num17(r[["r2"]]), within_r_squared = num17(if (is.na(r[["wr2"]])) r[["r2"]] else r[["wr2"]]))
}

d$rel <- ifelse(is.na(d$treat), -1000, d$year - d$treat)
d$rel_days <- ifelse(is.na(d$treat_day), -1000, as.integer(d$day - d$treat_day))
cases <- list()
cases$two_way <- case("y ~ x + event(year, treat)", "unit + year",
  y ~ x + i(rel, ref = c(-1, -1000)) | unit + year, d, se = "cluster", cluster = "unit")
cases$pooled <- case("y ~ x + event(year, treat)", "",
  y ~ x + i(rel, ref = c(-1, -1000)), d)
cases$reference_zero <- case("y ~ x + event(year, treat, ref: 0)", "unit + year",
  y ~ x + i(rel, ref = c(0, -1000)) | unit + year, d, se = "cluster", cluster = "unit")
cases$dates <- case("y ~ x + event(day, treat_day, ref: -7)", "unit + year",
  y ~ x + i(rel_days, ref = c(-7, -1000)) | unit + year, d, se = "cluster", cluster = "unit")

write_json(list(reference = list(software = R.version.string, fixest = as.character(packageVersion("fixest")),
                                 note = "fixest::feols with i(rel, ref = c(-1, -1000)), fixef.tol 1e-11; values written with 17 significant digits"),
                cases = cases),
           file.path(here(), "event.json"))
