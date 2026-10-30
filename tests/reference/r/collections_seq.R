# R's seq(from, to, by) and its any()/all() for release 0.5.0, step 13. The values of seq are
# written with 17 significant digits so that the Rust test reads each back as the exact double
# and compares bit for bit: Zeff follows seq.default, n = trunc((to - from)/by + 1e-10),
# x = from + (0:n) * by, those past `to` brought back to it. The grid has fractional steps and
# ends from 1e7 to 1e12. Zeff departs from R in two places the Rust test names: seq(5, 1) is an
# error (the step 1 of the catalogue, where R counts down), and a zero step is always an error
# (R returns `from` for seq(1, 1, by = 0)). Run with `Rscript tests/reference/r/collections_seq.R`.
source(file.path(dirname(sub("^--file=", "", commandArgs(FALSE)[grepl("^--file=", commandArgs(FALSE))])), "common.R"))

dec <- function(x) sprintf("%.17g", x)
cases <- list()
add_case <- function(from, to, by) {
  x <- seq(from, to, by = by)
  cases[[length(cases) + 1]] <<- list(
    from = dec(from), to = dec(to), by = dec(by),
    n = length(x), values = as.list(dec(x))
  )
}

# Working grid: the steps that do not divide their range exactly.
for (by in c(0.1, 0.2, 0.25, 0.3, 1 / 3, 0.7)) add_case(0, 1, by)
for (by in c(0.1, 0.5, 2.5)) add_case(-3, 4, by)
add_case(10, 0, -2.5)
add_case(1, 1, 1)
add_case(0, 1, 1)
add_case(2, 11, 3)
# A step that lands short of `to` by less than the tolerance, and one that lands on it.
add_case(0, 0.3, 0.1)
add_case(0, 1, 0.1)
add_case(1, 2, 0.1)

# Ends from 1e7 to 1e12.
for (e in c(7, 8, 9, 10, 11, 12)) {
  big <- 10^e
  add_case(big, big + 10, 0.5)
  add_case(0, big, big / 4)
  add_case(0, big, big / 3)
  add_case(-big, big, big / 2)
  add_case(big, 0, -big / 5)
}

# any() and all() with NA: the same answers as the truth table of section 3.7.
logic <- list(c(FALSE, NA), c(TRUE, NA), c(NA, NA), c(TRUE, TRUE), c(FALSE, FALSE), logical(0), c(TRUE, FALSE, NA))
text <- function(x) ifelse(is.na(x), "NA", ifelse(x, "true", "false"))
logic_cases <- lapply(logic, function(x) list(
  items = as.list(text(x)), any = text(any(x)), all = text(all(x))
))

write_json(list(
  reference = reference_block(),
  seq = cases,
  logic = logic_cases
), file.path(here(), "collections_seq.json"))
