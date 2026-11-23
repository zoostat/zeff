# R's values for row_number (release 0.9.0, step 31b; open decision #167), in base R: the
# position of each row, seq_len(nrow), and its position within its group in the order the rows
# stand, ave(seq_along(g), g, FUN = seq_along). A missing value of the grouping variable is a
# group of its own, as every grouped verb of the engine has it, where ave() would leave such a row
# out of every group: the missing value is written as a level before the groups are formed.
#
# The cases: no groups; groups of whole numbers in runs and interleaved; text groups; a missing
# group; two grouping variables; one group; every row a group of its own; an empty frame; rows
# sorted by another variable before they are numbered. Then the shipped wages, 18,412 rows: the
# row number within each worker, in the file's order and after sorting by year from the latest,
# as its sum, its largest value, the rows numbered 1 (one for each worker) and the values of the
# first forty rows. Every value is a whole number, so the comparison is exact.
# Run with `Rscript tests/reference/r/row_number.R`.
source(file.path(dirname(sub("^--file=", "", commandArgs(FALSE)[grepl("^--file=", commandArgs(FALSE))])), "common.R"))

key <- function(g) ifelse(is.na(g), "\001NA", as.character(g))
within <- function(...) {
  keys <- lapply(list(...), key)
  n <- length(keys[[1]])
  if (n == 0) return(integer(0))
  as.integer(ave(seq_len(n), do.call(paste, c(keys, sep = "\002")), FUN = seq_along))
}
written <- function(g) as.list(ifelse(is.na(g), "NA", as.character(g)))

one_key <- function(g, text = is.character(g)) {
  list(g = written(g), text_groups = text, rows = length(g),
       whole = as.list(seq_len(length(g))), by = as.list(within(g)))
}

cases <- list(
  runs = one_key(c(1L, 1L, 1L, 2L, 2L, 3L)),
  interleaved = one_key(c(2L, 1L, 2L, 3L, 1L, 2L, 3L, 3L, 1L, 2L)),
  text = one_key(c("north", "south", "east", "north", "east", "north", "south")),
  missing_group = one_key(c(1L, NA, 2L, NA, 1L, 2L, NA)),
  missing_text_group = one_key(c("a", NA, "b", "a", NA)),
  one_group = one_key(rep(7L, 5)),
  each_its_own = one_key(c(5L, 3L, 9L, 1L)),
  one_row = one_key(4L),
  empty = one_key(integer(0))
)

# Two grouping variables.
g1 <- c("a", "a", "b", "a", "b", "b", "a", "a")
g2 <- c(1L, 2L, 1L, 1L, 1L, 2L, 2L, 1L)
two <- list(g1 = written(g1), g2 = written(g2), by = as.list(within(g1, g2)))

# Sorted by another variable first: the order the rows then stand in decides the numbers.
patient <- c("a", "a", "b", "a", "b")
day <- c(3L, 1L, 2L, 9L, 5L)
o <- order(day)
sorted <- list(patient = written(patient), day = as.list(day),
               patient_sorted = written(patient[o]), day_sorted = as.list(day[o]),
               by = as.list(within(patient[o])))

# The shipped wages.
root <- normalizePath(file.path(here(), "..", "..", ".."))
wages <- read_dataset(file.path(root, "packages", "tier1", "datasets", "data", "wages.csv"))
k <- within(wages$worker)
latest_first <- order(-wages$year, seq_len(nrow(wages)))
k_latest <- within(wages$worker[latest_first])
by_industry <- within(wages$industry)
shipped <- list(
  file = "packages/tier1/datasets/data/wages.csv", rows = nrow(wages),
  by_worker = list(sum = as.integer(sum(k)), largest = max(k), ones = as.integer(sum(k == 1L)),
                   first = as.list(k[1:40])),
  by_worker_latest_first = list(sum = as.integer(sum(k_latest)), largest = max(k_latest),
                                first = as.list(k_latest[1:40]),
                                # The year of the row numbered 1 of each worker is that worker's latest.
                                years_of_ones = as.integer(sum(wages$year[latest_first][k_latest == 1L]))),
  # A text variable with missing values among its groups.
  by_industry = list(sum = num(sum(as.numeric(by_industry))), largest = max(by_industry),
                     ones = as.integer(sum(by_industry == 1L)), first = as.list(by_industry[1:40])),
  whole = list(sum = num(sum(as.numeric(seq_len(nrow(wages))))), last = nrow(wages))
)

write_json(list(reference = reference_block(), cases = cases, two = two, sorted = sorted, wages = shipped),
           file.path(here(), "row_number.json"))
