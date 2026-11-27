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

# The semi and the anti join (release 0.10.0, step 7): the left rows that have a match in the
# right frame, and those that have none, each once and in the left order, with no column of the
# right frame; an NA key matches nothing, so its row is among those without a match. Worked out
# twice in base R and the two held to each other: from the inner pairs above, and from the keys
# pasted together and looked up with %in%, which is dplyr's semi_join and anti_join with
# na_matches = "never". Keys that repeat in both frames are allowed here, since no row is
# repeated, so a many-to-many pair is among the cases; it is generated after every case above, so
# that those keep their random numbers.
kept_by <- function(x, y, keys, join) {
  x$li <- seq_len(nrow(x)); y$ri <- seq_len(nrow(y))
  ok_x <- stats::complete.cases(x[keys]); ok_y <- stats::complete.cases(y[keys])
  m <- merge(x[ok_x, c(keys, "li"), drop = FALSE], y[ok_y, c(keys, "ri"), drop = FALSE], by = keys, all = FALSE, sort = FALSE)
  with_match <- sort(unique(m$li))
  pasted <- function(d) do.call(paste, c(lapply(d[keys], as.character), sep = "\r"))
  second <- which(ok_x & pasted(x) %in% pasted(y)[ok_y])
  stopifnot(identical(as.integer(with_match), as.integer(second)))
  kept <- if (join == "semi") with_match else setdiff(x$li, with_match)
  list(join = join, keys = as.list(keys), with_match = length(with_match),
       without_match = nrow(x) - length(with_match), right_unmatched = length(setdiff(y$ri, m$ri)),
       li = as.list(as.integer(kept)))
}
filtering_shipped <- lapply(c("semi", "anti"), function(j) kept_by(wages, firms, "firm", j))
filtering_reversed <- lapply(c("semi", "anti"), function(j) kept_by(firms, wages, "firm", j))
filtering <- list()
for (case in cases[vapply(cases, function(c) c$join == "left", TRUE)]) {
  x <- data.frame(k = suppressWarnings(as.numeric(unlist(case$x$k))), g = unlist(case$x$g), stringsAsFactors = FALSE)
  y <- data.frame(k = suppressWarnings(as.numeric(unlist(case$y$k))), g = unlist(case$y$g), stringsAsFactors = FALSE)
  for (j in c("semi", "anti")) {
    p <- kept_by(x, y, unlist(case$keys), j)
    p$name <- case$name; p$x <- case$x; p$y <- case$y
    filtering[[length(filtering) + 1]] <- p
  }
}
many <- list(x = gen(30, 1:6, TRUE), y = gen(18, 3:9, TRUE))
for (j in c("semi", "anti")) {
  p <- kept_by(many$x, many$y, "k", j)
  p$name <- "many_to_many"
  p$x <- list(k = as.list(ifelse(is.na(many$x$k), "NA", as.character(many$x$k))), g = as.list(many$x$g))
  p$y <- list(k = as.list(ifelse(is.na(many$y$k), "NA", as.character(many$y$k))), g = as.list(many$y$g))
  filtering[[length(filtering) + 1]] <- p
}

write_json(list(reference = reference_block(), shipped = shipped, cases = cases,
                filtering_shipped = filtering_shipped, filtering_reversed = filtering_reversed, filtering = filtering),
           file.path(here(), "merge.json"))
