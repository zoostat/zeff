# R's answers for append, to_long and to_wide (release 0.6.0, step 14): rbind with the fill of
# missing columns written out, reshape(direction = "long") with the rows put in Zeff's order
# (each original row's stub columns in column order, as tidyr's pivot_longer lists them) and
# reshape(direction = "wide") with the ids in first-appearance order and the new columns in the
# sorted order of the names, on the sample script's sales_wide shape and on generated frames
# with an absent combination. Values are written as text; the engine is held to them exactly.
# Run with `Rscript tests/reference/r/reshape.R`.
source(file.path(dirname(sub("^--file=", "", commandArgs(FALSE)[grepl("^--file=", commandArgs(FALSE))])), "common.R"))

cell <- function(x) ifelse(is.na(x), "NA", as.character(x))
# A frame as its column names in order, then its columns, since a JSON reader may sort the keys.
columns_of <- function(d) list(names = as.list(names(d)), values = lapply(d, function(c) as.list(cell(c))))

# The sales_wide shape of the sample script: one row per firm, sales by year across the columns.
set.seed(20261104)
sales_wide <- data.frame(firm_id = 1:6, region = c("North", "South", "North", "East", "South", "East"),
                         sales_2010 = round(runif(6, 50, 150), 1), sales_2011 = round(runif(6, 50, 150), 1),
                         sales_2012 = round(runif(6, 50, 150), 1), sales_2013 = round(runif(6, 50, 150), 1),
                         stringsAsFactors = FALSE)
sales_wide$sales_2012[3] <- NA

long_of <- function(wide, stub, id, others, names_to, values_to) {
  stubs <- grep(paste0("^", stub), names(wide), value = TRUE)
  rows <- lapply(seq_len(nrow(wide)), function(i) {
    lapply(stubs, function(s) {
      r <- wide[i, c(id, others), drop = FALSE]
      r[[names_to]] <- as.integer(sub(paste0("^", stub), "", s))
      r[[values_to]] <- wide[i, s]
      r
    })
  })
  out <- do.call(rbind, unlist(rows, recursive = FALSE))
  rownames(out) <- NULL
  # The same rows as base R's reshape gives, checked here, in Zeff's order.
  r <- reshape(wide, direction = "long", varying = stubs, sep = "_", idvar = id, timevar = names_to, v.names = values_to)
  r <- r[order(match(r[[id]], wide[[id]]), r[[names_to]]), c(id, others, names_to, values_to)]
  stopifnot(all(r[[values_to]] == out[[values_to]], na.rm = TRUE), all(is.na(r[[values_to]]) == is.na(out[[values_to]])))
  out
}
sales_long <- long_of(sales_wide, "sales_", "firm_id", "region", "year", "sales")

wide_of <- function(long, values_from, id, names_from) {
  ids <- unique(long[[id]])
  names <- sort(unique(long[[names_from]]))
  out <- data.frame(id = ids)
  names(out) <- id
  for (nm in names) {
    column <- rep(NA, length(ids))
    for (i in seq_along(ids)) {
      hit <- which(long[[id]] == ids[i] & long[[names_from]] == nm)
      if (length(hit) == 1) column[i] <- long[[values_from]][hit]
    }
    out[[paste0(values_from, "_", nm)]] <- column
  }
  # Checked against base R's reshape wide, which sorts nothing and keeps the ids in order of first appearance.
  r <- reshape(long[, c(id, names_from, values_from)], direction = "wide", idvar = id, timevar = names_from, sep = "_")
  for (nm in names) stopifnot(all(r[[paste0(values_from, "_", nm)]] == out[[paste0(values_from, "_", nm)]], na.rm = TRUE))
  out
}
sales_back <- wide_of(sales_long, "sales", "firm_id", "year")

# A generated long frame with an absent combination and ids out of order.
long_gaps <- data.frame(unit = c(3, 1, 3, 2, 1, 2), period = c(2, 1, 1, 2, 2, 1), y = c(0.5, 1.25, -2, 7, 3.5, NA), stringsAsFactors = FALSE)
long_gaps <- long_gaps[-4, ]
wide_gaps <- wide_of(long_gaps, "y", "unit", "period")

# append: the right frame lacks `region` and brings `founded`; Int meets Float in `sales_2010`.
left <- sales_wide[1:3, c("firm_id", "region", "sales_2010")]
right <- data.frame(firm_id = 7:8, sales_2010 = c(99L, 101L), founded = c("1990-01-05", "2001-11-30"), stringsAsFactors = FALSE)
all_columns <- union(names(left), names(right))
fill <- function(d) { for (c in setdiff(all_columns, names(d))) d[[c]] <- NA; d[all_columns] }
stacked <- rbind(fill(left), fill(right))

write_json(list(reference = reference_block(),
                sales_wide = columns_of(sales_wide), sales_long = columns_of(sales_long), sales_back = columns_of(sales_back),
                long_gaps = columns_of(long_gaps), wide_gaps = columns_of(wide_gaps),
                append = list(left = columns_of(left), right = columns_of(right), stacked = columns_of(stacked))),
           file.path(here(), "reshape.json"))
