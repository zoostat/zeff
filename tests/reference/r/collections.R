# R's values for the collection functions of release 0.5.0, step 11: length(), nchar(type =
# "chars"), x[[1]], tail(x, 1), c() and rev() on fixed lists and strings, strings with combining
# marks and emoji among them. Every value is exact, so the comparison is exact. Run with
# `Rscript tests/reference/r/collections.R`.
source(file.path(dirname(sub("^--file=", "", commandArgs(FALSE)[grepl("^--file=", commandArgs(FALSE))])), "common.R"))

# Lists of numbers, as Zeff writes them; the Rust test writes each as a list literal.
lists <- list(
  c(3, 1, 2),
  c(1.5, -2.25, 1e12, 0.5),
  c(42),
  c(7, 7, 7, 7, 7, 7),
  c(1e-8, 1e7, -1e12, 0, 2.5)
)
# Strings: plain, accented, a combining mark, emoji, a flag (two regional indicators), empty.
strings <- c("abc", "café", "é", "\U0001F600", "\U0001F1EC\U0001F1E7", "", "a b c",
             "日本語")

list_cases <- lapply(lists, function(x) list(
  items = as.list(num(x)),
  len = length(x),
  first = num(x[[1]]),
  last = num(tail(x, 1)),
  appended = as.list(num(c(x, 9))),
  reversed = as.list(num(rev(x)))
))
string_cases <- lapply(strings, function(s) list(text = s, len = nchar(s, type = "chars")))

write_json(list(
  reference = reference_block(),
  lists = list_cases,
  strings = string_cases
), file.path(here(), "collections.json"))
