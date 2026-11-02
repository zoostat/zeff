# R's answers for merge (release 0.6.0, step 13): the pairs of row positions (left, right; NA
# where a side has no row) a join gives, in Zeff's order (the left frame's order, then the right
# frame's within one left row; a full join's unmatched right rows last, in the right order), on
# the shipped wages with firms on firm under the three joins, and on generated pairs with every
# relation, NA keys on both sides and two keys. Base R's merge(sort = FALSE) does not promise an
# order and matches NA with NA, so the keys are joined on the rows without NA, the NA rows added
# back as unmatched, and the pairs ordered as Zeff orders them. The engine is held to the pairs
# exactly. Run with `Rscript tests/reference/r/merge.R`.
source(file.path(dirname(sub("^--file=", "", commandArgs(FALSE)[grepl("^--file=", commandArgs(FALSE))])), "common.R"))

root <- normalizePath(file.path(here(), "..", "..", ".."))

pairs_of <- function(x, y, keys, join) {
  x$li <- seq_len(nrow(x)); y$ri <- seq_len(nrow(y))
  ok_x <- stats::complete.cases(x[keys]); ok_y <- stats::complete.cases(y[keys])
  m <- merge(x[ok_x, c(keys, "li"), drop = FALSE], y[ok_y, c(keys, "ri"), drop = FALSE], by = keys, all = FALSE, sort = FALSE)
  m <- m[order(m$li, m$ri), c("li", "ri")]
  # The rows of each side that no row of the other matched, whatever the join keeps.
  left_unmatched <- length(setdiff(x$li, m$li))
  right_unmatched <- length(setdiff(y$ri, m$ri))
  if (join %in% c("left", "full")) {
    unmatched <- setdiff(x$li, m$li)
    if (length(unmatched) > 0) m <- rbind(m, data.frame(li = unmatched, ri = rep(NA_integer_, length(unmatched))))
    m <- m[order(m$li, m$ri, na.last = TRUE), ]
  }
  if (join == "full") {
    unmatched <- setdiff(y$ri, m$ri)
    if (length(unmatched) > 0) m <- rbind(m, data.frame(li = rep(NA_integer_, length(unmatched)), ri = sort(unmatched)))
  }
  list(join = join, keys = as.list(keys), matched = sum(!is.na(m$li) & !is.na(m$ri)),
       left_unmatched = left_unmatched, right_unmatched = right_unmatched,
       li = as.list(ifelse(is.na(m$li), "NA", as.character(m$li))), ri = as.list(ifelse(is.na(m$ri), "NA", as.character(m$ri))))
}

wages <- read_dataset(file.path(root, "packages", "tier1", "datasets", "data", "wages.csv"))
firms <- read_dataset(file.path(root, "packages", "tier1", "datasets", "data", "firms.csv"))
shipped <- lapply(c("left", "inner", "full"), function(j) pairs_of(wages, firms, "firm", j))

set.seed(20261103)
gen <- function(n, keys_from, with_na) {
  d <- data.frame(k = sample(keys_from, n, replace = TRUE), g = sample(c("a", "b"), n, replace = TRUE), v = round(rnorm(n), 2), stringsAsFactors = FALSE)
  if (with_na) d$k[sample(n, max(1, n %/% 5))] <- NA
  d
}
cases <- list()
add_case <- function(name, x, y, keys) {
  for (j in c("left", "inner", "full")) {
    p <- pairs_of(x, y, keys, j)
    p$name <- name
    p$x <- list(k = as.list(ifelse(is.na(x$k), "NA", as.character(x$k))), g = as.list(x$g))
    p$y <- list(k = as.list(ifelse(is.na(y$k), "NA", as.character(y$k))), g = as.list(y$g))
    cases[[length(cases) + 1]] <<- p
  }
}
add_case("many_to_one", gen(25, 1:8, FALSE), data.frame(k = 1:10, g = sample(c("a", "b"), 10, replace = TRUE), stringsAsFactors = FALSE), "k")
add_case("one_to_many", data.frame(k = 1:10, g = sample(c("a", "b"), 10, replace = TRUE), stringsAsFactors = FALSE), gen(25, 1:8, FALSE), "k")
add_case("one_to_one", data.frame(k = c(3, 1, 4, 7), g = c("a", "b", "a", "b"), stringsAsFactors = FALSE), data.frame(k = c(1, 4, 9), g = c("b", "a", "a"), stringsAsFactors = FALSE), "k")
# NA keys on both sides, the right side's present keys distinct, since a key repeated on both
# sides is a many-to-many merge the engine refuses.
add_case("na_keys", gen(20, 1:6, TRUE), data.frame(k = c(1, NA, 3, 4, NA, 6, 8), g = c("a", "b", "a", "b", "a", "b", "a"), stringsAsFactors = FALSE), "k")
add_case("two_keys", gen(30, 1:4, FALSE), {
  y <- expand.grid(k = 1:4, g = c("a", "b"), stringsAsFactors = FALSE); y$v <- seq_len(nrow(y)); y
}, c("k", "g"))
add_case("no_match", data.frame(k = 1:5, g = rep("a", 5), stringsAsFactors = FALSE), data.frame(k = 11:13, g = rep("b", 3), stringsAsFactors = FALSE), "k")

write_json(list(reference = reference_block(), shipped = shipped, cases = cases), file.path(here(), "merge.json"))
