# R's values of the descriptive reducers and the nine quantile definitions (release 0.2.0, step
# 10) on the datasets of tests/reference/descriptive.py (written as CSV in ../data/) and on the
# numeric columns of tests/datasets/wages_large.csv. Run with `Rscript tests/reference/r/descriptive.R`.
source(file.path(dirname(sub("^--file=", "", commandArgs(FALSE)[grepl("^--file=", commandArgs(FALSE))])), "common.R"))

probs <- c(0, 0.05, 0.1, 0.25, 0.5, 0.75, 0.9, 0.95, 1)

column_reference <- function(x) {
  present <- x[!is.na(x)]
  stats <- list(
    n = length(present),
    missing = sum(is.na(x)),
    sum = num(sum(present)),
    mean = num(if (length(present) > 0) mean(present) else NA),
    variance = num(if (length(present) > 1) var(present) else NA),
    sd = num(if (length(present) > 1) sd(present) else NA),
    median = num(if (length(present) > 0) median(present) else NA),
    min = num(if (length(present) > 0) min(present) else NA),
    max = num(if (length(present) > 0) max(present) else NA),
    n_distinct = length(unique(present))
  )
  quantiles <- list(probs = as.list(num(probs)))
  for (type in 1:9) {
    q <- if (length(present) > 0) quantile(present, probs = probs, type = type, names = FALSE) else rep(NA, length(probs))
    quantiles[[paste0("type", type)]] <- as.list(num(q))
  }
  stats$quantiles <- quantiles
  stats
}

root <- normalizePath(file.path(here(), "..", "..", ".."))
datasets <- list()
for (name in c("small", "offset", "wide", "constant", "two")) {
  file <- file.path("tests", "reference", "data", paste0(name, ".csv"))
  x <- read_dataset(file.path(root, file))$x
  datasets[[name]] <- list(file = file, columns = list(x = column_reference(x)))
}
wages <- read_dataset(file.path(root, "tests", "datasets", "wages_large.csv"))
datasets[["wages_large"]] <- list(
  file = "tests/datasets/wages_large.csv",
  columns = list(
    wage = column_reference(wages$wage),
    hours = column_reference(wages$hours),
    educ = column_reference(wages$educ)
  )
)

# Weighted statistics (the owner's decision of 19 October 2026): on wages_large, the weights are
# educ (whole numbers, so they serve as frequency weights too). Frequency weights replicate rows:
# N = sum(w), variance divisor sum(w) - 1. Analytic and sampling weights give the mean
# weighted.mean(x, w) and the variance with the weights scaled to sum to n, divisor n - 1, as
# Stata's summarize [aweight]; the weighted sum is sum(w * x) (for analytic weights, scaled by
# n / sum(w)); the weighted quantile is the smallest value whose cumulative weight reaches p of the
# total. Rows with NA in the value are left out, as the reducers skip them.
weighted_reference <- function(x, w) {
  keep <- !is.na(x) & !is.na(w)
  x <- x[keep]
  w <- w[keep]
  n <- length(x)
  m <- weighted.mean(x, w)
  freq_var <- sum(w * (x - m)^2) / (sum(w) - 1)
  scaled <- w * n / sum(w)
  aw_var <- sum(scaled * (x - m)^2) / (n - 1)
  o <- order(x, method = "radix")
  cw <- cumsum(w[o])
  wq <- function(p) x[o][which(cw >= p * sum(w))[1]]
  list(
    n = n,
    weight_sum = num(sum(w)),
    mean = num(m),
    variance_frequency = num(freq_var),
    sd_frequency = num(sqrt(freq_var)),
    variance_analytic = num(aw_var),
    sd_analytic = num(sqrt(aw_var)),
    sum_frequency = num(sum(w * x)),
    sum_analytic = num(sum(w * x) * n / sum(w)),
    median = num(wq(0.5)),
    quantiles = list(probs = as.list(num(probs)), values = as.list(num(sapply(probs, wq))))
  )
}
datasets[["wages_large"]]$weighted <- list(
  weights = "educ",
  wage = weighted_reference(wages$wage, wages$educ),
  hours = weighted_reference(wages$hours, wages$educ)
)

write_json(list(reference = reference_block(), datasets = datasets), file.path(here(), "descriptive.json"))
