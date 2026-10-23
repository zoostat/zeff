# R's model matrices for the formulas of release 0.4.0, step 8 (Language Reference, section 7.2),
# on the shipped wages (packages/tier1/datasets/data/wages.csv, its industry levels in the order
# the Arrow file carries), tests/datasets/wages_large.csv and the edge-case files. Each case gives
# the columns under Zeff's names (section 7.2 and question 5 of the release), with every value of
# the first twenty rows to 17 significant digits, the column sums and sums of squares, and the
# rows used. The contrasts follow Zeff's conventions: treatment and sum omit the base, the first
# level; helmert and poly are R's contr.helmert and contr.poly. Run with
# `Rscript tests/reference/r/design.R`.
source(file.path(dirname(sub("^--file=", "", commandArgs(FALSE)[grepl("^--file=", commandArgs(FALSE))])), "common.R"))
Sys.setlocale("LC_COLLATE", "C")
root <- normalizePath(file.path(here(), "..", "..", ".."))

num17 <- function(x) ifelse(is.na(x), "NA", sprintf("%.17g", x))

industry_levels <- c("Agriculture", "Mining", "Utilities", "Construction", "Manufacturing", "Retail",
                     "Transport", "Finance", "Services", "Education", "Health", "Public administration")

# Zeff's sum contrast: the base (the first level) is the omitted level, so R's contr.sum, which
# omits the last, is given its rows with the last moved first.
zeff_sum <- function(k) {
  C <- contr.sum(k)
  C[c(k, seq_len(k - 1)), , drop = FALSE]
}

# A level inside a term name, quoted when it holds =, : or + (section 7.2).
level_name <- function(level) {
  if (grepl("[=:+]", level)) paste0('"', gsub('"', '\\\\"', gsub("\\\\", "\\\\\\\\", level)), '"') else level
}

# R's column names as Zeff writes them. `factors` is a list var => list(levels, contrast).
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
    if (part == "treatedTRUE") return("treated")
    part <- sub("^I\\((.*)\\)$", "\\1", part)
    # A variable's term name is its name as text (section 7.2), without the backticks R adds
    # to a name it cannot parse.
    gsub("`", "", part, fixed = TRUE)
  }
  # A level holding ":" is matched whole before the name is split on ":".
  for (var in names(factors)) {
    for (l in factors[[var]]$levels) if (name == paste0(var, l)) return(paste0(var, "=", level_name(l)))
  }
  paste(vapply(strsplit(name, ":", fixed = TRUE)[[1]], part_name, ""), collapse = ":")
}

case <- function(zeff, data_file, r_formula, data, factors = list()) {
  mm <- model.matrix(r_formula, data = data)
  names <- vapply(colnames(mm), zeff_name, "", factors = factors)
  columns <- list()
  for (j in seq_len(ncol(mm))) {
    v <- mm[, j]
    columns[[names[j]]] <- list(
      first_20 = as.list(unname(num17(head(v, 20)))),
      sum = num17(sum(v)),
      sum_squares = num17(sum(v * v))
    )
  }
  list(formula = zeff, data = data_file, rows_used = nrow(mm), columns = columns)
}

wages <- read_dataset(file.path(root, "packages", "tier1", "datasets", "data", "wages.csv"))
wages$industry <- factor(wages$industry, levels = industry_levels)
large <- read_dataset(file.path(root, "tests", "datasets", "wages_large.csv"))
large$female <- large$female %in% c("true", TRUE)
# The same frame with region a factor, as `category(region)` makes it, under its own name.
large_f <- large
large_f$region <- factor(large$region)
region <- list(levels = levels(large_f$region), contrast = "treatment")
awkward <- read_dataset(file.path(root, "tests", "datasets", "awkward_names.csv"))
one <- read_dataset(file.path(root, "tests", "datasets", "one_row.csv"))
symbols <- read_dataset(file.path(root, "tests", "datasets", "level_symbols.csv"))
symbols$grp <- factor(symbols$grp)

