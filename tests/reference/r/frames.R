# R's answers for the frame verbs of release 0.2.0 (steps 15 and 16) on tests/datasets/wages.csv
# and wages_large.csv: the rows head and tail show, the rows where keeps, and the order sort gives
# (stable, NA last, text by code point as Zeff sorts it: R's radix method in the C locale). Row
# numbers are 1-based positions in the file. Run with `Rscript tests/reference/r/frames.R`.
source(file.path(dirname(sub("^--file=", "", commandArgs(FALSE)[grepl("^--file=", commandArgs(FALSE))])), "common.R"))

root <- normalizePath(file.path(here(), "..", "..", ".."))

frame_reference <- function(d, keys) {
  n <- nrow(d)
  out <- list(
    n_rows = n,
    n_cols = ncol(d),
    variables = as.list(names(d)),
    head_3 = as.list(seq_len(min(3, n))),
    tail_3 = as.list(if (n == 0) integer(0) else seq.int(max(1, n - 2), n)),
    head_10 = as.list(seq_len(min(10, n))),
    tail_10 = as.list(if (n == 0) integer(0) else seq.int(max(1, n - 9), n))
  )
  sorts <- list()
  for (key in keys) {
    for (direction in c("asc", "desc")) {
      # order() with method = "radix" is stable and sorts text by code point; NA rows come last
      # in both directions, as Zeff's sort puts them.
      o <- order(d[[key]], decreasing = (direction == "desc"), na.last = TRUE, method = "radix")
      sorts[[paste0(key, "_", direction)]] <- list(
        first_10 = as.list(as.integer(head(o, 10))),
        last_10 = as.list(as.integer(tail(o, 10)))
      )
    }
  }
  out$sort <- sorts
  out
}

wages <- read_dataset(file.path(root, "tests", "datasets", "wages.csv"))
large <- read_dataset(file.path(root, "tests", "datasets", "wages_large.csv"))

large_ref <- frame_reference(large, c("wage", "hours", "region", "educ"))
# Two keys: region by code point, then wage within the region, both ascending.
o <- order(large$region, large$wage, na.last = TRUE, method = "radix")
large_ref$sort[["region_then_wage_asc"]] <- list(first_10 = as.list(as.integer(head(o, 10))),
                                                 last_10 = as.list(as.integer(tail(o, 10))))
# where: rows whose condition is true; rows with NA in the condition are dropped and counted.
kept <- which(!is.na(large$hours) & large$hours > 40)
large_ref$where <- list(
  hours_over_40 = list(count = length(kept), first_10 = as.list(as.integer(head(kept, 10))),
                       na_dropped = sum(is.na(large$hours))),
  female = list(count = sum(large$female %in% c("true", TRUE)), na_dropped = 0L),
  educ_at_least_16_and_wage_under_20 = list(
    count = sum(large$educ >= 16 & large$wage < 20),
    first_10 = as.list(as.integer(head(which(large$educ >= 16 & large$wage < 20), 10))),
    na_dropped = 0L
  )
)

out <- list(
  reference = reference_block(),
  datasets = list(
    wages = c(list(file = "tests/datasets/wages.csv"), frame_reference(wages, c("wage", "hours", "name"))),
    wages_large = c(list(file = "tests/datasets/wages_large.csv"), large_ref)
  )
)
write_json(out, file.path(here(), "frames.json"))
