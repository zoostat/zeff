# R's answers for proportion_test and chi2_test (release 0.7.0, step 10): prop.test() on one and
# two proportions, uncorrected and with Yates's correction, two-sided and one-sided, at other
# levels; binom.test() for the exact (Clopper-Pearson) interval; Newcombe's score interval for a
# difference computed here from Wilson's limits; chisq.test() for independence (with and without
# the correction) and for goodness of fit. On the shipped wages (union, female, industry), a coin
# of ten tosses and a 3 x 2 table with an empty cell. Numbers are written with 15 significant
# digits; the engine is held to a relative 1e-6 (the closed-form class). Run with
# `Rscript tests/reference/r/counts.R`.
source(file.path(dirname(sub("^--file=", "", commandArgs(FALSE)[grepl("^--file=", commandArgs(FALSE))])), "common.R"))
options(warn = -1)

root <- normalizePath(file.path(here(), "..", "..", ".."))

one <- function(name, x, n, p, correct = FALSE, alternative = "two.sided", level = 0.95) {
  t <- prop.test(x, n, p = p, correct = correct, alternative = alternative, conf.level = level)
  b <- binom.test(x, n, p = p, alternative = alternative, conf.level = level)
  z <- sign(x - n * p) * sqrt(unname(t$statistic))
  if (x == n * p) z <- 0
  phat <- x / n
  zq <- qnorm(if (alternative == "two.sided") 1 - (1 - level) / 2 else level)
  yates <- if (correct) 0.5 else 0
  wald <- c(max(phat - zq * sqrt(phat * (1 - phat) / n) - yates / n, 0),
            min(phat + zq * sqrt(phat * (1 - phat) / n) + yates / n, 1))
  if (alternative == "less") wald <- c(0, wald[2]) else if (alternative == "greater") wald <- c(wald[1], 1)
  list(name = name, x = as.integer(x), n = as.integer(n), p = num(p), z = num(z), p_value = num(t$p.value),
       wilson = as.list(num(as.vector(t$conf.int))), wald = as.list(num(wald)),
       exact = as.list(num(as.vector(b$conf.int))))
}

wilson_limits <- function(x, n, zq, yates) {
  phat <- x / n
  up <- function(pc) if (pc >= 1) 1 else (pc + zq^2 / (2 * n) + zq * sqrt(pc * (1 - pc) / n + zq^2 / (4 * n^2))) / (1 + zq^2 / n)
  lo <- function(pc) if (pc <= 0) 0 else (pc + zq^2 / (2 * n) - zq * sqrt(pc * (1 - pc) / n + zq^2 / (4 * n^2))) / (1 + zq^2 / n)
  c(max(lo(phat - yates / n), 0), min(up(phat + yates / n), 1))
}

two <- function(name, x, n, correct = FALSE, alternative = "two.sided", level = 0.95) {
  x <- unname(x); n <- unname(n)
  t <- prop.test(x, n, correct = correct, alternative = alternative, conf.level = level)
  delta <- x[1] / n[1] - x[2] / n[2]
  z <- sign(delta) * sqrt(unname(t$statistic))
  zq <- qnorm(if (alternative == "two.sided") 1 - (1 - level) / 2 else level)
  yates <- if (correct) 0.5 else 0
  l1 <- wilson_limits(x[1], n[1], zq, yates); l2 <- wilson_limits(x[2], n[2], zq, yates)
  p1 <- x[1] / n[1]; p2 <- x[2] / n[2]
  newcombe <- c(delta - sqrt((p1 - l1[1])^2 + (l2[2] - p2)^2), delta + sqrt((l1[2] - p1)^2 + (p2 - l2[1])^2))
  if (alternative == "less") newcombe <- c(-1, newcombe[2]) else if (alternative == "greater") newcombe <- c(newcombe[1], 1)
  list(name = name, x = as.list(as.integer(x)), n = as.list(as.integer(n)), z = num(z), p_value = num(t$p.value),
       delta = num(delta), wald = as.list(num(as.vector(t$conf.int))), newcombe = as.list(num(newcombe)))
}

chi <- function(name, tab, correct = FALSE, p = NULL) {
  t <- if (is.null(p)) chisq.test(tab, correct = correct) else chisq.test(tab, p = p)
  list(name = name, statistic = num(unname(t$statistic)), df = as.integer(unname(t$parameter)), p_value = num(t$p.value),
       expected = as.list(num(as.vector(t(t$expected)))))
}

wages <- read_dataset(file.path(root, "packages", "tier1", "datasets", "data", "wages.csv"))
union_x <- sum(wages$union); union_n <- length(wages$union)
by_female <- split(wages$union, wages$female)   # 0 = male first, as Zeff orders the groups by code
x2 <- sapply(by_female, sum); n2 <- sapply(by_female, length)

one_sample <- list(
  one("union_30", union_x, union_n, 0.3),
  one("union_30_corrected_greater", union_x, union_n, 0.3, correct = TRUE, alternative = "greater"),
  one("union_35_less_99", union_x, union_n, 0.35, alternative = "less", level = 0.99),
  one("coin_50", 7, 10, 0.5),
  one("coin_50_corrected", 7, 10, 0.5, correct = TRUE),
  one("coin_50_less", 7, 10, 0.5, alternative = "less"),
  one("coin_all", 10, 10, 0.5),
  one("coin_none_corrected", 0, 10, 0.5, correct = TRUE))
two_sample <- list(
  two("union_by_female", x2, n2),
  two("union_by_female_corrected_99", x2, n2, correct = TRUE, level = 0.99),
  two("union_by_female_less", x2, n2, alternative = "less"),
  two("small_corrected_greater", c(7, 3), c(10, 12), correct = TRUE, alternative = "greater"))

industry_female <- table(wages$industry, wages$female)
union_female <- table(wages$union, wages$female)
small <- matrix(c(1, 1, 1, 1, 1, 0), nrow = 3, byrow = TRUE)
chi_cases <- list(
  chi("industry_female", industry_female),
  chi("union_female", union_female),
  chi("union_female_corrected", union_female, correct = TRUE),
  chi("small_empty_cell", small),
  chi("female_half", table(wages$female), p = c(0.5, 0.5)),
  chi("industry_equal", table(wages$industry), p = rep(1 / 12, 12)))
industry_levels <- as.list(sort(unique(wages$industry)))

out <- list(reference = reference_block(), one_sample = one_sample, two_sample = two_sample,
            chi2 = chi_cases, industry_levels = industry_levels)
write_json(out, file.path(here(), "counts.json"))
