# R's na.omit() on the subset of variables (release 0.5.0, step 18): the rows of a data frame
# with no NA in the variables named, in their order, and how many went. Every value is exact.
# Run with `Rscript tests/reference/r/drop_missing.R`.
source(file.path(dirname(sub("^--file=", "", commandArgs(FALSE)[grepl("^--file=", commandArgs(FALSE))])), "common.R"))

frames <- list(
  data.frame(id = 1:4, x = c(1, NA, 3, 4), y = c(NA, 2, 3, NA), z = c(1, 1, 1, 1)),
  data.frame(id = 1:3, x = c(1, 2, 3), y = c(4, 5, 6)),
  data.frame(id = 1:3, x = c(NA, NA, NA), y = c(1, 2, 3)),
  data.frame(id = 1:5, x = c(1e12, NA, 1e-8, 0, NA), y = c(NA, 1, 2, 3, 4), z = c(1, 2, 3, 4, 5))
)
subsets <- list(c("x", "y"), c("x"), c("y"), c("id", "x", "y"))
cases <- list()
for (f in frames) for (vars in subsets) {
  present <- intersect(vars, names(f))
  if (length(present) == 0) next
  kept <- f[complete.cases(f[, present, drop = FALSE]), , drop = FALSE]
  cases[[length(cases) + 1]] <- list(
    columns = lapply(names(f), function(n) list(name = n, values = as.list(num(f[[n]])))),
    vars = as.list(present),
    kept_ids = as.list(as.integer(kept$id)),
    dropped = nrow(f) - nrow(kept)
  )
}
write_json(list(reference = reference_block(), cases = cases), file.path(here(), "drop_missing.json"))
