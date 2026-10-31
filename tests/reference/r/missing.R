# R's values for assert, n_missing and fill_missing (release 0.5.0, step 15): stopifnot() on
# checks that hold and fail, sum(is.na(x)) and replace(x, is.na(x), v) on fixed vectors with NA
# among the values. Every value is exact, so the comparison is exact; R's replace() of an
# integer vector by a double gives doubles, as Zeff's joining rule does, and a text cannot fill
# numbers in Zeff (Z0214) where R would coerce the numbers to text. Run with
# `Rscript tests/reference/r/missing.R`.
source(file.path(dirname(sub("^--file=", "", commandArgs(FALSE)[grepl("^--file=", commandArgs(FALSE))])), "common.R"))

vectors <- list(
  c(1, NA, 3, NA),
  c(1, 2, 3),
  c(NA, NA),
  c(1.5, NA, -2.25, 1e12),
  c(NA, 0, NA, 1e-8, 1e7),
  numeric(0)
)
fill <- 0
num_case <- function(x) list(
  items = as.list(num(x)),
  n_missing = sum(is.na(x)),
  filled = as.list(num(replace(x, is.na(x), fill))),
  n_missing_after = sum(is.na(replace(x, is.na(x), fill)))
)
texts <- list(c("a", NA, "c"), c(NA, NA), c("x"))
text_case <- function(s) list(
  items = as.list(ifelse(is.na(s), "NA", s)),
  n_missing = sum(is.na(s)),
  filled = as.list(replace(s, is.na(s), "none"))
)
checks <- list(
  list(condition = "1 + 1 == 2", holds = tryCatch({ stopifnot(1 + 1 == 2); TRUE }, error = function(e) FALSE)),
  list(condition = "2 > 3", holds = tryCatch({ stopifnot(2 > 3); TRUE }, error = function(e) FALSE)),
  list(condition = "all(c(1, 2) > 0)", holds = tryCatch({ stopifnot(all(c(1, 2) > 0)); TRUE }, error = function(e) FALSE))
)

write_json(list(
  reference = reference_block(),
  numbers = lapply(vectors, num_case),
  texts = lapply(texts, text_case),
  checks = checks
), file.path(here(), "missing.json"))
