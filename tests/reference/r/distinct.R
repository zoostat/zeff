# R's answers for distinct and duplicates (release 0.6.0, step 11) on the shipped wages (the
# workers numbered to 50) and on a small frame with NA among the keys: the rows !duplicated() keeps, as 1-based positions in
# the current order (R's duplicated treats NA as equal to NA, the identity relation of section
# 3.7), and the table of repeated key values with their counts, in key order (radix, NA last, text
# by code point), one row per key value with more than one copy. The keys are whole-number and
# text columns, whose values print the same in R and in Zeff. The engine is held to these exactly.
# Run with `Rscript tests/reference/r/distinct.R`.
source(file.path(dirname(sub("^--file=", "", commandArgs(FALSE)[grepl("^--file=", commandArgs(FALSE))])), "common.R"))

root <- normalizePath(file.path(here(), "..", "..", ".."))
cell <- function(x) ifelse(is.na(x), "NA", as.character(x))

reference_for <- function(d, keys) {
  kept <- which(!duplicated(d[keys]))
  key_text <- do.call(paste, c(lapply(d[keys], cell), sep = "\r"))
  first <- !duplicated(key_text)
  copies <- as.vector(table(key_text)[key_text[first]])
  u <- d[first, keys, drop = FALSE]
  u$copies <- copies
  u <- u[u$copies > 1, , drop = FALSE]
  o <- do.call(order, c(lapply(u[keys], function(k) k), list(na.last = TRUE, method = "radix")))
  u <- u[o, , drop = FALSE]
  list(keys = as.list(keys), kept = as.list(kept),
       repeated = lapply(seq_len(nrow(u)), function(i) list(key = as.list(unname(cell(unlist(u[i, keys])))), copies = u$copies[i])))
}

wages <- read_dataset(file.path(root, "packages", "tier1", "datasets", "data", "wages.csv"))
wages <- wages[wages$worker <= 50, ]
key_sets <- list("worker", c("worker", "year"), "industry", c("industry", "year"), "year")
wages_cases <- lapply(key_sets, function(keys) reference_for(wages, keys))

small <- data.frame(a = c(1, NA, 1, NA, 2, 1), b = c("x", "y", "x", "y", "x", "y"), stringsAsFactors = FALSE)
small_cases <- list(reference_for(small, "a"), reference_for(small, c("a", "b")), reference_for(small, "b"))

out <- list(reference = reference_block(),
            wages = wages_cases,
            small = list(a = as.list(cell(small$a)), b = as.list(small$b), cases = small_cases))
write_json(out, file.path(here(), "distinct.json"))
