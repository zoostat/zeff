# R's panel lags, leads, differences and fill forward on a generated unbalanced panel with gaps
# and missing values, and on a slice of the shipped wages (release 0.6.0, step 8): the previous
# and next values matched on the unit and the exact period (`time - 1`, `time + 1`, `time - 2`),
# so that a gap gives NA as Stata's L. and F. do and unlike dplyr's lag, which takes the previous
# row; the difference as a double subtraction; and the last value present carried forward within
# the unit in time order, written out as a loop. The rows are shuffled, so the row order plays no
# part. The engine is held to these values exactly, so the difference, the one value computed
# rather than copied, is written with 17 significant digits, which name a double uniquely; 15
# would round the last bit of a subtraction away. Run with `Rscript tests/reference/r/panel.R`.
source(file.path(dirname(sub("^--file=", "", commandArgs(FALSE)[grepl("^--file=", commandArgs(FALSE))])), "common.R"))

exact <- function(x) ifelse(is.na(x), "NA", sprintf("%.17g", x))

panel_outputs <- function(unit, time, x) {
  key <- paste(unit, time)
  shift <- function(k) x[match(paste(unit, time + k), key)]
  lag1 <- shift(-1)
  lead1 <- shift(1)
  lag2 <- shift(-2)
  diff1 <- x - lag1
  ord <- order(unit, time)
  filled <- x
  last <- NA
  previous <- NULL
  for (i in ord) {
    if (is.null(previous) || unit[i] != previous) {
      last <- NA
      previous <- unit[i]
    }
    if (is.na(filled[i])) filled[i] <- last else last <- filled[i]
  }
  list(lag1 = as.list(num(lag1)), lead1 = as.list(num(lead1)), lag2 = as.list(num(lag2)),
       diff1 = as.list(exact(diff1)), filled = as.list(num(filled)))
}

set.seed(20261101)
counts <- c(1L, 3L, 5L, 7L, 9L, 11L, 13L, 2L)
unit <- rep(seq_along(counts), times = counts)
time <- unlist(lapply(counts, function(n) sort(sample(2000:2014, n))))
x <- round(rnorm(length(unit)), 3)
x[sample(length(x), 10)] <- NA
shuffle <- sample(length(unit))
unit <- unit[shuffle]
time <- time[shuffle]
x <- x[shuffle]
generated <- c(list(unit = as.list(unit), time = as.list(time), x = as.list(num(x))), panel_outputs(unit, time, x))

wages <- read_dataset(file.path(here(), "..", "..", "..", "packages", "tier1", "datasets", "data", "wages.csv"))
wages <- wages[wages$worker <= 300, ]
slice <- c(list(worker = as.list(wages$worker), year = as.list(wages$year), wage = as.list(num(wages$wage))),
           panel_outputs(wages$worker, wages$year, wages$wage))

out <- list(reference = reference_block(), generated = generated, wages = slice)
write_json(out, file.path(here(), "panel.json"))
