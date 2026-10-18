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

write_json(list(reference = reference_block(), datasets = datasets), file.path(here(), "descriptive.json"))
