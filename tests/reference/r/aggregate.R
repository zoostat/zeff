# R's grouped statistics for aggregate (release 0.6.0, step 12): on the shipped wages by industry
# (as text, so that R and Zeff order the groups alike) and by industry and year, and on the
# Wooldridge wage1 by education, the reducers of the data package computed per group with NA
# removed as Zeff's reducers skip it: mean, median, sd, variance, sum, min, max, n, n_distinct,
# the ninetieth percentile (type 7), iqr, mad (constant 1.4826), and the adjusted skewness and
# excess kurtosis of the data package's bodies. The groups come in key order (radix, NA last).
# Values are written with 17 significant digits; the engine is held to the exact ones (n,
# n_distinct, min, max, median, mad) exactly and to the closed-form ones (the moments, the sum,
# the interpolated quantile and iqr) within a relative 1e-12. Run with `Rscript tests/reference/r/aggregate.R`.
source(file.path(dirname(sub("^--file=", "", commandArgs(FALSE)[grepl("^--file=", commandArgs(FALSE))])), "common.R"))

root <- normalizePath(file.path(here(), "..", "..", ".."))
dec <- function(x) ifelse(is.na(x), "NA", ifelse(is.infinite(x), ifelse(x > 0, "Inf", "-Inf"), sprintf("%.17g", x)))
cell <- function(x) ifelse(is.na(x), "NA", as.character(x))

skewness <- function(x) {
  x <- x[!is.na(x)]; n <- length(x)
  if (n < 3) return(NA)
  d <- x - mean(x)
  g1 <- mean(d^3) / mean(d^2)^1.5
  g1 * sqrt(n * (n - 1)) / (n - 2)
}
kurtosis <- function(x) {
  x <- x[!is.na(x)]; n <- length(x)
  if (n < 4) return(NA)
  d <- x - mean(x)
  g2 <- mean(d^4) / mean(d^2)^2 - 3
  ((n + 1) * g2 + 6) * (n - 1) / ((n - 2) * (n - 3))
}
stats_of <- function(v, distinct_of) {
  p <- v[!is.na(v)]
  one <- function(f) if (length(p) == 0) NA else f(p)
  list(mean = dec(one(mean)), median = dec(one(median)), sd = dec(if (length(p) < 2) NA else sd(p)),
       variance = dec(if (length(p) < 2) NA else var(p)), sum = dec(one(sum)), min = dec(one(min)), max = dec(one(max)),
       n = length(v), n_distinct = length(unique(distinct_of)),
       q90 = dec(one(function(z) quantile(z, 0.9, type = 7, names = FALSE))),
       iqr = dec(one(function(z) IQR(z, type = 7))), mad = dec(one(function(z) mad(z, constant = 1.4826))),
       skewness = dec(skewness(v)), kurtosis = dec(kurtosis(v)))
}
grouped <- function(d, keys, value, distinct_var) {
  key_text <- do.call(paste, c(lapply(d[keys], cell), sep = "\r"))
  u <- d[!duplicated(key_text), keys, drop = FALSE]
  o <- do.call(order, c(lapply(u[keys], function(k) k), list(na.last = TRUE, method = "radix")))
  u <- u[o, , drop = FALSE]
  u_text <- do.call(paste, c(lapply(u[keys], cell), sep = "\r"))
  groups <- lapply(seq_len(nrow(u)), function(i) {
    rows <- key_text == u_text[i]
    c(list(key = as.list(unname(cell(unlist(u[i, keys]))))), stats_of(d[[value]][rows], d[[distinct_var]][rows]))
  })
  list(keys = as.list(keys), value = value, distinct_of = distinct_var, groups = groups)
}

wages <- read_dataset(file.path(root, "packages", "tier1", "datasets", "data", "wages.csv"))
# wage1 from the wooldridge package in the harness's own library, the doubles the engine reads
# back from tests/reference/data/wage1.csv (written with 17 significant digits); R's own CSV reader
# is not correctly rounded in the last bit, so the package's values are the reference.
.libPaths(c(file.path(here(), "library"), .libPaths()))
library(wooldridge)
data(wage1)
cases <- list(
  grouped(wages, "industry", "wage", "year"),
  grouped(wages, c("industry", "year"), "hours", "worker"),
  grouped(wages, "year", "wage", "industry"),
  grouped(wage1, "educ", "wage", "exper"),
  grouped(wage1, c("female", "married"), "lwage", "educ")
)
write_json(list(reference = reference_block(), cases = cases), file.path(here(), "aggregate.json"))
