# R's values of the studentized range (release 0.7.0, step 18): ptukey() in R 4.6.0 on the cases
# of tests/reference/tukey.py, each in the tail the case names, and qtukey() on a small grid.
# R is the second check here, at its own accuracy. ptukey() sums fixed Gauss-Legendre rules: about
# eleven digits in the centre, fewer far out, and above 25,000 degrees of freedom the range of
# normal values in place of the studentized one. qtukey() stops a secant search at about four
# digits; on 2 degrees of freedom it is out by up to a fifth, and for a probability of 0.05 or
# 0.1 on 30 groups its search fails to converge (NaN), so its grid starts at 5 degrees of
# freedom and at the median. ptukey() refuses fewer than 2 degrees of freedom (NaN, written NA).
# The exact decimal reference of tukey.py is the strict one. Run with
# `Rscript tests/reference/r/tukey.R`.
source(file.path(dirname(sub("^--file=", "", commandArgs(FALSE)[grepl("^--file=", commandArgs(FALSE))])), "common.R"))
options(warn = -1)

cases <- read.csv(file.path(here(), "..", "data", "tukey_cases.csv"), colClasses = "character")
cases <- cases[cases$kind != "density", ]
dec <- function(x) sprintf("%.17g", x)

probability <- lapply(seq_len(nrow(cases)), function(i) {
  q <- as.numeric(cases$q[i]); k <- as.integer(cases$groups[i]); df <- as.numeric(cases$df[i])
  list(kind = cases$kind[i], q = dec(q), groups = k, df = dec(df),
       value = num(ptukey(q, k, df, lower.tail = cases$kind[i] == "lower")))
})

quantile <- list()
for (k in c(2L, 5L, 30L)) {
  for (df in c(5, 20, 1e6)) {
    for (p in c(0.5, 0.9, 0.95, 0.99)) {
      quantile[[length(quantile) + 1L]] <- list(p = dec(p), groups = k, df = dec(df),
                                                value = num(qtukey(p, k, df)))
    }
  }
}

out <- list(reference = reference_block(), ptukey = probability, qtukey = quantile)
write_json(out, file.path(here(), "tukey.json"))
