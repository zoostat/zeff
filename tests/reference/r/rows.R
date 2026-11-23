# R's rowMeans() and rowSums() with na.rm = TRUE (release 0.5.0, step 17; the function is
# row(vars, stat: mean | sum) from release 0.8.0, and the keys below keep the old names), and the min_valid rule
# written out in base R: a row's mean when at least k values are present, NA otherwise. The
# values are written with 17 significant digits and compared within a relative 1e-12 (R
# accumulates row sums in long double on x86, Zeff with a compensated sum, so the last bits may
# differ). Where Zeff departs from R the test names it: rowSums(na.rm = TRUE) of a row with no
# value is 0 in R and NA in Zeff. From release 0.9.0 the count of missing values in each row,
# rowSums(is.na()), for row(vars, stat: n_missing): whole numbers, compared exactly.
# Run with `Rscript tests/reference/r/rows.R`.
source(file.path(dirname(sub("^--file=", "", commandArgs(FALSE)[grepl("^--file=", commandArgs(FALSE))])), "common.R"))

dec <- function(x) ifelse(is.na(x), "NA", sprintf("%.17g", x))
matrices <- list(
  cbind(q1 = c(5, 4, NA, 1.5), q2 = c(3, NA, NA, 2.5), q3 = c(4, 4, NA, 1e12)),
  cbind(a = c(1, 2, 3), b = c(1, 2, 3), c = c(1, 2, 3)),
  cbind(a = c(1e7, -1e7, 1e-8), b = c(1e12, 1e12, 1e12), c = c(-1e12, NA, 0.1)),
  cbind(x = c(2.5, NA, 7))
)
case <- function(m) {
  k <- ncol(m)
  present <- rowSums(!is.na(m))
  means_k <- function(kk) ifelse(present >= kk, rowMeans(m, na.rm = TRUE), NA)
  list(
    columns = lapply(colnames(m), function(n) list(name = n, values = as.list(dec(m[, n])))),
    row_sum = as.list(dec(ifelse(present == 0, NA, rowSums(m, na.rm = TRUE)))),
    row_sum_r = as.list(dec(rowSums(m, na.rm = TRUE))),
    row_mean = as.list(dec(rowMeans(m, na.rm = TRUE))),
    row_mean_min = lapply(seq_len(k), function(kk) list(min_valid = kk, values = as.list(dec(means_k(kk))))),
    # The count of missing values in each row (release 0.9.0, step 31c; row(vars, stat: n_missing)).
    row_n_missing = as.list(as.integer(rowSums(is.na(m))))
  )
}

# The count of missing values across variables of several types (open decision #166):
# rowSums(is.na()) on a data frame with numbers, text, logical values and dates, over all of its
# variables, over two of them and over one; a row with nothing missing and one with everything.
mixed <- data.frame(
  q1 = c(5L, 4L, NA, NA, 2L, NA),
  q2 = c(3.5, NA, NA, 1.0, 0.5, NA),
  name = c("a", NA, "c", NA, "", NA),
  ok = c(TRUE, NA, FALSE, TRUE, NA, NA),
  day = as.Date(c("2026-01-02", NA, NA, "2026-01-05", "2026-01-06", NA)),
  stringsAsFactors = FALSE
)
written <- function(x) as.list(ifelse(is.na(x), "NA", as.character(x)))
counts <- function(vars) as.list(as.integer(rowSums(is.na(mixed[vars]))))
mixed_case <- list(
  q1 = written(mixed$q1), q2 = written(mixed$q2), name = written(mixed$name),
  ok = as.list(ifelse(is.na(mixed$ok), "NA", ifelse(mixed$ok, "true", "false"))),
  day = written(mixed$day),
  all = counts(c("q1", "q2", "name", "ok", "day")),
  numbers = counts(c("q1", "q2")),
  text_and_dates = counts(c("name", "day")),
  one = counts("ok")
)

# The shipped wages: the missing values among wage, hours and industry in each of its 18,412
# rows, as the number of rows with 0, 1, 2 and 3 of them missing and the sum of the counts.
root <- normalizePath(file.path(here(), "..", "..", ".."))
wages <- read_dataset(file.path(root, "packages", "tier1", "datasets", "data", "wages.csv"))
gaps <- rowSums(is.na(wages[c("wage", "hours", "industry")]))
shipped <- list(
  file = "packages/tier1/datasets/data/wages.csv", variables = list("wage", "hours", "industry"),
  rows = nrow(wages), sum = as.integer(sum(gaps)),
  rows_with = lapply(0:3, function(k) as.integer(sum(gaps == k))),
  # Over every variable of the file.
  sum_all = as.integer(sum(is.na(wages))), complete = as.integer(sum(complete.cases(wages)))
)

write_json(list(
  reference = reference_block(),
  cases = lapply(matrices, case),
  n_missing = list(mixed = mixed_case, wages = shipped)
), file.path(here(), "rows.json"))
