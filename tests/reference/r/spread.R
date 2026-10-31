# R's IQR() under the nine quantile types and mad() under several constants (release 0.5.0, step
# 19), on fixed vectors with NA among the values and values to 1e12, written with 17 significant
# digits and compared within a relative 1e-12. IQR(type = t) is quantile(type = t) at 0.75 less
# at 0.25, and mad(constant = c) is c * median(|x - median(x)|), with NA removed, as Zeff's
# bodies compute them on the quantile and median of the data package. Run with
# `Rscript tests/reference/r/spread.R`.
source(file.path(dirname(sub("^--file=", "", commandArgs(FALSE)[grepl("^--file=", commandArgs(FALSE))])), "common.R"))

dec <- function(x) ifelse(is.na(x), "NA", ifelse(is.infinite(x), ifelse(x > 0, "Inf", "-Inf"), sprintf("%.17g", x)))
vectors <- list(
  c(1, 2, 4, 7, NA, 11),
  c(2.5, 3.5, 1e12, 7.25, -3, 0.5, 1e7, 12),
  c(1e-8, 2e-8, 3e-8, NA, 5e-8),
  c(5),
  c(1, 3),
  c(4, 4, 4, 4),
  c(-1e12, 1e12, 0, 3, 7, 9, 11, 13, 15, 17)
)
case <- function(x) list(
  items = as.list(dec(x)),
  iqr = lapply(1:9, function(t) list(type = t, value = dec(IQR(x, na.rm = TRUE, type = t)))),
  mad = lapply(c(1.4826, 1, 2.5), function(k) list(constant = dec(k), value = dec(mad(x, na.rm = TRUE, constant = k))))
)
write_json(list(reference = reference_block(), cases = lapply(vectors, case)), file.path(here(), "spread.json"))