industry <- list(levels = industry_levels, contrast = "treatment")
cases <- list()

cases$arithmetic <- case("log(wage) ~ educ + exper + exper^2 + female", "wages",
  log(wage) ~ educ + exper + I(exper^2) + female, wages)
cases$treatment <- case("log(wage) ~ educ + industry", "wages",
  log(wage) ~ educ + industry, wages, list(industry = industry))
relevelled <- wages
relevelled$industry <- relevel(relevelled$industry, "Retail")
cases$base_retail <- case("log(wage) ~ educ + base(industry, \"Retail\")", "wages",
  log(wage) ~ educ + industry, relevelled,
  list(industry = list(levels = levels(relevelled$industry), contrast = "treatment")))
summed <- wages
contrasts(summed$industry) <- zeff_sum(12)
cases$sum <- case("log(wage) ~ category(industry, contrast: sum) + female", "wages",
  log(wage) ~ industry + female, summed, list(industry = list(levels = industry_levels, contrast = "sum")))
helm <- wages
contrasts(helm$industry) <- contr.helmert(12)
cases$helmert <- case("log(wage) ~ category(industry, contrast: helmert)", "wages",
  log(wage) ~ industry, helm, list(industry = list(levels = industry_levels, contrast = "helmert")))
polyed <- wages
contrasts(polyed$industry) <- contr.poly(12)
cases$poly <- case("log(wage) ~ category(industry, contrast: poly)", "wages",
  log(wage) ~ industry, polyed, list(industry = list(levels = industry_levels, contrast = "poly")))
cases$product <- case("log(wage) ~ industry * female", "wages",
  log(wage) ~ industry * female, wages, list(industry = industry))
cases$no_intercept <- case("wage ~ industry - 1", "wages",
  wage ~ industry - 1, wages, list(industry = list(levels = industry_levels, contrast = "full")))
cases$interaction_without_main <- case("wage ~ educ + industry:female", "wages",
  wage ~ educ + industry:female, wages, list(industry = list(levels = industry_levels, contrast = "full")))
cases$three_way <- case("log(wage) ~ educ * exper * female", "wages",
  log(wage) ~ educ * exper * female, wages)
cases$region_by_female <- case("wage ~ category(region) * female", "wages_large",
  wage ~ region * female, large_f, list(region = region))
cases$interact <- case("wage ~ interact([educ, hours, female], order: 2)", "wages_large",
  wage ~ (educ + hours + female)^2, large)
cases$interact_three_way <- case("log(wage) ~ interact([educ, exper, female], order: 3)", "wages",
  log(wage) ~ (educ + exper + female)^3, wages)
cases$interact_with_category <- case("wage ~ interact([category(region), female, hours], order: 2)", "wages_large",
  wage ~ (region + female + hours)^2, large_f, list(region = region))
cases$interact_and_more <- case("wage ~ educ + interact([category(region), hours], order: 2) + female", "wages_large",
  wage ~ educ + (region + hours)^2 + female, large_f, list(region = region))
cases$range <- case("hours ~ id..wage + educ", "wages_large",
  hours ~ id + wage + educ, large)
cases$na_rows <- case("wage ~ hours + educ", "wages_large", wage ~ hours + educ, large)
cases$awkward <- case("`Gross Sales (total)` ~ `x y` + `日本語` + `1st` + Wage + wage_2", "awkward_names",
  `Gross Sales (total)` ~ `x y` + 日本語 + `1st` + Wage + wage_2, awkward)
cases$one_row <- case("y ~ x", "one_row", y ~ x, one)
cases$level_symbols <- case("y ~ category(grp) + n", "level_symbols", y ~ grp + n, symbols,
  list(grp = list(levels = levels(symbols$grp), contrast = "treatment")))

out <- list(reference = reference_block(), cases = cases)
write_json(out, file.path(here(), "design.json"))
