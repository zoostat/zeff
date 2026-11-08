# R's answers for tabulate (release 0.7.0, step 8) on the shipped wages and on a small frame with
# NA among the values: table() with useNA = "ifany", so that NA is its own row or column as Zeff
# shows it, and prop.table() by row, by column and over the whole table, as percentages. The
# counts are whole numbers and are held exactly; the percentages are written with 15 significant
# digits and the engine's one-decimal cells are held to within half a unit of the last place.
# Levels are R's (text alphabetically, numbers in order); Zeff's test matches rows by their
# level, so the two orders need not agree. Run with `Rscript tests/reference/r/tabulate.R`.
source(file.path(dirname(sub("^--file=", "", commandArgs(FALSE)[grepl("^--file=", commandArgs(FALSE))])), "common.R"))

root <- normalizePath(file.path(here(), "..", "..", ".."))
level_text <- function(x) ifelse(is.na(x), "NA", as.character(x))
rows_of <- function(m) lapply(seq_len(nrow(m)), function(i) as.list(num(as.vector(m[i, ]))))
count_rows_of <- function(m) lapply(seq_len(nrow(m)), function(i) as.list(as.integer(m[i, ])))

one_way <- function(d, var) {
  t <- table(d[[var]], useNA = "ifany")
  list(var = var, levels = as.list(level_text(names(t))), counts = as.list(as.integer(t)),
       percent = as.list(num(100 * as.vector(prop.table(t)))))
}

two_way <- function(d, v1, v2) {
  t <- table(d[[v1]], d[[v2]], useNA = "ifany")
  list(vars = list(v1, v2),
       rows = as.list(level_text(dimnames(t)[[1]])), cols = as.list(level_text(dimnames(t)[[2]])),
       counts = count_rows_of(unclass(t)),
       row_percent = rows_of(100 * unclass(prop.table(t, 1))),
       col_percent = rows_of(100 * unclass(prop.table(t, 2))),
       total_percent = rows_of(100 * unclass(prop.table(t))))
}

wages <- read_dataset(file.path(root, "packages", "tier1", "datasets", "data", "wages.csv"))
wages_cases <- list(one_way(wages, "industry"), one_way(wages, "year"), one_way(wages, "educ"))
wages_two <- list(two_way(wages, "industry", "female"), two_way(wages, "educ", "union"))

small <- data.frame(a = c(1, NA, 1, 2, 2, 2, NA), b = c("x", "y", "x", "y", "x", NA, "y"), stringsAsFactors = FALSE)
small_cases <- list(one_way(small, "a"), one_way(small, "b"))
small_two <- list(two_way(small, "a", "b"), two_way(small, "b", "a"))

out <- list(reference = reference_block(),
            wages = list(one_way = wages_cases, two_way = wages_two),
            small = list(one_way = small_cases, two_way = small_two))
write_json(out, file.path(here(), "tabulate.json"))
