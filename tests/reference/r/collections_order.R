# R's values for the order functions of release 0.5.0, step 12: unique(), sort(na.last = TRUE)
# both ways, %in%, names() and identical() on fixed lists, with NA among the values. Where R's
# handling of NA matches Zeff's identity relation the case is compared; where it departs the
# Rust test names the departure (R's %in% and unique() treat NA as one value, as Zeff does; R's
# sort() drops NA unless na.last is set, so it is set; identical(-0, 0) is TRUE in R as in Zeff).
# Every value is exact, so the comparison is exact. Run with
# `Rscript tests/reference/r/collections_order.R`.
source(file.path(dirname(sub("^--file=", "", commandArgs(FALSE)[grepl("^--file=", commandArgs(FALSE))])), "common.R"))

lists <- list(
  c(3, 1, 3, 2, 1),
  c(3, NA, 1, 3, 2, NA),
  c(NA, NA),
  c(1.5, -2.25, 1e12, 0.5, 1e12),
  c(-0, 0, 1, -0),
  c(7),
  numeric(0),
  c(1e-8, 1e7, -1e12, 0, 2.5, 1e7)
)
probes <- c(3, 5, NA, 0, 1e12)
strings <- list(
  c("b", "a", "b", "B", "é", "a"),
  c("pear", "apple", "fig", "apple")
)
Sys.setlocale("LC_COLLATE", "C")

num_case <- function(x) list(
  items = as.list(num(x)),
  unique = as.list(num(unique(x))),
  sorted = as.list(num(sort(x, na.last = TRUE))),
  sorted_desc = as.list(num(sort(x, decreasing = TRUE, na.last = TRUE))),
  is_in = as.list(ifelse(probes %in% x, "true", "false")),
  identical_self = identical(x, x)
)
str_case <- function(s) list(
  items = as.list(s),
  unique = as.list(unique(s)),
  sorted = as.list(sort(s)),
  sorted_desc = as.list(sort(s, decreasing = TRUE))
)

named <- list(mu = 0, sd = 1, share = 0.5)
pairs <- list(
  list(a = c(1, 2, NA), b = c(1, 2, NA)),
  list(a = c(1, 2), b = c(2, 1)),
  list(a = c(-0), b = c(0)),
  list(a = c(1), b = c(1, 1))
)

write_json(list(
  reference = reference_block(),
  probes = as.list(num(probes)),
  lists = lapply(lists, num_case),
  strings = lapply(strings, str_case),
  record_keys = as.list(names(named)),
  identical_pairs = lapply(pairs, function(p) list(a = as.list(num(p$a)), b = as.list(num(p$b)), identical = identical(p$a, p$b)))
), file.path(here(), "collections_order.json"))
