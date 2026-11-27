# R's answers for mcnemar_test (release 0.10.0, step 9): for paired 2 x 2 tables, the chi-squared
# statistic of mcnemar.test() with and without its continuity correction and the p-value of
# each; the exact p-value, which is binom.test() of one count of pairs that differ among both;
# and the difference of the two paired proportions with its standard error and Wald interval,
# written out from the formula, since base R prints none. The tables are small and large,
# balanced and lopsided, with an empty cell, with a single pair that differs, and that of the
# shipped survey `engagement`, counted here from its file.
#
# Where the two counts of pairs that differ are equal, mcnemar.test() with the correction
# gives 1 / (b + c), having taken 1 from a difference of 0; the engine gives 0 there, and such
# a table carries R's uncorrected statistic alone ("corrected" is left out). Where no pair
# differs R gives NaN, and no such table is here. Run with `Rscript tests/reference/r/mcnemar.R`.
source(file.path(dirname(sub("^--file=", "", commandArgs(FALSE)[grepl("^--file=", commandArgs(FALSE))])), "common.R"))
options(warn = -1)

root <- normalizePath(file.path(here(), "..", "..", ".."))

# cells: n11, n12, n21, n22, by rows, the first variable down the side.
paired <- function(name, cells) {
  tab <- matrix(cells, nrow = 2, byrow = TRUE)
  b <- tab[1, 2]; c <- tab[2, 1]; n <- sum(tab)
  plain <- mcnemar.test(tab, correct = FALSE)
  out <- list(name = name, table = as.list(as.integer(cells)),
              uncorrected = list(statistic = num(unname(plain$statistic)), df = as.integer(plain$parameter),
                                 p_value = num(plain$p.value)))
  if (b != c) {
    yates <- mcnemar.test(tab, correct = TRUE)
    out$corrected <- list(statistic = num(unname(yates$statistic)), p_value = num(yates$p.value))
  }
  out$exact_p_value <- num(binom.test(c(b, c))$p.value)
  d <- (c - b) / n
  se <- sqrt(b + c - (b - c)^2 / n) / n
  z <- qnorm(0.975)
  out$difference <- list(estimate = num(d), std_error = num(se),
                         ci = as.list(num(c(max(d - z * se, -1), min(d + z * se, 1)))))
  out
}

survey <- read_dataset(file.path(root, "packages", "tier1", "datasets", "data", "engagement.csv"))
answers <- table(survey$recommend_before, survey$recommend_after)   # rows before 0, 1; columns after 0, 1

cases <- list(
  paired("engagement", as.vector(t(answers))),
  paired("small", c(12, 9, 1, 8)),
  paired("textbook", c(794, 86, 150, 570)),
  paired("balanced", c(20, 7, 7, 20)),
  paired("one_way", c(5, 6, 0, 9)),
  paired("other_way", c(5, 0, 6, 9)),
  paired("one_pair", c(10, 1, 0, 10)),
  paired("two_pairs_apart", c(3, 2, 0, 3)),
  paired("near", c(40, 11, 10, 39)),
  paired("lopsided", c(2, 40, 3, 1)),
  paired("no_agreement", c(0, 13, 4, 0)),
  paired("large", c(41000, 2250, 1980, 39500)),
  paired("far", c(100, 400, 20, 100)))

out <- list(reference = reference_block(), cases = cases)
write_json(out, file.path(here(), "mcnemar.json"))
