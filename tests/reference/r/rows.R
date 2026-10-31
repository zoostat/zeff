# R's rowMeans() and rowSums() with na.rm = TRUE (release 0.5.0, step 17), and the min_valid rule
# written out in base R: a row's mean when at least k values are present, NA otherwise. The
# values are written with 17 significant digits and compared within a relative 1e-12 (R
# accumulates row sums in long double on x86, Zeff with a compensated sum, so the last bits may
# differ). Where Zeff departs from R the test names it: rowSums(na.rm = TRUE) of a row with no
# value is 0 in R and NA in Zeff. Run with `Rscript tests/reference/r/rows.R`.
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
    row_mean_min = lapply(seq_len(k), function(kk) list(min_valid = kk, values = as.list(dec(means_k(kk)))))
  )
}

write_json(list(
  reference = reference_block(),
  cases = lapply(matrices, case)
), file.path(here(), "rows.json"))
